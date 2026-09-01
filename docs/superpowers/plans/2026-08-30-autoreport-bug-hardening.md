# Auto-Report-Bug Hardening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close the four confirmed gaps in the `gf-autoreport-bug` processing path — unbounded archive growth, CI leaking through the reporting gate, a silently-missing GitHub label, and an unsafe unattended default — then produce real end-to-end evidence the pipeline works.

**Architecture:** Two small additive changes to the existing Rust write path (`apps/cli/src/error_reporter.rs`: archive pruning, CI-env gate), one additive check in the existing Bash hook (`hooks/auto-report-bug.sh`: label pre-check), one prose fix in the skill doc (`skills/gf-autoreport-bug/SKILL.md`: fail-safe default), and a manual verification pass. No new files, no new modules — every change extends an existing, already-tested unit.

**Tech Stack:** Rust 2024 (`apps/cli`), Bash + Bats (`hooks/`), Markdown skill docs.

**Spec:** `docs/superpowers/specs/2026-08-30-autoreport-bug-hardening-design.md`

## Global Constraints

- Never use `unwrap()`/`expect()` in production code (CLAUDE.md); `write_to_disk`/pruning errors must degrade gracefully, matching the existing best-effort posture of this module.
- TDD mandatory: RED → GREEN → REFACTOR for every code task; run `make test` at each GREEN step.
- Run `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic` after any Rust change in this plan.
- `deny.toml`, `.pre-commit-config.yaml`, `rust-toolchain.toml` must not be touched.
- Do not commit, push, or create/modify any real GitHub Issue without explicit user permission at the moment of that action (this plan's approval covers code changes only, not Task 5's live-Issue step).
- Skill source of truth is `skills/gf-autoreport-bug/SKILL.md` — never edit the gitignored `.claude/skills/` copy as if it were source.

---

### Task 1: Prune archived `pending.*.json` reports beyond a retention cap

**Files:**
- Modify: `apps/cli/src/error_reporter.rs` (add pruning after the existing archive-rename in `write_to_disk`, around line 101)
- Test: same file, `#[cfg(test)] mod tests`

**Interfaces:**
- Produces: `const MAX_ARCHIVED_REPORTS: usize = 10;` and `fn prune_archived_reports(dir: &Path)` — private, called from `write_to_disk` right after the existing `std::fs::rename(&path, &archived)?;` line. No other task depends on these names.

- [ ] **Step 1: Write the failing test**

Add to the `tests` module in `apps/cli/src/error_reporter.rs`:

```rust
#[test]
fn test_should_prune_archived_reports_beyond_retention_cap() {
    let tmp = tempfile::tempdir().expect("tempdir");

    // Write MAX_ARCHIVED_REPORTS + 3 reports in sequence; each write
    // archives the previous pending.json, so this produces
    // MAX_ARCHIVED_REPORTS + 2 archived files before pruning kicks in.
    for i in 0..(MAX_ARCHIVED_REPORTS + 3) {
        let report = ErrorReport::from_error(
            "issue list",
            "github",
            &format!("failure {i}"),
            "CLI_ERROR",
        );
        report.write_to_disk(tmp.path()).expect("write_to_disk");
        // Ensure filesystem-visible millisecond timestamps differ so
        // archived filenames sort deterministically.
        std::thread::sleep(std::time::Duration::from_millis(2));
    }

    let dir = tmp.path().join(".cache/bug-reports");
    let archived: Vec<_> = std::fs::read_dir(&dir)
        .expect("read bug-reports dir")
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n.starts_with("pending.") && n != "pending.json")
        .collect();

    assert_eq!(
        archived.len(),
        MAX_ARCHIVED_REPORTS,
        "archived reports must be capped at MAX_ARCHIVED_REPORTS: {archived:?}"
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p gitflow-cli error_reporter::tests::test_should_prune_archived_reports_beyond_retention_cap -- --nocapture`
Expected: FAIL — either `MAX_ARCHIVED_REPORTS` is undefined (compile error) or the archived count is `MAX_ARCHIVED_REPORTS + 2` (no pruning yet).

- [ ] **Step 3: Write minimal implementation**

In `apps/cli/src/error_reporter.rs`, add near the top (after the `use` block, before `ErrorReport`):

```rust
/// Maximum number of archived `pending.*.json` reports kept on disk.
///
/// Older archives beyond this cap are deleted on the next write to bound
/// unbounded growth of `.cache/bug-reports/` (a burst of CLI failures
/// otherwise accumulates one archive per failure forever).
const MAX_ARCHIVED_REPORTS: usize = 10;
```

