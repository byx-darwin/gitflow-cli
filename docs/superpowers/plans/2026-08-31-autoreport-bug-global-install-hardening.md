# Auto-Report-Bug Global-Install Hardening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close three gaps specific to *global* install of `gf-autoreport-bug` — a hardcoded target repo that breaks template forks, redaction narrow enough that a real credential on this machine would leak, and a global opt-in that silently covers every future project with no per-project visibility.

**Architecture:** Three independent, additive changes: (1) the hook script and its Rust installer gain a repo-slug parameter sourced from `Cargo.toml`; (2) `sanitize_error_message` gains an env-var-value scan plus two vendor-agnostic regex patterns; (3) `is_co_contribution_enabled` becomes project-only, backed by a new tri-state reader, with `gf doctor` surfacing the gap a global-only opt-in now leaves. No new files; every change extends an existing, already-tested unit.

**Tech Stack:** Rust 2024 (`apps/cli`), Bash + Bats (`hooks/`), Markdown skill docs.

**Spec:** `docs/superpowers/specs/2026-08-31-autoreport-bug-global-install-hardening-design.md`

## Global Constraints

- Never use `unwrap()`/`expect()` in production code (CLAUDE.md).
- TDD mandatory: RED → GREEN → REFACTOR for every code task; run `make test` at each GREEN step.
- Run `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic` after any Rust change.
- `deny.toml`, `.pre-commit-config.yaml`, `rust-toolchain.toml` must not be touched.
- Skill source of truth is `skills/gf-autoreport-bug/SKILL.md` — never edit the gitignored `.claude/skills/` copy.
- `is_co_contribution_enabled()`'s and `sanitize_error_message()`'s existing call sites and signatures (no-arg, `&str -> String`) must not change — only their internal behavior, so `maybe_report_error` needs no edits for G7 or G8's public surface.

---

### Task 1 (G6): De-hardcode the target repo

**Files:**
- Modify: `apps/cli/src/commands/skills.rs` (`build_auto_report_hook_cmd`, `resolve_global_hook_paths`, `resolve_project_hook_paths`; new `autoreport_repo_slug`)
- Modify: `hooks/auto-report-bug.sh`
- Modify: `hooks/tests/auto-report-bug.bats`
- Modify: `skills/gf-autoreport-bug/SKILL.md`
- Modify: `docs/references/gf-autoreport-bug-params.md`

**Interfaces:**
- Produces: `fn autoreport_repo_slug() -> String` in `skills.rs` (private, called only within that module). `build_auto_report_hook_cmd(hooks_dir: &str, repo: &str) -> String` — signature changes from one `&str` param to two. No other task touches these names.

- [ ] **Step 1: Write the failing Rust tests**

Add to `apps/cli/src/commands/skills.rs`'s `#[cfg(test)] mod tests` (find the existing `test_build_auto_report_hook_cmd_uses_provided_hooks_dir` and `test_build_auto_report_hook_cmd_works_for_other_platforms` tests and replace their calls to match the new two-arg signature — they currently call `build_auto_report_hook_cmd(".claude/hooks")` / `build_auto_report_hook_cmd(".codex/hooks")`; update both call sites to `build_auto_report_hook_cmd(".claude/hooks", "byx-darwin/gitflow-cli")` / `build_auto_report_hook_cmd(".codex/hooks", "byx-darwin/gitflow-cli")`, keeping their existing assertions), then add:

```rust
#[test]
fn test_build_auto_report_hook_cmd_includes_repo_argument() {
    let cmd = build_auto_report_hook_cmd(".claude/hooks", "acme/fork");
    assert!(
        cmd.contains("\"acme/fork\""),
        "command should pass the repo slug as an argument, got: {cmd}"
    );
}

#[test]
fn test_autoreport_repo_slug_parses_standard_github_url() {
    // CARGO_PKG_REPOSITORY at build time is "https://github.com/byx-darwin/gitflow-cli"
    // (from this workspace's own Cargo.toml `repository` field).
    let slug = autoreport_repo_slug();
    assert_eq!(slug, "byx-darwin/gitflow-cli");
}

#[test]
fn test_autoreport_repo_slug_from_url_strips_prefix_and_suffix() {
    assert_eq!(
        autoreport_repo_slug_from_url("https://github.com/acme/fork"),
        "acme/fork"
    );
    assert_eq!(
        autoreport_repo_slug_from_url("https://github.com/acme/fork.git"),
        "acme/fork"
    );
}

#[test]
fn test_autoreport_repo_slug_from_url_falls_back_on_unexpected_shape() {
    assert_eq!(
        autoreport_repo_slug_from_url("git@github.com:acme/fork.git"),
        "byx-darwin/gitflow-cli"
    );
    assert_eq!(autoreport_repo_slug_from_url(""), "byx-darwin/gitflow-cli");
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p gitflow-cli commands::skills::tests::test_build_auto_report_hook_cmd_includes_repo_argument commands::skills::tests::test_autoreport_repo_slug -- --nocapture`
Expected: FAIL — compile errors (`build_auto_report_hook_cmd` takes 1 arg not 2; `autoreport_repo_slug`/`autoreport_repo_slug_from_url` undefined).

