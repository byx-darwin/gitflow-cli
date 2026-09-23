# Issue Triage Report — 2026-09-21

**Context**: `gf-workflow` Phase 4 post-delivery check for Issue #346 (feat(skills): diff 交互式审阅网页渲染 — `scripts/render-diff-review.py`), merged into `dev` at commit `1b6bf81`.

**Run**: `gf-issue-triage` skill (report-generation mode only — **no labels were applied**, per Phase 4 instruction). Full `gf issue list --state open` fetch, `pagination.truncated: false`, 27 open Issues returned — coverage complete.

**Scope**: this run is targeted, not a full batch re-triage. It focuses on (a) Issues that reference #346 and may now be stale/inaccurate now that #346 is *actually delivered* rather than only designed, and (b) a spot-check of the rest of the open backlog for obviously wrong/stale labels. Full batch re-triage (all 27 Issues re-classified from scratch) was not performed — none showed a `type:*`/`priority:*` mismatch severe enough to warrant it, see "Spot-check" below.

## Findings

### 1. #399 — `fix(scripts): diff 审阅页注释卡片噪音 + hunk 边界丢失` — priority likely underrated (`low` → recommend `medium`)

Verified against `scripts/render-diff-review.py` (current `dev` tip):

- `_render_annotation_cards` (line ~408) does iterate every line in `f.get("lines", [])`, including unchanged context lines, and does read `f.get("pseudocode")`/`f.get("call_tree")` — both are **file-level** fields set in `_new_file_entry` (line ~23) — inside that **line-level** loop. The Issue's root-cause description is accurate: once #398 fills these fields, an 800-line file will render the same pseudocode/call-tree block 800 times.
- `_render_file_section` — confirmed hunks are parsed with boundaries (`_parse_hunks`) but the file-level renderer joins all hunks' lines with no separator, so non-adjacent hunks appear visually contiguous. Also accurate.
- Current labels: `type:bug` + `priority:low`. Given (a) this already produces oversized pages today (1.38 MB observed for a ~2500-line diff) which works against #346's core "offline, quickly reviewable" value proposition, and (b) the bug will **multiply in effect** the moment #398 ships (turning a cosmetic noise issue into a correctness/duplication issue), recommend bumping to **`priority:medium`**. `type:bug` is correct and unchanged.
- No `triage:done` label yet (issue is 12 hours old at filing) — recommend adding once priority is finalized.

**Recommended action (not applied)**: `gf issue add-label 399 --label "priority:medium"` (remove `priority:low` first) `--label "triage:done"`. Do not touch `type:bug`.

### 2. #397 — `feat(workflow): 接入 diff 交互式审阅网页渲染到 gf-workflow Phase 4` — AC item on output-path convention is now answerable, not still open

#397's Acceptance Criteria includes: *"渲染产物的输出路径遵循 #345/#344 已建立的 `.cache/` 或 `docs/` 约定之一，是明确决定的，不是临时拍板"* — phrased as an open decision between two precedents (`.cache/` per #345's `render-workflow-dashboard.py`, or `docs/*-report-*.md` per #344's `code-review-report-*.md`/`security-report-*.md`/`regression-report-*.md`).

Now that #346 is actually built, this is no longer a two-way open choice: `render-diff-review.py`'s `main()` (verified, lines ~463-495) hardcodes the convention as `.cache/diff-review/<sanitized-range>.json` and `.cache/diff-review/<sanitized-range>.html`, matching the `Makefile`'s `render-diff-review` target (`Makefile:191-193`). This is the `.cache/` branch of #345's precedent, **not** the `docs/*-report-*.md` branch #344 established. The AC is still correct and satisfiable, but #397's Context section (written before #346 merged) presents this as symmetrical between the two options — it isn't anymore; #346 already picked one. Recommend adding a comment to #397 (not done here — report-only) pointing at `render-diff-review.py`'s actual output path so Phase 4 integration work doesn't re-litigate the choice; the remaining open question for #397 is only the *trigger condition* (always-run vs. change-surface-gated, per #344's pattern) and *whether/how the render output gets referenced from a review comment*.

Labels (`type:enhancement`, `priority:low`, no `triage:done`) are still appropriate — this is genuinely a nice-to-have orchestration follow-up, not urgent. No relabel recommended beyond adding `triage:done`.

### 3. #398 — `feat(skills): diff 审阅页伪代码/调用树自动生成` — still accurate, no changes needed

#398's premise ("`annotations.json` 预留了 `pseudocode`/`call_tree` 字段，`#346` 交付时恒为 `None`") is confirmed by source: `_new_file_entry` sets both to `None` and nothing else in `render-diff-review.py` ever assigns them. The design tension called out in the Issue body (tree-sitter/AST vs. LLM vs. #346's "zero third-party dependency, offline" stance) is real and unresolved by #346. Labels (`type:enhancement`, `priority:low`, no `triage:done`) remain correct. Recommend adding `triage:done` only — no relabel.

### 4. Spot-check of remaining 24 open Issues

Scanned all labels/dates for the rest of the open backlog (Issues #227, #240, #188, #343, #347, #349, #370, #371, #380, #382–#393, #394, #395, #396) — no stale or mismatched `type:*`/`priority:*` labels found. All carry `triage:done` and their type/priority combinations match their descriptions (e.g. #394/#396 are `type:bug`+`priority:high` for real reproduced crashes with repro steps; the #382-family Jev proposals are consistently `type:feature`+`priority:low`/`medium` as speculative roadmap items with no urgency signal). No action recommended for this group.

## Summary Table

| Issue | Type | Current Priority | Recommended Priority | `triage:done` | Action |
|---|---|---|---|---|---|
| #399 | `type:bug` | `low` | **`medium`** | missing | relabel priority + add `triage:done` |
| #397 | `type:enhancement` | `low` | `low` (no change) | missing | add `triage:done`; comment clarifying `.cache/diff-review/` convention is now settled |
| #398 | `type:enhancement` | `low` | `low` (no change) | missing | add `triage:done` |
| all others (24) | — | — | — | present | none |

**Findings count: 3** (one priority relabel recommendation for #399, one clarifying-comment recommendation for #397, plus the `triage:done` gap common to all three newly-filed Issues). No labels were changed by this run — all changes above are recommendations for a human or a subsequent explicitly-authorized run to apply.
