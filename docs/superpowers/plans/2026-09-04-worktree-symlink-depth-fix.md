# Worktree Symlink Depth Fix Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the hardcoded `../../` relative depth in `skills/gf-workflow`'s Worktree Preflight symlink creation with a depth computed from `worktree_path`'s actual segment count, and add a post-creation existence check that gives a specific diagnostic instead of letting a broken link masquerade as "missing contract".

**Architecture:** Pure documentation change. `skills/gf-workflow/SKILL.md` (Phase 3 Step 1) and `skills/gf-workflow/references.md` (Worktree Preflight example + explanatory prose) both embed the same shell snippet as instructional text for an LLM orchestrator to execute verbatim — there is no compiled code path. The fix swaps the hardcoded `../../` for a `segs+1` formula and appends a `test -d` guard, in both places, plus a documented rationale so future edits don't regress the depth count.

**Tech Stack:** Markdown, POSIX shell (documented commands only — no CI executes them directly).

**Spec:** `docs/superpowers/specs/2026-09-04-worktree-symlink-depth-fix-design.md`

## Global Constraints

- Edit only `skills/gf-workflow/SKILL.md` and `skills/gf-workflow/references.md` under the repo root — **never** `.claude/skills/gf-workflow/*` (that is a runtime copy, not the source; per `CLAUDE.md`).
- Formula: `ups = (number of "/"-separated segments in worktree_path) + 1` — verified empirically in the design doc for both a 2-segment (`.worktree/foo`) and 3-segment (`.worktree/feat/89-desc`) `worktree_path`.
- Do not touch `deny.toml`, `.pre-commit-config.yaml`, or `rust-toolchain.toml`.
- This is a docs-only change — per `CLAUDE.md` no Rust build/test/clippy is required; validate via proofreading + `make check-agent-sync`.
- No commit without explicit user permission (per `CLAUDE.md`) — this plan stops short of committing; committing happens under `gf-workflow` Phase 3's own governed flow (worktree preflight + delivery choice), not as a step here.

---

### Task 1: Replace hardcoded depth in `SKILL.md` Phase 3 Step 1

**Files:**
- Modify: `skills/gf-workflow/SKILL.md:344`

**Interfaces:**
- Consumes: nothing (standalone doc edit)
- Produces: the corrected symlink-creation command block that Task 3's proofread checks against `references.md`'s Task 2 output for consistency

- [ ] **Step 1: Read the current line 344 in full**

Run: `sed -n '344p' skills/gf-workflow/SKILL.md`

Confirm it still contains the literal substring:
```
mkdir -p <worktree-path>/.cache && ln -s ../../.cache/workflows <worktree-path>/.cache/workflows && ln -s ../../.claude <worktree-path>/.claude
```
If the substring is gone (someone already edited it), stop and re-plan — do not blind-replace.

- [ ] **Step 2: Replace the symlink sub-clause**

Replace this exact substring:
```
`mkdir -p <worktree-path>/.cache && ln -s ../../.cache/workflows <worktree-path>/.cache/workflows && ln -s ../../.claude <worktree-path>/.claude`
```
with:
```
`segs=$(awk -F/ '{print NF}' <<< "<worktree-path>"); ups=$((segs + 1)); rel=$(printf '../%.0s' $(seq 1 "$ups")); mkdir -p <worktree-path>/.cache && ln -s "${rel}.cache/workflows" <worktree-path>/.cache/workflows && ln -s "${rel}.claude" <worktree-path>/.claude; test -d <worktree-path>/.cache/workflows || { echo "ABORT: symlink depth miscalculated — worktree_path=<worktree-path> segs=$segs ups=$ups, expected to resolve to repo-root .cache/workflows but did not. Check worktree_path follows the .worktree/<branch-name> convention."; exit 1; }`
```

This keeps the sentence's surrounding prose ("**After worktree creation**: symlink shared directories...") unchanged — only the backtick-quoted command itself changes. The self-check replaces the *next* sentence's job partially — leave "**Immediately exclude them**..." as-is; the `info/exclude` write is unrelated to depth.

- [ ] **Step 3: Verify the line renders as valid Markdown**

Run: `grep -c '^| 1 |' skills/gf-workflow/SKILL.md` — expect unchanged count (`1`), confirming the table row structure (pipe count) wasn't broken by the edit. Also run:
`awk -F'|' 'NR==344{print NF}' skills/gf-workflow/SKILL.md`
and compare against the same command run before the edit (record the "before" count in Step 1) — they must match, since the edit only touches a backtick-quoted span within one existing table cell, not the cell boundaries.

- [ ] **Step 4: Commit**

Do not commit yet — commits in this workflow happen through `gf-workflow`'s own governed Phase 3 flow. Leave the change staged in the working tree for the orchestrator to handle.

---

### Task 2: Update `references.md` Worktree Preflight example + add depth-formula rationale

**Files:**
- Modify: `skills/gf-workflow/references.md:96-134` (example code block)
- Modify: `skills/gf-workflow/references.md:200` (insert new subsection before "Why These Symlinks Must Never Reach the Main Branch")