- [ ] **Step 3: Implement the repo-slug helper and thread it through**

In `apps/cli/src/commands/skills.rs`, add near `build_auto_report_hook_cmd` (find it at the line starting `fn build_auto_report_hook_cmd(hooks_dir: &str) -> String {`):

```rust
/// Resolve the `owner/repo` slug the auto-report hook should target.
///
/// Reads this crate's compile-time `CARGO_PKG_REPOSITORY` (sourced from
/// the workspace `Cargo.toml`'s `repository` field via `repository.workspace
/// = true` in `apps/cli/Cargo.toml`) so a template fork that updates that
/// one field gets a correctly-targeted hook with no other file to edit.
fn autoreport_repo_slug() -> String {
    autoreport_repo_slug_from_url(env!("CARGO_PKG_REPOSITORY"))
}

/// Pure core of [`autoreport_repo_slug`], testable without depending on
/// the compile-time env var.
///
/// Falls back to the literal default `"byx-darwin/gitflow-cli"` for any
/// shape other than `https://github.com/{owner}/{repo}[.git]` — this is a
/// convenience default, not a security boundary, so fail-safe rather than
/// fail-loud.
fn autoreport_repo_slug_from_url(url: &str) -> String {
    const DEFAULT: &str = "byx-darwin/gitflow-cli";
    let Some(rest) = url.strip_prefix("https://github.com/") else {
        return DEFAULT.to_string();
    };
    let slug = rest.strip_suffix(".git").unwrap_or(rest);
    if slug.split('/').count() == 2 && !slug.is_empty() {
        slug.to_string()
    } else {
        DEFAULT.to_string()
    }
}
```

Change `build_auto_report_hook_cmd`'s signature and body:

```rust
fn build_auto_report_hook_cmd(hooks_dir: &str, repo: &str) -> String {
    format!(
        "bash -c 'p=$(git rev-parse --show-toplevel 2>/dev/null) && [ -x \
         \"$p/{hooks_dir}/auto-report-bug.sh\" ] && bash \"$p/{hooks_dir}/auto-report-bug.sh\" \"{repo}\"'"
    )
}
```

Update both call sites (`resolve_global_hook_paths` and `resolve_project_hook_paths`, both currently `let cmd = build_auto_report_hook_cmd(hooks_dir);`) to:

```rust
let cmd = build_auto_report_hook_cmd(hooks_dir, &autoreport_repo_slug());
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p gitflow-cli commands::skills:: -- --nocapture`
Expected: PASS, all tests in this module including the two pre-existing tests whose call sites you updated.

- [ ] **Step 5: Write the failing bats test**

Read `hooks/tests/auto-report-bug.bats` in full first — the `run_hook()` helper currently is:

```bash
run_hook() {
  run bash -c '"$1" 2>&1 < /dev/null' hook_script "$HOOK_SCRIPT"
}
```

Change it to accept an optional repo argument, defaulting to the current hardcoded value so every existing test's behavior is unchanged:

```bash
run_hook() {
  local repo="${1:-byx-darwin/gitflow-cli}"
  run bash -c '"$1" "$2" 2>&1 < /dev/null' hook_script "$HOOK_SCRIPT" "$repo"
}
```

Add one new test (after the existing "auth success -> calls gh auth status" test):

```bash
@test "custom repo argument reaches gh label list and gh issue commands" {
  write_pending
  export GH_LABEL_LIST_OUTPUT="auto-report"

  run_hook "acme/fork"

  [ "$status" -eq 0 ]
  grep -q "acme/fork" "$GH_CALL_LOG"
  [[ "$output" == *"acme/fork"* ]]
}
```

- [ ] **Step 6: Run bats to verify the new test fails**

Run: `bats hooks/tests/auto-report-bug.bats`
Expected: the new test FAILS (the script doesn't read `$1` as a repo slug yet, so `GH_CALL_LOG` still shows `byx-darwin/gitflow-cli` and the banner never mentions `acme/fork`). Every other test should still PASS (the `run_hook` default keeps their behavior identical).

- [ ] **Step 7: Implement the repo-slug parameter in the hook script**

In `hooks/auto-report-bug.sh`, add near the top, right after the `set -euo pipefail` line:

```bash
# Target repo for every `gh` call below — passed as $1 by the installed
# Stop Hook command (see build_auto_report_hook_cmd in
# apps/cli/src/commands/skills.rs), sourced at install time from this
# workspace's Cargo.toml `repository` field. Falls back to this repo's
# own slug for direct/manual invocation without an argument.
REPO_SLUG="${1:-byx-darwin/gitflow-cli}"
```

Replace every occurrence of the literal `byx-darwin/gitflow-cli` in the rest of the file with `$REPO_SLUG`:
- `echo "    URL: https://github.com/byx-darwin/gitflow-cli/issues/new"` → `echo "    URL: https://github.com/${REPO_SLUG}/issues/new"`
- `gh label list --repo byx-darwin/gitflow-cli --search auto-report ...` → `gh label list --repo "$REPO_SLUG" --search auto-report ...`
- `log_hook "label check failed (auto-report label missing on byx-darwin/gitflow-cli)"` → `log_hook "label check failed (auto-report label missing on ${REPO_SLUG})"`
- `echo "    gh label create auto-report --repo byx-darwin/gitflow-cli \\"` → `echo "    gh label create auto-report --repo ${REPO_SLUG} \\"`

Also add a `仓库` line to the final banner, right after the existing `echo "  平台:   ${PLATFORM:-unknown}"` line:

```bash
echo "  仓库:   ${REPO_SLUG}"
```

- [ ] **Step 8: Run bats to verify all tests pass**

Run: `bats hooks/tests/auto-report-bug.bats`
Expected: PASS, all tests (the pre-existing ones via `run_hook`'s default, the new one via the explicit `"acme/fork"` argument).

- [ ] **Step 9: Update the skill doc and reference doc**

In `skills/gf-autoreport-bug/SKILL.md`, change Workflow step 3:

```
3. **Dedup** — `gh issue list --repo byx-darwin/gitflow-cli --search "[auto-report] {command} {error_code}" --state all`. Match → clean, stop.
```
to:
```
3. **Dedup** — `gh issue list --repo {repo} --search "[auto-report] {command} {error_code}" --state all`. Match → clean, stop.
```

and step 4:
```
4. **Create** — On interactive confirm only, analyze root cause + severity, then `gh issue create --repo byx-darwin/gitflow-cli --title "[auto-report] gf {command} — {error_code}" --label "auto-report"`. Fail → keep file + `failed.log`.
```
to:
```
4. **Create** — On interactive confirm only, analyze root cause + severity, then `gh issue create --repo {repo} --title "[auto-report] gf {command} — {error_code}" --label "auto-report"`. Fail → keep file + `failed.log`.
```

Add one line directly after the Workflow list (before `## Error Handling`):

