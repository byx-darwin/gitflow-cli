# Pipeline Report: 排除非终态 Run 统计口径修复 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix `gf pipeline report`'s success-rate calculation so that non-terminal (in-progress/running/pending/queued) CI runs no longer pollute the `total_runs` denominator, for both GitHub and GitLab providers.

**Architecture:** In both `GitHubPipelineProvider::report` and `GitLabPipelineProvider::report`, `total_runs` is currently computed from the raw time-windowed run/pipeline list, which includes non-terminal runs. `success_count`/`failure_counts` already only count terminal runs (GitHub: `conclusion.is_some()`; GitLab: `PipelineStatusEnum::Success`/`Failed`). The fix narrows `total_runs` to match — count only runs that have reached a terminal state — so `success_rate = success_count / total_runs` is no longer silently deflated by runs still in flight.

**Tech Stack:** Rust 2024, `async_trait`, `chrono`, `serde_json`, `tokio::test`, existing `MockCommandRunner` test harness (per-crate, in `runner.rs`).

**Spec:** `specs/pipeline-report-in-progress-fix-design.md` — read this alongside the plan; it documents the root cause, the confirmed scope boundary (no new fields, `PipelineStatusEnum` untouched, GitCode out of scope), and the acceptance criteria from Issue #285.

## Global Constraints

- Do not add new fields to `PipelineReport` (per design: 方案 A, no `in_progress_runs` field).
- Do not change `PipelineStatusEnum`, `gh_status_to_enum`, or `parse_pipeline_status`.
- Do not touch GitCode (`crates/gitcode/src/pipeline.rs::report` is an unimplemented stub — out of scope).
- `avg_duration_secs` and `top_failures` computation logic is unchanged; only the `total_runs` denominator changes.
- Every new/changed function must keep passing `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic` (project lint baseline).
- Follow TDD: RED (failing test) → GREEN (minimal fix) → REFACTOR, committing after each GREEN.

---

### Task 1: GitHub — exclude in-progress runs from `total_runs`

