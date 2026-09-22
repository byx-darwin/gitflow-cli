# Workflow progress assessment (Issue #392)

`gf workflow progress assess` reads a bounded, structured trace for the active phase and writes an advisory report to `.cache/workflows/progress/<workflow-id>.json`. The workflow contract and its phase gates are not modified. Input events must match the contract's workflow ID, current phase, and phase start time.

```sh
gf workflow progress assess --workflow-id wf-2026-09-22-001 --input .cache/progress-trace.json
gf workflow progress assess --workflow-id wf-2026-09-22-001 --input .cache/progress-trace.json --live
gf workflow progress assess --workflow-id wf-2026-09-22-001 --input .cache/progress-trace.json --response .cache/progress-response.json --now 2026-09-22T12:00:00Z
gf workflow progress evaluate --input .cache/progress-evaluation.json
```

Example trace (hashes below are illustrative SHA-256 values):

```json
{
  "schemaVersion": 1,
  "workflowId": "wf-2026-09-22-001",
  "phase": 1,
  "phaseStartedAt": "2026-09-22T10:00:00Z",
  "events": [
    {"id": "ev-1", "at": "2026-09-22T10:10:00Z", "kind": {"type": "command_failed", "exit_code": 1, "failure_hash": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", "hypothesis_hash": null, "hypothesis": null}},
    {"id": "ev-2", "at": "2026-09-22T10:20:00Z", "kind": {"type": "completion_claim", "verified": false, "proof_hash": null}}
  ]
}
```

Supported event types are `command_failed`, `test_result`, `file_edited`, `evidence_added`, and `completion_claim`. Events carry stable IDs and timestamps, plus hashes or short reviewed hypotheses. Raw logs, source code, credentials, URLs, and long free text are rejected. The trace is capped at 128 chronological events and 64 KiB. Optional Jev receives only the last 12 event synopses, counts, and deterministic signals. Live calls require both `--live` and `GF_DECISION_PROVIDER=jev`; otherwise a saved typed decision response can be supplied for replay.

Deterministic signals cover repeated exit codes, repeated test failures, edit reversals, missing new evidence, phase stalls, unverified completion claims, and the same hypothesis across distinct failures. Each signal cites event IDs. A valid provider response can classify the blocker using closed Score, Noul, and Choice answers. The report stores source input and window hashes, policy version, accepted model ID, token usage, evidence references, and a static recovery suggestion. It does not store the provider request or response.

`healthy` means no concern; `watch` means one concerning assessment window; `review` requires a second distinct ten-minute window with a shared deterministic signal and a valid concerning provider response. A 30-minute cooldown follows `review`. Missing or invalid provider responses produce `unavailable` while preserving deterministic telemetry. These states never stop an executor, undo edits, switch models, or send a message.

For offline evaluation, pass a JSON array of labeled cases. Each case has `id`, `expectedStuck`, `stuckSinceWindow` (zero-based index or `null`), and `windows` containing `{ "trace": <trace above>, "response": <typed decision response or null>, "now": <RFC3339> }`. The command replays each case in order and reports case-level precision, recall, false-positive rate, and mean detection delay in windows. The included core tests cover normal progress, a legitimate retry, repeated failures, an edit reversal, a wrong hypothesis, an unverified claim, provider failure, and consecutive-window behavior. Synthetic test metrics do not establish production accuracy; human-labeled historical traces should be added before relying on thresholds operationally.

The current two-case synthetic evaluation fixture in the core test reports 1 true positive, 1 true negative, precision 1.0, recall 1.0, false-positive rate 0.0, and mean detection delay of one ten-minute window. It tests the metric computation and state transition, not field performance.
