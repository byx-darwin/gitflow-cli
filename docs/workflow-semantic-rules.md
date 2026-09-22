# Workflow semantic rule checks (Issue #388)

`gf workflow semantic-check` adds optional Jev observations at the planning, execution, and delivery phases. It reads the repository's versioned allowlist at `config/workflow-semantic-rules.json` and records results in the active workflow contract's `semantic_checks` array. The contract's normal phase gates, tests, lint, security checks, user approvals, and permissions remain authoritative.

## Write a rule

The [JSON Schema](../config/workflow-semantic-rules.schema.json) and runtime validator both enforce `schemaVersion: 1`, a stable `policyVersion`, at most 16 unique rule IDs, a phase (`plan`, `execute`, `deliver`), repository-relative file or directory prefixes, a short question, question type, severity, threshold, and failure policy. A directory prefix ends in `/`; a file prefix must match exactly. No globs or arbitrary commands are supported. Review policy changes in the repository before running them. Increment `policyVersion` whenever rules, questions, or thresholds change.

- `noul`: yes means a possible violation; `threshold` is the minimum yes probability.
- `score`: levels run from poor compliance to good compliance; normalized score below `threshold` is a possible violation. Provider confidence below the threshold yields `needs_review`.
- `choice`: include `none` plus violation categories. A non-`none` category is a possible violation; low provider confidence yields `needs_review`.
- `observe`: record the inference. `warn`: display a warning for a sufficiently confident possible violation. Neither changes workflow gates.
- `require_review`: reserved. The runtime rejects it until a reviewed labeled corpus supports the specific rule under [offline calibration](./decision-offline-evaluation.md), the policy is explicitly promoted, and the implementation verifies that evidence. The current example policy uses only `observe` and `warn`.

Keep questions and summaries free of secrets, raw logs, unrelated snippets, URLs, and instructions to the agent. Input limits are 64 paths and a 2 KiB reviewed summary. Question text is limited to 512 bytes. The CLI rejects obvious credential-like content before a provider call; callers still need to review the summary. Provider responses are schema checked. An invalid, missing, timed-out, or failed provider result becomes `unavailable`. The model receives path names and the reviewed summary; it does not receive file contents automatically.

## Run

Prepare an input file:

```json
{"phase":"execute","changedPaths":["crates/core/src/lib.rs"],"summary":"Adds an optional local advisory check without changing phase gates."}
```

Then run from the repository root, with an active contract in phase 3:

```sh
gf workflow semantic-check --workflow-id wf-2026-09-22-001 --input .cache/workflows/semantic-input.json
GF_DECISION_PROVIDER=jev gf workflow semantic-check --workflow-id wf-2026-09-22-001 --input .cache/workflows/semantic-input.json --live
```

The first invocation records `unavailable` for matched rules because live provider use is opt-in. `--response FILE` accepts a JSON object keyed by rule ID, with each value a provider-neutral `DecisionResponse`, for offline replay. `--rules FILE` selects a different versioned repository policy during testing. Omit `--live` to disable Jev immediately. To disable all semantic checks, set `rules` to `[]` in the policy; the command records `not_applicable` with the policy version. Missing/failed Jev calls return successfully with `unavailable`; they do not pause the workflow.

Each audit record contains the actual policy version, phase, rule ID, matched path evidence, conclusion, confidence, status, advice, and an explicit `hypothesis` flag for possible violations. It excludes the summary, secrets, and raw provider text. Treat model findings as leads to verify against files and deterministic checks. A duplicate rule ID or unsupported review policy fails configuration validation before any provider call.

## Test and rollout

Run `cargo test -p gitflow-core workflow_semantic`, `cargo test -p gitflow-cli --bin gf`, and validate both JSON schema files. Replay labeled cases through `gf decide eval` and `gf decide calibrate` before considering stronger policy. Check hit/miss/not-applicable paths, high and low confidence, malformed responses, provider timeouts, and prompt injection summaries. The included examples and synthetic tests establish plumbing only; they do not measure real-world accuracy or justify `require_review`.