**Interfaces:**
- Consumes: the formula from Task 1's Step 2 (must stay textually identical in spirit — same `segs`/`ups`/`rel` variable names — so a reader comparing `SKILL.md` and `references.md` sees one consistent pattern)
- Produces: the worked example + rationale block that Task 3's proofread reads end-to-end

- [ ] **Step 1: Read current lines 95-135 to confirm exact text**

Run: `sed -n '95,135p' skills/gf-workflow/references.md`

Confirm the branch name used is the single-segment `feat-146-worktree-path` (dash-joined, no `/`) — this is the misleading example the fix must replace with a real slash-containing branch name.

- [ ] **Step 2: Replace the example block**

Replace lines 96-134 (the fenced ` ```bash ... ``` ` block) with:

````markdown
```bash
# Phase 3 Step 1: preflight, then create worktree
git status --porcelain                      # classify before forking (see below)
git worktree add .worktree/feat/146-worktree-path -b feat/146-worktree-path main

# Carry this workflow's Phase 1/2 documents INTO the worktree, then commit on the
# feature branch (structure-preserving and portable — macOS has no `cp --parents`)
WORKTREE_PATH=".worktree/feat/146-worktree-path"
for f in docs/superpowers/specs/146-x-design.md docs/superpowers/plans/146-x.md; do
  mkdir -p "$WORKTREE_PATH/$(dirname "$f")"
  cp "$f" "$WORKTREE_PATH/$f"
done

# Backstop: assert every contract-referenced document really landed in the worktree
for f in docs/superpowers/specs/146-x-design.md docs/superpowers/plans/146-x.md; do
  test -f "$WORKTREE_PATH/$f" || { echo "ABORT: $f missing in worktree"; exit 1; }
done

# Symlink shared directories (workflow contracts + Claude config).
# Depth is computed, not hardcoded: worktree_path can be multi-segment
# (branch names follow feat/<issue-number>-<short-description>, so
# .worktree/<branch-name> is routinely 2+ segments deep). See "Why the
# Symlink Depth Is Computed, Not Hardcoded" below for the formula and
# the empirical proof.
segs=$(awk -F/ '{print NF}' <<< "$WORKTREE_PATH")
ups=$((segs + 1))
rel=$(printf '../%.0s' $(seq 1 "$ups"))
mkdir -p "$WORKTREE_PATH/.cache"
ln -s "${rel}.cache/workflows" "$WORKTREE_PATH/.cache/workflows"
ln -s "${rel}.claude" "$WORKTREE_PATH/.claude"

# Existence self-check — a dangling symlink still passes `test -e`, so verify
# the *resolved target* is a real directory. A failure here means the depth
# formula or worktree_path itself is wrong, not that the contract is missing.
test -d "$WORKTREE_PATH/.cache/workflows" || {
  echo "ABORT: symlink depth miscalculated — worktree_path=$WORKTREE_PATH segs=$segs ups=$ups"
  echo "Expected to resolve to repo-root .cache/workflows but did not."
  exit 1
}

# Exclude them from git tracking — writes to the COMMON git dir's info/exclude
# (verified: worktrees do NOT have a per-worktree info/exclude; this file is shared
# by the main tree + all worktrees of this local clone), so it protects every
# worktree, not just this one, without touching the project's own .gitignore.
EXCLUDE_FILE="$(cd "$WORKTREE_PATH" && git rev-parse --git-common-dir)/info/exclude"
grep -qxF '.cache/workflows' "$EXCLUDE_FILE" || echo '.cache/workflows' >> "$EXCLUDE_FILE"
grep -qxF '.claude' "$EXCLUDE_FILE" || echo '.claude' >> "$EXCLUDE_FILE"

cd "$WORKTREE_PATH"
git add docs && git commit -m "docs(workflow): wf-2026-08-30-001 Phase 1-2 artifacts"
cd -
# Only now remove the main-tree copies, so the eventual merge cannot be blocked
rm docs/superpowers/specs/146-x-design.md docs/superpowers/plans/146-x.md

# Phase 4 Branch Finish: Remove worktree
git worktree remove "$WORKTREE_PATH"
```
````

- [ ] **Step 3: Verify the fenced block is well-formed**

Run: `awk '/^```bash$/{c++} /^```$/{c++} END{print c}' skills/gf-workflow/references.md`
Expected: an even number (every opening fence has a matching close). Compare against the same command run before Step 2's edit — the count must be unchanged (one block replaced by one block, not a net new fence).

- [ ] **Step 4: Insert the depth-formula rationale subsection**

