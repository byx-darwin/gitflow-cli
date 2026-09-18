# Dependency Edges

Progressive-disclosure reference for `gf-issue-decompose`. Loaded when ordering tickets
or decomposing a wide refactor.

## Blocked by Declaration

Each ticket declares its inbound edges explicitly, in its body, one per line:

```markdown
Blocked by: #12, #14
```

Rules:

- **Edges are code edges, not preference.** An edge exists only when this ticket's
  acceptance criteria cannot go red-then-green until the other ticket has merged.
  "It would be tidier to do that first" is not an edge.
- **An edge keeps the criterion here.** When a criterion observes *this* ticket's
  behavior but needs a blocker merged first, it stays on this ticket and the edge is
  declared. Only a criterion that observes the *blocker's own* behavior moves away —
  that is reject shape 2 in `falsifiable-criteria.md`, not an edge.
- **Declare on the blocked ticket**, never on the blocker. One direction only; a cycle
  means the slices are wrong, so re-slice rather than breaking the cycle arbitrarily.
- **Reference by number** once the blocker exists. When the blocker has not been created
  yet, create it first — see below.
- **A tracer bullet has no inbound edges.** If the first ticket is blocked, it is not a
  tracer bullet.

`Blocked by: #N` is the literal a topological sorter will parse. No such consumer exists
in this repository yet — `gf-workflow-batch` still takes the first pending Issue in list
order, and sorting it by dependency edges is tracked as Issue #337. Writing the edge as
prose ("depends on the schema work") produces data that sorter cannot read, so keep the
literal even though nothing consumes it today.

## Creation in Dependency Order

Tickets are created in topological order, blockers first, so that every `Blocked by`
reference points at a number that already exists.

1. Build the graph from the decomposition: nodes are tickets, edges are `Blocked by`.
2. Detect cycles. A cycle is a slicing defect — merge the cycle's members into one
   ticket, or re-slice, before creating anything.
3. Emit a topological order. Ties break by slice position: the tracer bullet first,
   then widenings in the order a user would encounter them.
4. Create tickets one at a time in that order, substituting the real issue numbers into
   each subsequent body as they are assigned.

Never create the whole set first and patch edges afterwards: a partially-created batch
that fails midway leaves dangling references to numbers that were never assigned.

## Wide Refactor

A refactor that touches many call sites has no user-observable slice of its own, so the
vertical-slice test cannot be applied directly. Decompose it as expand → migrate →
contract instead, one ticket per stage.

| Stage | Ticket content | Acceptance shape |
|---|---|---|
| **expand** | Introduce the new shape alongside the old one. Nothing is removed; both paths work. | New path is exercisable end to end; old path still passes its existing tests |
| **migrate** | Move call sites onto the new shape, in batches if the count is large. | No call site references the old shape; behavior at each migrated site is unchanged |
| **contract** | Delete the old shape and its compatibility layer. | The old symbol no longer exists; nothing references it |

Edges are linear: `migrate` is `Blocked by` `expand`, `contract` is `Blocked by`
`migrate`. When `migrate` is split into batches, each batch is blocked by `expand` only
and the batches are parallel; `contract` is blocked by all of them.

This is the one sanctioned exception to the observable-behavior rule, and it is bounded:
the three stages must all be created together. An `expand` ticket created without its
`contract` ticket is how a codebase acquires a permanent second way to do things.
