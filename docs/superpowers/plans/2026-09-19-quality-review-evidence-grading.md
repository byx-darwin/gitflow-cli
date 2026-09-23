# gf-quality / gf-pr-review Evidence Grading Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Backfill the `Measured`/`Inferred`/`Unverified` three-tier evidence vocabulary and failing-test commit-ancestry table (established by #329's `gf-walkthrough`) into the existing `gf-quality` and `gf-pr-review` skills, so their reports state how thoroughly each conclusion was verified.

**Architecture:** Pure documentation change — `skills/gf-quality/SKILL.md` and `skills/gf-pr-review/SKILL.md` are Markdown instruction files, not application code. No Rust source changes, no `cargo` tests. Verification is grep-based structural assertions in a new `make check-quality-review-evidence-skill` target (same pattern as `check-walkthrough-skill`), plus one real dogfooding run.

**Tech Stack:** Markdown, `make`, `bash`/`grep` for assertions, `git log`/`git merge-base` for the ancestry check (algorithm copied verbatim from `gf-walkthrough`).

**Spec:** `docs/superpowers/specs/2026-09-19-quality-review-evidence-grading-design.md`

## Global Constraints

- Modify ONLY `skills/gf-quality/SKILL.md` and `skills/gf-pr-review/SKILL.md`. Do NOT touch `skills/gf-review/SKILL.md` (out of scope per design doc — it submits verdicts only, no analysis).
- All three-tier vocabulary and the failing-test table columns MUST be copied **verbatim** from `skills/gf-walkthrough/SKILL.md:36-53` — no rewording (Issue AC5: "与 #329 的术语表一致，两者标注名称不同则为假").
- Never invent a fourth tier or rename `Measured`/`Inferred`/`Unverified`.
- `docs/references/pr-review-checklist.md` stays untouched — it is a static checklist of what to look for, not a verdict template; evidence tiers apply to `gf-pr-review`'s own Step 2/3 output, not this reference file.
- No `cargo clean`. No commit without running the project's pre-commit hooks (already wired into `git commit`, do not pass `--no-verify`).

---

### Task 1: RED — Add `check-quality-review-evidence-skill` Makefile target

**Files:**
- Modify: `Makefile` (new target near `check-walkthrough-skill` at line 421, and its `.PHONY` line at line ~629)

**Interfaces:**
- Consumes: nothing (reads `skills/gf-quality/SKILL.md` and `skills/gf-pr-review/SKILL.md` from disk)
- Produces: `make check-quality-review-evidence-skill` — a target later tasks run to verify GREEN. Exit 0 = pass, exit 1 = any assertion failed.

- [ ] **Step 1: Add the Makefile target**

