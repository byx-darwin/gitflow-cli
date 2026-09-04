# gf-workflow — Reference

Operations reference for the `gf-workflow` orchestrator. Main execution flow: see `SKILL.md`.

## Contract Operations API

### Create Contract

```bash
gf workflow create --title "<issue_title>" --mode <full|fast>
# 输出分配的 ID 与合同路径，例如：
# ✅ Workflow 已创建: wf-2026-08-08-002
#    合同: .cache/workflows/active/wf-2026-08-08-002.json
```

ID 由 CLI 自动分配（Issue #142）：扫描 `active/` 与 `archive/` 全部月份目录取当日
最大序号的下一个空位，并用 O_EXCL 原子写入——归档后序号不复用，并发创建不产生
同名合同。**不要手写 `WORKFLOW_ID` 或按 active 数量计数**（旧逻辑已废弃）。

### Update Contract (on Phase completion)

```bash
COMPLETED_AT="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
jq --arg phase "$PHASE_NUM" \
   --arg status "complete" \
   --arg completed_at "$COMPLETED_AT" \
   --argjson evidence "$EVIDENCE_JSON" \
  '.phases[$phase].status = $status | .phases[$phase].completed_at = $completed_at | .phases[$phase].evidence = $evidence | .updated_at = $completed_at' \
  ".cache/workflows/active/${WORKFLOW_ID}.json" > "${WORKFLOW_ID}.tmp" \
  && mv "${WORKFLOW_ID}.tmp" ".cache/workflows/active/${WORKFLOW_ID}.json"
```

### Advance to Next Phase (after gate passes)

```bash
jq --arg next "$((PHASE_NUM + 1))" \
   --arg started_at "$COMPLETED_AT" \
  '.current_phase = ($next | tonumber) | .phases[$next].status = "in_progress" | .phases[$next].started_at = $started_at | .updated_at = $started_at' \
  ".cache/workflows/active/${WORKFLOW_ID}.json" > "${WORKFLOW_ID}.tmp" \
  && mv "${WORKFLOW_ID}.tmp" ".cache/workflows/active/${WORKFLOW_ID}.json"
```

### Read Contract

```bash
jq '.current_phase, .phases | to_entries[] | select(.value.status == "in_progress") | .key' \
  ".cache/workflows/active/${WORKFLOW_ID}.json"
```

## Cross-Session Recovery

Workflows may be interrupted at any Phase. New sessions recover via contract + plan doc.

```
New Session Starts
    ↓
1. List .cache/workflows/active/*.json → find workflow with status != "complete"
2. Read current_phase and evidence
3. Load context based on mode and current_phase:
   • Phase 1: No doc needed (start fresh)
   • Phase 2: Read design_doc_path; check mode for Phase 1 exemptions
   • Phase 3: Read spec_path (plan document); check mode for fast skip
   • Phase 4: Read pr_url + review reports; use get_phase4_steps(mode) to determine remaining steps
4. Resume from current_phase, follow auto-trigger rules
```

**Key principles:** Contract = state machine. Plan doc = execution manual. Design doc = requirement source. All state in contract and documents, no external dependencies.

## Multi-Workflow Concurrency

```
.cache/workflows/
├── active/
│   ├── wf-2026-07-09-001.json  ← feat: TOON
│   └── wf-2026-07-09-002.json  ← fix: pr merge
└── archive/
    └── 2026-07/
        └── wf-2026-07-08-001.json
```

Each workflow uses its own worktree, branch, and contract file — no interference.

### Worktree Path Convention

All worktrees are created at a fixed location within the project: `.worktree/<branch-name>`

- **Path pattern:** `.worktree/feat/<issue-number>-<short-description>`
- **Branch naming:** `feat/<issue-number>-<short-description>` (e.g., `feat/146-worktree-path`)
- **Gitignore:** `.worktree/` is included in `.gitignore`
- **Benefits:**
  - Worktrees remain within the project directory for easy discovery and cleanup
  - Predictable paths for Phase 4 Branch Finish automation
  - No orphaned worktrees in parent directories

