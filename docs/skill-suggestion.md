# Optional gf Skill suggestion (#384)

`gf skills suggest` recommends IDs from the 30 `gf-*` skills bundled with this binary. It never installs, invokes, or authorizes a Skill. Read the recommended `SKILL.md` and apply its trigger and permission rules before use.

```sh
gf skills suggest --query "审查这个 PR 并提交结论" --output json
printf '%s\n' 'Review this PR' | gf skills suggest --stdin --output json
gf skills suggest --query "Review this PR" --response .cache/decision/saved-skill-response.json
```

`--query` and `--stdin` are mutually exclusive. Input is limited to 1,024 bytes; use `--stdin` when shell history should not hold task text. The query is rejected if it contains an obvious credential or an HTTP(S) URL. The provider request contains only the bounded task text and the bundled Skill IDs with their short `SKILL.md` descriptions as Choice options. It does not send source files, environment variables, authentication data, or conversation history.

By default the command makes no network call and returns `unavailable`. `--response` replays a saved provider-neutral `DecisionResponse` offline. `--live` explicitly enables Jev when the binary has the `gitflow-jev` feature and `GF_DECISION_PROVIDER=jev` plus credentials are configured. A missing or failing provider returns `unavailable` without affecting `gf skills list`, validation, or installation. There is no implicit live call.

The JSON result contains an overall `status` and per-intent `suggestions` with `skillId`, `confidence`, a short catalog-derived `reason`, up to three meaningful `alternatives`, and `decisionStatus` (`accepted`, `needs_review`, `unavailable`). A `none` choice or confidence below 0.85 requires review. Up to three intents can be separated with `并且`, `同时`, `然后`, ` and `, or ` then `; every multi-intent result has `conflictReviewRequired: true` and overall `needs_review` until a person checks whether the suggested skills overlap or conflict. Model output cannot name a Skill outside the current catalog.

The 0.85 cutoff is an advisory default, not a production-calibrated threshold. [Offline evaluation](./decision-offline-evaluation.md) and reviewed historical labels should be used before treating any confidence as reliable. To disable Jev, omit `--live`; the command still gives an explicit unavailable result. The recommendation does not supersede Skill instructions or user authorization.
