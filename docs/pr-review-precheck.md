# PR review semantic precheck (#386)

`gf pr precheck` is an optional, read-only input to `gf-pr-review`. It does
not fetch, comment on, approve, reject, or merge a PR. The existing review
skill still reads the full diff and tests before any formal verdict.

## Input and use

Collect `gf pr view <number>` and `gf pr diff <number>` locally, then prepare
a reviewed JSON file. Do not pass a raw platform response or an entire diff.
The caller supplies one entry per changed file; local code applies file,
hunk, size, and sensitive-line filters before constructing a provider request.

```json
{
  "title": "Fix export retry",
  "description": "Show a retry action when export fails",
  "visibility": "public",
  "testStatus": "passed",
  "files": [
    {
      "path": "src/export.rs",
      "additions": 2,
      "deletions": 1,
      "patch": "@@ -1 +1,2 @@\n-old()\n+new()\n+retry()",
      "binary": false
    }
  ]
}
```

Run `gf pr precheck --input <file> --output json` for deterministic facts
without a provider. Add `--live` for an explicit Jev call. This requires a
binary built with `gitflow-jev`, `GF_DECISION_PROVIDER=jev`, and a TypeSafe
key from `TYPESAFE_API_KEY` or the macOS `gitflow-cli-typesafe` Keychain item.
`visibility` must be `public`, `private`, or `unknown`.
Private and unknown-visibility PRs require `--allow-private` as well. This
flag allows sending only the reviewed, filtered excerpt; check the file
before using it. `--response <file>` replays a saved typed response offline.

At most 40 files enter the local collector. Each patch is capped at 6,000
bytes and 20 hunks; the combined provider excerpt is capped at 6,000 bytes,
with at most 1,200 bytes per file and 20 selected hunks overall. Binary,
generated, oversized, and empty patches are skipped. Lines containing
recognizable credentials, authorization headers, passwords, secrets, or URLs
are removed. Title, description, and file paths containing such content are
rejected. These checks cannot find every sensitive value; manually review
the input before enabling live use. The provider receives only title,
description, test status, and selected diff lines. It does not receive the
full PR response, author, repository URL, raw logs, or all files.

## Result

`facts` contains caller-provided file counts, change statistics, test status,
selected source positions, skipped-file reasons, and the redacted-line count.
`inferences` contains seven uncalibrated 0–3 risk scores: correctness,
tests, maintainability, security, performance, compatibility, and
documentation. It also includes a topic choice, a focus hunk, and specialist
review probabilities. `unverifiedHypotheses` always requires checking the
complete diff and test results. Dimension `sources` list excerpts available
to the model; they do not establish that a defect is present there.

Low confidence yields `needs_review`; invalid or unavailable provider
responses yield `unavailable` while preserving `facts`. All states continue
the normal review. No score authorizes a verdict or external write.

## Evaluation status — 2026-09-22

Offline fake-engine checks cover a small PR, empty diff, large diff, excess
hunks, binary and generated files, sensitive lines, Chinese and English
metadata, prompt injection, provider absence, low confidence, and invalid
responses. They verify filtering, typed-response handling, and fallback but
cannot establish semantic accuracy.

| Measure | Current result | Required live measurement |
|---|---|---|
| Risk recall / false positives | Not measured | Human-reviewed, redacted completed PRs |
| Critical security / compatibility misses | Not measured | Adjudicated positive cases, reported separately |
| Dimension calibration | Not measured | Score bins against reviewer ratings |
| Evidence localization validity | Not measured | Reviewers check selected hunks against full diffs |
| Incorrect blocking | 0 by construction | Confirm no live result changes workflow or PR state |
| Abstention | 100% without provider | Live `unavailable` and `needs_review` rates |
| Latency | Not measured for live Jev | Median and p95 end-to-end time |
| Cost | $0 in default tests | Provider token usage at the current configured price |

There is no production semantic threshold or accuracy claim. A live pilot
requires reviewed samples and a TypeSafe credential accessible to the running
`gf` process. Keep saved provider responses outside the repository unless
they have been reviewed for sensitive content.

A synthetic live smoke check reached `jev-1.13.0` through the macOS Keychain.
Repeated calls also exposed one invalid Score distribution; that response
correctly fell back to `unavailable`. The smoke check does not measure risk
recall or calibration.