**Example:**
```bash
# Phase 3 Step 1: preflight, then create worktree
git status --porcelain                      # classify before forking (see below)
git worktree add .worktree/feat-146-worktree-path -b feat/146-worktree-path main

# Carry this workflow's Phase 1/2 documents INTO the worktree, then commit on the
# feature branch (structure-preserving and portable — macOS has no `cp --parents`)
for f in docs/superpowers/specs/146-x-design.md docs/superpowers/plans/146-x.md; do
  mkdir -p ".worktree/feat-146-worktree-path/$(dirname "$f")"
  cp "$f" ".worktree/feat-146-worktree-path/$f"
done

# Backstop: assert every contract-referenced document really landed in the worktree
for f in docs/superpowers/specs/146-x-design.md docs/superpowers/plans/146-x.md; do
  test -f ".worktree/feat-146-worktree-path/$f" || { echo "ABORT: $f missing in worktree"; exit 1; }
done

# Symlink shared directories (workflow contracts + Claude config)
mkdir -p .worktree/feat-146-worktree-path/.cache
ln -s ../../.cache/workflows .worktree/feat-146-worktree-path/.cache/workflows
ln -s ../../.claude .worktree/feat-146-worktree-path/.claude

# Exclude them from git tracking — writes to the COMMON git dir's info/exclude
# (verified: worktrees do NOT have a per-worktree info/exclude; this file is shared
# by the main tree + all worktrees of this local clone), so it protects every
# worktree, not just this one, without touching the project's own .gitignore.
EXCLUDE_FILE="$(cd .worktree/feat-146-worktree-path && git rev-parse --git-common-dir)/info/exclude"
grep -qxF '.cache/workflows' "$EXCLUDE_FILE" || echo '.cache/workflows' >> "$EXCLUDE_FILE"
grep -qxF '.claude' "$EXCLUDE_FILE" || echo '.claude' >> "$EXCLUDE_FILE"

cd .worktree/feat-146-worktree-path
git add docs && git commit -m "docs(workflow): wf-2026-08-30-001 Phase 1-2 artifacts"
cd -
# Only now remove the main-tree copies, so the eventual merge cannot be blocked
rm docs/superpowers/specs/146-x-design.md docs/superpowers/plans/146-x.md

# Phase 4 Branch Finish: Remove worktree
git worktree remove .worktree/feat-146-worktree-path
```

### Worktree Preflight (Phase 3 Step 1)

`git worktree add` checks out a **committed** state. Anything uncommitted in the main
working tree stays behind — including this workflow's own Phase 1/2 documents, which the
contract references by path. The contract itself survives (it is symlinked), so without a
preflight the executor is handed a `spec_path` that does not exist in its working directory
and plans against a document it cannot read.

Run this before `git worktree add`. `git status --porcelain` empty ⇒ record
`worktree_preflight = "clean"` and proceed.

| Bucket | Match | Action |
|---|---|---|
| **A — workflow artifacts** | paths equal to contract `design_doc_path`, `spec_path`, `ticket_refs[]` | ✋ PAUSE and ask before committing (see below) |
| **B — unrelated dirty files** | everything else reported by `git status` | ✋ PAUSE and ask |
| **C — ignored** | covered by `.gitignore` (`.cache/`, `.worktree/`, `target/`) | Nothing to do (`git status` already omits them) |

**Bucket A requires explicit commit permission too** — per project policy, nothing is
committed without the user's go-ahead, and Phase 1/2 design docs are no exception just
because the workflow itself produced them. `git worktree add` only forks a **committed**
state, so these docs must be committed for the executor to see them — but the commit
itself still needs a yes. Before committing, show the paths and ask:

```
即将把以下设计文档提交到 feature 分支 <branch>，worktree 才能读取到它们：
  <bucket A paths>
是否提交？
  1) 提交    2) 中止（不创建 worktree，回 Phase 2 处理）
```

Choice 2 ⇒ do not run `git worktree add`; leave the contract in Phase 3 for resume.
Choice 1 ⇒ commit to the feature branch, never to `base_branch`: entering Phase 3 must
not leave commits on a shared branch as a side effect.

**Why the main-tree copies must be removed after committing.** Verified behaviour: if the
untracked originals stay, the later merge fails with
`untracked working tree files would be overwritten by merge … Aborting`. Removing them
after the worktree commit is safe because the content is preserved in git and comes back on
merge. Remove only after asserting the commit succeeded.

**Bucket B is never auto-committed and never deleted** — it is somebody else's work. Offer
exactly these four options, and record the choice in `worktree_preflight`:

```
主工作区有与本工作流无关的改动：
  <paths from git status --porcelain>

它们不会进入 worktree。请选择：
  1) 单独提交    2) git stash    3) 留在主工作区继续    4) 中止
```

Choice 3 ⇒ `worktree_preflight = "user_left_dirty"` and list the paths in
`unresolved_dirty_paths`, so Phase 4 and any later reader knows what was deliberately
abandoned in the main tree. Choice 2 ⇒ `"user_stashed"`. Choice 4 ⇒ `"aborted"`, keep
the contract in Phase 3 for resume.

