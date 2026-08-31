# Auto-Report-Bug Global-Install Hardening — Design

## Context

While explaining the 2026-08-30 hardening plan's outcome, the user asked
whether `gf-autoreport-bug` has problems specifically under **global**
install (`gf skills install -g`, registering the Stop Hook in
`~/.claude/settings.json` rather than a single project's `.claude/settings.json`).
Investigation of this machine's own global config confirmed three
concrete gaps, independent of the 2026-08-30 plan:

1. **Hardcoded target repo.** `hooks/auto-report-bug.sh` and
   `skills/gf-autoreport-bug/SKILL.md` hardcode `byx-darwin/gitflow-cli`
   in every `gh` invocation (dedup search, label list/create, Issue
   create, login-guide URL — 4 places in the hook script alone). This
   repo is documented as "a reusable Rust 2024 workspace template"
   (`CLAUDE.md`). `gf skills install` embeds the hook script byte-for-byte
   via `include_bytes!` regardless of install target, so anyone who forks
   this template, updates `Cargo.toml`'s `repository` field (the natural,
   expected step when instantiating a template), and globally installs
   skills would still have every auto-reported bug filed against the
   *original* `byx-darwin/gitflow-cli`, not their fork.
2. **Narrow redaction, global blast radius.** `sanitize_error_message` in
   `apps/cli/src/error_reporter.rs` redacts only home-directory paths and
   GitHub token formats (`ghp_`/`github_pat_`). Under global install, `gf`
   runs across every project on the machine, so any project's error
   output can end up in a `pending.json` that later becomes a *public*
   GitHub Issue. Concretely demonstrated on this machine: `~/.claude/settings.json`
   contains a plaintext `ANTHROPIC_AUTH_TOKEN` (a `sk-ant-…`-shaped
   credential) that the current redaction would not catch if it ever
   surfaced in an error message.
3. **Global opt-in silently covers every future project.** `is_co_contribution_enabled()`
   checks the project's `.claude/settings.json` first, then falls back to
   the global `~/.claude/settings.json`. Confirmed on this machine:
   `gitflow.co_contribution: true` is set only in the *global* file. That
   means every new project the user ever runs `gf` in — including private
   or unrelated ones never explicitly told about this feature — is
   already opted in to auto-reporting, with no per-project visibility
   unless the user manually runs `gf doctor`.

Full evidence: this conversation's investigation of
`apps/cli/src/commands/skills.rs` (`build_auto_report_hook_cmd`,
`install_hook`), `hooks/auto-report-bug.sh`, `Cargo.toml`, and this
machine's own `~/.claude/settings.json`.

## Goals

- G6: The target repo for every `gh` call in the hook script follows the
  installing project's own `Cargo.toml` `repository` field, not a literal
  string — a template fork gets correct behavior automatically once they
  update that one field, with no other file to edit.
- G7: Redaction covers more than GitHub-token-shaped secrets: any process
  environment variable that *looks* like a credential (name matches
  `TOKEN|KEY|SECRET|PASSWORD|CREDENTIAL`, case-insensitive) and whose
  value appears verbatim in an error message is redacted, plus common
  vendor-agnostic key shapes (`sk-…`, GitLab `glpat-…`).
- G8: A global-only opt-in no longer silently enables reporting in a
  project that has never seen it — reporting requires an explicit
  project-level decision; `gf doctor` surfaces the gap instead of staying
  silent.

## Non-goals

- Building a general-purpose secret-scanning engine (entropy analysis,
  vendor-signature database). G7's env-var-value scan and two added
  regexes are a bounded, concrete response to the demonstrated gap, not
  a security product.
- Adding an interactive first-run consent prompt to the CLI's error path
  itself (`maybe_report_error` must stay non-blocking, best-effort). G8's
  mechanism is a `gf doctor` surfaced gap, not a blocking prompt.
- Changing how `pending.json` is written, archived, or pruned (covered by
  the 2026-08-30 plan already merged).

## Design

### G6 — De-hardcode target repo

`Cargo.toml` already declares `repository = "https://github.com/byx-darwin/gitflow-cli"`
at the workspace root — the natural single source of truth a template
fork updates. Add `fn autoreport_repo_slug() -> String` in
`apps/cli/src/commands/skills.rs`: read `env!("CARGO_PKG_REPOSITORY")`
(a compile-time constant Cargo already provides from that field), strip
a leading `https://github.com/` (and a trailing `.git` if present); on
any other shape, fall back to the literal default `"byx-darwin/gitflow-cli"`
(fail-safe, not fail-loud — this is a convenience default, not a security
boundary).

