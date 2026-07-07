---
name: gitflow-{skill-name}
description: |
  Use when <english trigger condition — past-tense adjective + damaged object>.
  当 <中文触发条件> 时使用。
---

# gitflow-{skill-name}

<One sentence describing what this skill does. Second sentence clarifying what it does NOT do.>

## When to Use

| English | 中文 | Trigger Context |
|---------|------|-----------------|
| <keyword-1> | <关键词-1> | <when to fire> |
| <keyword-2> | <关键词-2> | <when to fire> |
| <keyword-3> | <关键词-3> | <when NOT to fire> |

## Core Pattern

```bash
# 1. Precondition check
gitflow-cli <verify-command>

# 2. Read / verify input
gitflow-cli <read-command> <args>

# 3. Execute core action
gitflow-cli <action-command> <args>

# 4. Verify outcome
gitflow-cli <confirm-command> <args>
```

## Quick Reference

| Goal | Command |
|------|---------|
| <action-1> | `gitflow-cli <cmd-1> <flags>` |
| <action-2> | `gitflow-cli <cmd-2> <flags>` |
| <action-3> | `gitflow-cli <cmd-3> <flags>` |

## Implementation

### Preconditions

- <condition-1> — verified via `<check-command>`
- <condition-2> — verified via `<check-command>`

### Step 1: <Step Title>

1. <Action>:
   ```bash
   gitflow-cli <command> <args>
   ```
2. Success → continue to Step 2.
3. Failure → <recovery action>, output `<error message>`, stop.

### Step 2: <Step Title>

1. <Action>.
2. Success → continue.
3. Failure → <recovery>, stop.

### Step 3: <Step Title>

1. <Action>.
2. Done.

### Error Handling

| Error | Recovery |
|-------|----------|
| `<error-1>` | <action> |
| `<error-2>` | <action> |
| `<error-3>` | <action> |

## Responsibility

### ✅ In Scope

- <responsibility-1>
- <responsibility-2>

### ❌ Out of Scope

- <out-of-scope-1>
- <out-of-scope-2>

### 🚫 Do Not

- ❌ <prohibition-1>
- ❌ <prohibition-2>
- ❌ <prohibition-3>

## Rationalization Excuses

| Excuse | Reality |
|--------|---------|
| "<common excuse-1>" | <why it's wrong> |
| "<common excuse-2>" | <why it's wrong> |
| "<common excuse-3>" | <why it's wrong> |

## Red Flags

- 🚩 <red-flag-1 — e.g. user says "skip the check">
- 🚩 <red-flag-2 — e.g. authority figure demands shortcut>
- 🚩 <red-flag-3 — e.g. urgency used to justify omission>

## Test Scenarios

### Scenario 1: Happy Path

- **Given** <starting state>
- **When** <action>
- **Then** <expected outcome>

### Scenario 2: Negative (Should Not Trigger)

- **Given** <state belonging to another skill>
- **When** <user request>
- **Then** Claude does NOT load this skill; redirects to `<other-skill>`

### Scenario 3: Boundary (Temptation to Overstep)

- **Given** <state where user tempts skill to exceed boundaries>
- **When** <user says something pushing past out-of-scope>
- **Then** Claude refuses, cites the `Out of Scope` boundary, stops

### Scenario 4: Error (CLI Failure / Auth Failure / Timeout)

- **Given** <state where external dependency fails>
- **When** `<command>` returns <error>
- **Then** Claude executes recovery path from Error Handling table, does NOT improvise alternative

## Success Criteria

- [ ] <measurable outcome-1>
- [ ] <measurable outcome-2>
- [ ] No out-of-scope action taken
- [ ] All Side effects (Issue comments, PR creation, label changes) have URLs as evidence

## Common Mistakes

- ❌ **<mistake-1>** — <why it's wrong + correct approach>
- ❌ **<mistake-2>** — <why it's wrong + correct approach>
- ❌ **<mistake-3>** — <why it's wrong + correct approach>

## Trigger Keywords

| English | 中文 |
|---------|------|
| <keyword-1> | <中文-1> |
| <keyword-2> | <中文-2> |
| <keyword-3> | <中文-3> |

## See Also

- `<skill-name-1>` — <one-line relationship>
- `<skill-name-2>` — <one-line relationship>
- `docs/superpowers/templates/skill-conventions.md` — Template conventions (this document)