A non-empty bucket A that the user approved and that was committed ⇒ `"artifacts_carried"`.
A non-empty bucket A that the user declined to commit ⇒ `"aborted"`, same as Bucket B
choice 4 — the contract stays in Phase 3 for resume.

**Every execution mode must run this preflight.** Modes ① and ② let the *executor* create
the worktree, so the orchestrator cannot rely on having checked the tree itself — the
handoff text must carry these steps verbatim. See `Phase 3 Execution Modes` below.

## Lifecycle Management

| Status | Location | Retention | Cleanup |
|--------|----------|-----------|---------|
| active | `.cache/workflows/active/` | In progress | Move to archive on completion |
| archive | `.cache/workflows/archive/YYYY-MM/` | 90 days | `gitflow workflow cleanup --older-than 90` |

## CLI Integration

```bash
gitflow workflow create --title "<issue_title>" --mode <full|fast>  # 创建合同（自动分配当日不重复 ID）
gitflow workflow list                      # List active workflows
gitflow workflow status <workflow_id>      # View contract details
gitflow workflow archive <workflow_id>     # Archive completed workflow（目标已存在时拒绝覆盖）
gitflow workflow cleanup --older-than 90   # Clean up expired archives
```

## Branch Finish Operations

Phase 4 Step 4 commands. All operations are local-only (no push).

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

## Dual-Source Skill Resolution (Issue #141)

gf-workflow runs on ONE external skill source: `superpowers` or `mattpocock/skills`.
SKILL.md phase steps use role aliases; actual names resolve from the tables below.
**This section is the single point of maintenance** — upstream renames are fixed here only.

### Sentinels (shared authoritative definition)

Mirrored by the Rust constants in `apps/cli/src/commands/skills.rs`
(`SUPERPOWERS_PLUGIN_PREFIX` / `MATTPOCOCK_PLUGIN_PREFIX` /
`SUPERPOWERS_BARE_SENTINELS` / `MATTPOCOCK_BARE_SENTINELS`) used by install-time
Step 0. Change both sides together.

| Source | Namespaced form | Bare form (double hit required) |
|---|---|---|
| superpowers | `superpowers:brainstorming` | `brainstorming` + `writing-plans` |
| mattpocock | `mattpocock-skills:to-spec` + `mattpocock-skills:grilling` | `to-spec` + `grilling` |

Bare forms cover skills.sh / symlink installs. A partial hit (e.g. only `to-spec`)
counts as absent; report which sentinel is missing.

### Detection & Recording

- Mechanism: introspect the session available-skills list at Bootstrap, BEFORE the
  contract exists. Filesystem probing is diagnostics-only.
- Both present → ask the user which source this workflow uses (no default priority).
- Neither present → ask: continue inline (`skill_source: "inline"`) or abort (no contract).
- Record after contract creation:

```bash
jq --arg src "<superpowers|mattpocock|inline>" \
   '.skill_source = $src | .updated_at = (now | todate)' \
   ".cache/workflows/active/${WORKFLOW_ID}.json" > tmp && mv tmp ".cache/workflows/active/${WORKFLOW_ID}.json"
```

- Resume: reuse `skill_source` from the contract; re-verify sentinels are still present;
  if vanished, re-run the neither-present prompt.

### Dual-Source Mapping Table

| Role alias | superpowers | mattpocock | Invocation form |
|---|---|---|---|
| Clarification | `brainstorming` | `grilling` | model-invoked / model-invoked |
| Spec | (merged into brainstorming design doc) | ✋ `/to-spec` (local-only) | — / user-invoked |
| Issue creation | `gf-issue-create` | `gf-issue-create` (unchanged; authority unified) | gf CLI |
| Issue review | `gf-issue-review` | `gf-issue-review` (unchanged) | gf CLI |
| Planning | `writing-plans` | ✋ `/to-tickets` | model-invoked / user-invoked |
| Quality gate | `gf-quality` | `gf-quality` (unchanged) | gf CLI |
| Execution engine | `subagent-driven-development` (same-session) / `executing-plans` (new window) / background agent — per GO gate | ✋ `/implement` per ticket (internal `/tdd` mandatory) | per mode / user-invoked |
| Execution review | SDD built-in two-stage review | `code-review` (driven inside `/implement`) | — |
| Delivery review | `gf-review` | `gf-review` (unchanged; no extra code-review pass) | gf skill |
| Triage (full mode) | `gf-issue-triage` | `gf-issue-triage` (unchanged; mattpocock `triage` NOT adopted) | gf skill |
| Pipeline analysis | `gf-pipeline-analyzer` | (unchanged) | gf skill |

