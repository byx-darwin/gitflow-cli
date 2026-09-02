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
stays uncovered on disk entirely, so it becomes `pending[0]` again next
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

## Serial Dispatch Loop (full pseudocode)

```
discussion_attempted = false
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
    candidates = [i for i in pending if i.number not in attempted]
    if candidates is empty:
        break           # everything remaining was already attempted this run — stuck
    issue = candidates[0]
    result = Agent(subagent_type: default, prompt: f"/gf-workflow #{issue.number}")
    attempted.add(issue.number)   # add regardless of outcome, before next iteration
    dispatched += 1
    summary.append({issue: issue.number, contract: result.contract_path,
                     delivery: result.pr_url or result.merge_commit,
                     outcome: result.outcome})   # success | failed | rejected
print_summary_table(summary)
```

## Parameters Reference

| Flag | Default | Effect |
|------|---------|--------|
| `--limit N` | unlimited | Stop after N Issues **dispatched this run** (not a per-round candidate cap) |
| `--label <label>` | none | Only consider Issues carrying `<label>` as candidates |
