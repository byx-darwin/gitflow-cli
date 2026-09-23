# 覆盖率度量口径统一 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 把 `gf-quality` 的覆盖率工具统一到 `cargo-llvm-cov`、Gate 3 口径由「增量」落实为总行覆盖 80%，并修复 `references/` 的三处功能性缺陷。

**Architecture:** 绝大部分改动是 skill 文档的文本同步；唯一的可执行产物是新增的 `scripts/validate-skill-links.sh`（走 TDD），它把「引用必须可解析」从人工审查变成 `make check-agent-sync` 的自动闸门。脚本先于文档修复落地，其对仓库现状的报错即是后续修复任务的 RED。

**Tech Stack:** Bash（macOS 自带 bash 3.2 兼容）、GNU/BSD 通用的 `grep -oE` / `find`、Make、Markdown。

**Spec:** `docs/superpowers/specs/2026-09-17-coverage-metric-unification-design.md`

**Issues:** #340（工具分裂）、#348（references 三处缺陷）、#354（Gate 3 口径）— milestone #2

## Global Constraints

- **Skill 源码只改 `skills/`**，绝不改 `.claude/skills/` 中的副本（CLAUDE.md 明令）。
- **绝不运行 `cargo clean`**（CLAUDE.md 明令）。
- **未经明确许可不得 commit / push / merge**。计划中的 commit 步骤须在获得许可后执行。
- **不改写史料**：`docs/reports-archive/`、`docs/superpowers/plans/`、`docs/superpowers/specs/2026-07-06-*`、`docs/research/`、`docs/issue-triage-report-2026-09-16.md` 中共 13 处 `tarpaulin` 保持原样。
- **覆盖率口径统一为「总行覆盖率」**，阈值 `COV_THRESHOLD` 默认 `80`，五个语言层用词一致，不得再出现 `incremental`。
- **`SKILL.md` 的「Report only. No auto-fix.」是最高约束**，任何 reference 不得出现与之冲突或反向授权的表述。
- **不新增 CI 覆盖率采集**（`.github/workflows/` 实测零命中，#340 末条按空条件成立判定）。
- 本仓库既有两处**与本次无关**的测试失败（`gitflow-gitlab` 的 `temp_env` 并行竞态、`e2e-gitcode` 的 `gc` 撞名）。**不得顺手修复**，如实报告即可。

---

### Task 1: `validate-skill-links.sh` —— skill 文档引用校验脚本

唯一的可执行产物，走完整 TDD。本任务**不接线到 Makefile**（留到 Task 7），以保证每次提交时 `make check-agent-sync` 都是绿的。

**Files:**
- Create: `scripts/validate-skill-links.sh`
- Test: `scripts/tests/validate-skill-links-test.sh`

**Interfaces:**
- Consumes: 无
- Produces: `scripts/validate-skill-links.sh [--root <dir>] [--verbose]`，退出码 `0`=全部可解析 / `1`=存在断链 / `2`=用法或环境错误。`--root` 供测试指向夹具树。

- [ ] **Step 1: 写失败的测试**

创建 `scripts/tests/validate-skill-links-test.sh`：

```bash
#!/usr/bin/env bash
# validate-skill-links-test.sh — fixture-based tests for validate-skill-links.sh
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TARGET="$SCRIPT_DIR/../validate-skill-links.sh"
PASS=0
FAIL=0

fixture_root=""
setup_fixture() {
    fixture_root="$(mktemp -d)"
}
teardown_fixture() {
    [ -n "$fixture_root" ] && rm -rf "$fixture_root"
    fixture_root=""
}

expect_exit() { # $1=expected code, $2=test name
    local expected="$1" name="$2" actual
    bash "$TARGET" --root "$fixture_root" >/dev/null 2>&1
    actual=$?
    if [ "$actual" -eq "$expected" ]; then
        echo "✅ $name"
        PASS=$((PASS + 1))
    else
        echo "❌ $name (expected exit $expected, got $actual)"
        FAIL=$((FAIL + 1))
    fi
}

# --- Case 1: all references resolve -> exit 0 ---
setup_fixture
mkdir -p "$fixture_root/gf-demo/references"
cat > "$fixture_root/gf-demo/SKILL.md" <<'EOF'
See [checklist](references/rust.md) for details.
Load `references/rust.md` when Cargo.toml is present.
EOF
touch "$fixture_root/gf-demo/references/rust.md"
expect_exit 0 "resolving link and load target pass"
teardown_fixture

# --- Case 2: broken backticked load target -> exit 1 ---
setup_fixture
mkdir -p "$fixture_root/gf-demo/references"
cat > "$fixture_root/gf-demo/references/detector.md" <<'EOF'
| `Gemfile` | Ruby | `references/ruby.md` |
EOF
expect_exit 1 "broken load target detected"
teardown_fixture

# --- Case 3: broken markdown link -> exit 1 ---
setup_fixture
mkdir -p "$fixture_root/gf-demo"
cat > "$fixture_root/gf-demo/SKILL.md" <<'EOF'
See [taxonomy](../references/missing.md).
EOF
expect_exit 1 "broken markdown link detected"
teardown_fixture

# --- Case 4: non-reference paths are skipped -> exit 0 ---
setup_fixture
mkdir -p "$fixture_root/gf-demo"
cat > "$fixture_root/gf-demo/SKILL.md" <<'EOF'
Visit [site](https://example.com/x.md) or [anchor](#section).
Write output to `/tmp/report.md` — absolute, not a reference.
Template path `docs/report-<YYYY-MM-DD>.md` uses a placeholder.
Glob `code-review-report-*.md` is not a path.
Run `rm -f /tmp/report.md` to clean up.
EOF
expect_exit 0 "absolute/placeholder/glob/command paths skipped"
teardown_fixture

# --- Case 5: load target resolves against skill root, not container dir -> exit 0 ---
setup_fixture
mkdir -p "$fixture_root/gf-demo/references"
cat > "$fixture_root/gf-demo/references/detector.md" <<'EOF'
| `Cargo.toml` | Rust | `references/rust.md` |
EOF
touch "$fixture_root/gf-demo/references/rust.md"
expect_exit 0 "load target resolves against skill root"
teardown_fixture

echo ""
echo "=== Summary: $PASS passed, $FAIL failed ==="
[ "$FAIL" -eq 0 ]
```

