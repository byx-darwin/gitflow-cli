# TASK-0: Create unified skill template and conventions

**ID:** TASK-0
**Effort:** 8h
**Depends on:** None
**Deliverables:** `docs/superpowers/templates/skill-template.md`, `docs/superpowers/templates/skill-conventions.md`

## Sub-tasks

1. Author canonical skill template with all required sections:
   - YAML frontmatter (name, description with "Use when..." trigger format)
   - ## Overview (1-2 sentences)
   - ## When to Use (trigger keywords, English + Chinese)
   - ## Core Pattern (executable skeleton)
   - ## Quick Reference (command cheat-sheet)
   - ## Implementation (step-by-step)
   - ## Responsibility (✅ In scope / ❌ Out of scope / 🚫 Do Not)
   - ## Red Flags
   - ## Trigger Keywords
   - ## See Also (cross-references)
   - ## Test Scenarios (≥ 4 scenarios)
   - ## Success Criteria
   - ## Common Mistakes

2. Define token budget policy (SKILL.md ≤ 500 words, externalize references)
3. Define cross-reference convention (See Also format)
4. Define test scenario format (baseline + happy-path + negative + stress)
5. Define trigger keyword convention (English + Chinese bilingual)
6. Define rationalization excuse counter-table format
7. Define Red Flags format
8. Review with superpowers:writing-skills methodology for template compliance

## Global Constraints

- Each skill must be independently loadable
- Token efficiency: SKILL.md ≤ 500 words
- Trigger condition description must use "Use when..." format only
- All documents must use Markdown format
- Bilingual trigger keywords (English + Chinese)

## Context

This is the foundation task. All subsequent tasks (TASK-1 through TASK-59) depend on the template and conventions defined here. The template serves as the canonical reference for all 26 skill refactoring tasks.
