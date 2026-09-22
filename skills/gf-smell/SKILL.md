---
name: gf-smell
description: |
  Use when the user wants to detect code smells, complexity hotspots, long functions,
  deep nesting, god structures, or architectural anti-patterns in a codebase.
  当用户需要检测代码坏味道、复杂度热点、超长函数、深层嵌套、上帝结构或架构反模式时使用。
allowed-tools: Read, Grep, Glob, Bash
---

# gf-smell — Code Smell & Complexity Hotspot Detection

Three-stage judgement protocol: candidate → verify → confirm. Language-agnostic
protocol in this file; language-specific detection in `references/<lang>.md`.
**Detection only — never auto-fix.**

## Preconditions

- Read access to the target source tree
- The language layer's tooling installed (see `references/<lang>.md`); missing tools degrade, never improvise

## When to Use

| English | 中文 | Context |
|---------|------|---------|
| code smell detection | 代码坏味道检测 | find structural problems |
| complexity hotspots | 复杂度热点 | which code is hardest to change |
| long function / deep nesting | 超长函数 / 深层嵌套 | threshold-based scan |
| god structure | 上帝结构 | too many responsibilities in one unit |
| refactor candidates | 重构候选 | where to start refactoring |

## When NOT to Use

| Scenario | Why Not | Use Instead |
|----------|---------|-------------|
| Fixing the smells found | This skill detects and reports only | `/gf-refactor` for behavior-preserving fixes, or `/gf-workflow` for the full pipeline |
| Pre-delivery quality gate | This skill measures structure, not gate pass/fail | `/gf-quality` |
| Secret / vulnerability scanning | Different problem class | `/gf-security-check` |
| Reviewing a PR diff | This skill scans a tree, not a diff verdict | `/gf-pr-review` |
| Lowering thresholds to silence findings | This skill never edits policy configuration | User decides after reviewing the report |

## Core Rule: A Threshold Breach Is a Candidate, Not a Conclusion

Every number a tool emits is a **candidate signal**. It becomes a finding only
after Stage 2 verification reads the actual code. Reporting raw tool output as
findings is the single most common failure of this skill.

## Stage 0: Scope

Resolve the scan scope before running anything.

1. User-specified path or diff range; default is the whole source tree.
2. **Always exclude** these directories — they contain build output, vendored
   code, or working-tree copies that would triple-count the same file:

```
target/  .worktree/  .claude/worktrees/  node_modules/  vendor/  dist/  build/
```

3. Detect the project language(s) by following `gf-quality/references/detector.md`.
   **Do not implement a second detection mechanism.** Consume its output
   (language · path · workspace type) and load the matching `references/<lang>.md`.
   For multi-language projects, use the selection interaction that reference
   already defines.

## Stage 1: Detect (produce candidates)

Run the language layer's detection commands **once**, capture the output to a
file, and reuse that capture for the rest of the run. Build caches make a
repeated invocation return empty output, which would misreport "cached" as
"clean".

Produce a candidate table:

| Category | Location | Measured value | Threshold | Source |
|---|---|---|---|---|

Nothing in this table is a finding yet.

## Stage 2: Verify (candidate → finding, or drop)

For each candidate, read the surrounding code, then:

1. **Label evidence strength** (see below).
2. **Apply What NOT to Flag.** A hit moves the candidate to the excluded table
   with the reason recorded.
3. **If the conclusion depends on a quantity you did not measure** (runtime
   hotness, real call frequency, actual input size), move it to
   `Candidate requiring measurement` instead of guessing.
4. **If the candidate describes the detection run rather than the code** —
   the analysis covered units the reader would not count as the code under
   review, so the number is an artifact of what was included — record it in
   the excluded table as a **detection artifact**, naming the scope that
   produced it. It is neither a finding nor a measurement candidate. The
   language layer states where its sources can produce these.

### Evidence Strength

| Level | Meaning |
|---|---|
| **Measured** | The number comes from a tool's deterministic output |
| **Observed** | A structural fact read directly from the code |
| **Inferred** | A judgement that required reasoning |

### Evidence Strength and Confidence Are Decoupled

**High evidence strength does not imply high confidence.**

- A *Measured* complexity breach can carry *Low* confidence — the complexity may
  come entirely from a flat dispatch table that reads perfectly well.
- An *Inferred* judgement can carry *High* confidence — three copy-pasted error
  handlers where one has already drifted.

