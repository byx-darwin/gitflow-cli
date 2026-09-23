# gf --version Git SHA Traceability Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make `gf --version` report the git commit it was built from, so a pipeline-analysis snapshot can be checked against the repo's current HEAD — closing the traceability gap that made Issue #378's "fix looks ineffective" reports impossible to diagnose.

**Architecture:** `apps/cli/build.rs` shells out to `git rev-parse --short HEAD` at compile time (never panics; falls back to `"unknown"` on any failure) and writes the result to a generated file included by `main.rs`. `main.rs` appends it to the version string passed to clap. Two `cargo:rerun-if-changed` directives (on `logs/HEAD` and `HEAD`, resolved via `git rev-parse --git-path`, which correctly handles both the main worktree and any `gf-workflow`-created linked worktree) ensure the SHA is refreshed on every new commit or checkout, not just when `skills/` changes — this is the same staleness class of bug this Issue is about, so getting it wrong here would defeat the fix's purpose.

**Tech Stack:** Rust 2024, `std::process::Command` (no new dependency — the plan's approved design explicitly prefers this over adding `built`'s `git2` feature, which pulls in `libgit2-sys` native bindings and adds cross-platform (Windows/macOS/Linux CI matrix) build risk for no benefit, since `built`'s own git2 code emits no `rerun-if-changed` either and would need the same manual fix anyway).

**Spec:** Design conclusion posted to Issue #378 as comment [5749481258](https://github.com/byx-darwin/gitflow-cli/issues/378#issuecomment-5749481258); requirement-quality review posted as comment 5749581084. Workflow contract: `.cache/workflows/active/wf-2026-09-20-005.json`.

## Global Constraints

- No public API breaking changes.
- No new runtime or build dependency (see Tech Stack rationale above).
- Do not modify `crates/github/src/pipeline.rs` success-rate calculation logic — already verified correct (filters non-terminal runs via `run.status == "completed"`) and already has full regression coverage: `test_should_exclude_runs_with_non_terminal_status_even_when_conclusion_is_present` and `test_should_not_attribute_failure_to_a_still_in_progress_job` (both currently passing — confirmed via `cargo test -p gitflow-github` during Phase 1 investigation). **Do not add a duplicate test for this** — Issue #378's AC item "新增回归测试" is already satisfied by pre-existing coverage; note this in the PR/Issue close-out instead of writing redundant tests.
- GitCode and GitLab providers are out of scope — untouched.
- `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic` must pass on `apps/cli`.

---

### Task 1: Build-time git short SHA capture

**Files:**
- Modify: `apps/cli/build.rs`
- Modify: `apps/cli/src/main.rs:22-43` (doc comment + new include module)
- Test: `apps/cli/tests/version_test.rs` (new file — integration test, since this behavior can only be observed via the built binary's `--version` output, not a unit test of build.rs itself)

**Interfaces:**
- Produces: `apps/cli` binary — running `gf --version` prints `gf <PKG_VERSION> (<sha>)` where `<sha>` is either a short git hash or the literal string `unknown`.
- Produces: `OUT_DIR/git_sha.rs` containing `pub const GIT_SHA: &str = "<value>";` (generated file, not checked into git — same pattern as the existing `OUT_DIR/skills_manifest.rs`).

- [ ] **Step 1: Write the failing integration test**

Create `apps/cli/tests/version_test.rs`:

```rust
//! Integration test for `gf --version` output format.

use std::process::Command;

#[test]
fn test_should_print_version_with_parenthesized_suffix() {
    let output = Command::new(env!("CARGO_BIN_EXE_gf"))
        .arg("--version")
        .output()
        .expect("failed to run gf --version");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Format is "gf <version> (<sha>)" — the sha is either a short git hash
    // (hex, typically 7-12 chars) or the literal "unknown" fallback.
    let trimmed = stdout.trim();
    assert!(
        trimmed.starts_with("gf "),
        "expected version output to start with 'gf ', got: {trimmed:?}"
    );
    assert!(
        trimmed.ends_with(')') && trimmed.contains('('),
        "expected version output to end with a parenthesized suffix, got: {trimmed:?}"
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p gitflow-cli --test version_test`
Expected: FAIL — current output is just `gf 1.9.0` with no parentheses.

- [ ] **Step 3: Add git SHA capture to `build.rs`**

In `apps/cli/build.rs`, add this function and call it from `main()` right after the existing `built::write_built_file()...` line:

```rust
/// Captures the short git commit hash of `HEAD` at compile time.
///
/// Falls back to `"unknown"` (never panics) when the build tree is not a
/// git checkout (e.g. a crates.io source tarball) or `git` is unavailable.
/// Also arranges for cargo to rerun this script whenever `HEAD` moves —
/// via `git rev-parse --git-path`, which resolves to the correct file for
/// both the main worktree and any linked worktree (e.g. the ones
/// `gf-workflow` creates under `.worktree/`) — so the embedded sha never
/// goes stale relative to the source it was actually built from.
fn write_git_sha(manifest_dir: &Path, out_dir: &Path) {
    let git_dir_relative_paths = ["HEAD", "logs/HEAD"];
    for rel in git_dir_relative_paths {
        if let Ok(output) = std::process::Command::new("git")
            .args(["rev-parse", "--git-path", rel])
            .current_dir(manifest_dir)
            .output()
        {
            if output.status.success() {
                if let Ok(path) = String::from_utf8(output.stdout) {
                    println!("cargo:rerun-if-changed={}", path.trim());
                }
            }
        }
    }

    let sha = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(manifest_dir)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_owned());

    let git_sha_path = out_dir.join("git_sha.rs");
    fs::write(
        &git_sha_path,
        format!("// @generated by build.rs - do not edit\npub const GIT_SHA: &str = \"{sha}\";\n"),
    )
    .expect("Failed to write git_sha.rs");
}
```

Then in `fn main()`, right after the `built::write_built_file()...` line, add:

```rust
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR env var"));
    write_git_sha(&manifest_dir, &out_dir);
```

Note: `build.rs` already declares `manifest_dir` and `out_dir`-equivalent locals later in `main()` for the skills manifest — reuse those existing bindings (move their declarations earlier) rather than redeclaring, to avoid clippy `unused variable`/shadowing warnings. Check the current variable names before editing (`manifest_dir` is already used at the point where skills are located; `out_dir` is declared inside the `if skills_dir.exists()` branch and in the `else` branch separately — hoist a single `out_dir` declaration to the top of `main()` and reuse it in both branches).

- [ ] **Step 4: Wire `GIT_SHA` into the version string in `main.rs`**

In `apps/cli/src/main.rs`, add a new module next to `built_info` (around line 43):

```rust
/// Git commit short SHA captured at build time by `build.rs`.
///
/// `"unknown"` when the build tree is not a git checkout (e.g. a published
/// crates.io source tarball).
mod build_git {
    include!(concat!(env!("OUT_DIR"), "/git_sha.rs"));
}
```

Update the doc comment on `built_info` (lines 22-27) to remove the incorrect claim that it exposes `GIT_COMMIT_HASH` — that constant does not exist because the `built` crate's `git2` feature is not enabled (verified: `target/*/build/gitflow-cli-*/out/built.rs` has no `GIT_COMMIT_HASH*` constants). Replace with:

```rust
/// Build-time metadata generated by the `built` crate.
///
/// Exposes `PKG_VERSION`, `TARGET`, and similar constants. Does **not**
/// expose git information — the `built` crate's `git2` feature is
/// intentionally not enabled (see `build_git` module for the lightweight
/// alternative this binary uses instead). Unused fields are expected
/// because the `built` crate generates a fixed set of metadata regardless
/// of what the binary actually reads.
```

Then change the version-building line (currently `.version(built_info::PKG_VERSION)`, around line 73) to:

```rust
    let version_string = format!("{} ({})", built_info::PKG_VERSION, build_git::GIT_SHA);

    // Build version string: pkg version + git sha
    let matches = match Cli::command()
        .version(version_string)
        .try_get_matches()
    {
        Ok(m) => m,
        Err(e) => e.exit(),
    };
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cargo test -p gitflow-cli --test version_test`
Expected: PASS

- [ ] **Step 6: Manual sanity check**

Run: `cargo run -p gitflow-cli -- --version`
Expected output: `gf 1.9.0 (<7-char-hex>)` where `<7-char-hex>` matches `git rev-parse --short HEAD` run in the same shell.

- [ ] **Step 7: Run full crate test suite + clippy**

Run: `cargo test -p gitflow-cli && cargo clippy -p gitflow-cli --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: all pass, no new warnings.

- [ ] **Step 8: Commit**

```bash
git add apps/cli/build.rs apps/cli/src/main.rs apps/cli/tests/version_test.rs
git commit -m "feat(cli): embed git short SHA in gf --version for build traceability"
```

---

### Task 2: Document the freshness check in `gf-pipeline-analyzer`

**Files:**
- Modify: `skills/gf-pipeline-analyzer/SKILL.md`

**Interfaces:**
- Consumes: Task 1's `gf --version` output format (`gf <version> (<sha>)`).

- [ ] **Step 1: Add a note under `## Preconditions`**

In `skills/gf-pipeline-analyzer/SKILL.md`, immediately after the existing `## Preconditions` block (`gf` installed / `gf` authenticated), add:

```markdown
- `gf --version` 的 commit sha 与仓库当前 `git rev-parse --short HEAD` 一致
  （尤其在 gf-workflow 按 Issue 创建的独立 worktree 中：全局安装的 `gf` 二进制
  只有显式 `cargo install --path apps/cli` 后才会更新，落后的二进制会让已经
  修复的问题看起来仍在复现——见 Issue #378）
```

- [ ] **Step 2: Verify word count is not pushed further out of budget**

Run (per `docs/superpowers/templates/skill-conventions.md`'s intent, using the corrected counting form since the documented one is known-broken per Issue #350):
`perl -0 -ne 's/^---\n.*?^---\n//ms; s/```.*?```//gs; s/`[^`]+`//g; print scalar(()=/\p{L}+/g), "\n"' skills/gf-pipeline-analyzer/SKILL.md`

This is informational only (Issue #350 owns fixing/enforcing the limit) — do not block this task on the pre-existing over-limit state of other skills.

- [ ] **Step 3: Commit**

```bash
git add skills/gf-pipeline-analyzer/SKILL.md
git commit -m "docs(gf-pipeline-analyzer): note gf --version freshness check before snapshotting"
```

---

### Task 3: Real-world in-progress verification (manual, required by Issue #378 AC — not satisfiable by unit tests alone)

**Files:** none (evidence-gathering step, output goes into the Issue/PR, not the repo)

- [ ] **Step 1:** After Task 1 and Task 2 are pushed and this branch's own CI run has started, while at least one job is still `in_progress`, run:
  ```bash
  gf --version
  gf pipeline report --branch <this-branch> --days 1
  ```
- [ ] **Step 2:** Confirm `gf --version`'s sha matches `git rev-parse --short HEAD` on this branch.
- [ ] **Step 3:** Confirm the `pipeline report` snapshot does **not** show a depressed success rate or a still-running job's name in `top_failures` (this exercises the pre-existing, already-passing logic end-to-end against live data, not mocks).
- [ ] **Step 4:** Paste both command outputs into a comment on Issue #378 as closing evidence, then close via the PR's `Closes #378` (per delivery step) — do not close manually ahead of merge.

## Self-Review Notes

- **Spec coverage:** AC item 1 (why the fix looked ineffective) — answered by the Phase 1 design comment, no code task needed. AC item 2 (exclude non-terminal from denominator) — already true in source, verified, no task needed. AC item 3 (regression test) — already exists and passes, explicitly called out in Global Constraints to prevent a duplicate test being added during execution. AC item 4 (real in-progress verification) — Task 3.
- **New scope beyond the original AC** (user-approved): git-sha traceability (Task 1) and the skill doc note (Task 2) — these are the actual fix for why #378 was hard to diagnose in the first place.
- **No placeholders**: all code blocks are complete and copy-pasteable; the one open detail (exact existing variable names to reuse in `build.rs`) is flagged as "check before editing" rather than guessed, since Task 1 Step 3 already shows the reader exactly what to grep for.
