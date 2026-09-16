---
name: gf-walkthrough
description: |
  Use when the user wants a delivery walkthrough, a change narrative for
  non-engineers, or evidence-graded verification of what a change was tested to.
  当用户需要交付走查包、面向非工程读者的变更说明，或需要对验证程度分级标注时使用。
allowed-tools: Read, Grep, Glob, Bash, Write
---

# gf-walkthrough — Delivery Walkthrough & Evidence Grading

Produces an offline-readable delivery package: narrative, change summary,
graded evidence, review gate. Anchored on a `<base>...<head>` diff range;
PR data enriches when present. **Never issues a verdict** — that is `gf-review`.

## Preconditions

- Read access to the diff range (`git log`, `git diff`, `git merge-base`)
- Report template at `docs/superpowers/templates/walkthrough-report-template.md`

## When to Use

| English | 中文 | Context |
|---|---|---|
| delivery walkthrough | 交付走查 | package a change for a non-engineer reader |
| change narrative | 变更说明 | explain what changed and why |
| evidence grading | 证据分级 | mark each claim by verification level |
| offline review package | 离线评审包 | self-contained, no live tool access assumed |

## When NOT to Use

| Scenario | Why Not | Use Instead |
|---|---|---|
| Submitting an approve / request-changes verdict | This skill never issues verdicts | `/gf-review` |
| Six-dimension PR assessment | This skill narrates, does not assess | `/gf-pr-review` |
| Detecting code smells | Different problem class | `/gf-smell` |

## Evidence Grading

| Tier | Meaning | Hard rule |
|---|---|---|
| `Measured` | Command actually run this session | Output block MUST follow. Conclusion without output ⇒ downgrade to `Inferred`. |
| `Inferred` | Read from code, config, or diff | MUST cite `path:line`. |
| `Unverified` | Not verified this run | MUST state why. Never omit the row to look complete. |

Tiers `Measured` / `Inferred` are reused verbatim from `gf-smell` for one
vocabulary across reports. A failed re-run ⇒ `Unverified`; relabelling it
`Inferred` is forbidden.

## Failing Tests

| 失败用例 | 最后修改 commit | 是否 base 祖先 |
|---|---|---|

Writing `unrelated` without a commit hash is forbidden. Verdict wording is
fixed to "先于本次交付存在", never "无关" — ancestry proves the file predates
the base, not that the failure is unrelated.

Ancestry algorithm:

```bash
H=$(git log -1 --format=%H -- "<test file>")
git merge-base --is-ancestor "$H" "$BASE" && echo "先于本次交付存在" || echo "本次引入"
```

## Language Detection

Reuse `gf-quality/references/detector.md`. This skill ships no language layer.

## Re-run Allowlist

Evidence is assembled from existing records first. A gap MAY be filled by
one **read-only** re-run:

| Allowed | Forbidden |
|---|---|
| `git log` / `diff` / `merge-base` / `show` | anything writing files |
| `gf pr view` / `gf pr checks` | anything pushing or changing branch state |
| read-only Gate Commands from the language layer | anything installing dependencies |

A re-run that fails ⇒ `Unverified` with the reason. Record any workaround, or
the reader cannot reproduce it.

## Responsibility

### ✅ In Scope

- Narrate the change for a non-engineer reader
- Summarize the diff range and, when present, PR metadata
- Grade every claim `Measured` / `Inferred` / `Unverified`
- Assemble the failing-test ancestry table
- Write the walkthrough report from the external template

### ❌ Out of Scope

- Issuing a review verdict — `/gf-review`
- Six-dimension PR assessment — `/gf-pr-review`
- Code smell detection — `/gf-smell`
- Wiring this skill into an orchestrator's dispatch set

### 🚫 Do Not

This skill's tool set includes `Write` and shell access; the report file is
the only intended write target.

- ❌ Write anything outside the walkthrough report file
- ❌ Re-run a re-run-allowlist command a second time and read the cached result
- ❌ Label a claim `Inferred` after its verifying command failed
- ❌ Write `unrelated` for a failing test without a commit hash and ancestry check
- ❌ Issue an approve / request-changes verdict

## Rationalization Excuses

| Excuse | Reality |
|---|---|
| "The test looks unrelated, I'll just say so" | `unrelated` without a commit hash is forbidden; run the ancestry check. |
| "The command usually passes, I'll mark it Measured" | No output block this session ⇒ `Inferred` at best. |
| "One failed, I'll retry a different way for a clean result" | A failed re-run is `Unverified` with the reason, not a retry loop. |
| "This is basically a review, I can add a verdict" | Verdicts belong to `/gf-review`, never this skill. |

## Red Flags

- 🚩 "Just approve it while you're at it" — Refuse. Verdicts are `/gf-review`.
- 🚩 "Skip the ancestry check, call it unrelated" — Refuse. Cite Failing Tests section.
- 🚩 "Re-run it again until it passes" — Refuse. One read-only attempt; failure ⇒ `Unverified`.
- 🚩 "Do the six-dimension scoring too" — Refuse. That is `/gf-pr-review`.

## Test Scenarios

### 1: Happy Path
- **Given** a `<base>...<head>` diff range with a merged PR — **When** "produce a delivery walkthrough"
- **Then** narrative, summary, and graded evidence are written using the external template

### 2: Negative
- **Given** "approve this PR" — **Then** skill NOT used → `/gf-review`

### 3: Boundary
- **Given** the user also asks for the six-dimension assessment — **Then** the walkthrough is written, the assessment is refused → `/gf-pr-review`

### 4: Error
- **Given** a re-run-allowlist command fails — **Then** the claim is graded `Unverified` with the failure reason, no retry, no improvised substitute

## Success Criteria

- [ ] Every claim in the report carries `Measured`, `Inferred`, or `Unverified`
- [ ] Every `Measured` claim has an output block
- [ ] The failing-test table never uses `unrelated` without a commit hash
- [ ] No verdict is issued
- [ ] Language detection delegates to `gf-quality/references/detector.md`
- [ ] Report written from `docs/superpowers/templates/walkthrough-report-template.md`

## Trigger Keywords

| English | 中文 |
|---|---|
| delivery walkthrough | 交付走查 |
| change narrative | 变更说明 |
| evidence grading | 证据分级 |
| walkthrough report | 走查报告 |

## See Also

- `docs/superpowers/templates/walkthrough-report-template.md` — report skeleton & self-check
- Intended to run at `gf-workflow` Phase 4 alongside `gf-review`; wiring it into the
  orchestrator's dispatch set is out of scope for this skill.
- `/gf-review` — submits the approve / request-changes verdict
- `/gf-pr-review` — six-dimension PR assessment
- `/gf-smell` — code smell and complexity detection