### Source Branch Semantics

**Invariants (both sources):** four-phase state machine, three gates, contract evidence
semantics, all `gf-*` steps, mandatory TDD + code review, mode matrix (full/standard/fast).

**mattpocock path:**

- **Prerequisite:** `docs/agents/issue-tracker.md` exists (`setup-mat-pocock-skills`
  output). Missing → ask the user: run `setup-mat-pocock-skills` now (one-time) or abort.
- **Phase 1:** `grilling` (auto) → ✋ PAUSE prompting:

  > 请运行 `/to-spec`：综合当前对话撰写 spec，写入本地文件
  > `docs/specs/<workflow-id>-spec.md`。**只写本地，不发布 tracker、不打标签。**

  Verify the local spec exists afterwards. **Fallback** (constraint failed / skill refused):
  the orchestrator writes the design doc itself from the grilling record, bypassing `to-spec`.
  Then `gf-issue-create` creates the Issue (authority unified — no duplicate) and
  `gf-issue-review` reviews it. Evidence: `issue_url`, `comment_id`, `design_doc_path`.
- **Phase 2:** ✋ PAUSE prompting `/to-tickets` with the Phase 1 spec reference.
  `to-tickets` publishes tickets per the configured tracker (local `.scratch/<feature>/issues/`
  files or real tracker issues) and includes its own breakdown quiz. Its rule "do NOT close
  or modify any parent issue" is compatible with gf-workflow. Orchestrator records
  `ticket_refs` (paths/URLs) and sets `spec_path` = the Phase 1 spec file. Gate 2→3
  presents the ticket list + blocking edges.
- **Phase 3:** worktree per chosen execution mode; per ticket in dependency order
  (frontier): ✋ PAUSE → user runs `/implement` (internal `/tdd` + `/code-review` + commit);
  suggest `/clear` between tickets (context recovery via contract + ticket files).
  `gf-pr-create` with `Closes #<issue>`, then `make test`.
- **Phase 4:** identical to superpowers ([pipeline ∥ triage[full] ∥ gf-review] →
  dogfooding[full] → Branch Finish → archive). `code-review` already ran inside `/implement`.
- **Evidence mapping:** `design_doc_path` ← local spec file; `spec_path` ← same spec file
  (what `to-tickets` consumed); `ticket_refs` ← ticket paths/URLs.

**inline source (both absent, user chose to continue):** orchestrator performs each phase
inline — self-interview, self-written design doc/plan, TDD loop with a review subagent.
Evidence fields follow the superpowers shape. Degraded but explicit.

### Phase 3 Execution Modes (GO gate)

Gate 2→3 = plan approval + execution-mode choice. Same-session execution left the default
menu (SDD objection: approving a plan ≠ authorizing hours of autonomous subagent fan-out;
same-session SDD hijacks the conversation once started).

| Mode | Description | Availability |
|---|---|---|
| ① Background agent ⭐default | Dispatch with `isolation: worktree` + `run_in_background`; handoff = contract path + plan doc + engine instructions + **Worktree Preflight steps verbatim** (this executor creates the worktree, so the orchestrator's tree state was never checked); `task-notification` returns to the original window; executor writes evidence back to the contract | superpowers only (`/implement` is user-invoked → unusable on mattpocock) |
| ② Manual new window | Print opening guidance: worktree path (or creation command) + contract recovery command (`gf workflow status <id>` + plan doc path) + **Worktree Preflight steps verbatim**; new window creates branch/worktree itself and runs `executing-plans` (superpowers) or per-ticket `/implement` (mattpocock); user reports back, orchestrator verifies evidence | both sources |
| ③ Same-session | Current behavior: orchestrator creates worktree and drives the engine inline | explicit request only |

Quality compensation: `executing-plans` (light path) lacks per-task review → gates
compensate (`make test` before PR + Phase 4 `gf-review`). SDD carries per-task review built in.

### Worktree Location Convention (Issue #146)

All gf-workflow worktrees are created at a **fixed path**: `.worktree/<branch-name>`.

- Branch name format: `feat/<issue-number>-<short-description>`
- Full path example: `.worktree/feat/141-dual-skill-sources`
- `.worktree/` is in `.gitignore` → worktrees are automatically excluded from version control
- Phase 4 Branch Finish cleanup uses this predictable path for `git worktree remove`
- Background agents and new-window executors create worktrees at this same location
