# gf-workflow-batch — Reference

## Pending Derivation Algorithm

```
open = gf issue list --state open --output json   # array of {number, url, title, labels}
covered = set()
for contract in glob(".cache/workflows/active/*.json") + glob(".cache/workflows/archive/**/*.json"):
    # a contract aborted before Phase 1 finished may lack phases["1"] or
    # .evidence entirely — treat a missing/absent issue_url as falsy, don't error
    issue_url = contract.get("phases", {}).get("1", {}).get("evidence", {}).get("issue_url")
    if issue_url:
        covered.add(issue_url)
    elif contract.title:
        covered.add(("title", contract.title))

pending = []
for issue in open:
    if issue.url in covered: continue
    if ("title", issue.title) in covered: continue
    if --label filter set and issue.labels does not contain it: continue
    pending.append(issue)

pending.sort(by=issue.number, ascending=True)
# NOTE: --limit is NOT applied here. Truncating the candidate list every
# round would just re-truncate to the next N candidates each time a
# dispatched Issue drops off pending — the loop would still exhaust the
# whole backlog. --limit bounds total dispatches across the run; see the
# Serial Dispatch Loop below.
```

**Coverage semantics**: an `active/*.json` contract with any phase
`status != "complete"` means the Issue is currently in progress somewhere —
skip it (don't double-dispatch). An `archive/**/*.json` contract (all phases
complete, moved to `archive/YYYY-MM/`) means the Issue was already
delivered — skip it too. A contract only counts as "covering" an Issue via
`phases["1"].evidence.issue_url`, written during Phase 1's `gf-issue-create`
step (for an already-existing Issue, that step records the existing URL
rather than creating a new one).

**Known limitation**: if a subagent's `/gf-workflow` run aborts before Phase
1 writes `issue_url` (e.g. the brainstorming step itself fails), the
fallback title match is used. If the Issue's title was edited between
dispatch and failure, neither match fires and the Issue may be dispatched
again on the next round. The larger case: this design has no in-run failure
memory at all — if the abort happens *before* `phases["1"].evidence.issue_url`
is ever written (e.g. before `gf-issue-create` runs in Phase 1), the Issue
stays uncovered on disk entirely, so it becomes `ready[0]` again next
round and can be re-dispatched repeatedly. Accepted per the design spec
(`specs/gf-workflow-batch-design.md` → Issue 覆盖判定 → 已知局限); mitigated
within a single invocation by the in-run `attempted` set in the Serial
Dispatch Loop below (not hardened further, e.g. across separate invocations,
in this iteration).

## Discussion Mode

Triggered only when `pending` is empty after the derivation above.

1. Invoke `superpowers:brainstorming` with the user's original ask (or ask
   what they'd like to work on next, if none was given). Follow that
   skill's own scope-decomposition guidance for "the request describes
   multiple independent subsystems" — that is exactly this mode's purpose.
2. For each decomposed sub-task, invoke `gf-issue-create` once. This step
   only creates the Issue; it does NOT dispatch `/gf-workflow` for it.
3. After all sub-task Issues are created, return to Pending Derivation —
   the new Issues now appear in `pending` (no contract's `evidence.issue_url`
   points to them yet).
4. Continue into the normal dispatch loop below.

## Dependency Resolution

Runs every round, between Pending Derivation and the Serial Dispatch Loop's
`candidates` selection. Scope is **all open Issues**, not just `pending` —
an Issue already covered by an active contract can still be another
pending Issue's blocker, or a cycle member, and would be missed if only
`pending` were scanned.

### Parsing `Blocked by` edges

```python
import re

open_issues = gf_issue_list_open()                     # existing, unchanged;
                                                         # already includes `body`
                                                         # (gh/glab/gc issue list shares
                                                         # the same field set as `view`,
                                                         # see crates/github/src/issue.rs
                                                         # ISSUE_FIELDS)
bodies = {i.number: i.body for i in open_issues}        # reuse — no extra `gf issue view` calls

def extract_edges(body: str) -> set[int]:
    edges = set()
    for group in re.findall(r'Blocked by:\s*((?:#\d+(?:,\s*)?)+)', body):
        for blocker in re.findall(r'#(\d+)', group):
            edges.add(int(blocker))
    return edges

edges = {i.number: extract_edges(bodies[i.number]) for i in open_issues}
# edges: blocked_issue_number -> set(blocker_issue_number)
```

Multiple `Blocked by:` lines in one body union their references (see
`skills/gf-issue-decompose/references/dependency-edges.md` for the
declaration format this parses — one edge per referenced number, declared
on the blocked ticket only).

The regex intentionally matches only the canonical `Blocked by: #N[, #M...]`
form documented there — case-sensitive `Blocked by:`, colon required,
comma-separated. Variants (`Blocked By:`, `Blocked by #12` without a colon,
space-separated `#12 #14`) are **not** recognized and are silently treated
as no dependency at all, same as a body with no `Blocked by` line at all.
This is intentional, not an oversight: the producing skill
(`gf-issue-decompose`) always emits the canonical form, so loosening the
regex would only widen the ambiguity surface for hand-edited bodies without
a corresponding real need — YAGNI.

### Resolving blocker completion

An Issue number that does not appear in `open_issues` is either closed or
does not exist — the two are distinguished with one extra lookup:

```python
open_numbers = {i.number for i in open_issues}

for blocked, blockers in list(edges.items()):
    for b in list(blockers):
        if b in open_numbers:
            continue   # still open — edge stays, feeds cycle detection below
        view = gf_issue_view(b)   # not in open list: closed, or doesn't exist
        if view is None:
            raise WorkflowBatchError(
                f"Issue #{blocked} 的 Blocked by 引用了不存在的 #{b}"
            )
        if view.state != "closed":
            continue          # not closed after all — keep the edge, stay blocked
        blockers.discard(b)
```