Locate line `### Why These Symlinks Must Never Reach the Main Branch` (currently line 200, may shift after Step 2's edit — search for the exact heading text rather than trusting the line number). Insert a new subsection immediately **before** it:

```markdown
### Why the Symlink Depth Is Computed, Not Hardcoded

A relative symlink resolves starting from the directory that *contains* the
symlink file, not from `worktree_path` itself. The symlinks above live at
`$WORKTREE_PATH/.cache/workflows` and `$WORKTREE_PATH/.claude`, so their
containing directory (`$WORKTREE_PATH/.cache/`) is **one segment deeper**
than `$WORKTREE_PATH`. The number of `../` needed to reach the repo root is
therefore:

```
ups = (number of "/"-separated segments in worktree_path) + 1
```

**Verified empirically** (not inferred from documentation) with real
`mkdir` + `ln -s`:

| `worktree_path` | segments | `ups` | Hardcoded `../../` resolves to |
|---|---|---|---|
| `.worktree/foo` (single-segment — the case the old hardcoded value was written for) | 2 | 3 | `.worktree/` — **not the repo root** |
| `.worktree/feat/89-desc` (branch name contains `/`, per the `feat/<issue-number>-<short-description>` convention) | 3 | 4 | `.worktree/feat/` — **not the repo root** |

The old hardcoded `../../` (2 levels) was wrong even for the single-segment
case it was presumably written for — it only reaches `.worktree/`, one level
short of the repo root, in every case. A branch name containing `/` (the
routine case, not an edge case — see the naming convention above) simply
made the shortfall larger and easier to hit. The `segs + 1` formula is
correct for both, and the post-creation `test -d "$WORKTREE_PATH/.cache/workflows"`
check catches any future regression of this formula by refusing to proceed
silently — a dangling symlink otherwise looks identical to a missing
contract to every downstream reader (see Issue #322's real-world report:
this exact ambiguity cost significant debugging time downstream).
```

- [ ] **Step 5: Verify heading structure**

Run: `grep -n '^### ' skills/gf-workflow/references.md` and confirm the new `### Why the Symlink Depth Is Computed, Not Hardcoded` heading appears exactly once, immediately before `### Why These Symlinks Must Never Reach the Main Branch`, with no duplicate or orphaned heading.

- [ ] **Step 6: Commit**

Do not commit yet — same as Task 1, leave staged for `gf-workflow`'s governed Phase 3 flow.

---

### Task 3: Validation pass

**Files:**
- Read-only: `skills/gf-workflow/SKILL.md`, `skills/gf-workflow/references.md`

**Interfaces:**
- Consumes: Task 1 and Task 2's final file states
- Produces: a pass/fail verification record (no new files)

- [ ] **Step 1: Re-run the empirical symlink test against the exact formula now documented**

```bash
cd /tmp && rm -rf gf-322-verify && mkdir gf-322-verify && cd gf-322-verify
mkdir -p repo-root/.cache/workflows
cd repo-root

for WORKTREE_PATH in ".worktree/foo" ".worktree/feat/89-desc"; do
  mkdir -p "$WORKTREE_PATH/.cache"
  segs=$(awk -F/ '{print NF}' <<< "$WORKTREE_PATH")
  ups=$((segs + 1))
  rel=$(printf '../%.0s' $(seq 1 "$ups"))
  ln -sf "${rel}.cache/workflows" "$WORKTREE_PATH/.cache/workflows"
  resolved=$(readlink -f "$WORKTREE_PATH/.cache/workflows")
  expected="$(pwd)/.cache/workflows"
  [ "$resolved" = "$expected" ] && echo "PASS: $WORKTREE_PATH -> $resolved" || echo "FAIL: $WORKTREE_PATH -> $resolved (expected $expected)"
done
cd / && rm -rf /tmp/gf-322-verify
```

Expected: both cases print `PASS`. If either prints `FAIL`, the formula in Task 1/Task 2 has a bug — stop and fix before proceeding.

- [ ] **Step 2: Proofread both changed files end-to-end**

Read `skills/gf-workflow/SKILL.md` lines 340-350 and `skills/gf-workflow/references.md` lines 90-240 in full. Confirm:
- No leftover reference to the old hardcoded `../../` anywhere in either file (`grep -n '\.\./\.\.' skills/gf-workflow/SKILL.md skills/gf-workflow/references.md` should return zero matches, since the only prior occurrences were the two now-replaced ones — the `.git-common-dir` and `info/exclude` lines don't contain this substring).
- The `SKILL.md` table row (Phase 3 Step 1) still reads as one coherent sentence sequence — no broken markdown table pipes.
- The `references.md` example block and the new rationale subsection are internally consistent (same variable names: `segs`, `ups`, `rel`, `WORKTREE_PATH`).

- [ ] **Step 3: Run the project's doc-relevant Make target**

Run: `make check-agent-sync`
Expected: passes (verifies `CLAUDE.md` exists — unaffected by this change, but per `CLAUDE.md`'s own validation guidance for skill edits, run it as the baseline check).

- [ ] **Step 4: Confirm no unrelated files changed**

Run: `git status --porcelain skills/gf-workflow/`
Expected: exactly two modified files — `SKILL.md` and `references.md`. No new/deleted files, no changes outside `skills/gf-workflow/`.

- [ ] **Step 5: Do not commit**

Leave the working tree as-is. Committing, Issue-closing, and PR creation are governed by `gf-workflow` Phase 3/4 (Worktree Preflight, delivery choice) — this plan's scope ends at a verified, staged-but-uncommitted change.
