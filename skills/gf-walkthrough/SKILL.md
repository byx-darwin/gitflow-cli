---
name: gf-walkthrough
description: |
  Use when the user wants a delivery walkthrough, a change narrative for
  non-engineers, or evidence-graded verification of what a change was tested to.
  当用户需要交付走查包、面向非工程读者的变更说明，或需要对验证程度分级标注时使用。
allowed-tools: Read, Grep, Glob, Bash, Write
---

# gf-walkthrough — Delivery Walkthrough & Evidence Grading

Offline package: narrative, summary, graded evidence, review gate.
Anchored on a `<base>...<head>` diff; PR data enriches when present.
**Never issues a verdict** — `gf-review` does.

## Preconditions

- Read access to `git log`/`diff`/`merge-base`
- Report template at `docs/superpowers/templates/walkthrough-report-template.md`

## When to Use

| English | 中文 | Context |
|---|---|---|
| delivery walkthrough | 交付走查 | non-engineer reader |
| evidence grading | 证据分级 | verification level per claim |

## When NOT to Use

| Scenario | Why Not | Use Instead |
|---|---|---|
| Approve / request-changes verdict | No verdicts | `/gf-review` |
| Six-dimension PR assessment | Narrates only | `/gf-pr-review` |
| Detecting code smells | Different problem | `/gf-smell` |

## Evidence Grading

| Tier | Meaning | Hard rule |
|---|---|---|
| `Measured` | Command run this session | No output → downgrade to `Inferred`. |
| `Inferred` | Read from code/config/diff | Must cite `path:line`. |
| `Unverified` | Not verified this run | Must state why; never omitted. |

Reused from `gf-smell`. Failed re-run → `Unverified`, never `Inferred`.

## Failing Tests

| 失败用例 | 最后修改 commit | 是否 base 祖先 |
|---|---|---|

Missing commit hash together with `unrelated` → forbidden. Wording fixed:
"先于本次交付存在" (ancestry ⇒ file predates base, not ⇒ failure unrelated).
Algorithm: `docs/superpowers/templates/walkthrough-report-template.md`.

## Language Detection

Reuse `gf-quality/references/detector.md`; no language layer here.

## Re-run Allowlist

One read-only re-run fills gaps, else `Unverified`:

| Allowed | Forbidden |
|---|---|
| `git log` / `diff` / `merge-base` / `show` | anything writing files |
| `gf pr view` / `gf pr diff` / `gf pipeline status` | pushing or changing branch state |
| read-only Gate Commands | installing dependencies |

Failed re-run → `Unverified` with reason and workaround.

## Responsibility

### ✅ In Scope

- Narrate the change for a non-engineer reader
- Summarize the diff and PR metadata when present
- Grade every claim `Measured` / `Inferred` / `Unverified`
- Assemble the failing-test ancestry table
- Write the report from the template

### ❌ Out of Scope

- Review verdict — `/gf-review`
- Six-dimension assessment — `/gf-pr-review`
- Smell detection — `/gf-smell`
- Orchestrator wiring

### 🚫 Do Not

- ❌ Write outside the report file
- ❌ Label a claim `Inferred` after its command failed
- ❌ Write `unrelated` without a commit hash and ancestry check
- ❌ Issue a verdict

## Rationalization Excuses

| Excuse | Reality |
|---|---|
| "Looks unrelated, I'll say so" | Forbidden without a hash; run ancestry check. |
| "Usually passes, mark it Measured" | No output this session → `Inferred` at best. |
| "Basically a review, add a verdict" | Verdicts belong to `/gf-review`. |

## Red Flags

- 🚩 "Approve it too" — Refuse. Verdicts are `/gf-review`.
- 🚩 "Skip ancestry, call it unrelated" — Refuse. See Failing Tests.
- 🚩 "Re-run until it passes" — Refuse. One attempt; else `Unverified`.
- 🚩 "Score six dimensions too" — Refuse. `/gf-pr-review`.

## Test Scenarios

### 1: Happy Path
Diff + merged PR → produce walkthrough → narrative, summary, graded
evidence from the template.

### 2: Negative
"Approve this PR" → not used → routed to `/gf-review`.

### 3: Boundary
Six-dimension assessment also requested → walkthrough written, assessment
refused → `/gf-pr-review`.

### 4: Error
Allowlist command fails → `Unverified` with reason, no retry.

## Success Criteria

- [ ] Every claim carries `Measured`, `Inferred`, or `Unverified`
- [ ] Every `Measured` claim has an output block
- [ ] Failing-test table never uses `unrelated` without a commit hash
- [ ] No verdict is issued
- [ ] Language detection delegates to `gf-quality/references/detector.md`
- [ ] Report from template

## Trigger Keywords

| English | 中文 |
|---|---|
| delivery walkthrough | 交付走查 |
| evidence grading | 证据分级 |

## See Also

Runs at `gf-workflow` Phase 4, alongside `/gf-review`; wiring out of scope.

- `docs/superpowers/templates/walkthrough-report-template.md` — report skeleton & self-check
- `/gf-review` — approve / request-changes verdict
- `/gf-pr-review` — six-dimension PR assessment
- `/gf-smell` — code smell and complexity detection
