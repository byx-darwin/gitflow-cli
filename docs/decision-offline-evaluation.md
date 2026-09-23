# Decision offline evaluation

`gf decide eval` scores versioned labeled requests against saved typed responses. The default path reads local JSON only; it does not contact Jev or require an API key. A runnable mixed Noul/Choice/Score pair is in [`tests/fixtures/decision/mixed-v1.json`](../tests/fixtures/decision/mixed-v1.json) and [`mixed-responses-v1.json`](../tests/fixtures/decision/mixed-responses-v1.json).

```sh
mkdir -p .cache/decision
gf decide eval --fixtures tests/fixtures/decision/mixed-v1.json --responses tests/fixtures/decision/mixed-responses-v1.json > .cache/decision/report.json
gf decide calibrate --report .cache/decision/report.json --question kind --target-accuracy 0.99
gf decide compare --baseline .cache/decision/report.json --candidate .cache/decision/report.json
```

The one-case example verifies the wire format but is too small for calibration. A recommendation requires a complete report and at least 30 labeled and accepted examples **within the selected question/language/risk group**. The reported 95% Wilson lower bound must meet the requested accepted-accuracy target. `insufficient_evidence`, `incomplete`, and `target_unmet` statuses provide no threshold. Recommendations are advisory and never modify production configuration.

## Artifact contract, version 1

- Fixture: `schemaVersion`, stable `datasetId`, and 1–500 `cases`. Each case has a unique `id`, `language` (`english`, `chinese`, `mixed`, `other`), `risk` (`low`, `medium`, `high`, `critical`), `source` (`synthetic`, `public_issue`, `reviewed_report`), a validated `DecisionRequest`, and `expected` answers for every question. Expected values use `{"type":"noul","yes":true}`, `{"type":"choice","choice":"bug"}`, or `{"type":"score","level":2}`. The score level is zero based.
- Saved responses: matching `schemaVersion` and `datasetId`, `provider`, optional `inputPricePerMillionUsd` and `outputPricePerMillionUsd`, and `results` keyed by `caseId`. An outcome is `{"status":"ok","response":<DecisionResponse>,"latencyMs":120}` or `{"status":"error","kind":"unavailable"}`. Error kinds are `unavailable`, `timeout`, `transport`, and `invalid_response`. Missing cases are allowed and make the report incomplete.
- Report: versions, dataset/provider, actual returned model IDs, question schema and full fixture content hashes, completeness, per-question metrics sliced by language and risk, token totals, optional estimated input, output, and combined token cost, and p50/p95 valid-response latency. Each metric has accuracy, a confusion matrix, false positives/negatives, and acceptance points containing threshold, accepted accuracy, coverage, abstention, and Wilson bounds. Missing/failed responses are excluded from accuracy and included in coverage denominators.

Keep `datasetId` stable only when the fixture is the same; use a versioned ID when publishing a revised dataset. `compare` also checks full fixture content and question schema hashes, so edits cannot silently pass as a comparable result. It records provider/model changes and reports accuracy, false-negative, or fixed-threshold coverage regressions for comparable reports. A regression remains report data, not a CI failure.

Fixture validation rejects obvious credential-like request content and any HTTP(S) URL anywhere in the serialized request, including public links. Review imported public issue text before building fixtures. Each artifact is limited to 4 MiB. Reports and saved responses can contain issue text or decisions, so keep them in the ignored `.cache/decision/` directory unless reviewed for sharing. The content hash is a change detector, not a security signature.

For a deliberate provider run, build with `gitflow-jev`, configure `GF_DECISION_PROVIDER=jev` and Jev credentials as described in [Jev decisions](./jev-decision.md), then use `gf decide eval --fixtures <file> --live --save-responses .cache/decision/<name>.json`. The live command caps a run at 50 cases and creates a private response file without overwriting an existing one. If the provider is unavailable or credentials are absent, it records `unavailable` outcomes and produces an incomplete report. Live evaluation is never selected implicitly.
