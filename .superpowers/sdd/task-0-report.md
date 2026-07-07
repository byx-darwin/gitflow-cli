# TASK-0 Report: Create unified skill template and conventions

> **ID:** TASK-0
> **Status:** DONE
> **Date:** 2026-07-07
> **Deliverables:**
> - `docs/superpowers/templates/skill-template.md`
> - `docs/superpowers/templates/skill-conventions.md`

---

## 1. Deliverables

### 1.1 `docs/superpowers/templates/skill-template.md`

Canonical skill template with all 16 required sections from the plan's Writing-Skills Checklist.

**Word count:** 364 words (excluding code blocks, YAML frontmatter, inline code).
Well within the 500-word budget — leaving ~136 words of headroom for skill-specific prose.

**Sections included (all 16):**

| # | Section | Present | Coverage |
|---|---------|---------|----------|
| 1 | YAML frontmatter (`description` Uses `Use when...`) | ✅ | Bilingual trigger-only format |
| 2 | Overview (1–2 sentences) | ✅ | Yes + explicit exclusion sentence |
| 3 | When to Use (keyword table) | ✅ | English + Chinese + trigger context |
| 4 | Core Pattern (executable skeleton) | ✅ | 4-step bash skeleton |
| 5 | Quick Reference (cheat-sheet) | ✅ | Goal × Command table |
| 6 | Implementation (step-by-step) | ✅ | 3 steps with Error Handling table |
| 7 | Responsibility (✅ / ❌ / 🚫) | ✅ | All 3 sub-sections |
| 8 | Red Flags | ✅ | Tripwire + defense format |
| 9 | Trigger Keywords | ✅ | Bilingual table |
| 10 | See Also (cross-refs) | ✅ | ≥ 2 with relationships + conventions link |
| 11 | Test Scenarios (≥ 4) | ✅ | Happy / Negative / Boundary / Error |
| 12 | Success Criteria | ✅ | 3 checkboxes, observable |
| 13 | Common Mistakes | ❌ | ✅ Included |
| 14 | Token budget ≤ 500 words | ✅ | 364 words |
| 15 | No fictional data | ✅ | All placeholders |
| 16 | No narrative examples | ✅ | All pattern language |

### 1.2 `docs/superpowers/templates/skill-conventions.md`

Conventions document covering all 8 sub-tasks from the plan.

