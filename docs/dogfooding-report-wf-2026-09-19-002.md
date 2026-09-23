## Dogfooding Summary — Phase 4 (Issue #334, wf-2026-09-19-002)

**Date:** 2026-09-19
**Executor:** gf-workflow Phase 4 (Claude, full mode)
**Result:** FAIL (GitLab item blocked by a newly discovered bug)

| Platform | Repo | Items | Passed | Failed | Notes |
|----------|------|-------|--------|--------|-------|
| GitHub   | byx-darwin/gitflow-cli (this repo) | 4 | 4 | 0 | Release create/delete, non-interactive `--yes` verification all clean |
| GitLab   | 192.168.230.23/iproost/iproost-docs (self-hosted) | 5 | 1 | 4 | `gf label create` fails 100% of the time on GitLab (not a Chinese-encoding issue — reproduced with ASCII name too); blocks the remaining 4 chained items (label-tagged issue create/view/close/label-delete) |
| GitCode  | gitcode.com/byx-darwin/NexaTrade (user-provided) | 4 | 4 | 0 | Non-interactive `pr merge` completed without confirmation prompt |

**Bugs Found:** 1 — [Issue #372](https://github.com/byx-darwin/gitflow-cli/issues/372): `gf label create` passes the label name as a positional argument to `glab label create`, which requires `--name`; every GitLab label creation fails regardless of name content. A secondary observation (not yet filed separately): `gf label create --help`'s color format guidance (`d73a4a`, no `#`) does not match what `glab` actually requires (`#428bca`).
**Release Decision:** BLOCKED for a full release cut (GitLab label CRUD is broken) — **does not block Issue #334's already-completed delivery**, since #334 is a documentation-only change to `gf-pr-apply-feedback` with no GitLab/label code touched; this dogfooding run exercises the generic release-time checklist per explicit user request, same as the immediately preceding #333 run.

### Notes

- This dogfooding run is **not scoped to the Issue #334 change itself** (a documentation-only closing of the review loop in `gf-pr-apply-feedback/SKILL.md` — no platform-command code touched). It exercises the generic release-time checklist across all three platforms, per explicit user request during Phase 4, matching the precedent set by the #333 dogfooding run (`docs/dogfooding-report-wf-2026-09-19-001.md`).
- `gitlab.com` remains unauthenticated (401) in this environment; only the self-hosted instance (`192.168.230.23`) was reachable, matching the pre-existing auth status. GitLab dogfooding used the same self-hosted test repo as the #333 run.
- Root cause was isolated by reproducing directly against `glab`: `glab label create "<name>" --color "<hex>"` (positional name) fails with `Unknown command "<name>" for "glab label create"`, while `glab label create --name "<name>" --color "#<hex>"` succeeds. This confirms the defect is in `gf`'s GitLab adapter argument construction, not in glab itself or in Chinese-character handling.
- Test artifacts cleaned up: GitHub release `v0.0.0-dogfood-334` (×2, both deleted), GitLab diagnostic label `dogfood-test-334` (created directly via `glab` for root-cause isolation, then deleted — no `gf`-created GitLab artifacts exist since `gf label create` never succeeded), GitCode PR #2 + branch `dogfood-334-gitcode-pr` (merged then deleted) + a follow-up commit removing the test file `DOGFOOD_334_TEST.md` from `main` (this repo's PR-merge step, like #333's, lands a real commit on `main`; unlike the #333 run, that commit's content was explicitly reverted here rather than left in place). No residue remains on any of the three platforms.