- [ ] **Step 2: 运行测试确认它失败**

Run: `bash scripts/tests/validate-skill-links-test.sh`
Expected: FAIL —— 5 个用例全部报 `expected exit N, got 127`（`validate-skill-links.sh` 尚不存在，`bash` 对缺失文件返回 127）。

- [ ] **Step 3: 写最小实现**

创建 `scripts/validate-skill-links.sh`：

```bash
#!/usr/bin/env bash
# validate-skill-links.sh — Validate that relative references in skill docs resolve.
#
# Usage: ./scripts/validate-skill-links.sh [--root <dir>] [--verbose]
# Exit codes: 0 = all references resolve, 1 = broken references found, 2 = usage/env error
#
# Scope — only navigational references the skill runtime actually follows:
#   1. Markdown link syntax `[text](path)` with a relative target
#   2. Backticked `references/*.md` progressive-disclosure load targets
# Paths merely *mentioned* (a skill's own output path, a conditional
# prerequisite) are not references and are deliberately out of scope.
#
# Resolution roots, first hit wins:
#   1. the directory containing the referring file
#   2. the skill root (`<root>/<skill-name>/`)
# Root 2 is required because `references/detector.md` refers to sibling
# references as `references/<lang>.md`, i.e. relative to the skill root.

set -euo pipefail

ROOT=""
VERBOSE=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --root)
            [[ $# -ge 2 ]] || { echo "ERROR: --root needs a value" >&2; exit 2; }
            ROOT="$2"; shift 2 ;;
        --verbose) VERBOSE=1; shift ;;
        *) echo "ERROR: unknown option: $1" >&2; exit 2 ;;
    esac
done

if [[ -z "$ROOT" ]]; then
    ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/skills"
fi
ROOT="${ROOT%/}"

if [[ ! -d "$ROOT" ]]; then
    echo "ERROR: skills root not found: $ROOT" >&2
    exit 2
fi

BROKEN=()
CHECKED=0

is_skippable() {
    case "$1" in
        ''|http://*|https://*|mailto:*|\#*|/*) return 0 ;;
        *'<'*|*'*'*|*' '*) return 0 ;;
    esac
    return 1
}

check_ref() {
    local file="$1" ref="$2" kind="$3" dir rel skill_root
    dir="$(dirname "$file")"
    rel="${file#"$ROOT"/}"
    skill_root="$ROOT/${rel%%/*}"

    CHECKED=$((CHECKED + 1))
    if [[ -f "$dir/$ref" || -f "$skill_root/$ref" ]]; then
        [[ $VERBOSE -eq 1 ]] && echo "  ok   ($kind) $rel -> $ref"
        return 0
    fi
    BROKEN+=("$rel -> $ref  [$kind]")
    return 0
}

while IFS= read -r file; do
    while IFS= read -r ref; do
        is_skippable "$ref" && continue
        check_ref "$file" "$ref" link
    done < <(grep -oE '\]\([^)]+\)' "$file" 2>/dev/null | sed -E 's/^\]\(//; s/\)$//' || true)

    while IFS= read -r ref; do
        is_skippable "$ref" && continue
        check_ref "$file" "$ref" load
    done < <(grep -oE '`references/[A-Za-z0-9_.-]+\.md`' "$file" 2>/dev/null | tr -d '`' || true)
done < <(find "$ROOT" -name '*.md' -type f | sort)

echo ""
echo "=== Skill Doc Link Summary ==="
if [[ ${#BROKEN[@]} -eq 0 ]]; then
    echo "✅ All $CHECKED skill doc reference(s) resolve"
    exit 0
fi

echo "❌ ${#BROKEN[@]} broken reference(s) out of $CHECKED checked:"
for entry in "${BROKEN[@]}"; do
    echo "   - $entry"
done
exit 1
```

- [ ] **Step 4: 运行测试确认它通过**

Run: `chmod +x scripts/validate-skill-links.sh scripts/tests/validate-skill-links-test.sh && bash scripts/tests/validate-skill-links-test.sh`
Expected: PASS —— `=== Summary: 5 passed, 0 failed ===`，退出码 0。

- [ ] **Step 5: 对真实仓库运行，确认它报出三处已知断链**

Run: `bash scripts/validate-skill-links.sh; echo "exit=$?"`
Expected: 退出码 `1`，且断链清单**恰好**是下面 5 条（4 处引用指向 2 个唯一目标 + `ruby.md`）：

```
   - gf-label-stats/SKILL.md -> ../references/gf-label-stats-taxonomy.md  [link]
   - gf-pr-review/SKILL.md -> ../references/pr-review-checklist.md  [link]
   - gf-pr-review/SKILL.md -> ../references/pr-review-checklist.md  [link]
   - gf-pr-review/SKILL.md -> ../references/pr-review-checklist.md  [link]
   - gf-quality/references/detector.md -> references/ruby.md  [load]
```

**若出现其他条目**：说明判据过宽（误报）或有新的断链，停下来核实后再继续，不要放宽判据来让它变绿。

这 5 条就是 Task 2 与 Task 6 的 RED。

- [ ] **Step 6: 提交**（须先取得用户许可）

```bash
git add scripts/validate-skill-links.sh scripts/tests/validate-skill-links-test.sh
git commit -m "feat(scripts): add skill doc link validator with fixture tests

Validates navigational references in skills/ — markdown links and
backticked references/*.md load targets. Resolves against both the
containing directory and the skill root.

Refs #348"
```

---

### Task 2: 修复 `../references/` 少一级的两处断链

Task 1 Step 5 报出的前 4 条。#348 未发现这一组，但它与缺陷 1 同属「引用指向不存在的文件」。

**Files:**
- Modify: `skills/gf-label-stats/SKILL.md:12`
- Modify: `skills/gf-pr-review/SKILL.md:65,79,83`

**Interfaces:**
- Consumes: Task 1 的 `scripts/validate-skill-links.sh`
- Produces: 无

- [ ] **Step 1: 确认目标文件的真实位置**

Run:
```bash
ls -1 docs/references/gf-label-stats-taxonomy.md docs/references/pr-review-checklist.md
```
Expected: 两行均输出路径（文件存在）。

从 `skills/<skill>/SKILL.md` 出发到达 `docs/references/` 需要上溯两级：`../../docs/references/`。现有的 `../references/` 只上溯一级，落在不存在的 `skills/references/`。

- [ ] **Step 2: 修正 `gf-label-stats`**

```bash
sed -i '' 's#(\.\./references/gf-label-stats-taxonomy\.md)#(../../docs/references/gf-label-stats-taxonomy.md)#g' \
  skills/gf-label-stats/SKILL.md
```

- [ ] **Step 3: 修正 `gf-pr-review`**

```bash
sed -i '' 's#(\.\./references/pr-review-checklist\.md)#(../../docs/references/pr-review-checklist.md)#g' \
  skills/gf-pr-review/SKILL.md
```

- [ ] **Step 4: 验证这 4 条已消失**

Run: `bash scripts/validate-skill-links.sh; echo "exit=$?"`
Expected: 退出码仍为 `1`，但断链清单**只剩 1 条**：

```
   - gf-quality/references/detector.md -> references/ruby.md  [load]
```

- [ ] **Step 5: 确认没有残留的旧路径**

Run: `grep -rn '\.\./references/' skills/ || echo "CLEAN"`
Expected: `CLEAN`

- [ ] **Step 6: 提交**（须先取得用户许可）

```bash
git add skills/gf-label-stats/SKILL.md skills/gf-pr-review/SKILL.md
git commit -m "fix(skills): correct ../references paths to ../../docs/references

Both resolved to skills/references/ which does not exist; the targets
live in docs/references/. Found by validate-skill-links.sh.

Refs #348"
```

---

### Task 3: `SKILL.md` —— Gate 3 口径改为总行覆盖率 + N/A 语义

确立后续五个语言层要对齐的权威措辞，故先于各语言层。

**Files:**
- Modify: `skills/gf-quality/SKILL.md:101`

**Interfaces:**
- Consumes: 无
- Produces: Gate 3 的权威判据文本 `Total line coverage ≥ 80% (COV_THRESHOLD overrides); N/A when the change touches no source file of that language` —— Task 4/5/6 的语言层须与之一致。

- [ ] **Step 1: 确认改前原文**

Run: `sed -n '101p' skills/gf-quality/SKILL.md`
Expected: `| 3 | **coverage** | Incremental coverage ≥ 80% (\`COV_THRESHOLD\` overrides) |`

- [ ] **Step 2: 替换 Gate 3 判据**

把第 101 行整行替换为：

```markdown
| 3 | **coverage** | Total line coverage ≥ 80% (`COV_THRESHOLD` overrides); N/A when the change touches no source file of that language |
```

- [ ] **Step 3: 在 Preconditions 段补充 N/A 判定规则**

在 `SKILL.md` 的 `**Preconditions:**` 列表中，`- If a tool is missing → mark gate SKIPPED, warn user, do NOT auto-install` 之后追加一条：

```markdown
- Gate 3 is `N/A` when the change set contains no source file of the detected language. Determine the change set with
  `git diff --name-only "$(git merge-base HEAD "${BASE_REF:-origin/main}")"` and match the language's source extension.
  Report `N/A` distinctly from a tool-missing `SKIPPED` — they have different causes and different follow-ups.
```

- [ ] **Step 4: 验证**

Run:
```bash
grep -n 'Total line coverage' skills/gf-quality/SKILL.md
grep -c 'Incremental coverage' skills/gf-quality/SKILL.md
grep -n 'BASE_REF' skills/gf-quality/SKILL.md
```
Expected: 第一条命中 1 行；第二条输出 `0`；第三条命中 1 行。

- [ ] **Step 5: 提交**（须先取得用户许可）

```bash
git add skills/gf-quality/SKILL.md
git commit -m "fix(gf-quality): Gate 3 judges total line coverage, N/A on no-op

The incremental claim was never implemented — every language layer's
command produced total coverage. Neither cargo-llvm-cov 0.9.0 nor
cargo-tarpaulin 0.37.2 has a native diff mode (verified via --help).

Closes #354"
```

---

### Task 4: `rust.md` —— tarpaulin 全部改为 llvm-cov

#340 的主体。6 处 `tarpaulin` 全在此文件。

**Files:**
- Modify: `skills/gf-quality/references/rust.md`（第 11、20、56、100、110、127 行）

**Interfaces:**
- Consumes: Task 3 确立的口径文本
- Produces: Rust Gate 3 命令 `cargo llvm-cov --workspace --fail-under-lines ${COV_THRESHOLD:-80}`

- [ ] **Step 1: 确认改前有 6 处**

Run: `grep -c tarpaulin skills/gf-quality/references/rust.md`
Expected: `6`

- [ ] **Step 2: 替换 Gate 3 命令行（第 11 行）**

原文：
```markdown
| 3 | coverage | `cargo tarpaulin --workspace 2>&1 \| tail -3` | > `COV_THRESHOLD` (default 80%) |
```
改为（阈值由工具原生强制，不再解析文本输出）：
```markdown
| 3 | coverage | `cargo llvm-cov --workspace --fail-under-lines ${COV_THRESHOLD:-80}` | exit 0 (total line coverage ≥ threshold); N/A if no `.rs` in change set |
```

- [ ] **Step 3: 替换两处安装表（第 20、56 行）**

两行均把
```markdown
| cargo-tarpaulin | `cargo install cargo-tarpaulin` | ... |
```
的工具名与安装命令改为：
```markdown
| cargo-llvm-cov | `cargo install cargo-llvm-cov` | ... |
```
（保持各自所在表格的其余列不变：第 20 行所在表为 `| Tool | Install Command | Required By |`，第 56 行所在表为 `| Tool | Install | Config File | Required |`。）

- [ ] **Step 4: 替换 Language-Specific Notes（第 100 行）**

原文：
```markdown
- Gate 3 requires `cargo-tarpaulin` — if missing, mark SKIPPED
```
改为：
```markdown
- Gate 3 requires `cargo-llvm-cov` — if missing, mark SKIPPED
- Gate 3 is N/A when the change set contains no `.rs` file — report N/A, not SKIPPED
```

- [ ] **Step 5: 替换 Troubleshooting（第 110 行）**

原文：
```markdown
| `cargo-tarpaulin: command not found` | Tool not installed | `cargo install cargo-tarpaulin` |
```
改为：
```markdown
| `cargo-llvm-cov: command not found` | Tool not installed | `cargo install cargo-llvm-cov` |
```

- [ ] **Step 6: 替换 FAQ（第 127 行）**

原文：
```markdown
A: Ensure `cargo-tarpaulin` is installed and project builds successfully. Check for `#[cfg(test)]` modules.
```
改为：
```markdown
A: Ensure `cargo-llvm-cov` is installed and project builds successfully. Check for `#[cfg(test)]` modules.
```

- [ ] **Step 7: 验证**

Run:
```bash
grep -c tarpaulin skills/gf-quality/references/rust.md
grep -c 'llvm-cov' skills/gf-quality/references/rust.md
grep -n 'fail-under-lines' skills/gf-quality/references/rust.md
```
Expected: 第一条 `0`；第二条 `5`；第三条命中 1 行。

- [ ] **Step 8: 实跑确认命令可用且当前通过**

Run:
```bash
cargo llvm-cov --workspace --ignore-run-fail --fail-under-lines 80 --summary-only 2>&1 | tail -3; echo "exit=${PIPESTATUS[0]}"
```
Expected: `TOTAL` 行的 line 覆盖率约 85.8%，退出码 `0`。

> `--ignore-run-fail` 仅因本仓库存在两处与本次无关的既有测试失败（`gitflow-gitlab` 的 `temp_env` 并行竞态、`e2e-gitcode` 的 `gc` 撞名）。**写进 `rust.md` 的命令不含该参数**——它只是本地验证的权宜，不是闸门的一部分。

- [ ] **Step 9: 提交**（须先取得用户许可）

```bash
git add skills/gf-quality/references/rust.md
git commit -m "fix(gf-quality): unify Rust coverage tool to cargo-llvm-cov

Makefile:158 already ran llvm-cov while rust.md pointed at tarpaulin in
6 places. Threshold is now enforced natively via --fail-under-lines
instead of parsing tail -3.

Closes #340"
```

---

### Task 5: 四个语言层 —— 覆盖率口径同步 + auto-fix 冲突去除

`go` / `node` / `python` / `java` 四份 reference 同时涉及口径与 auto-fix 两类改动，且 `go.md`、`python.md` 两者都要改。合并为一个任务以避免同文件二次返工。

**Files:**
- Modify: `skills/gf-quality/references/go.md`（第 11、27、28、34、89、90 行）
- Modify: `skills/gf-quality/references/node.md`（第 26、37、66 行）
- Modify: `skills/gf-quality/references/python.md`（第 11、30、36、96 行）
- Modify: `skills/gf-quality/references/java.md`（第 13、24 行）

**Interfaces:**
- Consumes: Task 3 确立的口径文本；Task 4 确立的 `rust.md` 句式
- Produces: 四个语言层与 Rust 一致的 Gate 3 判据与 Forbidden Actions 句式

- [ ] **Step 1: 确认改前状态**

Run:
```bash
grep -rn 'incremental ≥ 80%\|incremental coverage' skills/gf-quality/references/
grep -rn 'auto-fix' skills/gf-quality/references/
```
Expected: 第一条命中 8 行（`go.md` 3、`node.md` 2、`python.md` 1、`java.md` 2）；第二条命中 9 行（`go.md` 3、`node.md` 1、`python.md` 3、`rust.md` 2）。

> ⚠️ 不要用裸 `grep incremental` 做断言：`rust.md:133,139` 与 `java.md:171,172` 的 `incremental` 指的是**增量编译**（`profile.dev.incremental`、`org.gradle.caching`），与覆盖率无关，必须保留。断言务必带上 `≥ 80%` 或 `coverage` 限定。

- [ ] **Step 2: `go.md` —— Gate 3 命令加阈值强制（第 11 行）**

原文：
```markdown
| 3 | coverage | `go test ./... -coverprofile=coverage.out && go tool cover -func=coverage.out \| grep total` | incremental ≥ 80% |
```
改为（Go 无原生阈值参数，用 `awk` 比较）：
```markdown
| 3 | coverage | `go test ./... -coverprofile=coverage.out && go tool cover -func=coverage.out \| awk -v t="${COV_THRESHOLD:-80}" '/^total:/ {gsub(/%/,"",$3); exit ($3+0 < t+0)}'` | exit 0 (total line coverage ≥ threshold); N/A if no `.go` in change set |
```

- [ ] **Step 3: `go.md` —— Notes 两处（第 27-28、89-90 行）**

两处内容逐字重复，改法相同。原文：
```markdown
- Gate 3: compare against previous run; incremental coverage ≥ 80%
- Gate 4: auto-fix with `gofmt -w .` only after user confirmation
```
改为：
```markdown
- Gate 3: total line coverage ≥ `COV_THRESHOLD` (default 80%); N/A when the change set has no `.go` file
- Gate 4: report the `gofmt -l .` file list — never run `gofmt -w .`
```

- [ ] **Step 4: `go.md` —— Forbidden Actions（第 34 行）**

原文 `- ❌ Never auto-fix without showing diff first` 反向授权了「出示 diff 即可自动修复」。改为与 `rust.md:34-35` 一致：
```markdown
- ❌ Never auto-fix with `gofmt -w .` — report only
```

- [ ] **Step 5: `python.md` —— Gate 3 命令加阈值强制（第 11 行）**

原文：
```markdown
| 3 | coverage | `python -m pytest --cov=src/ --cov-report=term-missing` | incremental ≥ 80% |
```
改为：
```markdown
| 3 | coverage | `python -m pytest --cov=src/ --cov-report=term-missing --cov-fail-under=${COV_THRESHOLD:-80}` | exit 0 (total line coverage ≥ threshold); N/A if no `.py` in change set |
```

- [ ] **Step 6: `python.md` —— Notes 两处（第 30、96 行）**

两处均为：
```markdown
- Gate 4: auto-fix with `ruff format .` or `black .` only after user confirmation
```
改为：
```markdown
- Gate 4: report the `ruff format --check .` diff — never run `ruff format .` or `black .`
```

> `python.md` 的 Notes 段只有 Gate 1 / 4 / 5 三条，**没有** Gate 3 行——该文件唯一的覆盖率口径在第 11 行（已由 Step 5 处理）。

- [ ] **Step 7: `python.md` —— Forbidden Actions（第 36 行）**

原文 `- ❌ Never auto-fix without showing diff first` 改为：
```markdown
- ❌ Never auto-fix with `ruff format .` or `black .` — report only
```

- [ ] **Step 8: `node.md` —— Gate 3 两处（第 26、37 行）**

第 26 行原文：
```markdown
| 3 | coverage | `bun test --coverage` | incremental ≥ 80% |
```
改为：
```markdown
| 3 | coverage | `bun test --coverage --coverage-threshold=${COV_THRESHOLD:-80}` | exit 0 (total line coverage ≥ threshold); N/A if no `.js`/`.jsx`/`.ts`/`.tsx` in change set |
```

第 37 行原文：
```markdown
| 3 | coverage | `npm run test:coverage` or `npx jest --coverage` | incremental ≥ 80% |
```
改为：
```markdown
| 3 | coverage | `npm run test:coverage` or `npx jest --coverage --coverageThreshold='{"global":{"lines":80}}'` | exit 0 (total line coverage ≥ threshold); N/A if no `.js`/`.jsx`/`.ts`/`.tsx` in change set |
```

- [ ] **Step 9: `node.md` —— Forbidden Actions（第 66 行）**

原文 `- ❌ Never auto-fix lint issues without showing diff first` 改为：
```markdown
- ❌ Never auto-fix lint issues with `eslint --fix` — report only
```

- [ ] **Step 10: `java.md` —— Gate 3 两处（第 13、24 行）**

第 13 行原文：
```markdown
| 3 | coverage | `mvn verify -Pcoverage` (requires JaCoCo) | incremental ≥ 80% |
```
改为：
```markdown
| 3 | coverage | `mvn verify -Pcoverage` (requires a JaCoCo `check` rule with LINE COVEREDRATIO ≥ 0.80) | exit 0 (total line coverage ≥ threshold); N/A if no `.java` in change set |
```

第 24 行原文：
```markdown
| 3 | coverage | `./gradlew jacocoTestReport` | incremental ≥ 80% |
```
改为：
```markdown
| 3 | coverage | `./gradlew jacocoTestReport jacocoTestCoverageVerification` (violationRules: LINE COVEREDRATIO ≥ 0.80) | exit 0 (total line coverage ≥ threshold); N/A if no `.java` in change set |
```

- [ ] **Step 11: 验证口径与 auto-fix 全部收敛**

Run:
```bash
grep -rn 'incremental ≥ 80%\|incremental coverage' skills/gf-quality/ || echo "NO INCREMENTAL COVERAGE LEFT"
grep -rn 'auto-fix' skills/gf-quality/references/
grep -rn 'only after user confirmation' skills/gf-quality/references/ || echo "NO AUTO-FIX AUTHORIZATION LEFT"
grep -rn 'without showing diff first' skills/gf-quality/references/ || echo "NO REVERSE AUTHORIZATION LEFT"
```
Expected: 第一条输出 `NO INCREMENTAL COVERAGE LEFT`；第二条每一行都形如 `❌ Never auto-fix ... — report only`；第三、四条分别输出对应的 `LEFT` 提示。

> `rust.md:133,139` 与 `java.md:171,172` 的「incremental compilation」不受影响，仍应存在。

- [ ] **Step 12: 提交**（须先取得用户许可）

```bash
git add skills/gf-quality/references/go.md skills/gf-quality/references/node.md \
        skills/gf-quality/references/python.md skills/gf-quality/references/java.md
git commit -m "fix(gf-quality): align 4 language layers to total line coverage

Also removes auto-fix authorizations that contradicted SKILL.md's
'Report only. No auto-fix.' — present in python.md as well as go.md,
plus three 'without showing diff first' phrasings that授权 the opposite.

Closes #348"
```

---

### Task 6: 补写 `references/ruby.md`

Task 1 Step 5 报出的最后 1 条断链。放在语言层之后，以便直接沿用已确立的句式。

**Files:**
- Create: `skills/gf-quality/references/ruby.md`

**Interfaces:**
- Consumes: Task 3 的口径文本、Task 5 的语言层句式
- Produces: `references/ruby.md`，使 `detector.md:26` 的映射可解析

- [ ] **Step 1: 确认它仍是唯一剩余断链**

Run: `bash scripts/validate-skill-links.sh; echo "exit=$?"`
Expected: 退出码 `1`，清单仅 `gf-quality/references/detector.md -> references/ruby.md  [load]`。

- [ ] **Step 2: 创建文件**

````markdown
# Ruby Quality Toolchain

**Detection:** `Gemfile` in project root.

## Gate Commands

| # | Gate | Command | Pass Criteria |
|---|------|---------|---------------|
| 1 | build | `bundle install --quiet` | exit 0 |
| 2 | test | `bundle exec rspec` | all pass |
| 3 | coverage | `bundle exec rspec` with SimpleCov `minimum_coverage ${COV_THRESHOLD:-80}` in `spec/spec_helper.rb` | exit 0 (total line coverage ≥ threshold); N/A if no `.rb` in change set |
| 4 | format | `bundle exec rubocop --only Layout` | exit 0, no offenses |
| 5 | static | `bundle exec rubocop` | exit 0, no offenses |
| 6 | pre-commit | `pre-commit run --all-files` | all hooks pass (or N/A if no `.pre-commit-config.yaml`) |

## Tool Installation

| Tool | Install Command | Required By |
|------|----------------|-------------|
| bundler | `gem install bundler` | Gates 1–5 |
| rspec | `bundle add rspec --group development,test` | Gates 2, 3 |
| simplecov | `bundle add simplecov --group test` | Gate 3 (coverage) |
| rubocop | `bundle add rubocop --group development` | Gates 4, 5 |

If a tool is missing, **warn the user and recommend install** — do NOT auto-install.

## Environment Variables

| Variable | Effect | Default |
|----------|--------|---------|
| `COV_THRESHOLD` / `COVERAGE_THRESHOLD` | Override coverage threshold | 80% |
| `BUNDLE_GEMFILE` | Alternate Gemfile location | `./Gemfile` |
| `RAILS_ENV` / `RACK_ENV` | Environment for test runs | `test` |

## Forbidden Actions

- ❌ Never auto-fix with `rubocop -a` or `rubocop -A` — report only
- ❌ Never run `bundle update` — it mutates `Gemfile.lock` outside the user's intent
- ❌ Never modify `spec/spec_helper.rb` to lower `minimum_coverage`

## Makefile-First Rule

If project root contains a `Makefile` with matching targets, prefer `make` commands over direct tool invocations:

| Gate | Preferred Command | Fallback |
|------|-------------------|----------|
| build | `make build` | `bundle install --quiet` |
| test | `make test` | `bundle exec rspec` |
| format | `make fmt` | `bundle exec rubocop --only Layout` |
| static | `make lint` | `bundle exec rubocop` |

Detection: `make -n <target> >/dev/null 2>&1` returns 0 → target exists.

## Configuration

### Config File Examples

#### `.rubocop.yml`

```yaml
AllCops:
  NewCops: enable
  Exclude:
    - "vendor/**/*"
    - "db/schema.rb"
