# gf-workflow Local-Merge Delivery Path Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Let `gf-workflow`'s Phase 3 delivery step offer "local merge" as an alternative to "push + open PR", so single-developer / no-review-trail scenarios don't have to open an unwanted PR.

**Architecture:** Add a `delivery_mode` ("pr" | "local_merge") branch to three already-existing, already-documented points in the orchestrator: Phase 3 Step 3 (the delivery action itself), Gate 3→4 (the evidence check), and Phase 4 Step 4 (Branch Finish cleanup). No new subsystems, no code — this is a documentation/schema-only change confined to `skills/gf-workflow/`.

**Tech Stack:** Markdown (`SKILL.md`, `gates.md`), JSON Schema (`contract.schema.json`). No Rust, no `gf` binary changes.

**Spec:** `docs/superpowers/specs/2026-08-31-gf-workflow-local-merge-design.md`

## Global Constraints

- Edit only `skills/gf-workflow/{SKILL.md,gates.md,contract.schema.json}` — this is the source of truth per `CLAUDE.md`. **Never edit `.claude/skills/gf-workflow/`** — it is a separate copy with its own (unrelated) drift; touching it is explicitly forbidden by `CLAUDE.md` unless the user asks.
- `delivery_mode` absent on a contract MUST be read as `"pr"` — old contracts (pre-#265) stay valid without migration.
- `merge_commit` is required only when `delivery_mode == "local_merge"`; `pr_url` is required only when `delivery_mode == "pr"`. `tests_passed` is required in both cases (unchanged).
- Conflict during local merge → abort, leave `branch`/worktree untouched, no silent fallback to PR (per approved design).
- No Rust code, no `gf` binary changes — `cargo build`/`test`/`clippy` are out of scope for this plan (per `CLAUDE.md`: docs/spec/skill-only changes skip the Rust gate).

---

### Task 1: Extend `contract.schema.json` with `delivery_mode` and `merge_commit`

**Files:**
- Modify: `skills/gf-workflow/contract.schema.json` (the `phases[3].evidence` = `$defs.phase.properties.evidence.properties` object — same object already holding `pr_url`, `tests_passed`, `merge_queued`)

**Interfaces:**
- Produces: two new evidence field names, `delivery_mode` and `merge_commit`, that Task 2 (gates.md) and Task 3 (SKILL.md) reference by these exact names.

- [ ] **Step 1: Add the two fields to the schema**

In `skills/gf-workflow/contract.schema.json`, inside `$defs.phase.properties.evidence.properties`, add these two entries (alongside the existing `pr_url`, `merge_queued`, etc. — insert after `pr_url` for readability):

```json
            "delivery_mode": {
              "type": "string",
              "enum": ["pr", "local_merge"],
              "description": "Phase 3 交付方式；缺省视为 \"pr\"（向后兼容旧合同，字段本身无 default 关键字）"
            },
            "merge_commit": {
              "type": "string",
              "description": "本地合并时 (`delivery_mode == \"local_merge\"`) 产生的 merge commit SHA"
            },
```

**Step 2: Verify the file is still valid JSON and valid JSON Schema**

Run: `python3 -m json.tool skills/gf-workflow/contract.schema.json > /dev/null && echo VALID`
Expected: `VALID` with no output diff to the rest of the file (only the two new keys added).

Also confirm no duplicate keys and the `additionalProperties: false` on the evidence object still lists these two names (it doesn't need a change — `additionalProperties: false` combined with `properties` already permits any key declared under `properties`, so no other section needs editing).

- [ ] **Step 3: Commit**

```bash
git add skills/gf-workflow/contract.schema.json
git commit -m "feat(gf-workflow): add delivery_mode/merge_commit to Phase 3 evidence schema"
```

---

### Task 2: Update Gate 3→4 in `gates.md`

**Files:**
- Modify: `skills/gf-workflow/gates.md` (the `### Gate 3→4: 执行 → 交付` section and the `check_gate()` Python pseudocode block, `elif target_phase == 4:` branch)

**Interfaces:**
- Consumes: `delivery_mode`, `merge_commit` (Task 1)
- Produces: updated gate condition text + pseudocode that Task 3's SKILL.md prose can point readers to.

- [ ] **Step 1: Update the prose gate conditions**

In `skills/gf-workflow/gates.md`, under `### Gate 3→4: 执行 → 交付`, replace:

```markdown
**条件:**
- `phases.3.status` 为 `complete`
- `phases.3.evidence.pr_url` 非空
- `phases.3.evidence.tests_passed` 为 `true`
```

with:

