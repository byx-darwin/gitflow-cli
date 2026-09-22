# Jev Issue triage pilot evaluation — 2026-09-22

## Method

This is a small, deliberately stratified pilot against the repository's own
public Issues, not a production accuracy claim. The 22 selected Issues are
`#396 #394 #378 #372 #364 #399 #395 #370 #363 #362 #393 #382 #357 #332
#331 #398 #397 #369 #346 #345 #371 #361`. Each has one `type:*` and one
`priority:*` label. Those existing labels are the reference values; they are
not independently adjudicated ground truth. Selection emphasized bugs and
high priority cases, so the sample is not representative of the full backlog.

Only each public title was sent as `state.title`. Bodies, comments, authors,
assignees, URLs, and labels were excluded. Each request used the four-question
batch in [Optional Jev decisions](./jev-decision.md): type Choice, priority
Score, security-related Noul, and blocked Noul. The model returned
`jev-1.13.0` for all 22 calls. All calls succeeded. No labels were changed.

## Results on labeled Issues

| Measure | Result |
|---|---:|
| Type Choice agreement with existing label | 16/22 (72.7%) |
| Priority Score mapped by nearest level to existing label | 11/22 (50.0%) |
| Calls completed | 22/22 |
| End-to-end CLI latency, median | 0.690 s |
| End-to-end CLI latency, maximum | 0.800 s |
| Input / output tokens | 12,438 / 2,356 |
| Estimated API charge for these 22 calls | US$0.00052 |

The charge uses TypeSafe's [Jev 1.13 model price](https://docs.typesafe.ai/models)
of US$0.042 per million input tokens; the official model page says output
tokens are free. It excludes the separate synthetic checks below.

### Type confusion matrix

Rows are existing labels; columns are Jev selections. Empty classes had no
examples in this sample.

| Existing label | bug | feature | enhancement | docs |
|---|---:|---:|---:|---:|
| bug | 10 | 0 | 0 | 0 |
| feature | 1 | 4 | 0 | 0 |
| enhancement | 0 | 4 | 1 | 0 |
| docs | 1 | 0 | 0 | 1 |

The main disagreement was `enhancement` versus `feature` (#398, #397, #346,
#345). Existing labels sometimes describe additions to an existing Skill as
enhancements even when the title says `feat`; #357 describes missing behavior
but is labeled `feature`. A title alone cannot resolve these policy choices.

### Type confidence and acceptance

| Minimum Choice confidence | Coverage | Agreement among accepted |
|---:|---:|---:|
| 0.50 | 22/22 (100%) | 16/22 (72.7%) |
| 0.70 | 20/22 (90.9%) | 16/20 (80.0%) |
| 0.90 | 17/22 (77.3%) | 13/17 (76.5%) |

Higher confidence did not consistently improve agreement, so these values
must not be used as a global automatic labeling threshold.

### Priority confusion matrix

For this *diagnostic baseline only*, the probability-weighted Score was
rounded to the nearest level: low=0, medium=1, high=2, urgent=3. The workflow
does not use that rounding rule to assign labels.

| Existing label | low | medium | high | urgent |
|---|---:|---:|---:|---:|
| low | 0 | 7 | 0 | 0 |
| medium | 0 | 7 | 2 | 0 |
| high | 0 | 2 | 4 | 0 |
| urgent | 0 | 0 | 0 | 0 |

There are no `priority:urgent` examples in the retrieved labeled Issues, so
urgent false-negative rate on real Issues is **not measurable**. Low priority
was systematically overestimated from title-only state. No priority threshold
is calibrated for deployment.

## Explicit synthetic boundary checks

Six additional, author-labeled hypothetical Issue titles exercised clear
boundaries. These are *synthetic*, separate from the 22 real Issue results.

| Scenario | Expected urgent | Score | Security Noul | Blocked Noul |
|---|---:|---:|---:|---:|
| Exposed release signing key | yes | 3.00 | 0.83 | 0.17 |
| All users unable to list Issues | yes | 2.84 | 0.07 | 0.83 |
| Main CI down, releases blocked | yes | 2.98 | 0.05 | 0.87 |
| Minor installation guide typo | no | 0.00 | 0.04 | 0.08 |
| Optional dark theme request | no | 0.25 | 0.02 | 0.11 |
| Preventive signature verification, no known exposure | no | 0.46 | 0.05 | 0.06 |

Using an exploratory Score cutoff of 2.5, there were 0/3 urgent false
negatives on these obvious synthetic positives. A Noul cutoff of 0.5 separated
the one security positive and two blocked positives from these negatives.
These checks do not establish calibrated thresholds or real-world error rates.

## Decision

Keep Jev advisory-only. `gf-issue-triage` continues to use its existing human
or deterministic classification path whenever the provider is absent, fails,
or yields an uncertain answer. It does not write labels from Jev output.
No threshold is deployed for type, priority, security-related, or blocked.

Before considering automatic acceptance, collect a reviewed corpus with body
excerpts and independently adjudicated labels, including urgent, security,
blocked, question, and no-match positives. Evaluate separate thresholds for
each decision against a held-out set and report high/urgent false negatives.
Issue #393 covers the reusable evaluation and calibration tooling.
