# Falsifiable Acceptance Criteria

Progressive-disclosure reference for `gf-issue-decompose`. Loaded when writing or
auditing a ticket's `## Acceptance Criteria`.

## Falsifying Observation

Every criterion must answer one question: **what observation would prove it false?**

Write the criterion so the falsifying observation is mechanical — a command someone
else can run without reading the implementation.

| Criterion | Falsifying observation |
|---|---|
| "Creating an X with a duplicate slug returns 409 and no row is written" | Run the request twice; a 2xx on the second call, or a second row, proves it false |
| "The list endpoint caps `per_page` at 100" | Request `per_page=500`; more than 100 items proves it false |
| "The CLI exits non-zero when the token is absent" | Unset the variable and run it; exit code 0 proves it false |

If the falsifying observation cannot be named, the criterion is prose, not a criterion.
Rewrite it or drop it.

Format each criterion as a checkbox whose text carries the observable, not the intent:

```markdown
- [ ] <observable behavior>; <what would prove it false> proves it false
```

## Three Reject Shapes

A criterion is rejected — not softened, not merged — when it matches any of these.

| Shape | Signature | Why it is rejected | Fix |
|---|---|---|---|
| **Already true on base** | The stated observation already holds at the base commit | It cannot fail, so it measures nothing and the ticket can be closed without work | Tighten it until the base fails it, or delete it |
| **Belongs to another ticket** | The behavior it *observes* is another ticket's deliverable, not this one's | This ticket could go green without ever satisfying it; it is somebody else's criterion | Move it to that ticket. Do **not** confuse this with a criterion that observes *this* ticket's behavior but needs a blocker merged first — that one stays here and gets a `Blocked by` edge (`dependency-edges.md`) |
| **Restates the requirement** | It paraphrases the Goal section with no observable ("X works correctly", "the API is robust") | Nothing can be run to contradict it | Replace with the observation that would fail |

Reject at authoring time. A ticket published with any of these shapes teaches the
implementer that acceptance criteria are decorative.

## Red-on-Base Check

Before a ticket is created, every criterion must be red at the base commit.

```bash
# The base commit the ticket will be implemented on top of
BASE=$(git rev-parse HEAD)

# For each criterion, run its falsifying observation against BASE.
# Expected: the observation fires — i.e. the criterion does NOT hold yet.
```

Outcomes:

The test is whether the **falsifying observation fires**, not whether the command runs
cleanly. On a tracer bullet the subcommand, route, or button does not exist yet, so the
observation fires as `command not found`, `404`, or a compile error. That is the healthy
red — keep it.

| Result at base | Meaning | Action |
|---|---|---|
| Observation fires — including not-found, 404, or a compile error | Red. It measures work this ticket will do | Keep |
| Observation does not fire; the behavior is already there | Shape 1 — already true on base | Reject the criterion |
| The observation is only meaningful once another ticket's behavior exists | Shape 2 — it observes somebody else's deliverable | Move it to that ticket |
| No executable observation can be written at all | Shape 3 — it restates the requirement | Rewrite as an observation |

A criterion that observes *this* ticket's behavior but cannot be implemented until a
blocker merges is **not** Shape 2. It stays, and the ticket declares `Blocked by` — see
`dependency-edges.md`.

A ticket whose criteria all fail to fire at base is not a ticket. Report that the work is
already done rather than publishing it.
