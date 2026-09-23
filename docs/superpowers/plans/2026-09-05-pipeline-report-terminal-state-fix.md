# Pipeline Report Terminal-State Fix Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix `gf pipeline report` (GitHub provider) so that `running`/`queued` runs and jobs are never counted toward `total_runs`/`success_rate` or misattributed into `top_failures`, even when their `conclusion` field carries an anomalous non-null value while the run/job has not actually reached a terminal state.

**Architecture:** GitHub's `report()` currently gates "has this run concluded?" on `conclusion.is_some()` alone, and never requests the `status` field at all. This plan adds `status` to the queried fields and gates terminality on `status == "completed"` (mirroring how `status()` already works), applied at both the run level and the job level (inside failure attribution). A small shared `PipelineStatusEnum::is_terminal()` helper is added to `gitflow-core` and used to deduplicate GitLab's already-correct inline terminal check.

**Tech Stack:** Rust 2024, `serde`/`serde_json`, `tokio` async tests, existing `MockCommandRunner`/`SequencedMockCommandRunner` test doubles in `crates/github/src/runner.rs`.

**Spec:** `docs/superpowers/specs/2026-09-05-pipeline-report-terminal-state-fix-design.md`

## Global Constraints

- No public API signature changes (all touched items are crate-private or additive).
- `#![forbid(unsafe_code)]` — no unsafe code introduced.
- Every fallible path already returns `Result`; this fix does not add new fallibility.
- Follow RED → GREEN → REFACTOR: write/adjust the failing test before touching production code in Task 3.
- Run `cargo fmt` and `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic` on every touched crate before considering a task done.

---

### Task 1: Shared `is_terminal()` helper on `PipelineStatusEnum`

**Files:**
- Modify: `crates/core/src/pipeline.rs:18-29` (enum definition), `crates/core/src/pipeline.rs` tests module (after line 189)
- Test: same file, `#[cfg(test)] mod tests`

**Interfaces:**
- Produces: `PipelineStatusEnum::is_terminal(&self) -> bool` — `true` for `Success`/`Failed`/`Cancelled`, `false` for `Running`/`Pending`. Used by Task 2 (GitLab) directly; Task 3 (GitHub) does NOT use this method (see Task 3 rationale) — it gates on the raw `status` string instead, before a `PipelineStatusEnum` even exists for that value.

- [ ] **Step 1: Write the failing tests**

Add to `crates/core/src/pipeline.rs`, inside `#[cfg(test)] mod tests`, right after `test_should_deserialize_pipeline_status_enum_from_snake_case` (currently ends at line 189):

```rust
    #[test]
    fn test_should_report_terminal_states_as_terminal() {
        assert!(PipelineStatusEnum::Success.is_terminal());
        assert!(PipelineStatusEnum::Failed.is_terminal());
        assert!(PipelineStatusEnum::Cancelled.is_terminal());
    }

    #[test]
    fn test_should_report_in_flight_states_as_not_terminal() {
        assert!(!PipelineStatusEnum::Running.is_terminal());
        assert!(!PipelineStatusEnum::Pending.is_terminal());
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p gitflow-core --lib pipeline::tests::test_should_report -- --nocapture`
Expected: FAIL with "no method named `is_terminal` found"

- [ ] **Step 3: Implement `is_terminal()`**

Insert into `crates/core/src/pipeline.rs` right after the enum's closing `}` (currently line 29), before the `PipelineStatus` struct doc comment:

```rust
impl PipelineStatusEnum {
    /// 是否表示已收尾（终态）：`Success`/`Failed`/`Cancelled`。
    ///
    /// `Running`/`Pending` 表示流水线仍在执行或排队中，尚无最终结论；
    /// 调用方在统计成功率、进行失败归因等场景中应先用此方法过滤掉
    /// 未收尾项，避免把"进行中"误判为"已收尾"甚至"失败"。
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        !matches!(self, PipelineStatusEnum::Running | PipelineStatusEnum::Pending)
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p gitflow-core --lib pipeline::`
Expected: PASS (all pipeline tests, including the 2 new ones)