`WorkflowBatchError` here means: **stop before dispatching anything this
run**, surface the message to the user, do not enter Discussion Mode, do
not fall back to a partial dispatch.

### Cycle detection

Only edges where both ends are still open can participate in a cycle
(a closed blocker already had its edge dropped above). Three-color DFS:

```python
WHITE, GRAY, BLACK = 0, 1, 2

def find_cycle(edges: dict[int, set[int]]) -> list[int] | None:
    # Explicit-stack DFS, not recursive: an unbounded dependency chain must
    # raise WorkflowBatchError (a real cycle) or return None, never an
    # uncaught RecursionError from a deep-but-acyclic chain.
    color = {}

    for start in list(edges):
        if color.get(start, WHITE) != WHITE:
            continue
        stack = [(start, iter(edges.get(start, ())))]
        path = [start]
        color[start] = GRAY
        while stack:
            node, neighbors = stack[-1]
            b = next(neighbors, None)
            if b is None:
                color[node] = BLACK
                path.pop()
                stack.pop()
                continue
            if color.get(b, WHITE) == GRAY:
                return path[path.index(b):] + [b]
            if color.get(b, WHITE) == WHITE:
                color[b] = GRAY
                path.append(b)
                stack.append((b, iter(edges.get(b, ()))))
    return None

cycle = find_cycle(edges)
if cycle is not None:
    raise WorkflowBatchError(
        "依赖成环: " + " → ".join(f"#{n}" for n in cycle)
    )
```

Same stop semantics as the missing-reference case above: found a cycle →
stop before dispatching anything, surface the cycle path, do not
auto-break it (re-slicing is the user's call, per
`skills/gf-issue-decompose/references/dependency-edges.md`'s "one
direction only; a cycle means the slices are wrong").

### Computing the ready set

```python
ready = [i for i in pending if not edges.get(i.number)]
ready.sort(key=lambda i: i.number)   # same tie-break as existing pending sort;
                                      # Issues without any Blocked by declaration
                                      # keep their original relative order
```

`pending` Issues not in `ready` (unsatisfied blockers remain) stay in
`pending` but are excluded from this round's `candidates`. The next round
re-derives `open_issues`/`bodies` from scratch — once a blocker closes
(via `gf-workflow` delivery or a manual close), the edge no longer
survives the "not in `open_numbers`" check above and the blocked Issue
becomes ready with no extra persisted state.

## Serial Dispatch Loop (full pseudocode)

```
discussion_attempted = false
blocked_exit = false          # true only when the loop stops because `ready` is
                               # genuinely exhausted this round — not when it stops
                               # because `--limit` was reached (that round's `pending`
                               # is non-empty too, but for an unrelated reason: it just
                               # hasn't been reached yet, not blocked)
dispatched = 0                # count of Issues dispatched this run, bounds --limit
attempted = set()             # in-memory only, scoped to this invocation, never
                               # persisted to disk — guards against re-dispatching
                               # an Issue that aborted before it became "covered"
loop:
    if limit is set and dispatched >= limit: break
    pending = derive_pending()   # recomputed every iteration, see above
    if pending is empty:
        if not discussion_attempted:
            run_discussion_mode()
            discussion_attempted = true
            continue   # recompute pending, which now includes new Issues
        else:
            break       # nothing left even after discussion mode
    ready = resolve_dependencies(pending)   # only runs when there's something to
                                             # dispatch this round — an empty `pending`
                                             # means every open Issue is already
                                             # covered by some contract, so a cycle or
                                             # bad reference among them is irrelevant
                                             # to this round and must not block
                                             # Discussion Mode above. Re-lists open
                                             # Issues itself, every round; see
                                             # Dependency Resolution above; raises
                                             # WorkflowBatchError → abort the whole run,
                                             # no dispatch this run
    candidates = [i for i in ready if i.number not in attempted]
    if candidates is empty:
        blocked_exit = true   # this round genuinely ran out of dispatchable
                               # candidates, as opposed to `--limit` cutting the
                               # loop short before `pending` was even re-derived
        break           # ready set exhausted, or all remaining candidates already
                         # attempted this run — normal stop, not an error: pending
                         # may still hold Issues waiting on a blocker to close
    issue = candidates[0]
    result = Agent(subagent_type: default, prompt: f"/gf-workflow #{issue.number}")
    attempted.add(issue.number)   # add regardless of outcome, before next iteration
    dispatched += 1
    summary.append({issue: issue.number, contract: result.contract_path,
                     delivery: result.pr_url or result.merge_commit,
                     outcome: result.outcome})   # success | failed | rejected
print_summary_table(summary)
if blocked_exit and pending:   # loop exited via `candidates is empty` with
                                # unmet-dependency Issues still in `pending` —
                                # distinguish this from a `--limit` cutoff, where
                                # `pending` is also non-empty but for an unrelated
                                # reason (not reached yet, not blocked), and from a
                                # clean run where nothing was left to do
    print(f"⏸ {len(pending)} Issue(s) still blocked on unmet dependencies: "
          + ", ".join(f"#{i.number}" for i in pending))
    print("Re-run /gf-workflow-batch after their blockers close.")
```

## Parameters Reference

| Flag | Default | Effect |
|------|---------|--------|
| `--limit N` | unlimited | Stop after N Issues **dispatched this run** (not a per-round candidate cap) |
| `--label <label>` | none | Only consider Issues carrying `<label>` as candidates |
