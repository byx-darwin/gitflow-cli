# Skill suggestion synthetic evaluation (#384)

The [versioned fixture](../tests/fixtures/skill-suggestion/evaluation-v1.json) has seven synthetic tasks: Chinese PR review, English Issue statistics, mixed-language multi-intent routing, an ambiguous request, a prompt-injection attempt, a low-confidence request, and provider failure. The `test_should_replay_synthetic_skill_routing_metrics` test loads the current bundled catalog, constructs fake typed Choice responses, validates every response against the request, and checks these metrics offline.

| Measure | Synthetic result |
|---|---:|
| Top-1 exact match on answered, matchable intents | 5/6 (83.3%) |
| Top-3 recall on answered, matchable intents | 6/6 (100%) |
| Overall abstention / human review | 4/7 cases (57.1%) |
| Wrong high-confidence recommendations | 1/3 accepted cases (33.3%) |
| Median / p95 saved latency | 125 / 180 ms |
| Estimated input + output token cost | $0.0000612 total |

The injection case intentionally has a wrong high-confidence choice. It shows why a suggestion must not invoke a Skill or grant permission. The multi-intent case requires review even though both per-intent matches are confident. These fake responses verify the routing and measurement path; they do not estimate live Jev accuracy. Before changing the 0.85 advisory cutoff, collect reviewed historical queries, measure false high-confidence recommendations, and calibrate per question with an adequate sample.
