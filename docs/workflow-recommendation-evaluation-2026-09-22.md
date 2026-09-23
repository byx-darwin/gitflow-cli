# Workflow recommendation synthetic evaluation (#383)

Six synthetic, reviewed tasks cover a small docs change, a cross-domain feature, an ambiguous fix, a high-risk release, an injection attempt, and a mixed-language refactor. The [fixture](../tests/fixtures/workflow/evaluation-fixtures-v1.json) and [saved responses](../tests/fixtures/workflow/evaluation-responses-v1.json) are versioned and contain no credentials or private URLs. The responses simulate a provider; these figures test the evaluation path and safety behavior, not Jev's real-world quality.

Replay offline:

```sh
gf decide eval --fixtures tests/fixtures/workflow/evaluation-fixtures-v1.json --responses tests/fixtures/workflow/evaluation-responses-v1.json
```

| Measure | Synthetic result |
|---|---:|
| Mode exact accuracy | 5/6 (83.3%) |
| Sensitive-action accuracy | 5/6 (83.3%) |
| Sensitive-action false negatives | 1/2 positive cases |
| Rule/model mode conflicts | 1/6 |
| Low-confidence abstentions at 0.90 | 1/6 |
| High-confidence mode coverage at 0.90 | 5/6 (83.3%) |
| Human acceptance rate | Not measured in synthetic replay |
| Median / p95 latency | 145 / 220 ms |
| Estimated input + output token cost | $0.0000612 total |

The mixed-language case intentionally contains a model risk false negative. The deterministic `delete` flag still requires review, and `effectiveMode` remains the rule or explicit user choice. This tiny synthetic set is insufficient for threshold calibration or an automation claim; `gf decide calibrate` returns `insufficient_evidence` until a complete group has at least 30 accepted labeled examples and its Wilson lower bound meets the requested target. A production assessment needs reviewed historical cases and explicit human acceptance tracking.