Metrics/MethodLength:
  Max: 20
```

#### `spec/spec_helper.rb` (SimpleCov)

```ruby
require "simplecov"

SimpleCov.start do
  add_filter "/spec/"
  minimum_coverage Integer(ENV.fetch("COV_THRESHOLD", "80"))
end
```

### Language-Specific Notes

- Gate 3 requires `simplecov` wired into `spec_helper.rb` — if absent, mark SKIPPED
- Gate 3 is N/A when the change set contains no `.rb` file — report N/A, not SKIPPED
- SimpleCov reports **line** coverage, matching the other language layers
- Run gates from the directory containing `Gemfile`

## Troubleshooting

### Common Errors

| Error | Cause | Fix |
|-------|-------|-----|
| `bundler: command not found: rspec` | rspec not in bundle | `bundle add rspec --group development,test` |
| `Could not locate Gemfile` | Wrong working directory | `cd` to the directory containing `Gemfile` |
| `SimpleCov failed with exit 2` | Coverage below `minimum_coverage` | Add tests; do not lower the threshold |
| `Gemfile.lock out of date` | Dependencies drifted | Report to user — do NOT run `bundle update` |

### Exit Code Reference

| Code | Meaning | Action |
|------|---------|--------|
| 0 | Success | Continue to next gate |
| 1 | Test failure or RuboCop offense | Fix and re-run |
| 2 | SimpleCov below threshold | Add tests |

### FAQ

**Q: Why does coverage show 0%?**
A: `require "simplecov"` and `SimpleCov.start` must run **before** application code is loaded. Put them at the very top of `spec/spec_helper.rb`.

**Q: RuboCop and the formatter disagree?**
A: Gate 4 runs only `--only Layout`; Gate 5 runs the full rule set. Report both, fix neither.
````

- [ ] **Step 3: 验证断链清零**

Run: `bash scripts/validate-skill-links.sh; echo "exit=$?"`
Expected: 退出码 `0`，输出 `✅ All N skill doc reference(s) resolve`。

- [ ] **Step 4: 验证口径与全局约束一致**

Run:
```bash
grep -c 'incremental' skills/gf-quality/references/ruby.md
grep -n 'report only' skills/gf-quality/references/ruby.md
grep -c 'tarpaulin' skills/gf-quality/references/ruby.md
```
Expected: 依次为 `0`、命中 1 行、`0`。

- [ ] **Step 5: 提交**（须先取得用户许可）

```bash
git add skills/gf-quality/references/ruby.md
git commit -m "fix(gf-quality): add missing references/ruby.md

