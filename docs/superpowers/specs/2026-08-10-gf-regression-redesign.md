# gf-regression Skill Redesign

**Date**: 2026-08-10
**Issue**: #168
**Current Score**: 2.8/5
**Target Score**: 3.5+

## Problem Statement

The `gf-regression` skill has three critical issues:

1. **Insufficient documentation** - Users don't know when to use it
2. **Vague trigger words** - AI Agent easily mismatches the skill
3. **Limited scope** - Only supports `gf` CLI testing, poor generalizability

## Design Approach: Option B (Documentation + Trigger Refactor)

We chose the balanced approach that directly addresses the core issues without over-engineering:

- ✅ Rewrite When to Use with specific scenarios
- ✅ Add When NOT to Use section
- ✅ Refactor trigger system from keywords to scenarios
- ✅ Add comprehensive usage examples
- ⏸️ Generalization support (deferred to future issue)

## Design Decisions

### 1. When to Use Rewrite

**Problem**: Current When to Use table has only 4 rows with brief descriptions.

**Solution**: Expand to include 5 specific scenarios with trigger phrases and expected actions:

| # | Scenario | Trigger Phrase | Expected Action |
|---|----------|----------------|-----------------|
| 1 | Post-change verification | "verify my changes didn't break gf" | Run read-only smoke test, report PASS/FAIL |
| 2 | Pre-release gate | "run pre-release checks" | Run smoke test as part of release preparation |
| 3 | Debugging CLI issues | "gf commands are failing" | Run smoke test to identify which operations fail |
| 4 | Quick health check | "is gf working?" | Run minimal smoke test, report status |
| 5 | Regression detection | "check for regressions" | Run full smoke test, compare with baseline |

**Rationale**: Specific scenarios help AI Agent match user intent more accurately than generic keywords.

### 2. When NOT to Use (New Section)

**Problem**: No explicit "do not use" guidance leads to false matches.

**Solution**: Add clear exclusion table:

| Scenario | Why Not | Use Instead |
|----------|---------|-------------|
| Fixing a bug | This skill only detects and reports bugs, doesn't fix them | `/gf-workflow` for bug fixes |
| Code review | This skill runs tests, doesn't review code | `/gf-pr-review` for code review |
| Full quality gate | This skill only runs smoke tests, not full CI | `/gf-quality` for complete quality checks |
| Testing other projects | This skill is designed for `gf` CLI only | Use project-specific test commands |
| CI pipeline | Smoke tests with autoreport shouldn't run in CI | Use `scripts/smoke-test.sh` directly with exit code |
| Performance testing | This skill checks functionality, not performance | Use benchmarking tools |

**Rationale**: Explicit boundaries prevent mismatching and guide users to the right skill.

### 3. Trigger System Refactor

**Problem**: Current Trigger Keywords section duplicates When to Use and lacks decision logic.

**Solution**: Replace with a three-tier trigger system:

**Primary Triggers (High Confidence)**:
- `run smoke test` / `跑冒烟测试`
- `check for regressions` / `检查回归`
- `verify gf works` / `验证 gf 是否正常`
- `pre-release check` / `发版前检查`

**Secondary Triggers (Medium Confidence)**:
- `gf is broken` / `gf 坏了`
- `test failed after changes` / `改动后测试失败`

**Negative Triggers (Do NOT Load)**:
- `fix bug` → Use `/gf-workflow`
- `review code` → Use `/gf-pr-review`
- `run tests` (for user's project) → Use project-specific commands
- `quality check` → Use `/gf-quality`

**Decision Tree**:
```
User says something about "test" or "check"
    │
    ├─ Is it about gf CLI specifically?
    │   ├─ YES → Load gf-regression
    │   └─ NO → Is it about code review?
    │       ├─ YES → Load gf-pr-review
    │       └─ NO → Is it about full quality?
    │           ├─ YES → Load gf-quality
    │           └─ NO → Ask for clarification
    │
    └─ Is it about fixing a bug?
        └─ YES → Load gf-workflow
```

**Rationale**: Scenario-based triggers with confidence levels and decision logic reduce false matches.

### 4. Usage Examples

**Problem**: Quick Reference lacks complete examples with expected output.

**Solution**: Add 4 detailed examples:

1. **Quick Health Check**: "Is gf working?" → minimal test, status report
2. **Post-Change Verification**: "Verify nothing is broken" → verbose test, failure classification
3. **Pre-Release Check**: "Run pre-release checks" → smoke test as release gate
4. **Debugging CLI Issues**: "gf commands are failing" → diagnose + smoke test

Each example includes:
- User phrase
- Action (command)
- Expected output (success and failure cases)

**Rationale**: Concrete examples help users understand what to expect and how to interpret results.

## Success Criteria

- [ ] When to Use section has 5+ specific scenarios
- [ ] When NOT to Use section exists with 6+ exclusions
- [ ] Trigger system uses three-tier confidence model
- [ ] 4+ usage examples with expected output
- [ ] Review score improves from 2.8 to 3.5+
- [ ] AI Agent correctly matches skill in test scenarios

## Out of Scope

- Custom test script paths (deferred to future issue)
- Support for other projects (deferred to future issue)
- Configuration options (deferred to future issue)

## Implementation Plan

1. Rewrite When to Use section with 5 scenarios
2. Add When NOT to Use section with exclusion table
3. Replace Trigger Keywords with three-tier trigger system
4. Add 4 usage examples with expected output
5. Update Quick Start section
6. Test with 5 scenarios (happy path, negative, boundary, error cases)

## References

- Issue #168: fix(skills): gf-regression 文档重写与触发词优化
- Current skill: `skills/gf-regression/SKILL.md`
- Similar high-quality skill: `skills/gf-quality/SKILL.md`
