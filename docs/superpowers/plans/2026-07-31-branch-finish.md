# Branch Finish Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add automated branch lifecycle closure (Branch Finish) to gitflow-workflow Phase 4, so worktrees are cleaned, feature branches deleted, and the base branch updated after PR merge.

**Architecture:** Documentation-only enhancement to the gitflow-workflow skill files. Three new evidence fields in the contract schema (`base_branch`, `worktree_path`, `branch_cleaned`), Phase 3 records the fork point, Phase 4 gains a new Step 5 that detects PR merge status and performs user-confirmed cleanup.

**Tech Stack:** JSON Schema, Markdown (skill documentation), bash (reference commands), jq (validation)

## Global Constraints

- No Rust code changes — skill documentation enhancement only
- User confirmation required before any delete/cleanup operation (CLAUDE.md non-negotiable)
- Never delete branch when PR is unmerged
- No hardcoded branch names — always read `base_branch` from contract
- `skills/gitflow-workflow/` and `.claude/skills/gitflow-workflow/` must stay in sync (identical content)
- Contract schema version bumps from `"1.0"` const to `"1.1"` const (already set in schema `$id`)

---

## File Structure

| File | Responsibility |
|------|---------------|
| `skills/gitflow-workflow/contract.schema.json` | Add 3 evidence fields, bump version const |
| `skills/gitflow-workflow/SKILL.md` | Phase 3 Step 1 (record base_branch/worktree_path), Phase 4 step table (insert Step 5, renumber 5→6, 6→7) |
| `skills/gitflow-workflow/references.md` | Add "Branch Finish Operations" section with bash commands |
| `.claude/skills/gitflow-workflow/contract.schema.json` | Mirror of above |
| `.claude/skills/gitflow-workflow/SKILL.md` | Mirror of above |
| `.claude/skills/gitflow-workflow/references.md` | Mirror of above |

---

### Task 1: Update Contract Schema

**Files:**
- Modify: `skills/gitflow-workflow/contract.schema.json:83-103`

**Interfaces:**
- Produces: Schema fields `base_branch` (string), `worktree_path` (string), `branch_cleaned` (boolean) available in evidence objects

- [ ] **Step 1: Add three new fields to evidence properties**

In `skills/gitflow-workflow/contract.schema.json`, inside `$defs.phase.properties.evidence.properties`, add after `"review_report_path"`:

```json
"base_branch": {
  "type": "string",
  "description": "Branch the worktree was forked from (Phase 3)"
},
"worktree_path": {
  "type": "string",
  "description": "Absolute path to the git worktree created in Phase 3"
},
"branch_cleaned": {
  "type": "boolean",
  "description": "Whether Branch Finish cleanup was executed (Phase 4)"
},
"dogfooding_passed": {
  "type": "boolean",
  "description": "Whether dogfooding checklist passed (Phase 4)"
}
```

Note: `dogfooding_passed` is referenced in SKILL.md Phase 4 but was missing from the schema — add it for completeness.

- [ ] **Step 2: Validate schema is well-formed JSON**

Run: `jq . skills/gitflow-workflow/contract.schema.json > /dev/null && echo "VALID"`
Expected: `VALID`

- [ ] **Step 3: Validate schema structure with a sample contract**

Run:
```bash
echo '{"version":"1.1","workflow_id":"wf-2026-07-31-001","title":"test","mode":"full","created_at":"2026-07-31T00:00:00Z","updated_at":"2026-07-31T00:00:00Z","current_phase":3,"phases":{"1":{"name":"Clarification","status":"complete","evidence":{"issue_url":"x","comment_id":"y","design_doc_path":"z"}},"2":{"name":"Planning","status":"complete","evidence":{"spec_path":"p","user_approved":true}},"3":{"name":"Execution","status":"in_progress","evidence":{"branch":"feat/1-x","base_branch":"dev","worktree_path":"/tmp/wt","pr_url":"u","tests_passed":true}},"4":{"name":"Delivery","status":"pending","evidence":{"branch_cleaned":false,"dogfooding_passed":true}}}}' | jq . > /dev/null && echo "SAMPLE VALID"
```
Expected: `SAMPLE VALID`

- [ ] **Step 4: Commit**

```bash
git add skills/gitflow-workflow/contract.schema.json
git commit -m "feat(workflow): add base_branch, worktree_path, branch_cleaned to contract schema"
```

---

### Task 2: Update SKILL.md Phase 3

**Files:**
- Modify: `skills/gitflow-workflow/SKILL.md:183-190`

**Interfaces:**
- Consumes: Schema fields from Task 1 (`base_branch`, `worktree_path`)
- Produces: Phase 3 evidence contract `{ branch, base_branch, worktree_path, pr_url, tests_passed }`