```markdown
**条件:**
- `phases.3.status` 为 `complete`
- 交付证据二选一（`delivery_mode` 缺省视为 `"pr"`）：
  - `delivery_mode == "pr"` → `phases.3.evidence.pr_url` 非空
  - `delivery_mode == "local_merge"` → `phases.3.evidence.merge_commit` 非空
- `phases.3.evidence.tests_passed` 为 `true`（两种交付方式均必须）
```

- [ ] **Step 2: Update the `check_gate()` pseudocode**

In the same file, in the ```python check_gate(contract, target_phase)``` block, replace the `elif target_phase == 4:` branch:

```python
    elif target_phase == 4:
        evidence = contract["phases"]["3"]["evidence"]
        return contract["phases"]["3"]["status"] == "complete" \
               and evidence.get("pr_url") \
               and evidence.get("tests_passed")
```

with:

```python
    elif target_phase == 4:
        evidence = contract["phases"]["3"]["evidence"]
        delivery_mode = evidence.get("delivery_mode", "pr")
        if delivery_mode == "local_merge":
            delivery_ok = bool(evidence.get("merge_commit"))
        else:
            delivery_ok = bool(evidence.get("pr_url"))
        return contract["phases"]["3"]["status"] == "complete" \
               and delivery_ok \
               and evidence.get("tests_passed")
```

- [ ] **Step 3: Verify no other reference to the old two-line condition survives**

Run: `grep -n 'pr_url' skills/gf-workflow/gates.md`
Expected: only the new conditional lines from Step 1/2 mention `pr_url` (no leftover unconditional check).

- [ ] **Step 4: Commit**

```bash
git add skills/gf-workflow/gates.md
git commit -m "feat(gf-workflow): Gate 3→4 accepts local_merge delivery evidence"
```

---

### Task 3: Update `SKILL.md` — Phase 3 Step 3 and Phase 4 Step 4

**Files:**
- Modify: `skills/gf-workflow/SKILL.md` (Phase 3 table, row 3 around line 345; Phase 4 Step 4 "Branch Finish" section around lines 404-429)

**Interfaces:**
- Consumes: `delivery_mode`, `merge_commit` (Task 1), updated Gate 3→4 text (Task 2)

- [ ] **Step 1: Replace Phase 3 Step 3's table row**

In `skills/gf-workflow/SKILL.md`, find this row in the Phase 3 table:

