# Typed Issue and PR search (Issue #390)

`gf issue search` and `gf pr search` compile read-only natural language into the [versioned typed filter AST](../crates/core/resources/query-filter-plan.schema.json). The parser handles explicit operators, state words, issue numbers, verified user and label candidates, and `上周` / `last week` locally. Only unresolved spans and the supplied candidate values may go to the optional Jev provider. Jev returns typed Choice/Score/Noul answers; its choice must match a caller-owned option, pass schema checks, and have high confidence with low ambiguity. It cannot return shell text or a raw platform query.

## Preview and execute

```sh
gf issue search --query 'state:open label:bug' --known-label bug --explain
gf issue search --query 'state:open label:bug' --known-label bug --limit 50
gf pr search --query 'state:open draft:true' --explain
gf issue search --query 'state:open 高优先级' --known-label priority:high --response tests/fixtures/query/priority-response-v1.json --explain
```

`--explain` prints the AST, source span and confidence for every field, unresolved spans, and how each field maps to the selected platform. It makes no platform request. Execution verifies label names against the repository and rejects unknown labels, unsupported fields, and local filtering over a truncated 1000-row result set. User logins must be supplied in `--known-user` values; without a verified candidate, explicit author/assignee filters are rejected rather than guessed. Candidate lists should come from a trusted repository or organization roster. Number, author, assignee, creation date, negative label, draft, merged, and PR text predicates use bounded local filtering of existing provider list results. Issue state and positive labels map to native list arguments; Issue text uses provider search plus a local title/body check. PR CI, labels, and assignee are currently unsupported because the existing PR provider data cannot verify them. `merged:true` is supported only on GitHub because the other adapters do not provide reliable merged timestamps.

When no state is specified, execution requests all states so number, author, and date filters can also find closed items. Relative dates use the local calendar day at compile time.

Explicit filters include `state:open|closed|all`, `number:123` or `#123`, `author:LOGIN`, `assignee:LOGIN` (Issue), `label:NAME`, `-label:NAME`, `after:YYYY-MM-DD`, `before:YYYY-MM-DD`, `text:"quoted phrase"`, and PR `draft:true|false` or GitHub `merged:true|false`. Contradictory fields, impossible dates, unknown operators, and write-like text are rejected or left unresolved. An unresolved span prevents execution. For an optional model suggestion, pass `--live` with `GF_DECISION_PROVIDER=jev`, or `--response FILE` for an offline typed response. Review the plan with `--explain`; `--accept-inferred` is required before executing model-inferred fields. Provider absence leaves the deterministic subset visible and the remaining spans marked unresolved.

No natural language query invokes close, merge, label edits, or other write actions. The platform adapters receive only validated typed list arguments; local filters inspect returned structured objects. Provider results are never inserted into a command string.

## Offline evaluation

The [synthetic fixture](../tests/fixtures/query/evaluation-v1.json) fixes the date and candidate lists for replay. Ten cases cover English and Chinese, mixed language, relative dates, quoting, negation, contradictory state, invalid date, missing label, ambiguity, PR draft, and prompt injection. The deterministic parser matches all ten expected AST/rejection outcomes; field F1 is 1.0, clarification rate 0.6, and dangerous complete misparse rate 0/5 on this fixture. These are synthetic plumbing checks, not estimates of live Jev accuracy. A reviewed historical query corpus and saved typed Jev responses are still needed to assess actual exact AST match, field F1, clarification burden, and dangerous misparse rate before expanding automatic inference.
