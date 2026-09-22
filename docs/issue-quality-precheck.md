# Issue requirement-quality precheck (#387)

`gf issue precheck` is a read-only advisory step for `gf-issue-review`. The
existing four-dimension human review and confirmation before commenting remain
authoritative.

## Input and use

Fetch an Issue with `gf issue view <number> --output json`, then manually select
and redact only the fields needed for this precheck. Do not pass the raw Issue
response. The input file has this exact shape:

```json
{
  "title": "feat: export a report",
  "body": "Users need CSV export. Acceptance: selecting Export downloads a CSV with all visible rows.",
  "labels": ["type:feature"],
  "milestone": null,
  "comments": []
}
```

Run `gf issue precheck --input <file> --live --output json`. The provider is
contacted only if the binary has the `gitflow-jev` feature, the command has
`--live`, `GF_DECISION_PROVIDER=jev`, and `TYPESAFE_API_KEY` is set in the
environment. An unavailable provider produces `status: unavailable`; continue
the existing four-dimension review. Use `--response <file>` for offline replay
of a saved typed `DecisionResponse` without contacting a provider.

Input is bounded to a 200-byte title, 8,000-byte body, 16 labels, optional
128-byte milestone title, and three selected comments of at most 500 bytes
each. URLs and recognizable credentials are rejected before provider access.
The provider receives only these fields and ten fixed questions: five Score
dimensions (title, context, goal, acceptance, slice), four Noul signals
(missing acceptance, untestable acceptance, mixed goals, hidden dependency),
and a Choice for the main gap. No author, assignee, Issue URL, raw response,
environment value, or full comment history is sent.

## Reading the result

`dimensions` contains 0–3 scores and confidence values. `source` names the
input field used for a model inference; it is not a quotation or proof. The
four `signals` are probabilities, not facts. `clarifyingQuestions` is a
bounded list of suggested questions ordered by impact. The reviewer must
check each suggestion against the full Issue before including it in a report.
If any Score or Choice confidence is below 0.7, or a Noul probability falls
between 0.3 and 0.7, status is `needs_review`; the skill follows its existing
manual analysis. The 0.7 cutoff only controls whether to display an advisory
status. It is not calibrated for automatic decisions, and no score may block
work or authorize a write. Provider errors and malformed responses yield
`unavailable`.

## Evaluation status — 2026-09-22

The default test suite uses a fake decision engine and covers complete, blank,
short, mixed-goal, solution-only, Chinese, English, credential-bearing, and
prompt-injection inputs. It verifies schema, bounded output, fallback, and
read-only behavior; fake responses cannot establish semantic accuracy.

| Measure | Current result | Required next measurement |
|---|---|---|
| Dimension calibration | Not measured | Human-rated bilingual Issue corpus with per-dimension agreement and calibration bins |
| Missing-item recall | Not measured | Human labels for absent and untestable criteria, mixed goals, and hidden dependencies |
| Incorrect blocking | 0 by construction | Confirm live findings never change workflow or Issue state |
| Question usefulness | Not measured | Blind reviewer rating of ranked questions |
| Language difference | Not measured | Separate Chinese and English results |
| Abstention | 100% without provider | Fraction of live requests returning `unavailable` or `needs_review` |
| Latency | Not measured for live Jev | Median and p95 end-to-end time |
| Cost | $0 in default tests | Provider-reported tokens multiplied by the current configured price |

No live Jev key was available for this initial implementation. No production
threshold or semantic-quality claim is established. Live evaluation must use
reviewed, redacted public Issues and be explicitly enabled; saved responses
must stay outside the repository unless they have been reviewed for sensitive
content.