detector.md mapped Gemfile to references/ruby.md but the file never
existed — Ruby projects detected fine, then failed to load.

Closes #348"
```

---

### Task 7: 外围文档同步 + 接线到 `check-agent-sync`

最后一个任务：把剩余的规范性 `tarpaulin` 提及改掉，并把校验脚本接进 Make 目标。接线放在最后，保证此前每次提交时 `make check-agent-sync` 都是绿的。

**Files:**
- Modify: `docs/integration-guide.md:100`
- Modify: `docs/references/gf-quality-params.md:161`
- Modify: `docs/superpowers/tests/skills/gf-quality-test.md`（7 处）
- Modify: `Makefile`（`check-agent-sync` 目标，第 174-181 行）

**Interfaces:**
- Consumes: Task 1 的 `scripts/validate-skill-links.sh`
- Produces: `make check-agent-sync` 包含链接校验

- [ ] **Step 1: `docs/integration-guide.md` —— 改工具名并修正检查项数**

第 100 行所在的框写「5 项检查」但 `SKILL.md` 定义了 6 个 Gate（漏列 pre-commit）。把该框内的
```
│  5 项检查，快速失败:                                          │
```
改为
```
│  6 项检查，快速失败:                                          │
```
并把
```
│  3. coverage  cargo tarpaulin (>80%)               ✅/❌      │
```
改为
```
│  3. coverage  cargo llvm-cov (行覆盖 >80%)          ✅/❌      │
```
在 `5. static` 行之后补上第 6 项：
```
│  6. pre-commit  pre-commit run --all-files         ✅/❌      │
```

> 注意该框是 ASCII 表格，右边框 `│` 需保持列对齐。改完目视确认竖线仍成一列。

- [ ] **Step 2: `docs/references/gf-quality-params.md` —— 改第 161 行**

原文：
```markdown
| Rust | `cargo-tarpaulin` missing, nightly toolchain missing, compilation errors | Coverage shows 0%, skip doc tests, slow workspace builds |
```
改为：
```markdown
| Rust | `cargo-llvm-cov` missing, nightly toolchain missing, compilation errors | Coverage shows 0%, skip doc tests, slow workspace builds |
```

- [ ] **Step 3: `docs/superpowers/tests/skills/gf-quality-test.md` —— 7 处全量替换**

该文件是**现行生效**的 skill 测试场景（非史料），其中 `tarpaulin 未安装` 的场景需改为 `llvm-cov 未安装`：

```bash
sed -i '' 's/cargo-tarpaulin/cargo-llvm-cov/g; s/cargo tarpaulin/cargo llvm-cov/g; s/tarpaulin/llvm-cov/g' \
  docs/superpowers/tests/skills/gf-quality-test.md