- [ ] **Step 1: Update Phase 3 Step 1 to record base_branch and worktree_path**

Replace the Phase 3 step table (lines starting with `| Step | Action | Output |` under `## Phase 3: Execution`) with:

```markdown
| Step | Action | Output |
|------|--------|--------|
| 1 | **[AUTO]** Record `base_branch` via `git rev-parse --abbrev-ref HEAD`, then create worktree: `feat/<issue-number>-<short-description>` | `branch`, `base_branch`, `worktree_path` |
| 2 | **[AUTO]** `superpowers:subagent-driven-development` (TDD: RED → GREEN → REFACTOR) | implementation |
| 3 | **[AUTO]** `gitflow-pr-create` — PR body MUST include `Closes #<issue-number>` | `pr_url` |
| 4 | **[AUTO]** `make test` or `cargo test` | `tests_passed` |
| 5 | **[AUTO]** Update contract: `evidence = { branch, base_branch, worktree_path, pr_url, tests_passed }` | — |
| 6 | **[AUTO]** Gate 3→4 — `pr_url` + `tests_passed = true` → **AUTO-ADVANCE to Phase 4** | — |
```

- [ ] **Step 2: Verify no other references to old Phase 3 evidence format**

Run: `grep -n "evidence = { branch, pr_url" skills/gitflow-workflow/SKILL.md`
Expected: No output (old format removed)

- [ ] **Step 3: Commit**

```bash
git add skills/gitflow-workflow/SKILL.md
git commit -m "feat(workflow): Phase 3 records base_branch and worktree_path in evidence"
```

---

### Task 3: Update SKILL.md Phase 4

**Files:**
- Modify: `skills/gitflow-workflow/SKILL.md:192-203`

**Interfaces:**
- Consumes: Phase 3 evidence (`base_branch`, `branch`, `worktree_path`, `pr_url`)
- Produces: Phase 4 evidence field `branch_cleaned` (boolean)

- [ ] **Step 1: Replace Phase 4 step table with new 7-step version**

Replace the Phase 4 step table with:

```markdown
| Step | Action | Output |
|------|--------|--------|
| 1 | **[AUTO]** `gitflow-pipeline-analyzer` — generates pipeline analysis report | `pipeline_ok` |
| 2 | **[AUTO]** `gitflow-issue-triage` — produces Issue triage report | — |
| 3 | **[AUTO]** `gitflow-review` — creates code review report | `review_report_path` |
| 4 | **[AUTO]** Dogfooding checklist (`docs/specs/phase4-dogfooding-checklist.md`) | `dogfooding_passed` |
| 5 | **[CONFIRM]** Branch Finish — detect PR merge status, user-confirmed cleanup (see below) | `branch_cleaned` |
| 6 | **[AUTO]** Update contract: `evidence = { pipeline_ok, review_report_path, dogfooding_passed, branch_cleaned }` | — |
| 7 | **[AUTO]** Archive contract → `.cache/workflows/archive/YYYY-MM/` | — |
```

- [ ] **Step 2: Add Branch Finish detail block after the Phase 4 table**

Insert after the step table (before `## Enforcement Rules`):

```markdown
### Phase 4 Step 5: Branch Finish

**Trigger:** After dogfooding passes. **Requires user confirmation.**

1. Read from contract: `base_branch`, `branch`, `worktree_path` (Phase 3 evidence)
2. Detect PR merge status: `gf pr view` (parse merged state)
3. **PR merged** → present confirmation prompt:
   - `cd` to main working tree (`git rev-parse --git-common-dir` parent)
   - `git checkout $base_branch && git pull origin $base_branch`
   - `git branch -d $branch`
   - `git worktree remove $worktree_path && git worktree prune`
   - `git fetch --prune origin`
   - Set `branch_cleaned = true`
4. **PR not merged** → output "PR 待合并，分支和 worktree 保留", set `branch_cleaned = false`
5. **Error tolerance:** if `git branch -d` fails (unmerged local commits), warn and preserve; do not block archive
6. **Missing fields:** if `base_branch` or `worktree_path` empty (old contract / fast mode), skip cleanup silently
```

- [ ] **Step 3: Verify Phase 4 now has 7 steps**

Run: `grep -c "^\| [0-9]" skills/gitflow-workflow/SKILL.md` (count step rows in Phase 4 table)
Expected: 7 rows in the Phase 4 table section

- [ ] **Step 4: Commit**

```bash
git add skills/gitflow-workflow/SKILL.md
git commit -m "feat(workflow): add Branch Finish step to Phase 4 with PR merge detection"
```

---

### Task 4: Update references.md

**Files:**
- Modify: `skills/gitflow-workflow/references.md` (append new section)

