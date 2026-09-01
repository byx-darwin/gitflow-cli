# gf-workflow: Local-merge delivery path (Issue #265)

## Context

`gf-workflow` is the four-phase gated orchestrator skill shipped with this
repo (`skills/gf-workflow/`, synced to `.claude/skills/gf-workflow/`). Phase 3
Step 3 (delivery) is currently hardcoded to `gf-pr-create`, and Gate 3→4
(`gates.md`) requires `phases.3.evidence.pr_url` unconditionally. There is no
path for single-developer / no-PR-review scenarios that just want a local
`git merge`.

## Problem

Forcing "push + open PR" for every delivery is unnecessary overhead for
personal projects or fast internal iteration branches where no PR review
trail is needed. Today, the only way around it is to bypass Phase 3/4
entirely or open an unwanted PR.

## Scope

Documentation/schema-only change, confined to three files under
`skills/gf-workflow/` (source of truth; the `.claude/skills/gf-workflow/`
copy is synced separately, not edited directly):

- `SKILL.md` — Phase 3 Step 3, Phase 4 Step 4 (Branch Finish)
- `gates.md` — Gate 3→4 (+ `check_gate()` pseudocode)
- `contract.schema.json` — `phases[3].evidence`

No changes to the `gf` binary/CLI code.

## Design

### Phase 3 Step 3 (replaces the hardcoded `gf-pr-create` call)

After TDD implementation (Step 2) completes, the orchestrator asks the user
to choose a delivery path:

1. **PR path** (unchanged, and the schema default for backward compat):
   `gf-pr-create` → `pr_url`; `delivery_mode = "pr"`.
2. **Local-merge path** (new):
   - Ask the user which git strategy to use each time (`git merge --no-ff`
     or `git merge --squash`) — no fixed default, since this varies by task.
   - Run the merge in the **main working tree** (not the worktree — the
     worktree does not own `base_branch`): merge `branch` into `base_branch`.
   - Success → capture `git rev-parse HEAD` as `merge_commit`; set
     `delivery_mode = "local_merge"`.
   - Conflict → `git merge --abort`; leave `branch`/worktree untouched; tell
     the user to resolve manually and re-run this step. `merge_commit` stays
     empty, so Gate 3→4 blocks until they retry — no silent fallback to PR.

`tests_passed` (Step 4) and `merge_queued` (Step 5, PR-only — skipped when
`delivery_mode == "local_merge"`) proceed as today.

### Gate 3→4 (`gates.md`)

Evidence condition becomes:

```
(delivery_mode == "pr" AND pr_url) OR (delivery_mode == "local_merge" AND merge_commit)
```

`tests_passed` remains required in both cases. `check_gate()` pseudocode
updated to match. No exemptions by mode (same as today — Gate 3→4 has none).

### Phase 4 Step 4 (Branch Finish)

If `delivery_mode == "local_merge"`, skip the `gf pr view` / `mergedAt`
merge-status detection entirely (the merge already happened in Phase 3) and
go straight to the existing cleanup sequence: re-run worktree preflight →
`git checkout $base_branch && git pull` → `git branch -d $branch` →
`git worktree remove` → prune. The PR path is unchanged.

### Schema (`contract.schema.json`)

Add to `phases[3].evidence`:

- `delivery_mode`: `{"type": "string", "enum": ["pr", "local_merge"]}`
- `merge_commit`: `{"type": "string"}` (merge commit SHA for the local path)

JSON Schema has no runtime default keyword enforced here; backward
compatibility for old contracts (field absent) is handled in prose in
`SKILL.md`/`gates.md`: absence is treated as `"pr"`.

## Testing / Validation

Docs/schema-only change — no Rust build/test/clippy required per
`CLAUDE.md` toolchain rules. Validation:

- `contract.schema.json` remains valid JSON Schema (`python3 -m json.tool` /
  a JSON Schema validator).
- Proofread rendered Markdown in `SKILL.md` / `gates.md`.
- `make check-agent-sync` (per `CLAUDE.md`, required for skill edits) to
  confirm `skills/` source and `.claude/skills/` copy stay in sync per the
  repo's existing sync mechanism.
