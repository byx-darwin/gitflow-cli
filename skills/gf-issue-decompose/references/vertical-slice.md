# Vertical Slice

Progressive-disclosure reference for `gf-issue-decompose`. Loaded when a candidate
ticket's shape is in doubt.

## Slice Test

A ticket passes only if a single narrow path runs end to end through every layer the
change touches — storage, service/API, surface, and the test that observes it.

Apply the test in this order:

1. **Name the observable.** State what a user or caller can do after this ticket that
   they could not do before. If the answer is phrased in terms of internals ("the
   repository now has a `find_by_slug` method"), the slice is horizontal.
2. **Walk the path.** List the layers the observable traverses. A slice is vertical
   when the ticket owns *all* of them for that one path, and horizontal when it owns
   *one* of them for all paths.
3. **Check the width.** A vertical slice is narrow: one entity, one field, one route,
   one error case. Breadth belongs to sibling tickets, not to this one.
4. **Check the test.** A vertical slice can be demonstrated by an end-to-end or
   integration test. If only a unit test of one layer can observe it, re-slice.

## Horizontal-Layering Rejects

These shapes are rejected on sight. Each is a layer-wide ticket masquerading as a
deliverable.

| Rejected ticket | Why it fails | Re-slice as |
|---|---|---|
| "Add the database schema for X" | Storage only; nothing observable ships | "Create an X and read it back through the API" |
| "Implement the X API endpoints" | Service only; no persistence, no caller | One ticket per route, each reaching storage and a test |
| "Build the X UI" | Surface only; wired to nothing | One ticket per user action, reaching the backend |
| "Write tests for X" | Test-layer only; no behavior change | Fold the tests into the ticket that creates the behavior |
| "Refactor the X module" | Internal-only; no falsifiable observable | See `dependency-edges.md` → Wide Refactor |

A useful smell: if two candidate tickets could be implemented by two people who never
speak to each other, and neither produces something a user can do, they are layers.

## Tracer Bullet

The first ticket of any decomposition is a tracer bullet — the thinnest possible path
that touches every layer and lands a real, observable result.

Properties:

- **Thin, not fake.** Real storage, real transport, real assertion. A stub that returns
  a constant is not a tracer bullet; it defers exactly the risk the bullet exists to
  retire.
- **Ugly is fine.** Hardcoded config, one supported case, no pagination. Later tickets
  widen it.
- **It proves the wiring.** Its value is that every subsequent ticket now has a path to
  extend rather than a path to invent.

Subsequent tickets widen the bullet along one axis each: another field, another route,
another error case, another platform.
