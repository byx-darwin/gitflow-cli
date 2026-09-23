## Dogfooding Summary — Phase 4 (Issue #337, wf-2026-09-19-005)

**Date:** 2026-09-19
**Executor:** gf-workflow Phase 4 (Claude, full mode)
**Result:** FAIL (one new GitLab bug found; a previously-filed GitLab bug is confirmed already fixed in code)

| Platform | Repo | Items | Passed | Failed | Notes |
|----------|------|-------|--------|--------|-------|
| GitHub   | byx-darwin/gitflow-cli (this repo) | 4 | 4 | 0 | Release create/view/delete, non-interactive `--yes` verification, and idempotent repeat-delete all clean |
| GitLab   | 192.168.230.23/iproost/iproost-docs (self-hosted) | 5 | 4 | 1 | `gf label create` now works (Issue #372 appears fixed in code, though the tracker Issue is still open); `gf issue create` fails 100% of the time on GitLab when `--body` is omitted — new bug, filed as #375. Downstream items (label-tagged issue view, issue close, label delete) all passed once the create step was worked around with a non-empty `--body` |
| GitCode  | gitcode.com/byx-darwin/NexaTrade | 4 | 4 | 0 | `pr create` + non-interactive `pr merge` completed without confirmation prompt; branch and merge-artifact cleanup all succeeded |

**Bugs Found:** 1 new — [Issue #375](https://github.com/byx-darwin/gitflow-cli/issues/375): `gf issue create` against GitLab fails 100% of the time when `--body` is not supplied, because the adapter only passes `--description` to `glab issue create` when a body is given, but `glab` requires `--description` (or `--template`) in non-interactive mode even for an empty body.

**Release Decision:** BLOCKED for a full release cut (GitLab `issue create` is broken for the common no-body case) — **does not block Issue #337's already-completed delivery**, since #337 is a documentation/skill-instruction change to `gf-workflow-batch` touching no platform-command Rust code. This dogfooding run exercises the generic release-time checklist per explicit user request during this Phase 4, matching the precedent set by the #334 and #333 runs (`docs/dogfooding-report-wf-2026-09-19-002.md`, `docs/dogfooding-report-wf-2026-09-19-001.md`).

### Details

#### GitHub

- `gf release create --tag-name v0.0.0-dogfood-337 --body "test release"` — succeeded, release visible via `gf release view v0.0.0-dogfood-337`.
- `gf release delete v0.0.0-dogfood-337 --yes` — succeeded; `gf release view` afterward correctly errors ("release not found"), confirming deletion.
- Non-interactive mode: `echo "y" | gf release create --tag-name v0.0.0-dogfood-337 --body "test"` — completed without any interactive confirmation prompt blocking it.
- Cleanup: `gf release delete v0.0.0-dogfood-337 --yes` — succeeded. Repeated a third time on an already-deleted release and it still returned `{"deleted": true}` with exit code 0 (idempotent, no error), satisfying the checklist's "删除操作幂等，重复删除不报错" verification point. No leftover tag on GitHub (`git ls-remote --tags` confirms it's gone).

**Note on checklist syntax:** the checklist's example command (`gf release create v0.x.x --notes "test release"`) uses a positional tag argument and `--notes`; the actual CLI requires `--tag-name <TAG>` and `--body <TEXT>` (no `--notes` flag exists). This is a checklist documentation mismatch, not a code bug — flagged as a secondary note inside Issue #375 rather than filed separately, since it doesn't block anything once the correct flags are used.

#### GitLab

Cloned `192.168.230.23:iproost/iproost-docs.git` locally (via `git clone`, since this build of `gf` has no `repo clone` subcommand) and ran all commands from inside that clone so `gf` could auto-detect the GitLab remote.

1. `gf label create "测试标签" --color "#ff0000"` — **succeeded**. This is the exact scenario that failed 100% of the time in the prior #334 dogfooding run (Issue #372: label name passed positionally instead of via `--name`). Inspecting `crates/gitlab/src/label.rs` confirms the current `dev` code already calls `glab label create --name <name> --color <color>`, i.e. **#372 is already fixed in code**, even though the GitHub Issue #372 itself is still open. No code change was made by this run; noting it here as an observation for whoever does tracker hygiene.
2. `gf issue create --title "Dogfooding test" --label "测试标签"` (the checklist's literal command, `--labels`, doesn't even parse — see below) — **failed**. Root-caused with `RUST_LOG=debug` (isolating against the `gf` binary's own debug logs, not raw `glab`, since the log line showed the exact `glab` invocation and its stderr):
   ```
   $ gf issue create --title "Dogfooding test"
   × Failed to create issue: GitLab CLI 执行失败

   [debug] glab command failed raw_stderr="'--Title' and '--description' (or '--template') required for non-interactive mode."
   ```
   Passing any `--body` (including an explicit workaround value) succeeds. Filed as **Issue #375** with the precise code location (`crates/gitlab/src/issue.rs`, `create()`, the `if let Some(body) = &args.body` guard around `--description`).
   Additionally, the checklist's literal example uses `--labels` (plural); the actual flag is `--label` (singular, repeatable) — `gf issue create --title "..." --labels "..."` fails immediately with a clap parse error (`unexpected argument '--labels'`) before even reaching the network. Documented as a secondary note in #375 rather than a separate Issue.
   Worked around by supplying `--body "gf-workflow phase4 dogfooding for #337"`, which created issue #8 with the label attached.
3. `gf issue view 8` — **passed**. Label `测试标签` displayed correctly with no encoding corruption (confirmed the checklist's original "Chinese encoding" risk hypothesis is not the actual defect — same conclusion as the #334 run for `label create`).
4. `gf issue close 8` — **passed**.
5. `gf label delete "测试标签" --yes` — **passed**; confirmed absent afterward via `gf label list`.

An extra diagnostic issue (#7, created and closed during root-cause isolation of the `--body` bug, before the official test issue #8) also exists closed on the GitLab tracker — see Notes/cleanup below.

#### GitCode

Cloned `byx-darwin/NexaTrade` via the `gitcode` CLI (`gitcode repo clone byx-darwin/NexaTrade --git-protocol ssh`; the plain HTTPS clone path failed first with "could not read Username" since no HTTPS credential helper is configured for gitcode.com in this environment — SSH is the working protocol here, consistent with prior runs).

- Created branch `dogfood-337-gitcode-pr` with one test-file commit, pushed it, then `gf pr create --title "Dogfooding test" --body "test"` — **succeeded** (PR #3).
  - Minor observation (not filed): the JSON returned by `pr create` had empty `author`, `baseBranch`, `headBranch`, and `url` fields, while `gf pr view 3` immediately afterward returned all of them populated correctly. Both code paths parse the same `PrApiResponse` type from the underlying `gitcode <cmd> --json` output, so this looks like the underlying `gitcode pr create --json` response is simply thinner than `gitcode pr view --json`'s — cosmetic only (all data is present and correct via `pr view`), not filed as a separate Issue since it doesn't affect functional correctness and this run already has one bug (#375) covering the release-blocking scenario. Recommend a human triage pass to decide if it's worth its own low-priority Issue.
- `gf pr merge 3 < /dev/null` (no TTY) — **succeeded** immediately with `{"merged": true}`, no confirmation prompt, matching the checklist's non-interactive-mode verification point and the resolution of the historical Issue #70.
- Cleanup: reverted the test file with a follow-up commit on `main` (`chore: remove dogfooding test artifact (Issue #337)`), pushed it, then deleted the remote branch `dogfood-337-gitcode-pr` via `git push origin --delete`. Confirmed via `gf pr view 3` the PR shows `state: closed`, `mergedAt` set.

### Notes

- This dogfooding run is **not scoped to the Issue #337 change itself** (a documentation/skill-instruction change to `gf-workflow-batch`'s dependency-resolution notes — no platform-command code touched). It exercises the generic release-time checklist across all three platforms, per explicit user request during Phase 4, matching the precedent set by the #334 and #333 dogfooding runs.
- `gitlab.com` remains unauthenticated (401) in this environment; only the self-hosted instance (`192.168.230.23`) was reachable, matching the pre-existing, expected auth status. GitLab dogfooding used the same self-hosted test repo (`iproost/iproost-docs`) as the #334 and #333 runs.
- This build of `gf` has no top-level `repo` subcommand (`gf --help` lists `issue, pr, release, review, auth, label, milestone, commit, pipeline, workflow, doctor, skills, update` only), so local clones for GitLab and GitCode were made with plain `git clone` / the underlying `gitcode` CLI's `repo clone`, not `gf` itself — noted since the checklist implicitly assumes a working directory already pointed at the target repo.
- Locating the real GitCode CLI binary required extra care on this machine: `gf`'s `gitcode_binary()` probe candidates are `["gitcode", "gc"]`, and Homebrew's `gc` on this machine is an unrelated graph-counting tool (not GitCode's CLI) — a known naming collision already documented in prior session memory (`e2e-gitcode-noauth-gc-name-collision`). `gf auth status --platform gitcode` still worked correctly during this run (it evidently found the real `gitcode` binary at `~/Library/Python/3.14/bin/gitcode` via the probe's PATH/pip-user-dir search), so this was an observation made only because this run needed the raw binary directly for `repo clone`; it did not affect any `gf`-driven command output.
- Test artifacts cleaned up:
  - **GitHub:** release `v0.0.0-dogfood-337` (created ×2 across the two checklist steps, both deleted; a third repeat-delete on the already-gone release also succeeded, exercising idempotency). No leftover tags. Issue #375 (the filed bug report) is an intentional, permanent artifact of this run, not test residue.
  - **GitLab:** label `测试标签` created and deleted — confirmed absent. Issues #7 (created only for root-causing the `--body` bug, before the "official" test issue existed) and #8 (the checklist's designated test issue) are both **closed** but remain on the tracker — GitLab/`glab` has no issue-deletion capability exposed through `gf issue close`, and the checklist's own cleanup instruction for GitLab is `gf issue close <n>` (not delete), so this matches the checklist's defined notion of "cleanup," same as the #334 run. Issue #7 is extra residue beyond what the checklist calls for (a byproduct of bug isolation); it is closed and clearly labeled by its title/body, so it was left in the same state the checklist's own "delete = close" convention would leave the designated test issue in, rather than force a raw `glab` deletion outside the CLI-based process this run is scoped to.
  - **GitCode:** PR #3 merged and closed (permanent tracker record, expected — same as the #334 run's PR #2). Branch `dogfood-337-gitcode-pr` deleted from the remote. The merge-added test file `DOGFOOD_337_TEST.md` was explicitly reverted with a follow-up commit on `main` (as in the #334 run), so `main`'s tree has no test residue, though the two extra commits (the test-file add via the merge, and its removal) remain in `main`'s history — this matches how PR-merge-based cleanup works for GitCode in every prior run, since GitCode's `pr merge` lands a real, permanent commit.
  - All local scratch clones (`/tmp/gf-dogfood-337/`) were removed at the end of the run; no local filesystem residue remains outside the repo's own `docs/` report file.