`build_auto_report_hook_cmd(hooks_dir: &str)` gains a `repo: &str`
parameter and appends it as a positional argument to the generated
command: `bash "$p/{hooks_dir}/auto-report-bug.sh" "{repo}"`. Both
`resolve_global_hook_paths` and `resolve_project_hook_paths` call
`autoreport_repo_slug()` and pass it through.

`hooks/auto-report-bug.sh` reads `REPO_SLUG="${1:-byx-darwin/gitflow-cli}"`
near the top (same fail-safe default, so a manually-copied or
directly-invoked script without an argument still works exactly as
today) and every hardcoded `byx-darwin/gitflow-cli` becomes `$REPO_SLUG`.
The final banner gains a `仓库: ${REPO_SLUG}` line, so `gf-autoreport-bug`
can read the resolved repo from the banner instead of a hardcoded
literal.

`skills/gf-autoreport-bug/SKILL.md`'s Workflow steps 3/4 change
`--repo byx-darwin/gitflow-cli` to `--repo {repo}`, with a one-line note
that `{repo}` comes from the Stop Hook banner. `docs/references/gf-autoreport-bug-params.md`
updates its 命令速查 examples the same way and documents the Cargo.toml
source of truth.

### G7 — Broader redaction

`sanitize_error_message` gains two additions, applied after the existing
home-path and GitHub-token steps:

1. **Env-var-value redaction.** A new testable core,
   `fn redact_env_values(message: &str, vars: impl IntoIterator<Item = (String, String)>) -> String`,
   iterates the given `(name, value)` pairs; for each pair where the name
   case-insensitively contains `TOKEN`, `KEY`, `SECRET`, `PASSWORD`, or
   `CREDENTIAL`, and the value is at least 8 bytes (avoids redacting
   trivial short values like `"1"` or `""` that would false-positive
   against common substrings), every occurrence of that value in
   `message` is replaced with `[REDACTED]`. `sanitize_error_message`
   wires in `std::env::vars()`. This directly closes the gap demonstrated
   by this machine's `ANTHROPIC_AUTH_TOKEN` — regardless of the secret's
   shape, if it's sitting in an env var with a credential-looking name
   and it leaks into an error message, it's caught.
2. **Two additional static patterns**, following the existing
   `GITHUB_TOKEN_RE` shape (`LazyLock<Regex>`, `replace_all` to
   `[REDACTED]`): a generic `sk-[A-Za-z0-9_-]{10,}` pattern (covers
   Anthropic/OpenAI-style keys — including the exact shape found on this
   machine) and a GitLab personal-access-token pattern
   (`glpat-[A-Za-z0-9_-]{20,}` — GitLab is a first-class platform in this
   codebase, so it gets the same treatment GitHub already has).

### G8 — Project-level confirmation for a global opt-in

`is_co_contribution_enabled()` changes to check **only** the project-level
`.claude/settings.json` at the repo root — the global-settings fallback is
removed from this function. A new tri-state reader,
`fn read_co_contribution_field(path: &Path) -> Option<bool>` (`None` when
the file/field is missing or unparsable, `Some(bool)` otherwise),
backs both this and a new `pub(crate) fn global_co_contribution_pending_ack(repo_root: &Path) -> bool`:
true exactly when the global file has `Some(true)` and the project file
has `None` (present-but-false, or present-but-true, both count as an
explicit per-project decision already made — only *absence* is
"pending"). `read_co_contribution_flag` (used elsewhere) stays as
`read_co_contribution_field(path).unwrap_or(false)` — same public
behavior, now expressed in terms of the tri-state reader.

`doctor.rs`'s `CoContributionCheck` calls `global_co_contribution_pending_ack`
for the current project; when true, it reports a warning-level item (not
a silent pass) telling the user global opt-in is active but this project
has never confirmed, with the exact line to add to this project's
`.claude/settings.json` to explicitly enable or disable it.

## Testing strategy

- G6: unit tests for `autoreport_repo_slug` (normal `https://github.com/owner/repo`
  form; malformed form → fallback) and for `build_auto_report_hook_cmd`
  (repo appears in the generated command); bats test extending
  `hooks/tests/auto-report-bug.bats`'s `run_hook` helper to pass a repo
  argument, plus one new test asserting a custom repo slug reaches the
  `gh` calls.
- G7: unit tests per redaction case (env-var match/no-match, too-short
  value not redacted, unrelated var name not redacted even if its value
  coincidentally matches, `sk-…` and `glpat-…` regex hits, safe message
  unchanged).
- G8: unit tests for `read_co_contribution_field`'s three states and for
  `global_co_contribution_pending_ack`'s four combinations (global
  true/project absent → true; global true/project explicit true or false
  → false; global false → false regardless of project); a `doctor.rs`
  test for the new warning item.
