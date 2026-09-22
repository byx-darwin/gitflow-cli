# Optional pipeline failure analysis (#385)

`gf pipeline analyze-failures` adds read-only classification and root-cause
grouping suggestions to the existing `gf-pipeline-analyzer` workflow. The
existing `gf pipeline report` remains the source of run statistics and keeps
working if Jev is unavailable. This command never retries or cancels a run,
edits CI configuration, or publishes a report.

## Prepare an input

Use `gf pipeline jobs --pipeline-id <ID>` and
`gf pipeline logs --pipeline-id <ID>` to select failed jobs. Create a reviewed
JSON file containing only short log excerpts. Do not pass a complete platform
response or complete CI log.

```json
{
  "pipelineId": 42,
  "failures": [
    {
      "job": "linux-build",
      "step": "cargo check",
      "exitCode": 101,
      "durationSecs": 80,
      "log": "checking core\nerror: could not compile core"
    },
    {
      "job": "mac-build",
      "step": "cargo check",
      "exitCode": 101,
      "durationSecs": 95,
      "log": "checking core\nerror: could not compile core"
    }
  ]
}
```

Run `gf pipeline analyze-failures --input <file> --output json` to inspect
deterministic telemetry without a provider. Add `--live` for an explicit Jev
call, with the `gitflow-jev` build feature, `GF_DECISION_PROVIDER=jev`, and
`TYPESAFE_API_KEY`. Use `--response <saved-typed-response.json>` for offline
replay. If the provider is missing or fails, `decisionStatus` is
`unavailable`, all categories are `unknown`, and the evidence positions
remain available for the ordinary analysis.

Input accepts up to 12 failures and 16,000 bytes of reviewed log text per
failure. The deterministic extractor keeps at most four short error lines per
failure with their one-based line numbers. Lines containing recognizable
tokens, passwords, auth headers, private URLs, or environment assignments are
replaced with a redaction marker. Job and step names with sensitive content
are rejected. At most the first three failures are sent to Jev in a single
typed request; later failures remain `unknown`. Review the input manually
because pattern matching cannot recognize every sensitive value.

For each selected failure, Choice selects one of eight categories:
build/compile, test assertion, flaky/timeout, dependency/network,
environment/configuration, permission/authentication,
quality/security gate, or unknown. Two Noul answers estimate flaky and
external-dependency signals. Pairwise Score ratings suggest shared root
causes; a group forms only when all member pairs have a matching non-unknown
category and similarity at least 2.5 with confidence at least 0.7. The
report keeps the original job, step, exit code, duration, and redacted line
positions for each member. These are model suggestions, not verified causes.
Low-confidence or ambiguous answers set `decisionStatus: needs_review`.
If a selected failure has no usable line after redaction, its category stays
`unknown` even when a provider returns a confident guess.

## Evaluation status — 2026-09-22

Offline fake-engine tests cover compilation and assertion failures,
timeouts, network failures, configuration failures, permission errors,
quality gates, mixed failures, cross-job grouping, disagreeing pairs,
provider failure, low confidence, malformed answers, oversized logs, and
sensitive log redaction. These checks establish schema and fallback behavior,
not semantic accuracy.

| Measure | Current result | Required live measurement |
|---|---|---|
| Category accuracy | Not measured | Human-labeled redacted failures across all categories |
| Root-cause grouping precision / recall | Not measured | Adjudicated cross-job pairs |
| Flaky false positives / negatives | Not measured | Repeated-run history with reviewer labels |
| Unknown coverage | 100% without provider | Live fraction classified versus unknown |
| Redaction | Tested on known patterns | Review live excerpts for novel secret forms |
| Latency | Not measured for live Jev | Median and p95 end-to-end time |
| Cost | $0 in default tests | Provider token counts at the current price |

No automatic remediation threshold is deployed. A live pilot requires a
reviewed, redacted dataset and a TypeSafe credential visible to the running
`gf` process. Keep saved responses outside the repository unless reviewed
for sensitive content.
