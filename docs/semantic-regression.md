# Optional semantic regression oracle (Issue #389)

`gf regression semantic-eval` evaluates captured, user-visible CLI behavior after the deterministic harness has asserted the exit code, JSON schema, and side effects. It does not execute commands or change fixtures. A failed factual assertion appears as `deterministic_failure`, skips the provider, and gives the command a nonzero exit code. A semantic `finding` alone never changes the exit code or blocks release.

The [versioned suite schema](../tests/fixtures/regression/semantic-suite-v1.schema.json) defines stable contract IDs, owners, expected intent, an exact provider-state field allowlist, confidence thresholds, positive and negative golden examples, and captured cases. Cases carry a fixed suite seed, platform, bounded command/output summaries, three deterministic assertion results, an optional independent human label, and an adversarial flag. Only the allowlisted fields go into a provider request. The oracle asks three typed questions: Noul for contract satisfaction, Choice for regression category, and Score for clarity/actionability/consistency. Output summaries remain untrusted data, never instructions to the agent.

## Run and replay

First run the regular read-only smoke and contract tests. Build a suite from their **actual** captured results; never mark a failed assertion as passed to invite semantic analysis. Then run:

```sh
bash scripts/smoke-test.sh --platform github --read-only
gf regression semantic-eval --suite tests/fixtures/regression/semantic-suite-v1.json --responses tests/fixtures/regression/semantic-responses-v1.json
```

The checked-in fixture intentionally has one factual failure, so this example emits a JSON report and exits nonzero. To inspect only advisory behavior, prepare a capture suite whose factual assertions all passed. Missing responses become `unavailable`, and low confidence becomes `low_confidence`. Provider errors never alter factual results.

Live requests are opt-in and bounded to 50 cases, at most four concurrent requests. Save observations, including response and measured latency, for repeatable offline evaluation:

```sh
GF_DECISION_PROVIDER=jev gf regression semantic-eval --suite .cache/regression/captured-suite.json --live --parallel 2 --save-responses .cache/regression/live-responses.json
gf regression semantic-eval --suite .cache/regression/captured-suite.json --responses .cache/regression/live-responses.json
```

The CLI writes saved live responses as a new private file under `.cache/regression/`. It will not overwrite an existing file. Without `--live`, no provider call is made, so ordinary CI needs no Jev key. Use `--input-price-per-million-usd` and `--output-price-per-million-usd` together to add an estimated token cost to the report.

## Privacy and evaluation

Before transmission, the CLI redacts URL-like tokens, paths, email-like identities, environment assignments, and common credential labels and values. It rejects oversized artifacts and asks the provider only about short summaries, never complete terminal logs. Review summaries before live use; deterministic redaction cannot identify every sensitive free-form value. The report stores case IDs, factual outcomes, semantic status, category, confidence, quality, model ID, token counts, latency, optional estimated cost, and labeled-case precision/recall. It does not store raw output summaries.

The checked-in [synthetic suite](../tests/fixtures/regression/semantic-suite-v1.json) covers help text, error actionability, JSON explanation, cross-platform equivalence, adversarial text, and a factual failure. Its [saved responses](../tests/fixtures/regression/semantic-responses-v1.json) test replay plumbing, not model accuracy. A human-labeled, representative corpus is still needed to measure real precision, recall, false-positive rate, latency, cost, and calibration before considering any stronger gate. The current oracle has no mechanism to promote a model finding to a release block.
