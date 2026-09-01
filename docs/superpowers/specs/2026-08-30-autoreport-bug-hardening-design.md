# Auto-Report-Bug Hardening — Design

## Context

A multi-role review (Product/Security/SRE/QA/OSS-maintainer/End-user) of the
`gf-autoreport-bug` pipeline (`error_reporter.rs` → `hooks/auto-report-bug.sh`
→ `skills/gf-autoreport-bug/SKILL.md` → `gh issue create`) found the write
path (Rust) solid, but the processing path still has four concrete gaps,
confirmed against this working copy:

1. **No rate limit / cleanup.** `error_reporter::write_to_disk` archives a
   pre-existing `pending.json` instead of overwriting it (P1-5), but nothing
   ever prunes those archives. `.cache/bug-reports/` in this repo currently
   holds 78 orphaned `pending.<millis>.json` files accumulated since
   2026-08-18.
2. **CI exclusion is social, not technical.** `gf-regression` documents
   "never invoke autoreport from CI," but the only Rust-level gate,
   `should_skip_reporting()`, checks `stderr.is_terminal()` — true for local
   interactive shells, **false for CI**, which is exactly backwards from the
   stated intent. Nothing in code checks `CI`/`GITHUB_ACTIONS`/etc.
3. **`auto-report` label has no in-repo source of truth.** `gh issue create
   --label "auto-report"` assumes the label exists on
   `byx-darwin/gitflow-cli`. It was created out-of-band; nothing here
   verifies it, so a missing label surfaces as a raw `gh` 422 with no
   actionable guidance, discovered only by the LLM at Issue-creation time.
4. **Non-interactive default is unsafe.** `SKILL.md` step 3b ("Preview")
   defaults to `create` when unattended — exactly the case the Stop Hook
   triggers. There is no human-in-the-loop gate at the one moment
   (unattended CLI failure → public GitHub Issue) where one matters most.

Full evidence trail: `docs/2026-08-18-autoreport-bug-multi-role-eval-report.md`,
`docs/references/gf-autoreport-bug-params.md`.

## Goals

- G1: Cap `.cache/bug-reports/` archive growth automatically, no operator
  action required.
- G2: Make "never in CI" a code-enforced gate, not a documentation
  convention.
- G3: Fail loud and early when the `auto-report` label is missing, before
  the LLM attempts `gh issue create`.
- G4: Make the unattended default `skip`, not `create`.
- G5: Produce first real end-to-end evidence that the pipeline, as hardened,
  actually files a usable GitHub Issue — the stress-test table in
  `docs/superpowers/tests/skills/gf-autoreport-bug-test.md` has stood at
  `[待运行]` since 2026-08-18.

## Non-goals

- Broadening the redaction regex beyond GitHub tokens (separate follow-up;
  not blocking these four fixes).
- Replacing the `gh issue list --search` dedup with structured matching
  (separate follow-up).
- Forcing a hard (non-LLM) trigger from the Stop Hook into the skill
  (architectural change, out of scope here).

## Design

### G1 — Archive pruning

`ErrorReport::write_to_disk` already renames an existing `pending.json` to
`pending.<unix_millis>.json` before writing the new one. Add a prune step
run in the same call: after archiving, list `pending.*.json` (excluding
`pending.json` itself) in `.cache/bug-reports/`, parse the embedded
millisecond timestamp from each filename, sort ascending, and delete all but
the newest `MAX_ARCHIVED_REPORTS` (10). Pruning failures (e.g. a permission
error on one file) must not fail the write — `pending.json` itself is the
report that matters; log-and-continue semantics, matching the existing
best-effort posture of this module (`maybe_report_error`'s doc comment:
"Callers should ignore errors").

### G2 — Code-enforced CI gate

Add `is_ci_environment()` next to `should_skip_reporting()`, following the
same "injectable check, pure predicate" shape already used there for
testability. It returns `true` if any of these environment variables is set
and non-empty: `CI`, `GITHUB_ACTIONS`, `GITLAB_CI`, `CI_PIPELINE_ID`,
`CIRCLECI`, `BUILDKITE`, `JENKINS_URL`. `maybe_report_error` gains this as
an additional early-return alongside `should_skip_reporting()` and
`is_co_contribution_enabled()`. This does not replace the TTY check (a
piped-but-local subprocess should still report); it adds the missing CI
carve-out `gf-regression`'s Red Flag already assumes exists.

### G3 — Label existence pre-check

`hooks/auto-report-bug.sh` already performs one `gh` round-trip (auth
check) before emitting the "MUST load skill" banner. Add a second cheap
round-trip immediately after a successful auth check:
`gh label list --repo byx-darwin/gitflow-cli --search auto-report --json
name -q '.[].name'`. Empty output (label absent) prints an actionable
warning (the exact `gh label create` command to run) and returns `exit 0`
**without** emitting the "MUST load skill" banner — `pending.json` is left
in place so the next successful run retries. This mirrors the existing
auth-failure branch's shape (`AUTH_CHECK_FAILED`) for consistency; add a
matching `LABEL_CHECK_FAILED` branch. The check itself is read-only
(`label list`), so it does not touch the "no repo-state changes without
permission" boundary from CLAUDE.md — it only surfaces the problem instead
of creating the label automatically.

### G4 — Fail-safe unattended default

`SKILL.md` step 3b changes from "Non-interactive default: create" to
"Non-interactive default: **skip** — keep `pending.json`, log the reason to
`processing.log`, and stop." A human re-running the skill interactively (or
explicitly passing a future `--yes` style override, out of scope here) is
required to actually create the Issue when unattended. This directly closes
the gap: the Stop Hook is by definition an unattended context, so this is
the exact case that previously defaulted to "post publicly."

### G5 — End-to-end verification

Manual runbook, gated by an explicit stop-and-confirm before any live
`gh issue create` against `byx-darwin/gitflow-cli` (per CLAUDE.md: creating
an Issue is a shared-system, hard-to-fully-reverse action and requires
explicit permission at execution time, independent of this plan's
approval). Steps: build with the G1–G4 changes, drive one real CLI failure
through the full local pipeline (Rust write → hook → skill) in a scratch
sandbox to confirm the coded gates behave, then — only with the user's
explicit go-ahead at that moment — let the skill file one real Issue,
verify its shape against `docs/references/gf-autoreport-bug-params.md`'s
template, close it as a test artifact, and record the run in
`docs/superpowers/tests/skills/gf-autoreport-bug-test.md`.

## Testing strategy

- G1, G2: unit tests in `apps/cli/src/error_reporter.rs`, TDD per CLAUDE.md.
- G3: extend `hooks/tests/auto-report-bug.bats` with a mocked `gh label
  list` branch, TDD (failing test first).
- G4: doc-only change; verified by proofreading + `make check-agent-sync`.
- G5: manual verification; no automated test (this is what fills the gap
  G5 exists to close).