- [ ] **Step 5: Commit**

```bash
git add crates/core/src/pipeline.rs
git commit -m "feat(core): add PipelineStatusEnum::is_terminal helper"
```

---

### Task 2: GitLab `report()` — dedupe terminal check via `is_terminal()`

**Files:**
- Modify: `crates/gitlab/src/pipeline.rs:352-364`

**Interfaces:**
- Consumes: `PipelineStatusEnum::is_terminal(&self) -> bool` from Task 1.
- Produces: no new interface — pure refactor, `report()`'s external behavior is unchanged.

- [ ] **Step 1: Run existing tests to confirm current GREEN baseline**

Run: `cargo test -p gitflow-gitlab --lib pipeline::`
Expected: PASS (baseline before refactor — this is a behavior-preserving cleanup, not a new-feature RED/GREEN cycle)

- [ ] **Step 2: Replace the inline terminal check**

In `crates/gitlab/src/pipeline.rs`, replace:

```rust
        let total_runs = recent
            .iter()
            .filter(|p| {
                !matches!(
                    p.status,
                    PipelineStatusEnum::Running | PipelineStatusEnum::Pending
                )
            })
            .count() as u64;
```

with:

```rust
        let total_runs = recent
            .iter()
            .filter(|p| p.status.is_terminal())
            .count() as u64;
```

- [ ] **Step 3: Run tests to verify still green**

Run: `cargo test -p gitflow-gitlab --lib pipeline::`
Expected: PASS (identical results to Step 1 — behavior unchanged)

- [ ] **Step 4: Commit**

```bash
git add crates/gitlab/src/pipeline.rs
git commit -m "refactor(gitlab): use PipelineStatusEnum::is_terminal in report()"
```

---

### Task 3: GitHub `report()` / `attribute_top_failures()` — fix the root cause (Issue #324)

**Files:**
- Modify: `crates/github/src/pipeline.rs` — `ReportRun` struct (lines 224-232), `attribute_top_failures` (lines 179-221), `aggregate_report_metrics` (lines 242-278), `report()` (lines 361-446), plus the 4 existing test fixtures listed in Step 1
- Test: same file, `#[cfg(test)] mod tests` (new tests + fixture updates)

**Interfaces:**
- Consumes: nothing from Task 1/2 — this task gates terminality on the raw `status: String` field (`"completed"`), not on `PipelineStatusEnum`, because `gh_status_to_enum`'s fallback arm (`_ => PipelineStatusEnum::Running`) would misclassify a `"completed"` run with an unrecognized/legacy `conclusion` string as non-terminal — which is the opposite of what `report()` needs (such a run must still count as terminal + failure, matching `is_failure_conclusion`'s existing "treat unknown conclusions as failure" semantics). Routing through the enum here would trade one correctness bug for another.
- Produces: `ReportRun.status: String` (new field, deserialized from `"status"` JSON key); `aggregate_report_metrics(&[&ReportRun]) -> (u64, f64, u64)` (signature changed from `&[ReportRun]`); `attribute_top_failures(&self, &[&ReportRun]) -> Vec<String>` (signature changed from `&[ReportRun]`).

- [ ] **Step 1: Write/adjust the failing tests first**

1a. Update the 4 existing test fixtures so they compile once `ReportRun` gains a required `status` field (add `"status": "completed"` to every run object — these fixtures represent runs that have genuinely finished, so `"completed"` is correct for all of them):

In `test_should_exclude_in_progress_runs_from_report_total_runs` (around line 842-857), change the JSON to:

```rust
        let json = format!(
            r#"[
                {{"databaseId": 1, "status": "completed", "conclusion": "success", "createdAt": "{}", "updatedAt": "{}"}},
                {{"databaseId": 2, "status": "completed", "conclusion": "success", "createdAt": "{}", "updatedAt": "{}"}},
                {{"databaseId": 3, "status": "completed", "conclusion": "failure", "createdAt": "{}", "updatedAt": "{}"}},
                {{"databaseId": 4, "status": "in_progress", "conclusion": null, "createdAt": "{}", "updatedAt": "{}"}}
            ]"#,
            ts(600),
            ts(300),
            ts(500),
            ts(200),
            ts(400),
            ts(100),
            ts(60),
            ts(30),
        );
```

