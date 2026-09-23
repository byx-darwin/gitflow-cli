# Optional workflow recommendation (#383)

`gf workflow recommend` displays the current deterministic mode rule together with optional typed Jev advice. It is read-only and runs before `gf workflow create`; it never creates a contract, changes a mode, approves an action, or skips a gate.

```sh
gf workflow recommend --input tests/fixtures/workflow/recommend-input.json
gf workflow recommend --input tests/fixtures/workflow/recommend-input.json --response tests/fixtures/workflow/recommend-response.json
```

The input JSON contains a reviewed `title` (at most 160 bytes), `summary` (at most 512 bytes), `labels` (at most 16 safe identifiers), and optional `userMode` (`fast`, `standard`, `full`). The command rejects obvious credentials and HTTP(S) links before any provider request. Keep only task facts in the summary: no raw logs, source files, environment variables, account identifiers, or private URLs. Input and response files are bounded and path checked.

Without `--response` or `--live`, the command makes no network call and returns `status: unavailable` with the rule mode. `--response` accepts a saved provider-neutral `DecisionResponse`. `--live` is an explicit opt-in; it calls Jev only if the CLI was built with `gitflow-jev` and `GF_DECISION_PROVIDER=jev` plus credentials are configured. A missing or failing provider returns the rule result with `unavailable`, without blocking workflow startup. The request uses fixed Choice (`mode`), Score (`complexity`), and Noul (`sensitive_action`) questions; task text is data, never an instruction to the provider.

The output separates `ruleMode`, `suggestedMode`, and `effectiveMode`. `effectiveMode` is always the explicit `userMode` or deterministic rule. A model confidence below 0.9 yields `needs_review`; high confidence with a differing mode yields `conflict`. `riskFlags` are advisory and cannot lower existing permission requirements. `candidatePhases` always lists all four contract phases; fast mode can follow its existing Phase 2 exemption only under the contract gate rules. The existing confirmation flow chooses the final mode.

The deterministic rule follows issue labels first (`good first issue`/`good-first-issue`/`kind/typo` → fast; `kind/feature`/`type:feature` → full), then title (`feat`/breaking → full; small docs/chore/typo → fast), otherwise standard. Suggested skills are derived locally. Jev cannot select tools, authorize writes, or alter execution strategy.

For repeatable semantic evaluation, feed the fixed request schema into `gf decide eval` with labeled synthetic or reviewed historical tasks. Score mode accuracy and risk false negatives separately; record rule conflicts, abstentions (`needs_review`/`unavailable`), latency, and input/output cost. The sample response is synthetic wire-format data, not evidence of production accuracy. Calibrate mode and risk thresholds per question with `gf decide calibrate` only after enough labeled examples are available.
