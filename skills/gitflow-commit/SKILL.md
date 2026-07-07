---
name: gitflow-commit
description: |
  Use when the user wants to inspect (view/diff/patch) or comment on a specific commit.
  当用户需要查看、差异比较、补丁导出或行内评论某个 commit 时使用。
---

# gitflow-commit

Encapsulates `gitflow-cli commit` for viewing, diffing, patching, and line-commenting on commits. Read-only except `comment`, which publishes.

## When to Use

| English | 中文 | Trigger Context |
|---------|------|-----------------|
| view / inspect a commit | 查看 commit | details, files changed, stats |
| diff / what changed | commit diff | unified diff |
| patch / export commit | 导出 commit | raw patch |
| comment on a commit line | 在 commit 上评论 | publish inline comment |
| fix bug in commit | 修复 bug | **NOT** — out of scope |

## Core Pattern

```bash
command -v gitflow-cli && git cat-file -t <sha>   # preconditions
gitflow-cli commit view <sha>                       # verify + read
gitflow-cli commit diff <sha>                       # diff
gitflow-cli commit patch <sha>                      # patch
gitflow-cli comment <sha> --body <t> --path <p> --line <n>
```

## Quick Reference

| Goal | Command |
|------|---------|
| View details | `gitflow-cli commit view <sha>` |
| Diff | `gitflow-cli commit diff <sha>` |
| Patch | `gitflow-cli commit patch <sha>` |
| Comment | `gitflow-cli commit comment <sha> --body <t> --path <p> --line <n>` |

## Implementation

### Preconditions

- `gitflow-cli` installed — `command -v gitflow-cli`
- SHA valid — `git cat-file -t <sha>` = commit
- In repo — `git rev-parse --show-toplevel`

### Step 1: Validate SHA

`git cat-file -t <sha>`. Fail → "SHA not found in repo"; stop.

### Step 2: Execute Read Operation

Run `view`, `diff`, or `patch`. Output. Stop.

### Step 3: Comment on Commit (mutation)

1. Auth check — `gitflow-cli auth status`; fail → "Run `gitflow-cli auth login`."
2. Draft body. Show draft to user.
3. STOP. Await explicit confirmation.
4. POST via `gitflow-cli commit comment`.
5. Output URL.

### Error Handling

| Error | Recovery |
|-------|----------|
| Invalid SHA | "SHA not found. Verify via `git log`." |
| Auth failure | "Run `gitflow-cli auth login --platform {platform}`." |
| Post failure | "Body preserved. Retry or abort." |
| Line out of diff | "Line not in diff. Re-check." |

## Responsibility

### ✅ In Scope

- View commit metadata and file stats
- Fetch unified diff / raw patch
- Publish line-level comment after confirmation

### ❌ Out of Scope

- Create / fix commits — `/gitflow-workflow`
- PR review — `/gitflow-pr-inline-review`, `/gitflow-pr-review`
- PR lifecycle — `/gitflow-pr`
- Issue management — `/gitflow-issue`

### 🚫 Do Not

- ❌ Post `comment` without explicit user confirmation
- ❌ Assume SHA validity without `git cat-file -t`
- ❌ Use `commit comment` as PR-review decision — different skill
- ❌ Edit source code or create commits

## Rationalization

| Excuse | Reality |
|--------|---------|
| "Just post it directly" | Mutation requires confirmation |
| "SHA looks valid, skip check" | Always verify before API call |
| "I'll fix the bug too" | Out of scope — `/gitflow-workflow` |
| "Comment equals approval" | Use `/gitflow-review` |

## Red Flags

- 🚩 "Post without asking" — refuse; confirmation mandatory
- 🚩 "Skip the SHA check" — refuse; non-skippable
- 🚩 "Merge / approve" — redirect `/gitflow-pr`
- 🚩 "Fix the issue" — redirect `/gitflow-workflow`
- 🚩 "Add task" — redirect `/gitflow-issue`

## Test Scenarios

### Scenario 1: Happy Path — View & Diff

- **Given** valid SHA `abc123`
- **When** "show commit abc123 and its diff"
- **Then** `commit view` + `commit diff` run; no mutation

### Scenario 2: Negative — PR Review

- **Given** "review PR #101"
- **When** user asks for PR review — not commit inspection
- **Then** skill NOT loaded; redirect `/gitflow-pr-inline-review`

### Scenario 3: Boundary — Comment Without Confirmation

- **Given** SHA + `--body` + `--path` + `--line` ready
- **When** Claude POSTs without confirmation
- **Then** refuse; show draft; STOP

### Scenario 4: Error — Invalid SHA

- **Given** bogus SHA `0000000`
- **When** `git cat-file -t` fails
- **Then** stop "SHA not found"; no command run

## Success Criteria

- [ ] `view`/`diff`/`patch` return correct data
- [ ] Invalid SHA caught before API call
- [ ] `comment` only after explicit confirmation
- [ ] No PR / issue / fix actions performed

## Common Mistakes

- ❌ **Posting without confirmation** — always draft-then-confirm (Step 3.3)
- ❌ **Skipping SHA validation** — always run `git cat-file -t`

## Trigger Keywords

| English | 中文 |
|---------|------|
| view commit | 查看 commit |
| commit diff | commit 差异 |
| commit patch | commit 补丁 |
| comment on commit | 在 commit 上评论 |
| show changes in commit | 显示 commit 变更 |

## See Also

- `/gitflow-pr-inline-review` — inline comments on PRs
- `/gitflow-pr-review` — overall PR review
- `/gitflow-pr` — PR lifecycle
- `/gitflow-issue` — post-comment-on-issue pattern
- `docs/superpowers/templates/skill-conventions.md` — skill conventions
