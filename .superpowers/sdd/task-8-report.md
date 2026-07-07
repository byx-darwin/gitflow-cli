### Task 8 Report: Refactor gitflow-regression skill

**Status:** COMPLETE

**Issue number:** #25

**Commits:**
- `9fb05fb` — refactor(skill): gitflow-regression — conform to Superpowers template (#25)

**Deliverables:**
1. Refactored skill: `skills/gitflow-regression/SKILL.md` (134 insertions, 277 deletions)
2. Commit: `9fb05fb` on branch `fix/hook-path-and-auth-parsing`

**Word Count:**
- Before: 858 words (77% over 500-word limit)
- After: 496 words (within limit)

**Structural Changes:**

| Section | Before | After |
|---------|--------|-------|
| description | Functional + workflow mixed | "Use when..." trigger-only (bilingual) |
| When to Use | Missing | 4-row EN/ZH trigger table |
| Core Pattern | Missing | Executable bash skeleton |
| Quick Reference | Missing | 4-row command cheat-sheet |
| Implementation | Linear 5-step manual | 4-step decision-flow |
| Responsibility | Missing | ✅/❌/🚫 + Error Handling table |
| Red Flags | Missing | 3 skill-specific flags |
| Rationalization | Missing | 2-row counter-table |
| Test Scenarios | Missing | 4 scenarios (happy/negative/boundary/chain) |
| Success Criteria | Missing | 4 checkboxes |
| Flowchart | Missing | Mermaid (3+ decision branches) |
| See Also | Missing | 4 cross-references |
| Trigger Keywords | Missing | 5-row EN/ZH table |
| Common Mistakes | 9-item 注意事项 | 2-item focused list |

**P0 Items Completed:**
- [x] description rewritten as trigger condition
- [x] Chain boundary with autoreport-bug declared (CLI-bug-only; never auth/network)
- [x] Prohibition list (🚫 Do Not) added
- [x] Red flags added
- [x] Trigger keywords (bilingual) added
- [x] Cross-references (See Also) added
- [x] Token compression: 858 → 496 words
- [x] Testability: 4 test scenarios added

**P1 Items Completed:**
- [x] Structured template compliance (all 16 checklist items verified)
- [x] Error handling table added
- [x] Preconditions section added
- [x] Rationalization excuse counter-table added
- [x] Quick Reference added

**Chain Boundary Handling:**
- Regression skill declares: "invokes `gitflow-autoreport-bug` for CLI-bug failures only — never auth/network"
- Classification table explicitly separates auth/network/env failures (notify only) from CLI bugs (report)
- Scenario 4 (Chain Boundary) verifies only panic + 500 → report; timeout → notify only

**Bidirectional Cross-Reference Gap (coordination item):**
- `gitflow-autoreport-bug` skill has NO `## See Also` section
- Regression → autoreport-bug reference added in this task
- autoreport-bug → regression reference must be added when autoreport-bug is refactored (separate task)
- Documented for cluster coordination

**Self-Review Checklist (16/16):**
1. ✅ description matches `/^Use when/i`
2. ✅ Contains ## When to Use (with trigger keywords EN+ZH)
3. ✅ Contains ## Core Pattern (executable skeleton)
4. ✅ Contains ## Quick Reference (command cheat-sheet)
5. ✅ Contains ## Implementation (step-by-step)
6. ✅ Contains ## Common Mistakes
7. ✅ Contains ## Responsibility (all 3 sub-sections)
8. ✅ Contains ## Red Flags
9. ✅ Contains ## Trigger Keywords
10. ✅ Contains ## See Also (≥2 cross-refs)
11. ✅ Contains ## Test Scenarios (≥4, incl. 1 negative)
12. ✅ Contains ## Success Criteria
13. ✅ Word count ≤500 (496)
14. ✅ No fictional data
15. ✅ No narrative examples
16. ✅ Mermaid flowchart included (3+ branches)

**Concerns:** None — the refactor is self-contained and all checklist items pass.