In `test_should_attribute_top_failures_to_job_names_not_generic_conclusion` (around line 984-996), change the JSON to:

```rust
        let run_list_json = format!(
            r#"[
                {{"databaseId": 10, "status": "completed", "conclusion": "failure", "createdAt": "{}", "updatedAt": "{}"}},
                {{"databaseId": 11, "status": "completed", "conclusion": "failure", "createdAt": "{}", "updatedAt": "{}"}},
                {{"databaseId": 12, "status": "completed", "conclusion": "success", "createdAt": "{}", "updatedAt": "{}"}}
            ]"#,
            ts(600),
            ts(500),
            ts(400),
            ts(300),
            ts(200),
            ts(100),
        );
```

In `test_should_fall_back_to_generic_conclusion_when_jobs_fetch_fails` (around line 1048-1052), change the JSON to:

```rust
        let run_list_json = format!(
            r#"[{{"databaseId": 20, "status": "completed", "conclusion": "failure", "createdAt": "{}", "updatedAt": "{}"}}]"#,
            ts(600),
            ts(500),
        );
```

In `test_should_not_call_jobs_api_for_non_failure_runs` (around line 1083-1098), change the JSON to:

```rust
        let run_list_json = format!(
            r#"[
                {{"databaseId": 30, "status": "completed", "conclusion": "success", "createdAt": "{}", "updatedAt": "{}"}},
                {{"databaseId": 31, "status": "completed", "conclusion": "cancelled", "createdAt": "{}", "updatedAt": "{}"}},
                {{"databaseId": 32, "status": "completed", "conclusion": "skipped", "createdAt": "{}", "updatedAt": "{}"}},
                {{"databaseId": 33, "status": "completed", "conclusion": "neutral", "createdAt": "{}", "updatedAt": "{}"}}
            ]"#,
            ts(800),
            ts(700),
            ts(600),
            ts(500),
            ts(400),
            ts(300),
            ts(200),
            ts(100),
        );
```

1b. Add two NEW regression tests, right after `test_should_exclude_in_progress_runs_from_report_total_runs` (after its closing `}`, before the `// --- Failure-path tests ...` comment):