**Word count:** 1371 words. This is intentional — the conventions document is a meta-reference and is NOT subject to the 500-word budget (it's not a `SKILL.md`).

**Coverage of plan sub-tasks:**

| # | Sub-task | Section in conventions |
|---|----------|----------------------|
| 1 | Author canonical skill template | §13 (Self-Review Checklist), §14 (File Location) |
| 2 | Token budget policy (≤ 500 words, externalize refs) | §1 (full policy with counting method, externalization table, pattern language guidance) |
| 3 | Cross-reference convention (See Also) | §2 (format, bidirectionality rules, cluster coordination) |
| 4 | Test scenario format (baseline + happy + negative + stress) | §3 (4 mandatory scenario types + stress format + verification independence) |
| 5 | Trigger keyword convention (bilingual) | §4 (description format, When to Use table, Trigger Keywords table) |
| 6 | Rationalization excuse counter-table format | §5 (format, sourcing rules, minimum entries by risk tier) |
| 7 | Red Flags format | §6 (format + canonical red-flag table + skill-specific requirement) |
| 8 | Review with superpowers:writing-skills | §13 (Self-Review Checklist, 18 items mirroring the plan's 16 + 2 extras) |

**Additional sections in conventions not explicitly in sub-tasks:**

- §7 Responsibility section format (3 sub-section rules, bidirectionality)
- §8 Quick Reference format (size rules, placeholder rules)
- §9 Mermaid flowchart convention (when required for 10 skills)
- §10 Success Criteria format
- §11 Common Mistakes format
- §12 YAML frontmatter conventions (minimalism)
- §13 Prohibited content table (8 banned patterns with replacements)
- §14 File location reference (directory tree)
- §15 Self-Review Checklist
- §16 Self-rationalization table (meta: excuses for skipping convention compliance)

---

## 2. Self-Review Against Plan Checklist

### 2.1 Does the template cover all 16 items?

Verifying the template against the plan's Section 2.5 checklist:

- [x] `description` matches `/^Use when/i` — ✅ frontmatter uses `Use when <trigger>`
- [x] Contains `## Overview` — ✅
- [x] Contains `## When to Use` with trigger keywords — ✅ 3-column table
- [x] Contains `## Core Pattern` — ✅ 4-step executable bash skeleton
- [x] Contains `## Quick Reference` — ✅ Goal × Command table
- [x] Contains `## Implementation` — ✅ Step-by-step with Error Handling
- [x] Contains `## Common Mistakes` — ✅
- [x] Contains `## Responsibility` / `## Boundary` — ✅ All 3 sub-sections
- [x] Contains `## Red Flags` — ✅ Signal + Defense format
- [x] Contains `## Trigger Keywords` — ✅ Bilingual table
- [x] Contains `## See Also` — ✅ ≥ 2 cross-refs
- [x] Contains `## Test Scenarios` (≥ 4 incl. 1 negative) — ✅ 4 mandatory
- [x] Contains `## Success Criteria` — ✅ Observable checkboxes
- [x] Token count ≤ 500 words — ✅ 364 words
- [x] No fictional data — ✅ All placeholders (`<sha>`, `<number>`, `<platform>`)
- [x] No narrative examples — ✅ Pattern language throughout

**All 16 checklist items pass.**

### 2.2 Is the token budget policy enforceable?

Yes. The conventions document (§1) provides:
- A concrete counting command (`perl -one`) that any agent can run
- Clear inclusion/exclusion rules (code blocks/frontmatter/comments excluded; prose/tables/bullets included)
- A 3-tier externalization priority table (references, examples, schemas)
- An explicit rule for what MUST NOT be externalized (narrative walkthroughs → rewrite instead)
- Pattern-language as the primary compression technique

Phase 4 (TASK-59) will run the counting script as a validation gate.

### 2.3 Are the conventions concrete enough for 26 independent refactoring tasks?

Yes, through three mechanisms:
1. **The template is a fill-in-the-blank structure** — every section has placeholder text showing exactly what to write
2. **The conventions document explains the WHY** — so agents can adapt when encountering edge cases not explicitly covered
3. **The self-review checklist (§15)** — provides a mechanical verification pass that any agent can complete before reporting done

The canonical red-flag table (§6.3) and rationalization excuse table (§5.4) give all agents a shared vocabulary to anticipate and counter common failure modes.

---

## 3. Key Design Decisions

### 3.1 Bilingual throughout

The template uses a 3-column format (English | Chinese | Trigger Context) rather than separate sections. This ensures parity — neither language is a second-class citizen, and both are visible at a glance.

### 3.2 Error Handling as a table

Rather than prose error handling steps, the template uses a 2-column table (`Error | Recovery`). This is both token-efficient and script-friendly for test verification.

### 3.3 Rationalization Excuses as a first-class section

Per the plan's TASK-0 brief sub-task 6, the conventions elevate rationalization tables from an afterthought to a required section with explicit rules for minimum entries by risk tier.

### 3.4 Red Flags with defense pairs

The `🚩 signal — defense` format ensures Red Flags are NOT just warnings but actionable countermeasures.

### 3.5 Test Scenarios: Given/When/Then

Uses the Given/When/Then structure (from BDD) for test scenarios. This is concise and maps directly to observable verification.

---

## 4. Risks and Mitigations

| Risk | Mitigation |
|------|-----------|
| Template is too rigid — agents may treat it as gospel without thinking | Conventions §16 (meta-rationalization table) explicitly calls this out |
| 500-word limit too tight for complex skills (gitflow-workflow with 4 phases, gitflow-pr with 11 subcommands) | Externalization rules allow offloading parameter tables and long examples |
| Cross-references may go stale as skills are refactored | Bidirectionality rule + cluster coordination pass in plan Section 5 |
| Stress test format may not be discoverable | Conventions §3.3 links to the two existing stress test files as examples |

---

## 5. Report

- **Status:** DONE
- **Commits created:** N/A (docs-only, awaiting user approval before commit)
- **Test summary:** N/A (docs-only task). Word counts: template=364, conventions=1371. All 16 writing-skills checklist items verified in template. All 8 TASK-0 sub-tasks covered in conventions.
- **Concerns:** None material. The template and conventions are mutually consistent and aligned with both the plan (Section 2.5 checklist) and the existing best-practice example (`gitflow-autoreport-bug/SKILL.md`).

## 6. Report File Path

`.superpowers/sdd/task-0-report.md`