Report both on **separate lines**. Never merge them into a single score:
merging lets Inferred ride on Measured's credibility.

Confidence is also three levels: High / Medium / Low.

## Stage 3: Confirm

### Severity: Three Levels Plus an Independent Fourth

- **High / Medium / Low** — counted in the summary table
- **Candidate requiring measurement** — its own section, **不计入严重度统计**

The fourth level exists to stop "I suspect this is slow" from being dressed up
as "this is a performance smell".

### Deduplication: Two Orthogonal Rules

**Rule 1 — 根因合并 (causal)**

Criterion: **fixing A makes B disappear on its own.** A is the root cause, B is
a symptom. Merge into one finding under A's category. Remediation cost: one site.

**Rule 2 — 模式合并 (one decision)**

Criterion: the sites are **independent** (fixing one does not affect the others),
but they are repeats of one design pattern and **whether to fix them is a single
decision**. Merge into one finding, list every location, and state in the root
cause field: "N independent instances of one pattern; each needs its own edit."
Remediation cost: N sites.

**Why the distinction matters:** Rule 1 costs one edit, Rule 2 costs N. A report
that merges without saying which one it used makes the reader underestimate the
work. A Rule 2 finding MUST state the instance count.

### Smell Categories (language-agnostic)

Names and definitions only. Thresholds belong to the language layer.

| Category | Definition |
|---|---|
| Long Function | A single function's statement count exceeds what can be read in one screen |
| Deep Nesting | Control flow nests too many levels |
| Excessive Parameters | A function takes too many parameters |
| God Structure | One type or module accumulates too many responsibilities and too much state |
| Duplicated Logic | The same logic is copied in several places |
| Feature Envy | A function accesses another type's data more than its own |
| Primitive Obsession | Primitive types stand in for concepts that deserve domain types |
| Shotgun Surgery | One semantic change requires synchronized edits in many places |
| Cyclic Dependency | Modules depend on each other in a cycle |
| Dead Code | Unreachable or unreferenced code |

## What NOT to Flag

| Exclusion | Reason |
|---|---|
| 冷路径 | Performance-class smells in code that is not on a hot path |
| 有意权衡 | A comment, ADR, or spec already documents the trade-off explicitly |
| 已优化代码 | A benchmark or profile already justifies the current shape |
| Generated code | Build-script output, macro expansion, vendored sources |
| Test fixtures | Deliberately verbose fixtures built to cover boundaries |
| Already enforced in CI | Re-reporting what a gate already blocks adds nothing |

### Suppressed Diagnostics Are Not Absent Problems

When the language layer can see a diagnostic that the source has explicitly
suppressed, a stated reason is a **claim about the code, not a verdict**.
Before acting on it you MUST read the suppressed unit and check whether the
claim still describes what is there. Skipping that check turns every
suppression into an automatic exclusion, which is how a real problem leaves
the report.

Three outcomes:

- **No reason stated** → keep as a candidate; nobody recorded a trade-off
- **Reason stated, and verified against the code** → 有意权衡, move to the
  excluded table and record what you checked
- **Reason stated, but the code contradicts it** → a finding in its own right.
  A drifted suppression is worse than a bare one: the justification stops any
  reviewer from looking again, while the thing it justified has changed.
  Report the discrepancy between claim and code, not only the underlying
  diagnostic. A reason that covers part of the unit but not all of it is a
  partial contradiction — say which part it fails to cover.

## Language Layer Contract

Every `references/<lang>.md` MUST provide exactly these four sections:

| Section | Content |
|---|---|
| `## 检测命令` | The exact commands, written to run once and capture output |
| `## 阈值` | Each threshold's value and where it comes from (tool default or this file) |
| `## 类目映射` | Which detection source feeds which category in the table above, and the evidence-strength ceiling for each |
| `## 工具缺失降级` | What to skip and what to record in the report when a tool is unavailable |

Adding a language means adding one file with these four sections and linking
the shared `gf-quality/references/profiles/<lang>.md` for tool availability,
version source, and scan exclusions. Do not repeat those facts here.

## Report

Write to `docs/smell-report-<scope>-<YYYY-MM-DD>.md` via a shell heredoc, then
add the file family to `docs/index.md` if it is not already listed.