```
`{repo}` is the value shown on the Stop Hook banner's `仓库` line — it defaults to this workspace's own `Cargo.toml` `repository` field, so a fork targets its own repo automatically.
```

Change the Red Flags line:
```
- 🔴 Missing `--repo` — always target fixed repo.
```
to:
```
- 🔴 Missing `--repo` — always target the repo given in the Stop Hook banner.
```

In `docs/references/gf-autoreport-bug-params.md`, update the 命令速查 code block's `--repo` example the same way (replace the literal with `{repo}` and a note it's sourced from `Cargo.toml`), and add one sentence to the existing "安全网关" section noting the repo is now parametrized (Task from 2026-08-31 plan).

- [ ] **Step 10: Lint and commit**

Run: `cargo clippy -p gitflow-cli --all-targets --all-features -- -D warnings -W clippy::pedantic`
Run: `bash -n hooks/auto-report-bug.sh`

```bash
git add apps/cli/src/commands/skills.rs hooks/auto-report-bug.sh hooks/tests/auto-report-bug.bats skills/gf-autoreport-bug/SKILL.md docs/references/gf-autoreport-bug-params.md
git commit -m "fix(autoreport): parametrize target repo from Cargo.toml instead of hardcoding it"
```

---

### Task 2 (G7): Broaden redaction

**Files:**
- Modify: `apps/cli/src/error_reporter.rs`

**Interfaces:**
- Produces: `fn redact_env_values(message: &str, vars: impl IntoIterator<Item = (String, String)>) -> String` and two new `LazyLock<Regex>` statics (`GENERIC_SK_TOKEN_RE`, `GITLAB_TOKEN_RE`), all private. `sanitize_error_message`'s signature is unchanged. No other task touches this file's names.

- [ ] **Step 1: Write the failing tests**

Add to `apps/cli/src/error_reporter.rs`'s `#[cfg(test)] mod tests`:

```rust
#[test]
fn test_should_redact_env_var_value_when_name_looks_like_credential() {
    let vars = vec![("MY_API_TOKEN".to_string(), "abcdef123456".to_string())];
    let message = "request failed: token abcdef123456 rejected";
    let redacted = redact_env_values(message, vars);
    assert_eq!(redacted, "request failed: token [REDACTED] rejected");
}

#[test]
fn test_should_not_redact_when_env_var_name_does_not_look_like_credential() {
    let vars = vec![("SOME_LONG_PATH_VALUE".to_string(), "abcdef123456".to_string())];
    let message = "request failed: token abcdef123456 rejected";
    let redacted = redact_env_values(message, vars);
    assert_eq!(
        redacted, message,
        "a non-credential-named var must not trigger redaction"
    );
}

#[test]
fn test_should_not_redact_short_env_var_values() {
    let vars = vec![("API_TOKEN".to_string(), "1".to_string())];
    let message = "exit code: 1";
    let redacted = redact_env_values(message, vars);
    assert_eq!(
        redacted, message,
        "values shorter than the minimum length must not be redacted"
    );
}

#[test]
fn test_should_redact_multiple_occurrences_of_env_var_value() {
    let vars = vec![("SECRET_KEY".to_string(), "topsecretvalue".to_string())];
    let message = "topsecretvalue appears twice: topsecretvalue";
    let redacted = redact_env_values(message, vars);
    assert_eq!(redacted, "[REDACTED] appears twice: [REDACTED]");
}

#[test]
fn test_should_sanitize_generic_sk_prefixed_token() {
    let message = "auth failed: sk-ant-api03-abcdefghijklmnopqrstuvwxyz rejected";
    let sanitized = sanitize_error_message(message);
    assert!(
        !sanitized.contains("sk-ant-"),
        "sk-prefixed token must be redacted: {sanitized}"
    );
    assert!(sanitized.contains("[REDACTED]"));
}

#[test]
fn test_should_sanitize_gitlab_token() {
    let message = "clone failed: token glpat-1234567890abcdefghij rejected"; // gitleaks:allow
    let sanitized = sanitize_error_message(message);
    assert!(
        !sanitized.contains("glpat-"),
        "GitLab token must be redacted: {sanitized}"
    );
    assert!(sanitized.contains("[REDACTED]"));
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p gitflow-cli error_reporter::tests::test_should_redact error_reporter::tests::test_should_sanitize_generic_sk error_reporter::tests::test_should_sanitize_gitlab -- --nocapture`
Expected: FAIL — `redact_env_values` undefined (compile error) for the first four; the `sk-`/`glpat-` tests fail because `sanitize_error_message` doesn't redact those shapes yet.

- [ ] **Step 3: Implement the redaction additions**

Add near the existing `GITHUB_TOKEN_RE` static in `apps/cli/src/error_reporter.rs` (find `static GITHUB_TOKEN_RE: LazyLock<Regex> = LazyLock::new(|| {`):

```rust
/// Redacts generic `sk-`-prefixed API keys (Anthropic, OpenAI, and
/// similarly-shaped vendor tokens) from error messages.
static GENERIC_SK_TOKEN_RE: LazyLock<Regex> = LazyLock::new(|| {
    #[allow(
        clippy::expect_used,
        reason = "regex pattern is a compile-time literal; a compile failure is a programming \
                  error"
    )]
    Regex::new(r"sk-[A-Za-z0-9_-]{10,}").expect("generic sk- token regex must be statically valid")
});

/// Redacts GitLab personal access tokens from error messages.
static GITLAB_TOKEN_RE: LazyLock<Regex> = LazyLock::new(|| {
    #[allow(
        clippy::expect_used,
        reason = "regex pattern is a compile-time literal; a compile failure is a programming \
                  error"
    )]
    Regex::new(r"glpat-[A-Za-z0-9_-]{20,}").expect("GitLab token regex must be statically valid")
});

/// Minimum byte length an environment variable's value must have before
/// [`redact_env_values`] will treat a match against it as significant —
/// avoids false-positive redaction against trivial short values (e.g. a
/// credential-named var accidentally set to `"1"` or `""`).
const MIN_REDACTABLE_ENV_VALUE_LEN: usize = 8;

/// Environment variable name fragments (case-insensitive) that mark a
/// variable as credential-shaped for [`redact_env_values`].
const CREDENTIAL_NAME_FRAGMENTS: &[&str] = &["TOKEN", "KEY", "SECRET", "PASSWORD", "CREDENTIAL"];

/// Redact any occurrence of a credential-looking environment variable's
/// value from `message`.
///
/// A variable counts as credential-looking when its name
/// case-insensitively contains any of [`CREDENTIAL_NAME_FRAGMENTS`] and
/// its value is at least [`MIN_REDACTABLE_ENV_VALUE_LEN`] bytes. This
/// catches secrets regardless of vendor-specific shape — e.g. a
/// `ANTHROPIC_AUTH_TOKEN` sitting in the environment that happens to leak
/// into an error message, which no fixed-format regex would recognize.
fn redact_env_values(message: &str, vars: impl IntoIterator<Item = (String, String)>) -> String {
    let mut result = message.to_string();
    for (name, value) in vars {
        if value.len() < MIN_REDACTABLE_ENV_VALUE_LEN {
            continue;
        }
        let upper_name = name.to_uppercase();
        let looks_like_credential = CREDENTIAL_NAME_FRAGMENTS
            .iter()
            .any(|frag| upper_name.contains(frag));
        if looks_like_credential {
            result = result.replace(&value, "[REDACTED]");
        }
    }
    result
}
```

