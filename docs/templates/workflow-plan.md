# Workflow Plan Template

> **Companion:** `skills/gitflow-workflow/SKILL.md`
> **Version:** 1.0.0

Use this template when Phase 2 (Plan) generates the plan document via `superpowers:writing-plans`.

---

## Required Structure

```markdown
# [Feature Name] Implementation Plan

## Task List

### Task 1–3: Issue Management
- [ ] Task 1: Create Issue
- [ ] Task 2: Set status to in-progress
- [ ] Task 3: Update Issue description (if needed)

### Task 4–N: Development (each task includes)
- [ ] TDD cycle
  - [ ] Write failing test (RED)
  - [ ] Write minimal impl (GREEN)
  - [ ] Refactor (REFACTOR)
  - [ ] Verify: run tests
- [ ] Code review
  - [ ] Call `superpowers:requesting-code-review`
  - [ ] Fix findings
- [ ] Commit
  - [ ] `git add -A`
  - [ ] `git commit -m "feat: ... (#N)"`

### Task N+1: Quality Gate
- [ ] Call `/gitflow-quality` — run 6 checks
  - Build / Test / Coverage (>80%) / Format / Static / Pre-commit
- [ ] Confirm report = `ALL CHECKS PASSED`
- [ ] On failure: apply fixes, rerun

### Task N+2: Delivery
- [ ] Create PR via `/gitflow-pr-create`
- [ ] PR review via `/gitflow-pr-review`
- [ ] Merge via `gitflow-cli pr merge`

### Task N+3: Closure
- [ ] Set Issue status to done
- [ ] Close Issue
- [ ] Verify all acceptance criteria checked
```

## Gate Evidence Requirements

### Gate 1 → 2 (Requirement → Plan)
- [ ] Issue URL accessible
- [ ] Requirement analysis posted as Issue comment
- [ ] Verify: `gitflow-cli issue view <number>`

### Gate 2 → 3 (Plan → Execution)
- [ ] Plan document exists
- [ ] Quality gate task (Task N+1) present in plan

### Gate 3 → 4 (Execution → Post-delivery)
- [ ] All plan tasks checked
- [ ] PR merged
- [ ] Issue closed

## Phase Compliance Checklist

```
Phase N compliance:
  [ ] Step 1: <skill> invoked
  [ ] Step 2: <skill> invoked
  [ ] Artifact recorded: <url or path>
  Missing: <list or "none">
  Gate passed: [y/n]
```

## Audit Log Template

Post to Issue after each phase:

```bash
cat > /tmp/phase-report.md << 'REPORT'
## Phase N: <Name> complete

### Artifacts
- <artifact>: <url or path>

✅ Gate passed — proceed to Phase N+1
REPORT

gitflow-cli issue comment <number> --body-file /tmp/phase-report.md
rm -f /tmp/phase-report.md
```

## Rollback Template

```bash
gitflow-cli issue comment <number> --body "## Rollback

**From Phase X → Phase Y**

### Reason
<why>

### TODOs
- [ ] <item>
- [ ] <item>"
```