```markdown
| 3 | **[AUTO]** `gf-pr-create` — PR body MUST include `Closes #<issue-number>` | `pr_url` |
```

Replace it with:

```markdown
| 3 | **[AUTO]** Delivery choice — ask user: ① 推送 + 建 PR（默认）② 本地合并. **① PR**: `gf-pr-create`, PR body MUST include `Closes #<issue-number>`; `delivery_mode = "pr"`. **② Local merge**: ask which git 策略（`git merge --no-ff` / `git merge --squash`，无固定默认，每次询问）; in the **main working tree** (not the worktree — it doesn't own `base_branch`), merge `branch` into `base_branch`. Success → `merge_commit = $(git rev-parse HEAD)`, `delivery_mode = "local_merge"`. Conflict → `git merge --abort`, leave `branch`/worktree untouched, tell user to resolve manually and re-run this step (no silent fallback to PR). | `pr_url` or (`delivery_mode`, `merge_commit`) |
```

- [ ] **Step 2: Update the contract-evidence-update row (Step 6) to include the new fields**

Find:

```markdown
| 6 | **[AUTO]** Update contract: `evidence = { branch, base_branch, worktree_path, worktree_preflight, unresolved_dirty_paths, pr_url, tests_passed, merge_queued }` | — |
```

Replace with:

```markdown
| 6 | **[AUTO]** Update contract: `evidence = { branch, base_branch, worktree_path, worktree_preflight, unresolved_dirty_paths, delivery_mode, pr_url, merge_commit, tests_passed, merge_queued }` (only the fields matching the chosen `delivery_mode` are populated; the other of `pr_url`/`merge_commit` stays absent) | — |
```

- [ ] **Step 3: Update Gate 3→4 auto-advance row (Step 7) wording**

Find:

```markdown
| 7 | **[AUTO]** Gate 3→4 — `pr_url` + `tests_passed = true` → **AUTO-ADVANCE to Phase 4**。（真正的合并闸门是平台必需检查 + 排队合并，不由本 workflow 判定） | — |
```

Replace with:

```markdown
| 7 | **[AUTO]** Gate 3→4 — 交付证据二选一（`pr_url` 或 `merge_commit`，按 `delivery_mode`）+ `tests_passed = true` → **AUTO-ADVANCE to Phase 4**。（PR 路径下，真正的合并闸门是平台必需检查 + 排队合并，不由本 workflow 判定；local_merge 路径下合并已在本 Step 完成） | — |
```

- [ ] **Step 4: Update Phase 4 Step 4 "Branch Finish" to branch on `delivery_mode`**

Find the numbered list starting at `### Phase 4 Step 4: Branch Finish` (steps 1-6). Insert a new step 2 that short-circuits merge-status detection for `local_merge`, and renumber the rest. Replace the whole numbered list (currently steps 1-6) with:

```markdown
1. Read from contract: `base_branch`, `branch`, `worktree_path`, `delivery_mode` (Phase 3 evidence)
   - Note: `worktree_path` follows the convention `.worktree/<branch-name>`
2. **`delivery_mode == "local_merge"`** → skip PR merge-status detection entirely (the merge already happened in Phase 3 Step 3); go straight to Step 4 ("PR merged" cleanup sequence) below.
3. **`delivery_mode == "pr"` (or absent)** → detect PR merge status: `gf pr view <n>` → 读 **`mergedAt`**
   - `mergedAt` 非空 → 判定**已合并**
   - `mergedAt` 为空且 `state == Closed` → **无法判定**：`State` 把 `MERGED` alias 进
     `Closed`，"关了没合"与"已合并"在 `state` 上完全同形；而 GitLab/GitCode 可能不返回
     `mergedAt`，此时 `None` 只代表"平台未上报"。→ ✋ **必须问用户**，给出 PR URL 与 state，
     由人确认后才允许删分支。**绝不靠推断删除**（`git branch -d` 的"未合并则失败"只是最后兜底，
     不是判定手段）
   - `state == Open` → 未合并，走下面第 5 步
4. **PR merged, or `delivery_mode == "local_merge"`** → present confirmation prompt:
   - `cd` to main working tree (`git rev-parse --git-common-dir` parent)
   - **Re-run the Worktree Preflight classification** before touching branch state: `git checkout`/`git pull` are blocked by the same dirty tree that blocks `git worktree add`. Bucket A is empty by now (its commit is merged), except for `local_merge` where the merge commit already carries it; anything left is bucket B → ✋ PAUSE, never auto-commit, never delete.
   - If `unresolved_dirty_paths` is non-empty, list it here — those are files Phase 3 deliberately left in the main tree.
   - `git checkout $base_branch && git pull origin $base_branch`
   - `git branch -d $branch`
   - `git worktree remove $worktree_path && git worktree prune`
   - `git fetch --prune origin`
   - Set `branch_cleaned = true`
5. **PR not merged** (PR path only) → output "PR 待合并，分支和 worktree 保留", set `branch_cleaned = false`
6. **Error tolerance:** if `git branch -d` fails (unmerged local commits), warn and preserve; do not block archive
7. **Missing fields:** if `base_branch` or `worktree_path` empty (old contract / fast mode), skip cleanup silently
```

- [ ] **Step 5: Verify Markdown table alignment and internal consistency**

Run: `grep -n 'delivery_mode\|merge_commit' skills/gf-workflow/SKILL.md`
Expected: hits in the Phase 3 Step 3 row, Step 6 row, Step 7 row, and the new Phase 4 Step 4 numbered list — no other stale mentions of an unconditional `pr_url` requirement remain in Phase 3/4 (a plain `grep -n pr_url` should show only conditional/branching mentions).

Proofread the rendered Markdown (visually re-read the two edited sections) to confirm tables still parse as tables (no broken `|` columns from the multi-sentence cell added in Step 1).

- [ ] **Step 6: Commit**

```bash
git add skills/gf-workflow/SKILL.md
git commit -m "feat(gf-workflow): Phase 3 delivery choice + Phase 4 Branch Finish local_merge path"
```

---

### Task 4: Final cross-file consistency check

**Files:**
- Read-only: `skills/gf-workflow/{SKILL.md,gates.md,contract.schema.json}`

**Interfaces:**
- Consumes: outputs of Tasks 1-3

- [ ] **Step 1: Confirm schema/gates/SKILL agree on field names**

Run:
```bash
grep -c 'delivery_mode' skills/gf-workflow/contract.schema.json skills/gf-workflow/gates.md skills/gf-workflow/SKILL.md
grep -c 'merge_commit' skills/gf-workflow/contract.schema.json skills/gf-workflow/gates.md skills/gf-workflow/SKILL.md
```
Expected: nonzero count in all three files for both field names (no file missed).

- [ ] **Step 2: Confirm `.claude/skills/gf-workflow/` was NOT touched**

Run: `git status --porcelain .claude/skills/gf-workflow/`
Expected: empty output (no changes to the copy — out of scope per `CLAUDE.md`).

- [ ] **Step 3: Validate schema is still parseable**

Run: `python3 -m json.tool skills/gf-workflow/contract.schema.json > /dev/null && echo VALID`
Expected: `VALID`

- [ ] **Step 4: No commit needed** — this task is verification-only; if any check fails, fix the relevant file from Task 1-3 and re-commit there.