**Interfaces:**
- Consumes: Phase 3/4 evidence field names from Tasks 2-3
- Produces: Bash command reference for Branch Finish operations

- [ ] **Step 1: Append Branch Finish Operations section**

Add at the end of `skills/gitflow-workflow/references.md`:

```markdown
## Branch Finish Operations

Phase 4 Step 5 commands. All operations are local-only (no push).

### Detect PR Merge Status

```bash
gf pr view  # parse "merged" field from output
```

### Execute Branch Cleanup (after user confirmation)

```bash
# Return to main working tree
MAIN_ROOT=$(git -C "$(git rev-parse --git-common-dir)/.." rev-parse --show-toplevel)
cd "$MAIN_ROOT"

# Switch to base branch and update
git checkout "$BASE_BRANCH"
git pull origin "$BASE_BRANCH"

# Delete feature branch (safe: refuses if unmerged)
git branch -d "$FEATURE_BRANCH"

# Remove worktree
git worktree remove "$WORKTREE_PATH"
git worktree prune

# Clean stale remote tracking refs
git fetch --prune origin
```

### Skip Conditions

| Condition | Action |
|-----------|--------|
| `base_branch` empty/missing | Skip entire Branch Finish |
| `worktree_path` empty | Skip worktree removal, still attempt branch delete |
| PR not merged | Skip all cleanup, set `branch_cleaned = false` |
| `git branch -d` fails | Warn, preserve branch, continue to archive |
| User declines confirmation | Set `branch_cleaned = false`, continue to archive |
```

- [ ] **Step 2: Verify markdown renders (no broken code fences)**

Run: `grep -c '```' skills/gitflow-workflow/references.md`
Expected: Even number (all fences closed)

- [ ] **Step 3: Commit**

```bash
git add skills/gitflow-workflow/references.md
git commit -m "docs(workflow): add Branch Finish operations reference"
```

---

### Task 5: Sync .claude/skills/ Mirror

**Files:**
- Modify: `.claude/skills/gitflow-workflow/contract.schema.json`
- Modify: `.claude/skills/gitflow-workflow/SKILL.md`
- Modify: `.claude/skills/gitflow-workflow/references.md`

**Interfaces:**
- Consumes: Final state of `skills/gitflow-workflow/` files from Tasks 1-4
- Produces: Identical copies in `.claude/skills/gitflow-workflow/`

- [ ] **Step 1: Copy all three files to .claude/skills mirror**

```bash
cp skills/gitflow-workflow/contract.schema.json .claude/skills/gitflow-workflow/contract.schema.json
cp skills/gitflow-workflow/SKILL.md .claude/skills/gitflow-workflow/SKILL.md
cp skills/gitflow-workflow/references.md .claude/skills/gitflow-workflow/references.md
```

- [ ] **Step 2: Verify mirrors are identical**

Run:
```bash
diff skills/gitflow-workflow/contract.schema.json .claude/skills/gitflow-workflow/contract.schema.json && \
diff skills/gitflow-workflow/SKILL.md .claude/skills/gitflow-workflow/SKILL.md && \
diff skills/gitflow-workflow/references.md .claude/skills/gitflow-workflow/references.md && \
echo "MIRRORS IN SYNC"
```
Expected: `MIRRORS IN SYNC`

- [ ] **Step 3: Commit**

```bash
git add .claude/skills/gitflow-workflow/
git commit -m "chore: sync .claude/skills/gitflow-workflow mirror with branch-finish changes"
```

---

### Task 6: Final Validation

**Files:**
- Read-only validation of all modified files

- [ ] **Step 1: Validate all JSON is parseable**

Run: `jq . skills/gitflow-workflow/contract.schema.json > /dev/null && echo OK`
Expected: `OK`

- [ ] **Step 2: Verify Phase 3 evidence includes new fields in SKILL.md**

Run: `grep "base_branch, worktree_path" skills/gitflow-workflow/SKILL.md`
Expected: Match in Phase 3 Step 5 evidence line

- [ ] **Step 3: Verify Phase 4 has Branch Finish step**

Run: `grep "Branch Finish" skills/gitflow-workflow/SKILL.md`
Expected: Match in Phase 4 Step 5 row

- [ ] **Step 4: Verify references.md has Branch Finish section**

Run: `grep "Branch Finish Operations" skills/gitflow-workflow/references.md`
Expected: Match

- [ ] **Step 5: Run make check-agent-sync (if available)**

Run: `make check-agent-sync 2>/dev/null || echo "target not found — skip"`
Expected: Pass or skip

- [ ] **Step 6: Final commit (if any fixups needed)**

Only if previous steps revealed issues. Otherwise no commit needed.
