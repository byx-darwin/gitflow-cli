# Branch Finish — Phase 4 自动分支收尾步骤

**Date:** 2026-07-31
**Status:** Approved
**Scope:** gf-workflow skill (documentation-only change, no Rust code)

## Problem

The current gf-workflow Phase 3 creates a worktree and PR, Phase 4 runs
pipeline/triage/review/dogfooding checks, then archives the contract. After the
workflow completes, the user is still sitting inside the worktree, the feature
branch still exists locally, and the base branch has not been updated. There is
no automated branch lifecycle closure.

## Design

### 1. Contract Schema Changes

Add three new fields to the evidence object in `contract.schema.json`:

| Field | Type | Phase | Description |
|-------|------|-------|-------------|
| `base_branch` | `string` | 3 | The branch the worktree was forked from (e.g. `main`, `master`, `dev`) |
| `worktree_path` | `string` | 3 | Absolute path to the git worktree created in Phase 3 Step 1 |
| `branch_cleaned` | `boolean` | 4 | Whether branch finish cleanup was executed successfully |

The schema's `additionalProperties: false` constraint must be updated to allow
these fields.

### 2. Phase 3 Changes

Step 1 (Create worktree) additionally records:

- `base_branch`: captured via `git rev-parse --abbrev-ref HEAD` **before**
  checking out the new feature branch.
- `worktree_path`: the absolute path returned by `git worktree add`.

Phase 3 evidence becomes:

```json
{
  "branch": "feat/90-example",
  "base_branch": "dev",
  "worktree_path": "/path/to/repo/.worktrees/feat-90-example",
  "pr_url": "https://...",
  "tests_passed": true
}
```

### 3. Phase 4 New Step: Branch Finish

Inserted after dogfooding (Step 4) and before contract update (old Step 5,
renumbered to Step 6). New numbering:

| Step | Action | Output |
|------|--------|--------|
| 1 | `gf-pipeline-analyzer` | `pipeline_ok` |
| 2 | `gf-issue-triage` | — |
| 3 | `gf-review` | `review_report_path` |
| 4 | Dogfooding checklist | `dogfooding_passed` |
| **5** | **Branch Finish** (new) | **`branch_cleaned`** |
| 6 | Update contract + archive | — |

#### Branch Finish Logic

```text
1. Read from contract:
   base_branch  = phases.3.evidence.base_branch
   branch       = phases.3.evidence.branch
   worktree_path = phases.3.evidence.worktree_path
   pr_url       = phases.3.evidence.pr_url

2. Detect PR merge status:
   Run `gf pr view` (parse merged state from output)

3a. IF PR is merged:
   → Prompt user for confirmation (CLAUDE.md: no merge/push without permission)
   → On confirm:
     MAIN_ROOT=$(git -C "$(git rev-parse --git-common-dir)/.." rev-parse --show-toplevel)
     cd "$MAIN_ROOT"
     git checkout "$base_branch"
     git pull origin "$base_branch"
     git branch -d "$branch"
     git worktree remove "$worktree_path"
     git worktree prune
     git fetch --prune origin
   → Set branch_cleaned = true

3b. IF PR is NOT merged:
   → Output: "PR 待合并，分支 $branch 和 worktree 保留"
   → Do NOT delete branch or worktree
   → Set branch_cleaned = false
```

#### Confirmation Prompt (PR merged case)

```text
PR <pr_url> 已合并。是否执行分支收尾？

  ✓ 切回 <base_branch> 并拉取最新代码
  ✓ 删除本地分支 <branch>
  ✓ 清理 worktree <worktree_path>
  ✓ 清理远程跟踪引用 (fetch --prune)

确认执行？(y/n)
```

### 4. Files to Modify

| File | Change |
|------|--------|
| `skills/gf-workflow/SKILL.md` | Phase 3 Step 1 (record base_branch/worktree_path), Phase 4 step table (insert Step 5, renumber) |
| `skills/gf-workflow/contract.schema.json` | Add `base_branch`, `worktree_path`, `branch_cleaned` to evidence properties |
| `skills/gf-workflow/references.md` | Add Branch Finish command reference section |
| `skills/gf-workflow/gates.md` | No change (Gate 3→4 conditions unchanged) |
| `.claude/skills/gf-workflow/` | Mirror all above changes to the `.claude/skills` copy |

### 5. Constraints

- **User confirmation required** before any merge/push/delete operation
  (CLAUDE.md non-negotiable rule).
- **Never delete branch when PR is unmerged** — preserve work for iteration.
- **No hardcoded branch names** — always read `base_branch` from contract.
  Supports `main`, `master`, `dev`, or any custom branch.
- **Worktree cleanup only for workflow-managed worktrees** — if `worktree_path`
  is empty or missing (e.g. fast mode skipped worktree), skip cleanup silently.
- **Error tolerance** — if `git branch -d` fails (branch not fully merged
  locally despite remote merge), report warning but do not block archive.

### 6. Edge Cases

| Scenario | Behavior |
|----------|----------|
| `base_branch` missing from contract (old contracts) | Skip branch finish, log warning |
| Worktree already manually removed | `git worktree remove` fails silently, continue |
| PR merged but local branch has unpushed commits | `git branch -d` refuses; report and preserve |
| User declines confirmation | Set `branch_cleaned = false`, continue to archive |
| Fast mode without worktree | `worktree_path` empty → skip worktree cleanup, still attempt branch delete if PR merged |

### 7. Non-Goals

- Does NOT merge the PR itself (PR merge happens on remote via GitHub/GitLab/GitCode UI or CLI).
- Does NOT push anything — only local cleanup operations.
- Does NOT modify `finishing-a-development-branch` superpowers skill.
- No Rust code changes — this is a skill documentation enhancement only.