Insert this new target directly after the `check-walkthrough-skill` target block (after its `exit $$FAIL` line, before the `check-decompose-skill:` line, i.e. right before line 421's neighboring block ends — insert as its own target, keeping one blank line before and after):

```makefile
check-quality-review-evidence-skill: ## Verify gf-quality/gf-pr-review evidence grading meets Issue #333 acceptance criteria
	@QS=skills/gf-quality/SKILL.md; PS=skills/gf-pr-review/SKILL.md; WS=skills/gf-walkthrough/SKILL.md; FAIL=0; \
	if [ ! -f "$$QS" ]; then echo "✗ missing $$QS"; exit 1; fi; \
	if [ ! -f "$$PS" ]; then echo "✗ missing $$PS"; exit 1; fi; \
	for F in "$$QS" "$$PS"; do \
		TIER=0; \
		for K in Measured Inferred Unverified; do \
			grep -qF "$$K" "$$F" || { echo "✗ #1 $$F 缺少证据档位 $$K"; TIER=1; FAIL=1; }; \
		done; \
		[ $$TIER -eq 0 ] && echo "✓ #1 $$F 三档标记齐备"; \
	done; \
	HEADER=0; \
	for H in '失败用例' '最后修改 commit' '是否 base 祖先'; do \
		grep -qF "$$H" "$$QS" || { echo "✗ #3 gf-quality 缺少失败测试列「$$H」"; HEADER=1; FAIL=1; }; \
	done; \
	[ $$HEADER -eq 0 ] && echo "✓ #3 gf-quality 失败测试三列齐备"; \
	grep -qF 'unrelated' "$$QS" \
		&& echo "✓ #4 gf-quality 禁用词规则已声明" \
		|| { echo "✗ #4 gf-quality 未声明禁止写 unrelated"; FAIL=1; }; \
	for F in "$$QS" "$$PS"; do \
		grep -qF 'gf-walkthrough' "$$F" \
			&& echo "✓ #5 $$F 声明复用 gf-walkthrough 词汇" \
			|| { echo "✗ #5 $$F 未声明复用词汇（术语可能与 #329 不一致）"; FAIL=1; }; \
	done; \
	for T in Measured Inferred Unverified; do \
		QLINE=`grep -m1 "| \\\`$$T\\\`" "$$QS" | tr -d '[:space:]'`; \
		WLINE=`grep -m1 "| \\\`$$T\\\`" "$$WS" | tr -d '[:space:]'`; \
		if [ "$$QLINE" != "$$WLINE" ]; then echo "✗ #5 gf-quality 的 $$T 行与 gf-walkthrough 不逐字一致"; FAIL=1; fi; \
	done; \
	grep -qE 'Evidence Tier|证据等级' "$$PS" \
		&& echo "✓ #1 gf-pr-review 逐维度声明证据等级" \
		|| { echo "✗ #1 gf-pr-review 未在维度评估中声明证据等级"; FAIL=1; }; \
	[ $$FAIL -eq 0 ] && echo "全部硬约束通过" || echo "存在未通过项"; \
	exit $$FAIL

```

Also add `check-quality-review-evidence-skill` to the `.PHONY` line (currently reads `... check-walkthrough-skill check-decompose-skill check-skills-drift ...` around line 629) — insert it right after `check-walkthrough-skill`:

```
check-agent-sync check-smell-skill check-refactor-skill check-architecture-diagram-skill check-walkthrough-skill check-quality-review-evidence-skill check-decompose-skill check-skills-drift release release-quick release-rehearse \
```

- [ ] **Step 2: Run it and confirm it fails (RED)**

Run: `make check-quality-review-evidence-skill`
Expected: FAIL (exit 1) — at minimum `#1` fails for both files (no `Measured`/`Inferred`/`Unverified` yet in either `SKILL.md`), and `#3`/`#4` fail for `gf-quality` (no failing-test ancestry table yet).

- [ ] **Step 3: Commit**

```bash
git add Makefile
git commit -m "test(quality-review): add check-quality-review-evidence-skill assertion (RED)"
```

---

### Task 2: GREEN — Add evidence grading to `gf-quality`

**Files:**
- Modify: `skills/gf-quality/SKILL.md`

**Interfaces:**
- Consumes: nothing new
- Produces: the `Evidence Grading` section other skills/readers can point to; the Step 2 gate table gains an `Evidence Tier` column; the Step 3 report templates gain an `Evidence` column plus a failing-test ancestry table

- [ ] **Step 1: Insert the "Evidence Grading" section**

Insert immediately after the `## When NOT to Use` table (after its last row, before the `## Quality Pipeline` heading — i.e. between current lines 36 and 38):

```markdown

## Evidence Grading

| Tier | Meaning | Hard rule |
|---|---|---|
| `Measured` | Command run this session | No output → downgrade to `Inferred`. |
| `Inferred` | Read from code/config/diff | Must cite `path:line`. |
| `Unverified` | Not verified this run | Must state why; never omitted. |

Reused verbatim from `gf-walkthrough`/`gf-smell`. Every gate result in Step 3's
report carries one of these three tiers. Never label a gate `Measured`
without that gate's command output present in the report this session.
```

- [ ] **Step 2: Add the Evidence Tier column to the Step 2 gate table**

Replace the existing table (current lines 97-104):

```markdown
| # | Gate | What It Checks |
|---|------|---------------|
| 1 | **build** | Code compiles (exit 0) |
| 2 | **test** | All tests pass |
| 3 | **coverage** | Total coverage ≥ 80% (`COV_THRESHOLD` overrides); line coverage for Rust, Python, Java, Ruby and Node.js, **statement** coverage for Go — the units differ, so always report which; N/A when the change touches no source file of that language |
| 4 | **format** | No formatting diff |
| 5 | **static** | No lint/analysis warnings |
| 6 | **pre-commit** | All hooks pass (N/A if no `.pre-commit-config.yaml`) |
```

with:

```markdown
| # | Gate | What It Checks | Evidence Tier |
|---|------|---------------|----------------|
| 1 | **build** | Code compiles (exit 0) | `Measured` when the command ran this session; `SKIPPED`/`N/A` → `Unverified` with reason |
| 2 | **test** | All tests pass | `Measured` when the command ran this session; `SKIPPED`/`N/A` → `Unverified` with reason |
| 3 | **coverage** | Total coverage ≥ 80% (`COV_THRESHOLD` overrides); line coverage for Rust, Python, Java, Ruby and Node.js, **statement** coverage for Go — the units differ, so always report which; N/A when the change touches no source file of that language | Coverage value `Measured` from tool output; the threshold itself `Inferred` — cite `COV_THRESHOLD` or this row's 80% default |
| 4 | **format** | No formatting diff | `Measured` when the command ran this session; `SKIPPED` → `Unverified` with reason |
| 5 | **static** | No lint/analysis warnings | `Measured` when the command ran this session; `SKIPPED` → `Unverified` with reason |
| 6 | **pre-commit** | All hooks pass (N/A if no `.pre-commit-config.yaml`) | `Measured` when the command ran this session; `N/A` → `Unverified` (no config present) |
```

- [ ] **Step 3: Add Evidence column + failing-test ancestry table to the Single-Language Report template**

Replace the current Single-Language Report block (current lines 151-171):

```markdown
```markdown
## Quality Gate Report

- Date: <date>
- Language: <detected language>
- Project: <repo name>

| Gate | Status | Details |
|------|--------|---------|
| 1. build | ✅/❌/N/A | <errors if any> |
| 2. test | ✅/❌/N/A | <failed tests if any> |
| 3. coverage | ✅/❌/N/A | <value vs threshold> |
| 4. format | ✅/❌/N/A | <files if diff> |
| 5. static | ✅/❌/N/A | <warnings if any> |
| 6. pre-commit | ✅/❌/N/A | <hook failures if any> |

### Result
- [ ] ALL CHECKS PASSED — ready for PR
- [ ] WARNINGS — recommend fixing before PR
- [ ] ERRORS — must fix before PR
```
```

with:

```markdown
```markdown
## Quality Gate Report

- Date: <date>
- Language: <detected language>
- Project: <repo name>

| Gate | Status | Evidence | Details |
|------|--------|----------|---------|
| 1. build | ✅/❌/N/A | Measured/Unverified | <errors if any> |
| 2. test | ✅/❌/N/A | Measured/Unverified | <failed tests if any> |
| 3. coverage | ✅/❌/N/A | Measured/Inferred/Unverified | <value vs threshold> |
| 4. format | ✅/❌/N/A | Measured/Unverified | <files if diff> |
| 5. static | ✅/❌/N/A | Measured/Unverified | <warnings if any> |
| 6. pre-commit | ✅/❌/N/A | Measured/Unverified | <hook failures if any> |

### Failing Tests (Gate 2 only)

| 失败用例 | 最后修改 commit | 是否 base 祖先 |
|---|---|---|

Never write "unrelated" / "与本次改动无关" without the commit hash and
ancestry check below (reused from `gf-walkthrough`):

\`\`\`bash
H=$(git log -1 --format=%H -- "<test file>")
git merge-base --is-ancestor "$H" "$BASE" && echo "先于本次交付存在" || echo "本次引入"
\`\`\`

### Result
- [ ] ALL CHECKS PASSED — ready for PR
- [ ] WARNINGS — recommend fixing before PR
- [ ] ERRORS — must fix before PR
```
```

- [ ] **Step 4: Add Evidence column + failing-test ancestry table to the Multi-Language Aggregate Report template**

In the `### Gate Results` table (current lines 190-195), add an `Evidence` column after `Result` is too disruptive to the wide table — instead add a per-language `Evidence` note row under each language's `#### N. <Language>` gate table in `### Per-Language Details` (current lines 197-213). Replace:

```markdown
#### 2. Node.js (apps/desktop/, bun)

| Gate | Status | Details |
|------|--------|---------|
| test | ❌ | 2 tests failed |

**Failed tests:**
- `test_add`: Expected 5, got 4
```

with:

```markdown
#### 2. Node.js (apps/desktop/, bun)

| Gate | Status | Evidence | Details |
|------|--------|----------|---------|
| test | ❌ | Measured | 2 tests failed |

**Failed tests:**

| 失败用例 | 最后修改 commit | 是否 base 祖先 |
|---|---|---|
| `test_add` | `<hash>` | `先于本次交付存在` / `本次引入` |

Never write "unrelated" without the commit hash and ancestry check (see
Single-Language Report → Failing Tests above for the command).
```

And add `| Evidence |` as a column analogous to the Rust example row right above it (current line 199-204's `#### 1. Rust` table), so both example blocks stay structurally consistent:

```markdown
#### 1. Rust (./, workspace)

| Gate | Status | Evidence | Details |
|------|--------|----------|---------|
| build | ✅ | Measured | 3 crates compiled |
| test | ✅ | Measured | 47 tests passed |
```

- [ ] **Step 5: Add Rationalization Table entries**

In the `## Rationalization Table` (current lines 231-241), add two rows before the closing of the table:

```markdown
| "看似与改动无关，写 unrelated" | 禁止；必须先跑祖先检查并附 commit hash |
| "SKIPPED 也算 Measured，反正跑过检测" | `SKIPPED` = 工具缺失、未产出真实输出，只能标 `Unverified` |
```

- [ ] **Step 6: Add Red Flags entries**

In `## Red Flags — STOP` (current lines 242-248), add:

```markdown
- 🚩 "报告写 unrelated 却没给 commit" — 禁止，先跑祖先检查
- 🚩 "把 N/A/SKIPPED 标成 Measured" — 无输出即非 Measured
```

- [ ] **Step 7: Add Common Mistakes entries**

In `## Common Mistakes` (current lines 250-256), add:

```markdown
- ❌ **失败测试写「与改动无关」但无 commit 溯源** — 必须先跑祖先检查
- ❌ **把跳过的 Gate 标成 Measured** — 无输出即为 `Unverified`
```

- [ ] **Step 8: Commit**

```bash
git add skills/gf-quality/SKILL.md
git commit -m "feat(gf-quality): add three-tier evidence grading and failing-test ancestry table"
```

---

### Task 3: GREEN — Add evidence grading to `gf-pr-review`

**Files:**
- Modify: `skills/gf-pr-review/SKILL.md`

**Interfaces:**
- Consumes: nothing new
- Produces: the `Evidence Grading` section; Step 2/3 now require a tier per dimension verdict

- [ ] **Step 1: Insert the "Evidence Grading" section**

Insert immediately after the `## When NOT to Use` table (after its last row, before the `## Core Pattern` heading — i.e. between current lines 46 and 48):

```markdown

## Evidence Grading

| Tier | Meaning | Hard rule |
|---|---|---|
| `Measured` | Command run this session | No output → downgrade to `Inferred`. |
| `Inferred` | Read from code/config/diff | Must cite `path:line`. |
| `Unverified` | Not verified this run | Must state why; never omitted. |

Reused verbatim from `gf-walkthrough`/`gf-smell`/`gf-quality`. Every
dimension verdict in Step 2/3 carries one of these three tiers.
```

- [ ] **Step 2: Update Step 2 (Assess 6 Dimensions)**

Replace (current lines 77-79):

```markdown
### Step 2: Assess 6 Dimensions

For each dimension (correctness, security, performance, maintainability, test-coverage, docs): ✅ or ⚠️ with `path:line`. See [checklist](../../docs/references/pr-review-checklist.md).
```

with:

```markdown
### Step 2: Assess 6 Dimensions

For each dimension (correctness, security, performance, maintainability, test-coverage, docs): ✅ or ⚠️ with `path:line`, plus an Evidence Tier — `Inferred` for a diff-reading judgment (the default; cites `path:line`), `Measured` when backed by a command actually run this session (command + output required), `Unverified` for an unresolved suspicion (state why, never omit). See [checklist](../../docs/references/pr-review-checklist.md).
```

- [ ] **Step 3: Update Step 3 (Draft Conclusion)**

Replace (current lines 81-83):

```markdown
### Step 3: Draft Conclusion

Per-dimension verdicts with `path:line` for ⚠️ items. See [template](../../docs/references/pr-review-checklist.md).
```

with:

```markdown
### Step 3: Draft Conclusion

Per-dimension verdicts with `path:line` for ⚠️ items, carrying the same Evidence Tier assigned in Step 2 — never upgrade a tier between Step 2 and Step 3. See [checklist](../../docs/references/pr-review-checklist.md).
```

- [ ] **Step 4: Add a Rationalization Excuses entry**

In `## Rationalization Excuses` (current lines 132-137), add:

```markdown
| "读了 diff 就是 Measured" | 读 diff 得出的判断是 `Inferred`；`Measured` 需要实际跑过验证命令并有输出 |
```

- [ ] **Step 5: Add a Red Flags entry**

In `## Red Flags` (current lines 139-143), add:

```markdown
- 🚩 "结论没写证据等级" — Refuse. 每条 ✅/⚠️ 判断都要标 `Measured`/`Inferred`/`Unverified`
```

- [ ] **Step 6: Add a Common Mistakes entry**

In `## Common Mistakes` (current lines 178-181), add:

```markdown
- ❌ **把 diff 阅读判断标成 Measured** — 未实际执行验证命令的判断只能是 `Inferred`
```

- [ ] **Step 7: Commit**

```bash
git add skills/gf-pr-review/SKILL.md
git commit -m "feat(gf-pr-review): tag per-dimension verdicts with evidence grading"
```

---

### Task 4: GREEN verification + real dogfooding + index update

**Files:**
- Modify: `specs/index.md` (add plan entry under `## 实现计划`)
- No new files — dogfooding output stays in chat/PR body, matching `gf-quality`'s existing "report to terminal, no fixed archive file" convention (only `gf-review`'s code-review-report-*.md has a docs/ archiving policy, which is unaffected)

**Interfaces:**
- Consumes: the modified `skills/gf-quality/SKILL.md` and `skills/gf-pr-review/SKILL.md` from Tasks 2-3
- Produces: a passing `make check-quality-review-evidence-skill`, one real `gf-quality` run showing genuine `Measured` tags with output, `specs/index.md` updated

- [ ] **Step 1: Run the assertion target and confirm GREEN**

Run: `make check-quality-review-evidence-skill`
Expected: PASS (exit 0), every line prefixed `✓`, ending "全部硬约束通过".

- [ ] **Step 2: Dogfood `gf-quality` for real on this repo**

Follow the now-updated `skills/gf-quality/SKILL.md` verbatim: detect language (this repo is a Cargo workspace → `references/rust.md`), run the 6 gates for real (`cargo build`, `cargo test`, coverage tool if configured, `cargo fmt --check`, `cargo clippy`, pre-commit hooks), and produce one real `Quality Gate Report` with the new `Evidence` column filled from genuine command output — not fabricated. Paste this report into the task completion notes / eventual PR body as the dogfooding evidence for AC1/AC2.

If any gate is `SKIPPED`/`N/A` in this real run, that is expected and correct — it demonstrates the `Unverified` tier working as designed, not a plan failure.

- [ ] **Step 3: Note gf-pr-review dogfooding path (no separate task needed)**

`gf-pr-review` requires a real open PR to dogfood meaningfully. This workflow's own Phase 4 (`gf-workflow` full mode → parallel `Review` step) will invoke `gf-pr-review` against this Issue's own delivery PR, exercising the new Evidence Tier template on a real diff. Record in the plan's completion notes that this is the intended dogfooding point — no contrived PR needed here.

- [ ] **Step 4: Add plan entry to `specs/index.md`**

Under `## 实现计划` (append after the `gf-refactor 实施计划` line), add:

```markdown
- [gf-quality/gf-pr-review 证据分级实施计划](../docs/superpowers/plans/2026-09-19-quality-review-evidence-grading.md) — Issue #333，4 个 Task：验收断言（RED）→ gf-quality SKILL.md（逐 Gate 证据等级 + 失败测试溯源表）→ gf-pr-review SKILL.md（逐维度证据等级）→ 校验 GREEN + gf-quality 实跑 dogfooding。
```

- [ ] **Step 5: Commit**

```bash
git add specs/index.md
git commit -m "docs(specs): index the quality-review evidence grading plan"
```

---

## Completion Checklist

- [ ] `make check-quality-review-evidence-skill` passes
- [ ] `skills/gf-quality/SKILL.md`: Evidence Grading section + per-gate tier column + failing-test ancestry table, vocabulary verbatim-identical to `gf-walkthrough`
- [ ] `skills/gf-pr-review/SKILL.md`: Evidence Grading section + per-dimension tier requirement, vocabulary verbatim-identical to `gf-walkthrough`
- [ ] `skills/gf-review/SKILL.md` untouched
- [ ] Real `gf-quality` dogfooding report pasted into completion notes, evidence column populated from genuine output
- [ ] `specs/index.md` updated