```

改完人工通读一遍第 11、142、145、149、152、157、167 行，确认语义仍通顺（例如「6-gate 闸门（fmt / clippy / test / docs / llvm-cov / pre-commit）」）。

- [ ] **Step 4: 验证规范性文件已无 tarpaulin**

Run:
```bash
grep -rn tarpaulin skills/ docs/integration-guide.md docs/references/ docs/superpowers/tests/ || echo "NORMATIVE CLEAN"
```
Expected: `NORMATIVE CLEAN`

- [ ] **Step 5: 验证史料原样保留**

Run:
```bash
grep -rc tarpaulin docs/reports-archive/ docs/superpowers/plans/ docs/research/ \
  docs/issue-triage-report-2026-09-16.md docs/superpowers/specs/2026-07-06-skill-constraints-sync-design.md \
  | grep -v ':0'
```
Expected: 共 13 处，分布为 `reports-archive` 1、`plans` 3（两个文件）、`research` 6（两个文件）、`issue-triage-report` 2、`2026-07-06-spec` 1。**一处都不应减少。**

- [ ] **Step 6: 接线到 `check-agent-sync`**

在 `Makefile` 的 `check-agent-sync` 目标中，`@bash scripts/validate-skill-commands.sh` 之后追加一行：

```makefile
	@bash scripts/validate-skill-links.sh