```rust
    #[tokio::test]
    async fn test_should_exclude_runs_with_non_terminal_status_even_when_conclusion_is_present() {
        // Reproduces issue #324: `gh run list` can report a non-null
        // `conclusion` for a run whose `status` has not reached `"completed"`.
        // Trusting `conclusion.is_some()` alone (the pre-fix behavior) would
        // misclassify this still-running run as terminal, and even as a
        // failure, inflating the denominator and corrupting `success_rate`.
        let now = chrono::Utc::now();
        let ts = |offset_secs: i64| (now - chrono::Duration::seconds(offset_secs)).to_rfc3339();

        let json = format!(
            r#"[
                {{"databaseId": 1, "status": "completed", "conclusion": "success", "createdAt": "{}", "updatedAt": "{}"}},
                {{"databaseId": 2, "status": "in_progress", "conclusion": "failure", "createdAt": "{}", "updatedAt": "{}"}}
            ]"#,
            ts(600),
            ts(300),
            ts(200),
            ts(100),
        );

        let runner = MockCommandRunner::success(&json);
        let provider = GitHubPipelineProvider::with_runner("owner/repo", runner);

        let report = provider
            .report("main", 7)
            .await
            .expect("report should succeed");

        // Only the `completed` run counts; the `in_progress` run must be
        // excluded from total_runs/success_rate despite carrying a non-null
        // `conclusion`, and must not appear in top_failures either.
        assert_eq!(report.total_runs, 1);
        assert!((report.success_rate - 1.0).abs() < f64::EPSILON);
        assert!(report.top_failures.is_empty());
    }

    #[tokio::test]
    async fn test_should_not_attribute_failure_to_a_still_in_progress_job() {
        use crate::runner::SequencedMockCommandRunner;

        let now = chrono::Utc::now();
        let ts = |offset_secs: i64| (now - chrono::Duration::seconds(offset_secs)).to_rfc3339();

        let run_list_json = format!(
            r#"[{{"databaseId": 40, "status": "completed", "conclusion": "failure", "createdAt": "{}", "updatedAt": "{}"}}]"#,
            ts(600),
            ts(500),
        );

        // The run has concluded overall, but job-level data lags behind: one
        // job already succeeded, the other is still `in_progress` yet
        // (matching the real-world anomaly behind issue #324) already carries
        // a non-null `conclusion` value. Attribution must not label the
        // still-running job as the failure — it must fall back to the run's
        // generic conclusion instead.
        let jobs_json = r#"{
            "jobs": [
                {
                    "databaseId": 1,
                    "name": "MSRV",
                    "status": "completed",
                    "conclusion": "success",
                    "startedAt": "2026-07-01T10:00:00Z",
                    "completedAt": "2026-07-01T10:01:00Z",
                    "url": "https://example.com/job/1"
                },
                {
                    "databaseId": 2,
                    "name": "Test (windows-latest)",
                    "status": "in_progress",
                    "conclusion": "failure",
                    "startedAt": "2026-07-01T10:00:00Z",
                    "url": "https://example.com/job/2"
                }
            ]
        }"#;

        let runner = SequencedMockCommandRunner::from_results(&[
            (true, &run_list_json),
            (true, jobs_json),
        ]);
        let provider = GitHubPipelineProvider::with_runner("owner/repo", runner);

        let report = provider
            .report("main", 7)
            .await
            .expect("report should succeed");

        assert_eq!(report.total_runs, 1);
        assert_eq!(report.top_failures, vec!["failure".to_string()]);
        assert!(!report.top_failures.contains(&"MSRV".to_string()));
        assert!(!report.top_failures.contains(&"Test (windows-latest)".to_string()));
    }
```

- [ ] **Step 2: Run tests to verify the new/changed ones fail**

