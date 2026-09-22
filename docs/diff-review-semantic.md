# Optional semantic notes for the diff review page

Issue #346's `scan` and `render` steps remain offline, deterministic, and
Python standard-library only. Issue #398 adds a separate, **opt-in** enrichment
step that fills each source file's `pseudocode` and `call_tree` fields in the
same `annotations.json`. The renderer already displays those fields when set.

## Mechanism

This step uses the authenticated Claude Code CLI as an LLM provider. A model
can summarize behavior and identify calls visible in a patch across languages;
a manifest parser or a regular expression cannot reliably infer either from a
diff. The output is a review aid, not a verified whole-program call graph.
It may vary between runs and requires the CLI, authentication, network access,
and model usage. The base renderer has none of those requirements.

Run it only when sending the changed source to the configured model is
appropriate for the project:

```bash
make render-diff-review-semantic RANGE=dev..HEAD
```

Run this from a separate terminal, outside an active Claude Code session.
Claude Code rejects a nested `claude --print` process; when detected, this
step leaves the optional fields empty and the base HTML still renders.

This runs `scan → enrich → render` and writes the JSON and HTML under
`.cache/diff-review/`. For an existing annotations file:

```bash
python3 scripts/enrich-diff-review.py .cache/diff-review/dev..HEAD.json
python3 scripts/render-diff-review.py render .cache/diff-review/dev..HEAD.json
```

The enrichment command invokes `claude --print` with a JSON schema, no tools,
and no session persistence. By default it processes at most 10 changed source
files and 12,000 patch characters per file; `--max-files`, `--max-chars`,
`--model`, and `--timeout` can override those limits. It skips binary files,
non-source files, and files without changed lines. It writes the JSON atomically.

If the CLI is absent, times out, returns invalid JSON, or fails for a file,
that file's optional fields remain `null`; the other files and the HTML render
continue. The command reports generated/failed/skipped counts. Re-running the
base `scan` resets enrichment and never invokes the model. `gf-workflow` uses
the base scan/render path unless a user explicitly chooses this semantic step.