Modify `write_to_disk` to call pruning right after the existing archive rename:

```rust
        if path.exists() {
            let archived = dir.join(format!(
                "pending.{}.json",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_or(0, |d| d.as_millis())
            ));
            std::fs::rename(&path, &archived)?;
            prune_archived_reports(&dir);
        }
```

Add the pruning function (near `set_pending_file_permissions`):

```rust
/// Delete archived `pending.<millis>.json` reports beyond
/// [`MAX_ARCHIVED_REPORTS`], oldest first.
///
/// Best-effort: any I/O error while listing or removing a file is
/// swallowed. The current `pending.json` is never touched by this
/// function — only files matching `pending.<digits>.json`.
fn prune_archived_reports(dir: &Path) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    let mut archives: Vec<(u128, PathBuf)> = entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let name = entry.file_name();
            let name = name.to_str()?;
            let millis = name
                .strip_prefix("pending.")
                .and_then(|rest| rest.strip_suffix(".json"))
                .and_then(|ts| ts.parse::<u128>().ok())?;
            Some((millis, entry.path()))
        })
        .collect();

    if archives.len() <= MAX_ARCHIVED_REPORTS {
        return;
    }

    archives.sort_by_key(|(millis, _)| *millis);
    let excess = archives.len() - MAX_ARCHIVED_REPORTS;
    for (_, path) in archives.into_iter().take(excess) {
        let _ = std::fs::remove_file(path);
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p gitflow-cli error_reporter:: -- --nocapture`
Expected: PASS, including the pre-existing `test_should_archive_previous_pending_on_second_write` (must still pass unchanged — 2 writes never triggers pruning since `MAX_ARCHIVED_REPORTS = 10`).

- [ ] **Step 5: Lint and commit**

Run: `cargo clippy -p gitflow-cli --all-targets --all-features -- -D warnings -W clippy::pedantic`
Fix any pedantic findings on the new code, then:

```bash
git add apps/cli/src/error_reporter.rs
git commit -m "fix(autoreport): prune archived pending.json reports beyond retention cap"
```

---

### Task 2: One-time cleanup of already-accumulated archives in this working copy

**Files:** none (operates on gitignored `.cache/bug-reports/`, not tracked by git)

**Interfaces:**
- Consumes: nothing from Task 1's code (this is a local hygiene action, independent of the fix landing).

- [ ] **Step 1: Confirm `.cache/` is gitignored (safe to prune locally)**

Run: `git check-ignore -v .cache/bug-reports/pending.json`
Expected: a match against `.gitignore` (already confirmed during investigation — `.cache/` is ignored).

- [ ] **Step 2: Count and remove stale archives, keeping the live `pending.json`**

```bash
ls .cache/bug-reports/pending.*.json 2>/dev/null | grep -v '^\.cache/bug-reports/pending\.json$' | wc -l
rm -f .cache/bug-reports/pending.*.json
```

This intentionally also removes the *current* `pending.json` in this working copy — it was observed during investigation to be a self-triggered `AUTH_FAILED` report from this very environment, not a real product bug. If a `pending.json` reflecting a genuine unresolved bug exists at execution time, keep it (only glob-delete `pending.[0-9]*.json`, i.e. the archives, and leave `pending.json` alone).

- [ ] **Step 3: Verify**