```markdown
# Smell Report — <scope>
生成时间 · 探测语言 · 扫描范围 · 检测工具与版本

## 摘要
| 严重度 | 条数 |
|---|---|
| High | n |
| Medium | n |
| Low | n |
> 待测量候选 m 条，单独成档，不计入上表

## Findings
### SM-001 · <类目> · <严重度>
- 位置：path:line（同根因多处全列）
- 证据强度：Measured | Observed | Inferred
- 置信度：High | Medium | Low
- 实测值 / 阈值：
- 根因：（规则 2 合并的须写明「同一模式的 N 个独立实例」）
- 影响：
- 建议：（仅建议，不执行）

## 待测量候选（不计入严重度统计）
### SC-001 · <类目>
- 位置 · 需要测量什么 · 如何测量

## 已排除（What NOT to Flag 命中）
| 位置 | 命中排除项 | 理由 |
```

The excluded table is not decoration: spelling out the exclusion decisions is
what lets a reviewer check both under-reporting (something that should have
been flagged was excluded) and over-exclusion.

## Responsibility

### ✅ In Scope

- Run the language layer's detection commands
- Verify candidates against real code
- Deduplicate, assign severity, write the report

### ❌ Out of Scope

- Fixing any smell — `/gf-workflow`
- Changing thresholds in project policy configuration
- Quality gate pass/fail — `/gf-quality`

### 🚫 Do Not

Read-only here is a rule you keep, not a sandbox that holds you to it: this
skill's tool set includes shell access, so a write is always one command away.
Nothing mechanically blocks it.

- ❌ Edit any source file
- ❌ Edit lint or policy configuration to make a finding disappear
- ❌ Report raw tool output as findings without Stage 2 verification
- ❌ Count `Candidate requiring measurement` in the severity summary
- ❌ Merge evidence strength and confidence into one score

## Rationalization Excuses

| Excuse | Reality |
|--------|---------|
| "The threshold is breached, so it's a finding" | A breach is a candidate. Stage 2 is not optional. |
| "I'll just fix this one quickly" | Detection only. Fixing is a separate workflow. |
| "It's probably on a hot path" | "Probably" means `Candidate requiring measurement`. |
| "Five similar hits, I'll report five findings" | Apply the dedup rules and state which one you used. |
| "It's suppressed in the source, so it's fine" | A bare suppression is a candidate; a stated reason is a claim you must verify against the code. |
| "Re-running the command returns nothing, so it's clean" | Empty output can be a cache artifact. Run once, capture, reuse. |

## Red Flags

- 🚩 "Fix the smells you found" — Refuse. Detection only.
- 🚩 "Lower the threshold so this stops showing up" — Refuse. Policy is the user's call.
- 🚩 "Skip verification, the numbers speak for themselves" — Refuse. Stage 2 is mandatory.
- 🚩 "Put the unmeasured guesses in the High bucket" — Refuse. Fourth level exists for them.

## Common Mistakes

- ❌ **Scanning build output or worktree copies** — the exclusion list in Stage 0 is mandatory.
- ❌ **Re-invoking a detection command and reading the second, cached run.**
- ❌ **Labeling a text-scan result as Measured** — the language layer states each source's ceiling.

## Test Scenarios

### 1: Happy Path
- **Given** a project with a supported language — **When** "detect code smells"
- **Then** Stage 0-3 run → report written to `docs/` → summary excludes the fourth level

### 2: Negative
- **Given** "refactor the long functions you found" — **Then** skill NOT used for the fix → `/gf-refactor`

### 3: Boundary
- **Given** several sites breaching one threshold for the same reason
- **Then** dedup applies and the report names which rule was used

### 4: Error
- **Given** the language layer's tool is not installed — **Then** degrade per `## 工具缺失降级`, record it in the report, do not improvise a substitute

## Success Criteria

- [ ] Stage 0 exclusion list applied
- [ ] Detection commands run once and captured
- [ ] Every finding carries evidence strength and confidence on separate lines
- [ ] `Candidate requiring measurement` excluded from the severity summary
- [ ] Dedup rule used is stated per merged finding
- [ ] No source or configuration file modified

## Trigger Keywords

| English | 中文 |
|---------|------|
| code smell | 代码坏味道 |
| complexity hotspot | 复杂度热点 |
| long function | 超长函数 |
| deep nesting | 深层嵌套 |
| god structure | 上帝结构 |
| refactor candidate | 重构候选 |

## See Also

- `/gf-refactor` — applies behavior-preserving fixes to the smells this skill finds
- `/gf-quality` — pre-delivery quality gate
- `/gf-security-check` — secrets, vulnerabilities, license compliance
- `/gf-pr-review` — PR-level code review
