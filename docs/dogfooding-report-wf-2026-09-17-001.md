# Dogfooding Summary — wf-2026-09-17-001

**Date:** 2026-09-17
**Executor:** gf-workflow `wf-2026-09-17-001` Phase 4 (full mode)
**Binary under test:** `target/debug/gf` (branch `fix/340-coverage-metric-unification`, merged as `eb14dc0`) — not the globally installed `~/.cargo/bin/gf`, which predates this delivery's GitCode fix
**Result:** PASS (GitHub, GitLab) · BLOCKED (GitCode)

| Platform | Items | Passed | Failed | Blocked | Notes |
|----------|-------|--------|--------|---------|-------|
| GitHub   | 4     | 4      | 0      | 0       | on `byx-darwin/gitflow-cli` |
| GitLab   | 5     | 5      | 0      | 0       | on `iproost/iproost-docs` (self-hosted `192.168.230.23`) |
| GitCode  | 4     | 0      | 0      | 4       | not authenticated (`loggedIn: false`) |

**Bugs Found:** 0 functional failures. 2 checklist-drift items + 3 observations (below).
**Release Decision:** N/A — this run delivered documentation plus one binary-discovery fix via local merge, not a release.

---

## GitHub Dogfooding — 4/4 PASS

Risk item: `release` command non-interactive compatibility.

| # | Item | Result |
|---|------|--------|
| 1 | Create release (`--tag-name v0.0.0-dogfooding-20260917`) | ✅ created, id `390537401` |
| 2 | Visible after creation (`release view`) | ✅ tag and name returned |
| 3 | Non-interactive delete (stdin `/dev/null`, no TTY) | ✅ exit 0, no confirmation prompt |
| 4 | Idempotent repeat delete | ✅ exit 0, no error |

Extra check beyond the checklist: **no orphan git tag left on the remote** after `release delete` (`git ls-remote --tags origin` clean).

## GitLab Dogfooding — 5/5 PASS

Risk item: Chinese label encoding through the GitLab API.

| # | Item | Result |
|---|------|--------|
| 1 | Create Chinese label `测试标签` | ✅ no mojibake in response |
| 2 | Create issue carrying that label | ✅ issue #4, Chinese title/body/label all intact |
| 3 | View issue, confirm label | ✅ `labels=测试标签` |
| 4 | Close issue (non-interactive) | ✅ `state=closed` |
| 5 | Delete label (non-interactive) | ✅ deleted; `label list` confirms removal |

Full CRUD round-trip on Chinese text with no encoding error at any step. Cleanup verified — neither the test issue nor the test label remains open.

## GitCode Dogfooding — 4/4 BLOCKED

Risk item: `pr merge` non-interactive mode (Issue #70).

`gf auth status --platform gitcode` reports `loggedIn: false`. The GitCode CLI is installed
(`~/Library/Python/3.14/bin/gitcode`) but not authenticated, so no item in this section can run.

Worth noting: this section is the **only** one this delivery could have affected — commit `8f74699`
changed GitCode binary discovery so that Graphviz's `/opt/homebrew/bin/gc` no longer shadows the
real CLI. That fix is verified by unit tests and by direct probe (`gf auth status --platform gitcode`
now exits 1 with a genuine "not authenticated" error instead of exiting 0 while reporting
`loggedIn: false`), but its end-to-end behaviour under a logged-in GitCode account remains unverified.

---

## Checklist drift found (the checklist, not the CLI, is wrong)

| Checklist says | CLI actually requires |
|---|---|
| `gf release create v0.x.x --notes "test release"` | tag must be passed as `--tag-name`, not positionally; body flag is `--body` (there is no `--notes`) |
| `gf issue create --title "..." --labels "测试标签"` | flag is `--label` (singular) |

Both sections' `--yes` usage and the "`--yes` flag correctly passed" verification points are **valid** —
`-y, --yes` exists on both `release delete` and `label delete`.

## Observations (not failures)

1. **Repeat `release delete` on a nonexistent release returns `"deleted": true`.** This satisfies the
   checklist's idempotency requirement ("重复删除不报错"), but the payload claims to have deleted
   something that was not there. A `deleted: false` or a distinct "already absent" signal would be
   more honest to a machine consumer.
2. **IP-based self-hosted remotes are not recognised.** `git@192.168.230.23:iproost/iproost-docs.git`
   produces `warning: unrecognized domain in remote URL ... Defaulting to GitLab adapter`. The default
   happened to be correct here, but platform selection for self-hosted instances rests on a fallback
   rather than on detection.
3. **`gf auth status --platform gitlab` ignores `--output json` on the failure path.** GitHub and
   GitCode both emit JSON; GitLab emits a human-readable error block instead. It also folds a
   multi-host state into a single verdict — `glab` is authenticated to `192.168.230.23` and not to
   `gitlab.com`, and `gf` reports simply "未认证", which is misleading for self-hosted users.

## References

- Checklist: `docs/specs/phase4-dogfooding-checklist.md`
- Contract: `.cache/workflows/active/wf-2026-09-17-001.json`
- Delivery: merge commit `eb14dc0` into `dev`