Run: `ls .cache/bug-reports/ | wc -l`
Expected: 0 (or 1 if a genuine live `pending.json` was deliberately kept per Step 2's caveat).

No commit — nothing here is git-tracked.

---

### Task 3: Skip auto-reporting when running inside CI

**Files:**
- Modify: `apps/cli/src/error_reporter.rs`
- Test: same file, `#[cfg(test)] mod tests`

**Interfaces:**
- Produces: `fn is_ci_environment() -> bool` and, for testability, `fn is_ci_environment_with(has_var: impl Fn(&str) -> bool) -> bool`. `maybe_report_error` gains one more early-return calling `is_ci_environment()`. No other task depends on these names.

- [ ] **Step 1: Write the failing test**

Add to the `tests` module:

```rust
#[test]
fn test_should_detect_ci_from_known_env_vars() {
    for var in ["CI", "GITHUB_ACTIONS", "GITLAB_CI", "CI_PIPELINE_ID", "CIRCLECI", "BUILDKITE", "JENKINS_URL"] {
        let present = |name: &str| name == var;
        assert!(
            is_ci_environment_with(present),
            "{var} must be recognized as a CI indicator"
        );
    }
}

#[test]
fn test_should_not_detect_ci_when_no_known_vars_set() {
    let present = |_: &str| false;
    assert!(!is_ci_environment_with(present));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p gitflow-cli error_reporter::tests::test_should_detect_ci -- --nocapture`
Expected: FAIL — `is_ci_environment_with` not found.

- [ ] **Step 3: Write minimal implementation**

Add near `should_skip_reporting`:

```rust
/// Environment variables set by common CI providers, checked to keep
/// `gf-autoreport-bug` from ever firing inside a pipeline run.
///
/// `gf-regression`'s skill doc already documents "never autoreport from
/// CI" as a hard rule; this makes that rule code-enforced rather than
/// relying on the LLM to honor the documentation. Note this is
/// independent of [`should_skip_reporting`] (the TTY check): CI runs are
/// almost always non-interactive, so without this check the TTY gate
/// alone would let CI failures straight through.
const CI_ENV_VARS: &[&str] = &[
    "CI",
    "GITHUB_ACTIONS",
    "GITLAB_CI",
    "CI_PIPELINE_ID",
    "CIRCLECI",
    "BUILDKITE",
    "JENKINS_URL",
];

/// Returns `true` when any known CI environment variable is present.
///
/// Extracted for testability — see [`is_ci_environment_with`].
fn is_ci_environment() -> bool {
    is_ci_environment_with(|name| std::env::var_os(name).is_some())
}

/// Testable core of [`is_ci_environment`]: takes a presence-check
/// closure instead of reading the real environment directly, so tests
/// don't need to mutate global process state.
fn is_ci_environment_with(has_var: impl Fn(&str) -> bool) -> bool {
    CI_ENV_VARS.iter().any(|var| has_var(var))
}
```

Wire it into the gate in `maybe_report_error`, right after the `should_skip_reporting()` check:

```rust
    if should_skip_reporting() {
        return Ok(());
    }

    if is_ci_environment() {
        return Ok(());
    }

    // Only report if user has joined the co-contribution plan
    if !is_co_contribution_enabled() {
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p gitflow-cli error_reporter:: -- --nocapture`
Expected: PASS, all tests including the two new ones and all pre-existing ones (`should_skip_reporting`'s existing test is unaffected — different function).

- [ ] **Step 5: Lint and commit**

Run: `cargo clippy -p gitflow-cli --all-targets --all-features -- -D warnings -W clippy::pedantic`

```bash
git add apps/cli/src/error_reporter.rs
git commit -m "fix(autoreport): skip error reporting when running in CI"
```

---

### Task 4: Fail loud when the `auto-report` GitHub label is missing

**Files:**
- Modify: `hooks/auto-report-bug.sh`
- Test: `hooks/tests/auto-report-bug.bats`

**Interfaces:**
- Consumes: the existing `AUTH_CHECK_FAILED` branch structure (lines 68–130 of `hooks/auto-report-bug.sh`) as the pattern to mirror.
- Produces: a new `LABEL_CHECK_FAILED` branch. No other task depends on this.

- [ ] **Step 1: Write the failing test**

Add to `hooks/tests/auto-report-bug.bats` (after the existing auth-success test — read the file first to match its exact helper style, e.g. `run_hook`, `PENDING_FILE`, `GH_CALL_LOG`):

```bash
@test "warns and does not emit banner when auto-report label is missing" {
  mkdir -p "$(dirname "$PENDING_FILE")"
  cat > "$PENDING_FILE" <<'JSON'
{"id":"abc","command":"issue list","platform":"github","error_code":"500","error_message":"boom","timestamp":"2026-08-30T00:00:00Z"}
JSON
  export GH_AUTH_STATUS="ok"
  export GH_LABEL_LIST_OUTPUT=""

  run_hook

  [ "$status" -eq 0 ]
  [[ "$output" == *"auto-report"*"label"* ]] || [[ "$output" == *"标签"* ]]
  [[ "$output" != *"MUST load the gf-autoreport-bug skill"* ]]
  [ -f "$PENDING_FILE" ]
}

@test "emits banner when auto-report label exists" {
  mkdir -p "$(dirname "$PENDING_FILE")"
  cat > "$PENDING_FILE" <<'JSON'
{"id":"abc","command":"issue list","platform":"github","error_code":"500","error_message":"boom","timestamp":"2026-08-30T00:00:00Z"}
JSON
  export GH_AUTH_STATUS="ok"
  export GH_LABEL_LIST_OUTPUT="auto-report"

  run_hook

  [ "$status" -eq 0 ]
  [[ "$output" == *"MUST load the gf-autoreport-bug skill"* ]]
}
```

Update the mock `gh` in `setup()` (same file) to answer `label list`:

```bash
  cat > "$bindir/gh" <<'MOCK'
#!/usr/bin/env bash
printf '%s\n' "$*" >> "$GH_CALL_LOG"
if [ "$1" = "label" ] && [ "$2" = "list" ]; then
  printf '%s\n' "${GH_LABEL_LIST_OUTPUT:-auto-report}"
  exit 0
fi
if [ "${GH_AUTH_STATUS:-ok}" = "fail" ]; then
  exit 1
fi
exit 0
MOCK
```

(This replaces the existing mock `gh` body in `setup()` — keep the rest of `setup()` unchanged.)

- [ ] **Step 2: Run test to verify it fails**

Run: `bats hooks/tests/auto-report-bug.bats`
Expected: the two new tests FAIL (no label-list call is made yet, so `GH_LABEL_LIST_OUTPUT=""` has no effect and the banner is emitted regardless).

- [ ] **Step 3: Write minimal implementation**

In `hooks/auto-report-bug.sh`, after the existing `AUTH_CHECK_FAILED` block (which `exit 0`s on failure) and before the banner block (currently starting at the blank `echo ""` before `━━━` on what is line 132), insert:

```bash
# Label existence pre-check (G3) — verify the `auto-report` label exists
# on the target repo before asking the LLM to `gh issue create` with it.
# A missing label previously surfaced as a raw 422 at Issue-creation time
# with no actionable guidance; this fails loud, earlier, with a fix.
LABEL_CHECK_FAILED=false
if command -v gh >/dev/null 2>&1; then
  LABEL_LIST_OUTPUT=$(gh label list --repo byx-darwin/gitflow-cli --search auto-report --json name -q '.[].name' 2>/dev/null || true)
  if [ -z "$LABEL_LIST_OUTPUT" ]; then
    LABEL_CHECK_FAILED=true
    log_hook "label check failed (auto-report label missing on byx-darwin/gitflow-cli)"
  fi
fi

if [ "$LABEL_CHECK_FAILED" = "true" ]; then
  echo ""
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo "  ⚠️  仓库缺少 auto-report 标签，无法自动创建 Issue"
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo ""
  echo "  请先创建该标签，然后重新触发："
  echo "    gh label create auto-report --repo byx-darwin/gitflow-cli \\"
  echo "      --description \"Automatically filed by gf-autoreport-bug\" --color FBCA04"
  echo ""
  exit 0
fi
```

- [ ] **Step 4: Run test to verify it passes**

Run: `bats hooks/tests/auto-report-bug.bats`
Expected: PASS, all tests including the two new ones and every pre-existing test in the file (auth-cache-hit tests must not now also require a `label list` call to succeed — confirm the mock's `label list` branch runs unconditionally regardless of `GH_AUTH_STATUS`, since label-check only runs after auth already succeeded in the real script).

- [ ] **Step 5: Commit**

```bash
git add hooks/auto-report-bug.sh hooks/tests/auto-report-bug.bats
git commit -m "fix(autoreport): fail loud when the auto-report label is missing"
```

---

### Task 5: Make the unattended Preview default fail-safe (skip, not create)

**Files:**
- Modify: `skills/gf-autoreport-bug/SKILL.md`
- Modify: `docs/references/gf-autoreport-bug-params.md`

**Interfaces:** none (prose only).

- [ ] **Step 1: Update the Workflow step in `skills/gf-autoreport-bug/SKILL.md`**

Change line 57 from:

```
3b. **Preview** — Print sanitized summary + planned title/body. Ask: `create / skip / modify`. Non-interactive default: create.
```

to:

```
3b. **Preview** — Print sanitized summary + planned title/body. Ask: `create / skip / modify`. Non-interactive default: **skip** — keep `pending.json`, append `[timestamp] preview skipped (non-interactive)` to `processing.log`, stop. A human must re-run interactively to actually create the Issue.
```

Also update the Mermaid decision flow (lines 36-50) so the `P[Preview]` node reflects the new default — change:

```
    P --> J[Create Issue]
```

to:

```
    P -->|interactive confirm| J[Create Issue]
    P -->|non-interactive default| I
```

And add one row to the **Error Handling** table (after the `Dedup hit` row):

```
| Non-interactive preview | Keep file + log, stop (fail-safe default) |
```

- [ ] **Step 2: Update `docs/references/gf-autoreport-bug-params.md`**

In the "命令速查" section, after the existing `gh issue create` example, add a short note documenting the new default and the two hardening gates from Tasks 3–4:

```markdown
## 安全网关（2026-08-30 加固）

- **CI 跳过**：`error_reporter` 检测到 `CI`/`GITHUB_ACTIONS`/`GITLAB_CI`/`CI_PIPELINE_ID`/`CIRCLECI`/`BUILDKITE`/`JENKINS_URL` 任一环境变量存在时，直接跳过写入 `pending.json`，不会产生上报。
- **标签预检查**：Stop Hook 在认证成功后会先执行 `gh label list --repo byx-darwin/gitflow-cli --search auto-report`，标签不存在则打印修复命令并停止，不再触发 skill。
- **非交互默认值**：Preview 阶段在非交互场景（Stop Hook 触发即是此场景）下默认 `skip`，不会自动创建 Issue；只有交互式确认才会创建。
```

- [ ] **Step 3: Proofread and sync check**

Run: `make check-agent-sync`
Expected: passes (verifies `CLAUDE.md` presence; this task does not touch `CLAUDE.md` itself).

Read both edited files back once to confirm the Mermaid diagram still parses (balanced brackets, no dangling arrows) and the workflow numbering (`3b`) still reads coherently.

- [ ] **Step 4: Commit**

```bash
git add skills/gf-autoreport-bug/SKILL.md docs/references/gf-autoreport-bug-params.md
git commit -m "docs(autoreport): default unattended preview to skip, document CI/label gates"
```

---

### Task 6: End-to-end verification (manual, gated)

**Files:** none (verification only); result recorded in `docs/superpowers/tests/skills/gf-autoreport-bug-test.md`.

**Interfaces:**
- Consumes: the built `gf` binary with Tasks 1–5 applied (`make build`).

- [ ] **Step 1: Full local test suite**

Run: `make test` and `bats hooks/tests/auto-report-bug.bats`
Expected: all green, including the new tests from Tasks 1, 3, 4.

- [ ] **Step 2: Local sandbox dry run of the coded gates (no `gh` network calls)**

In a scratch temp directory (a throwaway `git init`'d repo, *not* this working copy), set `gitflow.co_contribution: true` in a local `.claude/settings.json`, then run a `gf` subcommand known to fail with a non-`USER_INPUT_ERROR` code (e.g. an unauthenticated `gf issue list` against a fake remote) with stderr piped (non-interactive) and `CI` unset. Confirm:
- `pending.json` is written under `.cache/bug-reports/` in that scratch repo.
- Re-running with `CI=true` set does **not** write a new `pending.json` (Task 3's gate).
- Writing 12 failures in a row leaves exactly 10 archived files (Task 1's cap).

- [ ] **Step 3: STOP — explicit confirmation gate before any live GitHub write**

Do not proceed past this step without the user explicitly confirming, at this point in execution, that a real `gh issue create` against `byx-darwin/gitflow-cli` should run. This is independent of this plan's earlier approval — CLAUDE.md requires explicit permission at the moment of any action that changes shared/public state (filing a public Issue qualifies), and it must be closed out afterward as a test artifact, not left as a real bug report.

- [ ] **Step 4: Live run (only after Step 3's explicit go-ahead)**

Drive the scratch repo's `pending.json` through the real hook (`gh` on `PATH`, real auth) and let the `gf-autoreport-bug` skill process it end to end: validate → auth → dedup → label pre-check (Task 4) → interactive Preview confirm → create. Confirm the created Issue's title matches `[auto-report] gf {command} — {error_code}`, has the `auto-report` label applied, and its body matches the template in `docs/references/gf-autoreport-bug-params.md`.

- [ ] **Step 5: Close out the test Issue and record evidence**

Close the created Issue with a comment noting it was a deliberate end-to-end verification run, not a real bug. Update `docs/superpowers/tests/skills/gf-autoreport-bug-test.md`'s 运行记录 table: change the relevant `[待运行]` row(s) to `[已运行]` with the date, outcome, and a link to the closed Issue.

- [ ] **Step 6: Commit the evidence update**

```bash
git add docs/superpowers/tests/skills/gf-autoreport-bug-test.md
git commit -m "test(autoreport): record first end-to-end verification run"
```
