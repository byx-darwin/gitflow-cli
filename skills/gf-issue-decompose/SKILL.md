---
name: gf-issue-decompose
description: |
  Use when the user has a requirement, spec, or design document and wants it broken down into a set of Issues with dependency edges.
  当用户手上有一份需求、规格或设计文档，希望批量拆解成带依赖边的 Issue 时使用。
allowed-tools: Bash, Read, Grep, Glob
---

# gf-issue-decompose

Decomposes a document into vertically sliced, falsifiable, ordered tickets.

**MUST use `gf`, NOT `gh`** — `gh` breaks GitLab/GitCode.

## Preconditions

- `command -v gf` · `gf auth status`
- A document to decompose

## When to Use

| English | 中文 | Context |
|---------|------|---------|
| break this spec into issues | 把规格拆成 Issue | document, no tickets |
| decompose the requirement | 拆解需求 | many deliverables |
| split into tickets with dependencies | 拆票并标依赖 | ordering matters |
| file one issue | 提一个 Issue | does NOT fire |

## Trigger Keywords

| English | 中文 |
|---------|------|
| decompose, break down | 拆解, 拆分 |
| slice into tickets | 切票 |
| blocked by, dependency order | 阻塞, 依赖序 |

## When NOT to Use

| Scenario | Use Instead |
|----------|-------------|
| One ticket, interactive | `/gf-issue-create` |
| Judging an Issue's quality | `/gf-issue-review` |
| Labelling the backlog | `/gf-issue-triage` |
| Edit / close / comment | `/gf-issue` |

## Procedure

**0 — Floor.** Fits in **one context window** → **do not split**. Emit one ticket.

**1 — Slice.** One narrow path per ticket, through every layer. Slicing by layer is the
**default error**: a **horizontal layer** is not a **vertical slice**. Ticket one is a
tracer bullet. → `references/vertical-slice.md`

**2 — Falsify.** Name **what observation would prove it false**; fire it at the
**base commit**, where it **must be red** (not-found and compile errors count). Reject one
**already true on base**, one observing what **belongs to another ticket**, one that
**restates the requirement**. → `references/falsifiable-criteria.md`

**3 — Order.** Declare `Blocked by: #N` when this ticket cannot go green before the
other merges; the criterion stays here. Create in **dependency order**. Wide refactors
split expand → migrate → contract. → `references/dependency-edges.md`

**4 — Confirm.** **Before creating** anything, ask and wait:

1. **granularity** — ticket count right?
2. **demo path** — what does each ticket demonstrate?
3. **edges** — blockers real, order acyclic?

On approval, create in that order:

```bash
gf issue create --title "<prefix>(scope): summary" --body-file <path> [--label <l>]
```

## Error Handling

| Error | Recovery |
|-------|----------|
| `gf auth status` fails | Stop. Prompt `gf auth login`. No retry. |
| Creation fails mid-batch | Stop. Report existing numbers. No re-create. |
| Cycle among edges | Stop. Re-slice, never cut it. |

## Responsibility

### ✅ In Scope
Slicing · criteria · edges · confirmed creation

### ❌ Out of Scope
Everything in **When NOT to Use**.

### 🚫 Do Not
- ❌ **skip the confirmation**
- ❌ Split below the floor rule
- ❌ Publish criteria green at base
- ❌ Dependencies as prose, not `Blocked by`

## Rationalization Excuses

| Excuse | Reality |
|--------|---------|
| "Schema-only is a clean unit" | Ships nothing observable. |
| "Clearly not met yet" | Obvious is not measured. Fire it. |
| "Add the edges afterwards" | Mid-batch failure dangles them. |
| "User is in a hurry" | Creation is a remote side effect. |
| "'Works correctly' is fine" | Nothing contradicts it. |

## Red Flags

- 🚩 "Just file them all" — three questions first.
- 🚩 Tickets buildable by strangers — layers.

## Test Scenarios

### 1: Happy Path
**Given** a three-capability spec — **When** asked to decompose — **Then** three vertical tickets, falsifiable criteria, declared edges, no creation before the questions.

### 2: Negative
**Given** no document — **When** "file a bug" — **Then** not loaded → `/gf-issue-create`.

### 3: Boundary — Below the Floor
**Given** two files, one module — **When** asked to decompose — **Then** one ticket.

### 4: Error
**Given** an expired token — **When** the batch starts — **Then** stop → `gf auth login`, no retry.

### 5: Boundary — Green at Base
**Given** an observation not firing at base — **When** assembled — **Then** reject, re-author.

## Success Criteria

- [ ] No layer-only ticket published
- [ ] Each criterion names its observation, red at base
- [ ] Edges as `Blocked by`, created in dependency order
- [ ] Three questions answered first

## See Also

- `/gf-issue-create` — authors one Issue
- `/gf-issue-review` — audits Issue quality
- `/gf-issue-triage` — classifies the backlog
- `/gf-issue` — Issue CRUD
- `docs/superpowers/templates/skill-conventions.md` — skill conventions
