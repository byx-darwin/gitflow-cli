# Code Review Report — Issue #399

- Date: 2026-09-22
- Delivered via: local merge into `dev` (no PR — merge commit `b645c9016a2c00b1e977dc3e0fd3c890e3b3c6cf`, range `2da839c..b645c90`)
- Commits: `3bd7cf3` (fix), `f7e526c` (follow-up cleanup from independent code review)

## Scope

`scripts/render-diff-review.py` + `scripts/tests/test_render_diff_review.py`, fixing two bugs in the diff-review HTML renderer established by Issue #346:

1. Annotation cards (`pseudocode`/`call_tree`) were rendered once per diff line instead of once per file, producing noise (a wall of "暂无说明" cards on unrelated context lines) and a latent duplication bug once #398 populates those fields on real files.
2. Non-adjacent hunks within the same file were rendered as one continuous stream with no visual boundary.

## Independent verification (this pass)

Read the full merged diff (`git diff 2da839c..b645c90`) directly, not relying solely on the subagent review that ran during execution.

- **`hunk_header` attach/reset logic**: `is_first_line_of_hunk` is initialized `True` at the start of each hunk's inner loop and only flipped to `False` after an actual line entry (add/remove/context) is appended; the `\ No newline at end of file` continuation line is skipped via `pass` without touching the flag or emitting a spurious entry. Re-entering the outer `while` for the next `@@` re-derives `hunk_header_text` and resets the flag, so there is no cross-hunk leakage. Confirmed correct by direct read, not just by trusting the test.
- **Annotation card scope reduction**: `_render_annotation_cards` now iterates files once, skips a file entirely when it has neither `pseudocode` nor `call_tree`, and falls back to a single `<p class="empty">当前没有注释内容</p>` placeholder when the whole diff has no annotation content. Matches the approved design exactly.
- **HTML escaping**: the new hunk-header text goes through `html.escape` in `_render_hunk_line`; the file-path-based `data-anchor` in `_render_annotation_cards` goes through `html.escape(f["path"], quote=True)`. Consistent with the file's existing escaping discipline — no injection surface introduced.
- **Prior review finding, verified fixed**: the independent subagent review (during execution) flagged that lifting cards to file-level dropped their `data-anchor`, leaving `.annotation-card` click/hover listeners as dead code with a misleading `cursor: pointer`. Commit `f7e526c` repoints `data-anchor` at the file path and rewires the click handler to scroll to that file's `.file-section` (mirroring the existing file-tree click behavior), and removes `.annotation-card` from the `highlightAnchor` query set. Verified this actually restores working behavior, not just silences the symptom.
- **Test coverage**: 5 new/modified tests (`test_hunk_header_marks_only_the_first_line_of_each_hunk`, `test_single_hunk_dual_line_numbers` updated, `test_no_annotation_content_shows_placeholder_message`, `test_pseudocode_renders_once_per_file_not_per_line`, `test_annotation_card_anchors_to_file_section_not_a_line`, `test_hunk_separator_shown_between_non_adjacent_hunks`) each pin down one specific behavior change and would catch a regression in that behavior. Full suite: 45/45 passing (`python3 scripts/tests/test_render_diff_review.py`).

## New finding (minor, non-blocking)

`STYLE` still defines `.annotation-card.hover-highlight { background: #e0ecff; }`, but no JS path adds `hover-highlight` to `.annotation-card` anymore (that class is only ever toggled on `.diff-line` elements now, per the `f7e526c` fix). This is dead CSS — cosmetic only, no functional impact, no security concern. Not worth a blocking follow-up on its own; can be swept up the next time this file is touched.

## Verdict

**APPROVE.** Both original bugs are fixed as designed, the prior review's own finding was correctly addressed rather than papered over, escaping discipline is maintained, and test coverage is adequate. `make check-agent-sync` baseline unchanged (78/29/194/0 PASSED). No PR exists to attach a formal `gf review` verdict to (local-merge delivery); this report stands as the Phase 4 sign-off.