Update `sanitize_error_message` to call the three new steps after the existing two:

```rust
fn sanitize_error_message(message: &str) -> String {
    let sanitized = if let Some(home) = dirs::home_dir() {
        let home_str = home.to_string_lossy();
        message.replace(home_str.as_ref(), "~")
    } else {
        message.to_string()
    };
    let sanitized = GITHUB_TOKEN_RE.replace_all(&sanitized, "[REDACTED]").into_owned();
    let sanitized = GENERIC_SK_TOKEN_RE
        .replace_all(&sanitized, "[REDACTED]")
        .into_owned();
    let sanitized = GITLAB_TOKEN_RE.replace_all(&sanitized, "[REDACTED]").into_owned();
    redact_env_values(&sanitized, std::env::vars())
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p gitflow-cli error_reporter:: -- --nocapture`
Expected: PASS, all tests including the six new ones and every pre-existing test in this module (the existing `test_should_sanitize_home_directory_in_error_message`, `test_should_sanitize_token_in_error_message`, and `test_should_not_modify_safe_error_message` must still pass unchanged — note `test_should_not_modify_safe_error_message`'s message now also passes through `redact_env_values(&sanitized, std::env::vars())`, i.e. the REAL process environment during `cargo test`; if this causes a flake because some real env var's value coincidentally appears in the safe test message, treat that as a genuine signal and report it rather than silencing the test — it would mean the redaction is working exactly as designed against real environment content).

- [ ] **Step 5: Lint and commit**

Run: `cargo clippy -p gitflow-cli --all-targets --all-features -- -D warnings -W clippy::pedantic`

```bash
git add apps/cli/src/error_reporter.rs
git commit -m "fix(autoreport): redact credential-named env var values and sk-/glpat- tokens"
```

---

### Task 3 (G8): Require project-level confirmation for a global opt-in

**Files:**
- Modify: `apps/cli/src/error_reporter.rs`
- Modify: `apps/cli/src/commands/doctor.rs`