**Files:**
- Modify: `crates/github/src/pipeline.rs:364` (the `report` method's `total_runs` computation)
- Test: `crates/github/src/pipeline.rs` (`#[cfg(test)] mod tests`, same file — this crate's convention keeps unit tests in-file)

**Interfaces:**
- Consumes: existing `ReportRun { conclusion: Option<String>, created_at: String, updated_at: String }` (already defined at line 181), existing `MockCommandRunner::success(stdout: &str) -> Self` (test harness, `crate::runner::MockCommandRunner`)
- Produces: no new public interface — `GitHubPipelineProvider::report` (trait method, unchanged signature) now returns a corrected `total_runs`/`success_rate` in `PipelineReport`

- [ ] **Step 1: Write the failing test**

Add this test to the `mod tests` block in `crates/github/src/pipeline.rs` (after `test_should_count_all_failure_types_in_report_logic`, before the `MockCommandRunner`-based failure-path tests):

```rust
    #[tokio::test]
    async fn test_should_exclude_in_progress_runs_from_report_total_runs() {
        // 4 runs in the report window: 2 success, 1 failure, 1 still in-progress
        // (GitHub only sets `conclusion` once a run is `completed`, so an
        // in-progress run serializes with `"conclusion": null`).
        let now = chrono::Utc::now();
        let ts = |offset_secs: i64| (now - chrono::Duration::seconds(offset_secs)).to_rfc3339();

        let json = format!(
            r#"[
                {{"conclusion": "success", "createdAt": "{}", "updatedAt": "{}"}},
                {{"conclusion": "success", "createdAt": "{}", "updatedAt": "{}"}},
                {{"conclusion": "failure", "createdAt": "{}", "updatedAt": "{}"}},
                {{"conclusion": null, "createdAt": "{}", "updatedAt": "{}"}}
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

        let runner = MockCommandRunner::success(&json);
        let provider = GitHubPipelineProvider::with_runner("owner/repo", runner);

        let report = provider
            .report("main", 7)
            .await
            .expect("report should succeed");

        // Only 3 of the 4 runs have reached a terminal state (conclusion is
        // Some); the in-progress run (conclusion: null) must be excluded
        // from total_runs, not just from success/failure counts.
        assert_eq!(report.total_runs, 3);
        assert!((report.success_rate - (2.0 / 3.0)).abs() < f64::EPSILON);
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p gitflow-github test_should_exclude_in_progress_runs_from_report_total_runs -- --nocapture`
Expected: FAIL — `assertion left == right failed` with `left: 4, right: 3` (current code counts all 4 raw runs into `total_runs`).

- [ ] **Step 3: Write minimal implementation**

In `crates/github/src/pipeline.rs`, replace line 364:

```rust
        let total_runs = runs.len() as u64;
```

with:

```rust
        // Only runs that have reached a terminal state carry a `conclusion`
        // (GitHub sets it once `status == "completed"`). An in-progress run
        // serializes with `conclusion: null` and must not inflate the
        // denominator used for `success_rate`.
        let total_runs = runs.iter().filter(|r| r.conclusion.is_some()).count() as u64;
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p gitflow-github test_should_exclude_in_progress_runs_from_report_total_runs -- --nocapture`
Expected: PASS

Also re-run the full crate suite to confirm no regression:

Run: `cargo test -p gitflow-github`
Expected: all tests PASS (including `test_should_compute_report_from_runs`, which already asserts `total == runs.len()` on a fixture with a `null`-conclusion run at index 3 counted into `total` — **check this test**: it computes `total` locally as `runs.len() as u64` inside the test body itself, not via `aggregate_report_metrics`/`report()`, so it is unaffected by this change and will still pass with `total == 4`. Do not modify it — it documents pre-fix local arithmetic for illustration only, not the production `report()` path.)

- [ ] **Step 5: Commit**

```bash
git add crates/github/src/pipeline.rs
git commit -m "fix(pipeline): exclude in-progress runs from GitHub report total_runs"
```

---

### Task 2: GitLab — exclude non-terminal pipelines from `total_runs`

**Files:**
- Modify: `crates/gitlab/src/pipeline.rs:335-360` (the `report` method's `total_runs`/early-return logic)
- Test: `crates/gitlab/src/pipeline.rs` (`#[cfg(test)] mod tests`, same file)

**Interfaces:**
- Consumes: existing `PipelineStatusEnum::{Running, Pending, Success, Failed, Cancelled}` (`gitflow_core::pipeline`), existing `PipelineApiResponse` JSON shape consumed by `status()` (`id`, `ref_name`/`ref`, `status`, `created_at`, `updated_at`, `web_url`), existing `MockCommandRunner::success(stdout: &str) -> Self`
- Produces: no new public interface — `GitLabPipelineProvider::report` (trait method, unchanged signature) now returns a corrected `total_runs`/`success_rate`

- [ ] **Step 1: Write the failing test**

Add this test to the `mod tests` block in `crates/gitlab/src/pipeline.rs` (after `test_should_return_serialization_error_on_invalid_json_for_report`, at the end of the file, before the closing `}` of `mod tests`):

```rust
    #[tokio::test]
    async fn test_should_exclude_non_terminal_pipelines_from_report_total_runs() {
        // 4 pipelines in the report window: 2 success, 1 failed, 1 still running.
        let now = Utc::now();
        let ts = |offset_secs: i64| (now - chrono::Duration::seconds(offset_secs)).to_rfc3339();

        let json = format!(
            r#"[
                {{"id": 1, "ref_name": "main", "status": "success", "created_at": "{}", "updated_at": "{}"}},
                {{"id": 2, "ref_name": "main", "status": "success", "created_at": "{}", "updated_at": "{}"}},
                {{"id": 3, "ref_name": "main", "status": "failed", "created_at": "{}", "updated_at": "{}"}},
                {{"id": 4, "ref_name": "main", "status": "running", "created_at": "{}", "updated_at": "{}"}}
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

        let runner = MockCommandRunner::success(&json);
        let provider = GitLabPipelineProvider::with_runner("owner/repo", runner);

        let report = provider
            .report("main", 7)
            .await
            .expect("report should succeed");

        // Only 3 of the 4 pipelines have reached a terminal state
        // (Success/Failed/Cancelled); the running one must be excluded
        // from total_runs, not just from success/failure counts.
        assert_eq!(report.total_runs, 3);
        assert!((report.success_rate - (2.0 / 3.0)).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn test_should_zero_report_when_all_pipelines_are_running() {
        // No terminal pipelines at all in the window -> total_runs must be
        // 0 (not the raw count of running pipelines), success_rate 0.0, and
        // no division-by-zero NaN leaking into the report.
        let now = Utc::now();
        let ts = |offset_secs: i64| (now - chrono::Duration::seconds(offset_secs)).to_rfc3339();

        let json = format!(
            r#"[{{"id": 1, "ref_name": "main", "status": "running", "created_at": "{}", "updated_at": "{}"}}]"#,
            ts(60),
            ts(30),
        );

        let runner = MockCommandRunner::success(&json);
        let provider = GitLabPipelineProvider::with_runner("owner/repo", runner);

        let report = provider
            .report("main", 7)
            .await
            .expect("report should succeed");

        assert_eq!(report.total_runs, 0);
        assert!((report.success_rate - 0.0).abs() < f64::EPSILON);
        assert!(!report.success_rate.is_nan());
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p gitflow-gitlab test_should_exclude_non_terminal_pipelines_from_report_total_runs test_should_zero_report_when_all_pipelines_are_running -- --nocapture`
Expected: FAIL — the first test fails with `left: 4, right: 3` (current code counts the running pipeline into `total_runs`); the second test fails with `left: 1, right: 0` (current code counts the sole running pipeline as `total_runs = 1`, and `success_rate` would compute as `0/1 = 0.0` by coincidence — the `is_nan()` assertion is the forward-looking guard for the general case; the `total_runs` assertion is what actually fails here).

- [ ] **Step 3: Write minimal implementation**

In `crates/gitlab/src/pipeline.rs`, replace lines 352-360:

```rust
        let total_runs = recent.len() as u64;
        if total_runs == 0 {
            return Ok(PipelineReport {
                total_runs: 0,
                success_rate: 0.0,
                avg_duration_secs: 0.0,
                top_failures: vec![],
            });
        }
```

with:

```rust
        // Only pipelines that have reached a terminal state (Success/Failed/
        // Cancelled) count toward the denominator used for `success_rate`.
        // Running/Pending pipelines are still in flight and must not
        // silently deflate the reported rate.
        let total_runs = recent
            .iter()
            .filter(|p| !matches!(p.status, PipelineStatusEnum::Running | PipelineStatusEnum::Pending))
            .count() as u64;
        if total_runs == 0 {
            return Ok(PipelineReport {
                total_runs: 0,
                success_rate: 0.0,
                avg_duration_secs: 0.0,
                top_failures: vec![],
            });
        }
```

Note: `avg_duration_secs` and `top_failures` below this block still derive from `recent` (the full time-windowed list, unchanged) — this task only narrows the `total_runs`/`success_rate` denominator, per the confirmed design scope.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p gitflow-gitlab test_should_exclude_non_terminal_pipelines_from_report_total_runs test_should_zero_report_when_all_pipelines_are_running -- --nocapture`
Expected: PASS

Also re-run the full crate suite to confirm no regression:

Run: `cargo test -p gitflow-gitlab`
Expected: all tests PASS

- [ ] **Step 5: Commit**

```bash
git add crates/gitlab/src/pipeline.rs
git commit -m "fix(pipeline): exclude non-terminal pipelines from GitLab report total_runs"
```

---

### Task 3: Workspace-wide verification and regenerated pipeline-analysis-report

**Files:**
- None modified — this task only runs verification and produces the evidence required by Issue #285's 4th acceptance criterion.

**Interfaces:**
- Consumes: `gf pipeline report` CLI command (`apps/cli/src/commands/pipeline.rs`, unchanged), the fixes from Task 1 and Task 2
- Produces: a fresh pipeline-analysis-report (via `gf-pipeline-analyzer`, run in Phase 4 of the workflow — this task only confirms the underlying fix is correct at the workspace level; the actual regenerated report is produced by the `gf-pipeline-analyzer` skill during Phase 4 post-delivery checks, not here)

- [ ] **Step 1: Run full workspace test suite**

Run: `cargo test --workspace`
Expected: PASS, including all pre-existing GitHub/GitLab pipeline tests plus the 3 new tests from Task 1/2.

- [ ] **Step 2: Run clippy pedantic on touched crates**

Run: `cargo clippy -p gitflow-github -p gitflow-gitlab --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: no warnings/errors on the touched code paths.

- [ ] **Step 3: Run formatting check**

Run: `cargo +nightly fmt --check -p gitflow-github -p gitflow-gitlab`
Expected: no diff. If a diff appears, run `cargo +nightly fmt -p gitflow-github -p gitflow-gitlab` and re-check.

- [ ] **Step 4: Commit any formatting fixes (if Step 3 produced changes)**

```bash
git add crates/github/src/pipeline.rs crates/gitlab/src/pipeline.rs
git commit -m "style(pipeline): apply rustfmt after report total_runs fix"
```

(Skip this step entirely if Step 3 reported no diff.)