```

- [ ] **Step 7: 跑通 Make 目标**

Run: `make check-agent-sync`
Expected: 退出码 `0`，输出依次包含 `✓ CLAUDE.md 存在`、`When NOT to Use` 校验结果、`gf` 命令校验结果，以及 `✅ All N skill doc reference(s) resolve`。

- [ ] **Step 8: 跑脚本自测确认未回归**

Run: `bash scripts/tests/validate-skill-links-test.sh`
Expected: `=== Summary: 5 passed, 0 failed ===`

- [ ] **Step 9: 提交**（须先取得用户许可）

```bash
git add docs/integration-guide.md docs/references/gf-quality-params.md \
        docs/superpowers/tests/skills/gf-quality-test.md Makefile
git commit -m "fix(quality): sync remaining normative docs to llvm-cov, wire link check

Archival documents (reports-archive, past plans, research, dated triage
reports) keep their tarpaulin references — rewriting them would make the
record disagree with what actually ran at the time.

Closes #340"
```

---

## 交付前检查清单

全部 7 个任务完成后，逐条确认：

- [ ] `bash scripts/tests/validate-skill-links-test.sh` → 5 passed, 0 failed
- [ ] `make check-agent-sync` → 退出码 0
- [ ] `grep -rn tarpaulin skills/ docs/integration-guide.md docs/references/ docs/superpowers/tests/` → 无输出
- [ ] 史料 13 处 `tarpaulin` 一处未减
- [ ] `grep -rn 'incremental ≥ 80%\|incremental coverage' skills/gf-quality/` → 无输出（`rust.md` / `java.md` 的增量**编译**表述不受影响，应仍在）
- [ ] `grep -rn 'only after user confirmation\|without showing diff first' skills/gf-quality/references/` → 无输出
- [ ] `cargo llvm-cov --workspace --ignore-run-fail --fail-under-lines 80 --summary-only` → 退出码 0
- [ ] `git diff --summary <base>...HEAD | grep 'create mode 120000'` → 无输出（无符号链接误提交）
- [ ] `.claude/skills/` 未被改动：`git status --porcelain .claude/` 为空

## 已知的、不在本次范围内的失败

以下两处失败**先于本次变更存在**，不得在本计划中修复，亦不得作为本次变更的回归：

| 失败 | 位置 | 成因 |
|---|---|---|
| `gitflow-gitlab` 4 个 auth 测试 | `crates/gitlab/src/auth.rs:347` | `temp_env::with_var` 改的是进程级环境变量，与同进程并行的 async 测试竞态；失败条数在 3～4 间抖动 |
| `e2e-gitcode` noauth 2 个测试 | `crates/e2e-gitcode/tests/noauth.rs` | Graphviz 的 `gc` 与 GitCode CLI 命令撞名（已知环境噪声） |

两者都会让 `gf-quality` Gate 2（test）判负。如实报告，Phase 4 单独提 Issue。