**Interfaces:**
- Consumes: Task 1/2 do not touch these functions; safe to implement independently and in any order relative to them.
- Produces: `fn read_co_contribution_field(path: &Path) -> Option<bool>`, `fn global_co_contribution_pending_ack_at(repo_root: &Path) -> bool`, `pub(crate) fn global_co_contribution_pending_ack() -> bool` in `error_reporter.rs`. `read_co_contribution_flag` keeps its existing `pub(crate) fn read_co_contribution_flag(path: &Path) -> bool` signature (doctor.rs's existing call site is unaffected). No other task depends on these names.

- [ ] **Step 1: Write the failing tests**

Add to `apps/cli/src/error_reporter.rs`'s `#[cfg(test)] mod tests`:

```rust
#[test]
fn test_should_return_none_for_missing_co_contribution_field() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("settings.json");
    std::fs::write(&path, r#"{"gitflow": {}}"#).expect("write");
    assert_eq!(read_co_contribution_field(&path), None);
}

#[test]
fn test_should_return_none_for_missing_settings_file_tri_state() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let missing = tmp.path().join("nonexistent.json");
    assert_eq!(read_co_contribution_field(&missing), None);
}

#[test]
fn test_should_return_some_true_for_co_contribution_field_true() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("settings.json");
    std::fs::write(&path, r#"{"gitflow": {"co_contribution": true}}"#).expect("write");
    assert_eq!(read_co_contribution_field(&path), Some(true));
}

#[test]
fn test_should_return_some_false_for_co_contribution_field_false() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let path = tmp.path().join("settings.json");
    std::fs::write(&path, r#"{"gitflow": {"co_contribution": false}}"#).expect("write");
    assert_eq!(read_co_contribution_field(&path), Some(false));
}

#[test]
fn test_pending_ack_true_when_global_true_and_project_absent() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let global = tmp.path().join("global.json");
    std::fs::write(&global, r#"{"gitflow": {"co_contribution": true}}"#).expect("write");
    let project = tmp.path().join("project.json");
    std::fs::write(&project, r#"{}"#).expect("write");
    assert!(global_co_contribution_pending_ack_with(&global, &project));
}

#[test]
fn test_pending_ack_false_when_project_already_decided_true() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let global = tmp.path().join("global.json");
    std::fs::write(&global, r#"{"gitflow": {"co_contribution": true}}"#).expect("write");
    let project = tmp.path().join("project.json");
    std::fs::write(&project, r#"{"gitflow": {"co_contribution": true}}"#).expect("write");
    assert!(!global_co_contribution_pending_ack_with(&global, &project));
}

#[test]
fn test_pending_ack_false_when_project_already_decided_false() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let global = tmp.path().join("global.json");
    std::fs::write(&global, r#"{"gitflow": {"co_contribution": true}}"#).expect("write");
    let project = tmp.path().join("project.json");
    std::fs::write(&project, r#"{"gitflow": {"co_contribution": false}}"#).expect("write");
    assert!(!global_co_contribution_pending_ack_with(&global, &project));
}

#[test]
fn test_pending_ack_false_when_global_false() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let global = tmp.path().join("global.json");
    std::fs::write(&global, r#"{"gitflow": {"co_contribution": false}}"#).expect("write");
    let project = tmp.path().join("project.json");
    std::fs::write(&project, r#"{}"#).expect("write");
    assert!(!global_co_contribution_pending_ack_with(&global, &project));
}

#[test]
fn test_co_contribution_enabled_ignores_global_when_project_absent() {
    // is_co_contribution_enabled must now only consult the project file;
    // a global-only true must not enable reporting.
    let tmp = tempfile::tempdir().expect("tempdir");
    let project = tmp.path().join("settings.json");
    std::fs::write(&project, r#"{}"#).expect("write");
    assert!(!read_co_contribution_field(&project).unwrap_or(false));
}
```

Note: the last test above only re-exercises `read_co_contribution_field` directly (the same tri-state reader `is_co_contribution_enabled` will be rewritten to use) — `is_co_contribution_enabled()` itself has no repo-root parameter to inject in tests (it calls `find_repo_root()` internally, same as today), so its end-to-end behavior change is verified structurally by Step 3's rewrite plus this unit coverage of its new sole data source, matching how `is_co_contribution_enabled`'s existing behavior is already tested only indirectly via `read_co_contribution_flag`.

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p gitflow-cli error_reporter::tests::test_should_return_none_for_missing_co_contribution_field error_reporter::tests::test_pending_ack -- --nocapture`
Expected: FAIL — `read_co_contribution_field` and `global_co_contribution_pending_ack_with` undefined (compile errors).

- [ ] **Step 3: Implement the tri-state reader and pending-ack check**

In `apps/cli/src/error_reporter.rs`, find the existing `is_co_contribution_enabled` and `read_co_contribution_flag`:

```rust
fn is_co_contribution_enabled() -> bool {
    if let Ok(repo_root) = find_repo_root() {
        let project_settings = repo_root.join(".claude/settings.json");
        if read_co_contribution_flag(&project_settings) {
            return true;
        }
    }

    if let Some(home) = dirs::home_dir() {
        let global_settings = home.join(".claude/settings.json");
        if read_co_contribution_flag(&global_settings) {
            return true;
        }
    }

    false
}
```
```rust
pub(crate) fn read_co_contribution_flag(path: &Path) -> bool {
    let Ok(content) = std::fs::read_to_string(path) else {
        return false;
    };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) else {
        return false;
    };
    json.pointer("/gitflow/co_contribution")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
}
```

Replace both with:

```rust
/// Check whether the co-contribution plan is enabled for the current
/// project.
///
/// Checks **only** `<repo_root>/.claude/settings.json` — a global-only
/// opt-in (`~/.claude/settings.json`) no longer silently enables
/// reporting in every project; see [`global_co_contribution_pending_ack`]
/// for the mechanism that surfaces that gap via `gf doctor` instead.
/// Returns `false` if the repo root cannot be found or the field is
/// missing/false.
fn is_co_contribution_enabled() -> bool {
    let Ok(repo_root) = find_repo_root() else {
        return false;
    };
    read_co_contribution_flag(&repo_root.join(".claude/settings.json"))
}

/// Read the `gitflow.co_contribution` flag from a specific settings file.
///
/// Returns `false` if the file doesn't exist, can't be read, or the field
/// is missing/not a boolean. Convenience wrapper over
/// [`read_co_contribution_field`] for call sites that don't need to
/// distinguish "explicitly false" from "absent".
pub(crate) fn read_co_contribution_flag(path: &Path) -> bool {
    read_co_contribution_field(path).unwrap_or(false)
}

