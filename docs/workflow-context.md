# Selective workflow context (Issue #391)

`gf workflow context build` creates a derived active-context manifest for one active workflow. It reads the existing contract and a reviewed catalog of evidence files. The contract and all source files remain unchanged. The manifest is written to `.cache/workflows/context/<workflow-id>.json`; it contains paths, SHA-256 source hashes, selection reasons, model identifiers, and token usage, but no source body or raw provider request/response.

Put the catalog under `.cache/` so it does not alter the worktree fingerprint. Each `source` is a repository-relative file of at most 64 KiB. Classify every user constraint, permission decision, failure, test conclusion, unresolved blocker, and security finding with its corresponding `kind`. These six categories and the workflow contract are always retained. The four newest noncritical items are also retained. Older items are parked unless a valid optional Score/Noul response finds them relevant. A missing or failed provider leaves the deterministic selection usable and marks its status `unavailable` or `partial`.

```json
{
  "schemaVersion": 1,
  "objective": "Implement the current workflow phase",
  "evidence": [
    {
      "id": "user_constraint_1",
      "kind": "user_constraint",
      "source": "docs/requirements.md",
      "summary": "Preserve the confirmed user constraints",
      "recordedAt": "2026-09-22T10:00:00Z"
    }
  ]
}
```

```sh
gf workflow context build --workflow-id wf-2026-09-22-001 --input .cache/context-input.json
gf workflow context build --workflow-id wf-2026-09-22-001 --input .cache/context-input.json --live
gf workflow context build --workflow-id wf-2026-09-22-001 --input .cache/context-input.json --labels .cache/context-labels.json
gf workflow context restore --workflow-id wf-2026-09-22-001 --input .cache/context-input.json --id old_result
gf workflow context restore --workflow-id wf-2026-09-22-001 --input .cache/context-input.json --source docs/old-result.md
gf workflow context restore --workflow-id wf-2026-09-22-001 --input .cache/context-input.json --keyword build
```

Live scoring requires `GF_DECISION_PROVIDER=jev` and `--live`. A saved response map can be supplied with `--response FILE` for offline replay. Only the reviewed `objective`, `summary`, kind, and phase enter a provider request. Summaries reject credential-like text, URLs, environment assignments, and control characters. Source file contents and complete logs never enter the request. Score thresholds vary by noncritical evidence kind. Invalid responses fall back to deterministic selection and never override critical retention.

`restore` checks the current contract, phase, objective, catalog, source hashes, HEAD, and changed file contents before reading a parked source. Rebuild after any of these change. Restoration is read-only and prints recovered content to the caller. The build output includes byte and approximate token reduction (bytes divided by four), critical evidence recall, erroneous critical drops, elapsed time, reported provider token usage, and nullable cost. Supply optional `--labels` JSON such as `{"schemaVersion":1,"requiredIds":["user_constraint_1"]}` to measure human-labeled required-evidence recall and erroneous drop rate. Without labels, those metrics are null. Cost remains null because the decision adapter does not provide billing data. Search parked items by exact ID, exact source path, or summary keyword.

The [Jevable instant compaction example](https://jevable.com/project/2100694549362553153) motivates scoring old tool results; [Jev compaction for OMP](https://jevable.com/project/2100987079488409741) describes parking less useful history. This implementation keeps the workflow contract and source files as the audit record and uses the model only for optional selection advice.