Run: `cargo test -p gitflow-github --lib pipeline::tests`
Expected: compile error (missing `status` field on `ReportRun` struct literal is fine since JSON fixtures don't require a matching Rust struct field to compile — the actual expected failures are the **assertions** in the two new tests: `test_should_exclude_runs_with_non_terminal_status_even_when_conclusion_is_present` fails with `total_runs == 2` (not 1); `test_should_not_attribute_failure_to_a_still_in_progress_job` fails with `top_failures == ["Test (windows-latest)"]` (not `["failure"]`)

- [ ] **Step 3: Implement the fix**

3a. Add `status` to `ReportRun` (around line 224-232):

```rust
/// `gh run list` 的 report 统计所需最小字段集。
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReportRun {
    database_id: u64,
    /// GitHub Actions run 的整体状态（`"queued"`/`"in_progress"`/`"completed"` 等）。
    ///
    /// 只有 `"completed"` 才代表该 run 已收尾——`conclusion` 字段不能单独
    /// 作为收尾判据：观测到 `gh run list` 会在 run 仍未收尾时也为
    /// `conclusion` 填入非 null 值（issue #324）。
    status: String,
    conclusion: Option<String>,
    created_at: String,
    updated_at: String,
}
```

3b. Update the `--json` field list in `report()` (around line 380-384):

```rust
                    "databaseId,status,conclusion,createdAt,updatedAt",
```

3c. Replace the total_runs/aggregate block in `report()` (around lines 407-413):

```rust
        // A run only carries a meaningful `conclusion` once its `status` is
        // `"completed"`. Gating on `status` (rather than `conclusion.is_some()`)
        // is required because `gh run list` can populate `conclusion` for a
        // run that has not actually finished (issue #324) — trusting presence
        // alone re-admits in-progress runs into the denominator.
        let terminal_runs: Vec<&ReportRun> = runs
            .iter()
            .filter(|run| run.status == "completed")
            .collect();

        let total_runs = terminal_runs.len() as u64;

        let (success_count, total_duration_secs, has_duration) =
            aggregate_report_metrics(&terminal_runs);
```

3d. Update `aggregate_report_metrics`'s signature (line 248) — body is unchanged, only the parameter type changes:

```rust
fn aggregate_report_metrics(runs: &[&ReportRun]) -> (u64, f64, u64) {
```

3e. Update the `attribute_top_failures` call site (around line 438) to pass the pre-filtered terminal runs:

```rust
        let top_failures = self.attribute_top_failures(&terminal_runs).await;
```

3f. Update `attribute_top_failures`'s signature and job-level gate (lines 179-221):

```rust
    /// 为一批已收尾且失败类的 run 归因到具体失败 job 名称，用于 [`PipelineReport::top_failures`]。
    ///
    /// 只对结论落在失败类（见 [`is_failure_conclusion`]）的 run 发起 `jobs` 查询，
    /// 成功和非失败终态（`cancelled`/`skipped`/`neutral`）的 run 不消耗额外 API
    /// 调用。若某次 run 的 job 级数据无法获取、其中没有失败类 job，或匹配到的
    /// job 自身尚未收尾（`status != "completed"`，issue #324），则回退为该 run
    /// 的通用 `conclusion` 字符串，确保该样本仍计入统计而不是被静默丢弃或
    /// 误将一个仍在执行的 job 当作失败来源。
    async fn attribute_top_failures(&self, runs: &[&ReportRun]) -> Vec<String> {
        let mut failure_counts: std::collections::HashMap<String, u64> =
            std::collections::HashMap::new();

        for run in runs {
            let Some(conclusion) = run.conclusion.as_deref() else {
                continue;
            };
            if !is_failure_conclusion(conclusion) {
                continue;
            }

            let label = match self.jobs(run.database_id).await {
                Ok(jobs) => jobs
                    .iter()
                    .find(|job| {
                        job.status == "completed"
                            && job.conclusion.as_deref().is_some_and(is_failure_conclusion)
                    })
                    .map_or_else(|| conclusion.to_owned(), |job| job.name.clone()),
                Err(err) => {
                    debug!(
                        repo = %self.repo,
                        pipeline_id = run.database_id,
                        error = %err,
                        "failed to fetch jobs for failure attribution, falling back to generic conclusion"
                    );
                    conclusion.to_owned()
                }
            };

            *failure_counts.entry(label).or_insert(0) += 1;
        }

        // 按失败次数降序排列；次数相同时按标签字母序，保证输出稳定。
        let mut failures: Vec<_> = failure_counts.into_iter().collect();
        failures.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        failures.into_iter().map(|(label, _)| label).collect()
    }
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p gitflow-github --lib pipeline::`
Expected: PASS (all existing + 2 new tests)

- [ ] **Step 5: Commit**

```bash
git add crates/github/src/pipeline.rs
git commit -m "fix(github): gate pipeline report terminal state on run/job status, not conclusion presence

Closes #324"
```

---

### Task 4: Full verification across touched crates

**Files:** none (verification only)

- [ ] **Step 1: Run the full test suite for the 3 touched crates**

Run: `cargo test -p gitflow-core -p gitflow-github -p gitflow-gitlab`
Expected: PASS, 0 failures

- [ ] **Step 2: Format**

Run: `cargo +nightly fmt -- --check` (or `cargo fmt` to apply if it reports diffs)
Expected: no diff after `cargo fmt`

- [ ] **Step 3: Clippy pedantic on touched crates**

Run: `cargo clippy -p gitflow-core -p gitflow-github -p gitflow-gitlab --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: no warnings

- [ ] **Step 4: Full workspace build sanity check**

Run: `cargo build --workspace`
Expected: builds cleanly (confirms no downstream crate depends on the changed private signatures in an incompatible way — none expected since all changed items are private to their modules)

- [ ] **Step 5: Commit (only if fmt/clippy produced fixes not already committed in Task 1-3)**

```bash
git add -A
git commit -m "chore: fmt + clippy fixes for pipeline report terminal-state fix"
```