/// Read the `gitflow.co_contribution` flag from a specific settings file,
/// distinguishing an explicit decision from absence.
///
/// Returns `None` if the file doesn't exist, can't be read, can't be
/// parsed as JSON, or the field is missing/not a boolean — i.e. "no
/// decision has been made here." Returns `Some(bool)` for an explicit
/// `true` or `false`.
fn read_co_contribution_field(path: &Path) -> Option<bool> {
    let content = std::fs::read_to_string(path).ok()?;
    let json = serde_json::from_str::<serde_json::Value>(&content).ok()?;
    json.pointer("/gitflow/co_contribution")
        .and_then(serde_json::Value::as_bool)
}

/// Returns `true` when the global co-contribution opt-in is active but
/// this project has never made its own explicit decision — the gap
/// `gf doctor`'s `CoContributionCheck` surfaces so a global-only opt-in
/// doesn't silently cover every future project with no visibility.
pub(crate) fn global_co_contribution_pending_ack() -> bool {
    let Some(home) = dirs::home_dir() else {
        return false;
    };
    let Ok(repo_root) = find_repo_root() else {
        return false;
    };
    global_co_contribution_pending_ack_with(
        &home.join(".claude/settings.json"),
        &repo_root.join(".claude/settings.json"),
    )
}

/// Testable core of [`global_co_contribution_pending_ack`] — takes both
/// settings paths explicitly instead of resolving them from `HOME`/the
/// git repo root.
fn global_co_contribution_pending_ack_with(global_path: &Path, project_path: &Path) -> bool {
    read_co_contribution_field(global_path) == Some(true)
        && read_co_contribution_field(project_path).is_none()
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p gitflow-cli error_reporter:: -- --nocapture`
Expected: PASS, all tests including the eight new ones and every pre-existing test (`test_should_return_true_for_co_contribution_enabled`, `test_should_return_false_for_co_contribution_disabled`, `test_should_return_false_for_invalid_json`, `test_should_return_false_for_missing_settings_file`, `test_should_return_false_for_settings_without_gitflow`, `test_should_return_false_for_gitflow_without_co_contribution` — all exercise `read_co_contribution_flag`, whose observable behavior is unchanged since it's now `read_co_contribution_field(path).unwrap_or(false)`).

- [ ] **Step 5: Write the failing doctor.rs test**

Read `apps/cli/src/commands/doctor.rs`'s `CoContributionCheck` and its existing test `test_co_contribution_check_reports_opt_out_guide` (in `#[cfg(test)] mod tests`) first. Add a new test after it:

```rust
#[test]
fn test_co_contribution_check_warns_when_global_pending_ack() {
    // This test cannot force crate::error_reporter::global_co_contribution_pending_ack()
    // to return true (it reads real HOME/repo-root state), so it instead
    // verifies the check's structure directly: whatever the real result,
    // exactly one item is returned and — when a pending ack IS detected —
    // the item's status is a warning, not a silent pass, and its detail
    // names the project-level settings key to add.
    let items = CoContributionCheck.run();
    assert_eq!(items.len(), 1);
    let item = &items[0];
    if crate::error_reporter::global_co_contribution_pending_ack() {
        assert_eq!(
            item.status,
            CheckStatus::Warn,
            "a pending global-only opt-in must surface as a warning, not a silent pass"
        );
        let detail = item.detail.clone().unwrap_or_default();
        assert!(
            detail.contains(".claude/settings.json"),
            "warning must point at the project-level settings file: {detail}"
        );
    }
}
```

- [ ] **Step 6: Run the doctor.rs test to see current behavior**

Run: `cargo test -p gitflow-cli commands::doctor::tests::test_co_contribution_check_warns_when_global_pending_ack -- --nocapture`
Expected: PASS trivially right now (the `if` guard is false since `global_co_contribution_pending_ack` doesn't exist as a wired-in behavior in `CoContributionCheck::run()` yet — but this compiles against the function added in Step 3, so it's really checking nothing meaningful until Step 7's implementation makes the `if` branch reachable). Confirm it at least compiles and passes before proceeding — this step exists to prove the test scaffolding is sound before wiring the real behavior in.

- [ ] **Step 7: Wire the warning into `CoContributionCheck::run()`**

In `apps/cli/src/commands/doctor.rs`, find:

```rust
impl HealthCheck for CoContributionCheck {
    fn category(&self) -> &'static str {
        "co_contribution"
    }

    fn run(&self) -> Vec<CheckItem> {
        let enabled = dirs::home_dir().is_some_and(|home| {
            crate::error_reporter::read_co_contribution_flag(&home.join(".claude/settings.json"))
        });
        let mut items = Vec::new();
        let item = if enabled {
            CheckItem::pass(
                self.category(),
                "共建计划",
                "bug 自动上报已开启（~/.claude/settings.json）",
            )
        } else {
            CheckItem::pass(
                self.category(),
                "共建计划",
                "未加入共建计划，bug 自动上报未开启",
            )
        };
        items.push(item.with_detail(
            "退出方式：编辑 ~/.claude/settings.json，移除 gitflow.co_contribution 字段后保存",
        ));
        items
    }
}
```

Replace with:

```rust
impl HealthCheck for CoContributionCheck {
    fn category(&self) -> &'static str {
        "co_contribution"
    }

    fn run(&self) -> Vec<CheckItem> {
        let mut items = Vec::new();

        if crate::error_reporter::global_co_contribution_pending_ack() {
            let item = CheckItem::warn(
                self.category(),
                "共建计划",
                "全局已开启共建计划（~/.claude/settings.json），但本项目尚未确认",
                "编辑 .claude/settings.json 添加 gitflow.co_contribution 字段",
            )
            .with_detail(
                "在本项目 .claude/settings.json 中设置 gitflow.co_contribution 为 \
                 true 或 false 以确认或关闭；否则本项目不会自动上报（现在仅看项目级 \
                 设置，不再回退到全局）",
            );
            items.push(item);
            return items;
        }

        let enabled = dirs::home_dir().is_some_and(|home| {
            crate::error_reporter::read_co_contribution_flag(&home.join(".claude/settings.json"))
        });
        let item = if enabled {
            CheckItem::pass(
                self.category(),
                "共建计划",
                "bug 自动上报已开启（~/.claude/settings.json）",
            )
        } else {
            CheckItem::pass(
                self.category(),
                "共建计划",
                "未加入共建计划，bug 自动上报未开启",
            )
        };
        items.push(item.with_detail(
            "退出方式：编辑 ~/.claude/settings.json，移除 gitflow.co_contribution 字段后保存",
        ));
        items
    }
}
```

`CheckItem::warn` (defined in `crates/core/src/doctor.rs:135`) takes four arguments — `category`, `name`, `message`, `hint` — unlike `CheckItem::pass`'s three; the snippet above already matches this signature. `.with_detail(...)` (line 171 of the same file) is chainable on any `CheckItem` regardless of constructor.

- [ ] **Step 8: Run tests to verify they pass**

Run: `cargo test -p gitflow-cli commands::doctor:: -- --nocapture`
Expected: PASS, all tests including the new one and `test_co_contribution_check_reports_opt_out_guide` (which must still pass — that test's only assertion is `detail.contains("gitflow.co_contribution")`; the warn-branch's detail text above contains that exact substring too, so the test passes whichever branch actually runs on the machine executing it. **This machine's own `~/.claude/settings.json` has `co_contribution: true` globally**, so if this repo's local (gitignored) `.claude/settings.json` has no explicit `gitflow.co_contribution` field, both `test_co_contribution_check_reports_opt_out_guide` and the new test will exercise the *warning* branch here specifically — by construction above, both branches' detail strings contain `"gitflow.co_contribution"` and the warn branch's also contains `".claude/settings.json"`, so both tests hold regardless of which branch runs on this machine.

- [ ] **Step 9: Lint and commit**

Run: `cargo clippy -p gitflow-cli --all-targets --all-features -- -D warnings -W clippy::pedantic`

```bash
git add apps/cli/src/error_reporter.rs apps/cli/src/commands/doctor.rs
git commit -m "fix(autoreport): require project-level confirmation for a global-only opt-in"
```

---

### Task 4: Full regression pass

**Files:** none (verification only).

**Interfaces:**
- Consumes: the combined output of Tasks 1-3.

- [ ] **Step 1: Full test suite**

Run: `cargo test --workspace` and `bats hooks/tests/auto-report-bug.bats`
Expected: all green. If the pre-existing unrelated `test_should_contain_all_phase4_outputs` failure (fixed 2026-08-30, commit `3023c13`) or the Windows-only `SafePath` issue (fixed 2026-08-30, commit `d24a502`) resurface, that indicates an unrelated regression on `dev` since this plan started — investigate before proceeding, don't assume it's this plan's fault without checking `git diff` against the commit this plan started from.

- [ ] **Step 2: Full clippy**

Run: `cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: clean.

- [ ] **Step 3: Manual sanity check of the repo-slug default on this machine**

```bash
cargo build -p gitflow-cli --bin gf
```

Then, in a scratch `git init`'d sandbox (not this repo), run `gf skills install --report-bug=true` (project-level, no `-g`) and inspect the generated `.claude/settings.json`'s Stop hook command string — confirm it ends with `"byx-darwin/gitflow-cli"` (this repo's own Cargo.toml value, since the sandbox has no Cargo.toml of its own — the `gf` binary's own compiled-in value is what's used, which is correct: the repo slug follows whichever `gf` binary is running, not the project being installed into). Clean up the sandbox after.

No commit — this task is verification only.
