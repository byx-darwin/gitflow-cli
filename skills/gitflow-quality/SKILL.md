---
name: gitflow-quality
description: 质量关卡 — 5 项质量检查闸门（build / test / coverage / format / static），全部通过才能进入交付阶段
---

# gitflow-cli quality 质量关卡

在交付前运行 5 项质量检查，按顺序执行、失败即停（fast-fail），最终生成 Quality Report。编排层只做指挥，所有检查通过标准 CLI 命令执行。

## 前置条件

```bash
# 确认在 git 仓库中
git rev-parse --show-toplevel

# 确认工作区干净（无未提交变更）
git status --porcelain
```

如果工作区有未提交变更，先提交或暂存后再运行质量关卡。

---

## 检查清单

| # | 检查项 | 命令 | 通过标准 |
|---|--------|------|---------|
| 1 | build | `cargo build --workspace` | 退出码 0，无 error |
| 2 | test | `cargo test --workspace` | 全部测试通过 |
| 3 | coverage | `cargo tarpaulin --workspace` | 覆盖率 > 80% |
| 4 | format | `cargo +nightly fmt -- --check` | 退出码 0，无 diff |
| 5 | static | `cargo clippy --workspace --all-targets -- -D warnings` | 退出码 0，无 warning |

---

## 执行流程

### 步骤 0：检测项目语言

根据项目特征文件判断语言，适配检查工具：

```bash
# 检测 Rust 项目
if [ -f "Cargo.toml" ]; then
    LANG="rust"
# 检测 Node.js 项目
elif [ -f "package.json" ]; then
    LANG="node"
# 检测 Python 项目
elif [ -f "pyproject.toml" ] || [ -f "setup.py" ]; then
    LANG="python"
# 检测 Go 项目
elif [ -f "go.mod" ]; then
    LANG="go"
else
    LANG="unknown"
fi
```

**非 Rust 项目的适配命令**：

| 检查项 | Node.js | Python | Go |
|--------|---------|--------|-----|
| build | `npm run build` | `python -m py_compile src/` | `go build ./...` |
| test | `npm test` | `pytest` | `go test ./...` |
| coverage | `npx jest --coverage` 或 `npx vitest run --coverage`（取决于测试框架） | `pytest --cov` | `go test -coverprofile=coverage.out ./... && go tool cover -func=coverage.out \| grep total` |
| format | `npx prettier --check .` | `black --check .` | `test -z "$(gofmt -l .)"` |
| static | `npx eslint .` | `ruff check .` | `golangci-lint run` |

后续步骤以 Rust 为例，非 Rust 项目替换对应命令即可。

---

### 步骤 1：Build — 编译检查

```bash
cargo build --workspace 2>&1
```

- **通过**：退出码 0 → 记录 `✅ build`
- **失败**：退出码非 0 → 记录 `❌ build` + 错误摘要 → **fast-fail，跳过后续检查**

提取关键信息：
- 错误数量
- 警告数量（如有）

---

### 步骤 2：Test — 单元测试

```bash
cargo test --workspace 2>&1
```

- **通过**：全部测试通过 → 记录 `✅ test`
- **失败**：有测试失败 → 记录 `❌ test` + 失败摘要 → **fast-fail**

提取关键信息：
- 通过数量：`X passed`
- 失败数量：`Y failed`
- 忽略数量：`Z ignored`

---

### 步骤 3：Coverage — 代码覆盖率

```bash
# 读取自定义阈值，默认 80%
THRESHOLD=${COVERAGE_THRESHOLD:-80}

cargo tarpaulin --workspace 2>&1
```

- **通过**：覆盖率 > `$THRESHOLD` → 记录 `✅ coverage`
- **失败**：覆盖率 <= `$THRESHOLD` → 记录 `❌ coverage` + 当前覆盖率 → **fast-fail**

提取关键信息：
- 当前覆盖率百分比
- 阈值（`$THRESHOLD`%，来自 `COVERAGE_THRESHOLD` 环境变量或默认 80%）

**注意**：如果 `cargo tarpaulin` 未安装，提示用户安装：

```bash
cargo install cargo-tarpaulin
```

或者使用 `cargo llvm-cov` 作为替代方案。

---

### 步骤 4：Format — 代码格式

```bash
cargo +nightly fmt -- --check 2>&1
```

- **通过**：退出码 0，无格式差异 → 记录 `✅ format`
- **失败**：有格式差异 → 记录 `❌ format` + 不符文件列表 → **fast-fail**

**自动修复提示**：

```
运行 `cargo +nightly fmt` 自动修复格式问题
```

---

### 步骤 5：Static — 静态分析

```bash
cargo clippy --workspace --all-targets -- -D warnings 2>&1
```

- **通过**：退出码 0，无警告 → 记录 `✅ static`
- **失败**：有警告或错误 → 记录 `❌ static` + 警告摘要 → **fast-fail**

**自动修复提示**：

```
运行 `cargo clippy --fix --workspace --all-targets` 自动修复部分 lint 问题
```

---

## Quality Report 格式

所有检查完成后（或 fast-fail 后），生成 Quality Report：

```markdown
## Quality Report — YYYY-MM-DD

| Check    | Status | Details |
|----------|--------|---------|
| build    | ✅     | 0 errors, 0 warnings |
| test     | ✅     | 47 passed, 0 failed |
| coverage | ✅     | 85.3% (threshold: 80%) |
| format   | ✅     | No diff |
| static   | ✅     | No warnings |
```

**Status 列**：
- ✅ 表示通过
- ❌ 表示失败

**Details 列示例**：
- build：`0 errors, 2 warnings` 或 `3 errors`
- test：`47 passed, 0 failed, 2 ignored` 或 `2 failed: test_foo, test_bar`
- coverage：`85.3% (threshold: 80%)` 或 `72.1% (threshold: 80%) — BELOW THRESHOLD`
- format：`No diff` 或 `3 files need formatting: src/main.rs, src/lib.rs, ...`
- static：`No warnings` 或 `5 warnings in 2 files`

**总体结论**：

```markdown
**Result: ✅ ALL CHECKS PASSED — Ready for delivery**

或

**Result: ❌ QUALITY GATE FAILED — Return to Phase 2 for fixes**
```

---

## 失败处理

### Fast-fail 策略

检查按顺序执行，遇到第一个失败项立即停止：

1. 如果 build 失败 → 跳过 test、coverage、format、static
2. 如果 test 失败 → 跳过 coverage、format、static
3. 如果 coverage 失败 → 跳过 format、static
4. 如果 format 失败 → 跳过 static

失败项在报告中标记为 `⏭️ SKIPPED`。

### 修复建议

对于每个失败项，提供具体修复建议：

| 失败项 | 修复命令 |
|--------|---------|
| build | `cargo build --workspace` 查看详细错误 |
| test | `cargo test --workspace -- --nocapture` 查看失败详情 |
| coverage | 增加测试用例覆盖未测试代码路径 |
| format | `cargo +nightly fmt` 自动修复 |
| static | `cargo clippy --fix --workspace --all-targets` 自动修复 |

---

## 自动发布到关联 Issue

检查完成后，检测是否存在关联 Issue：

```bash
# 检查关联 Issue 文件
if [ -f ".claude/gh-issue/current-issue.txt" ]; then
    ISSUE_NUMBER=$(cat .claude/gh-issue/current-issue.txt)
    echo "关联 Issue: #${ISSUE_NUMBER}"
fi
```

**如果存在关联 Issue**：

1. 检查 `gitflow` CLI 是否可用：

```bash
if ! command -v gitflow-cli &>/dev/null; then
    echo "⚠️ gitflow-cli 未安装，跳过自动发布，直接输出报告"
    # 跳转到终端输出
fi
```

2. 将 Quality Report 写入临时文件 `quality-report.md`
3. 发布到 Issue 评论：

```bash
gitflow-cli issue comment "${ISSUE_NUMBER}" --body-file quality-report.md
```

4. 清理临时文件：

```bash
rm -f quality-report.md
```

**如果不存在关联 Issue**：

直接输出 Quality Report 到终端，不发布评论。

---

## 使用示例

### 运行完整质量检查

```
使用 gitflow-quality 技能，对当前分支运行 5 项质量检查。
```

### 运行单个检查步骤

质量关卡按固定 5 项顺序执行，不支持参数化跳过。如需单独验证某项，可直接运行对应命令：

```bash
# 单独验证 build
cargo build --workspace

# 单独验证 format
cargo +nightly fmt -- --check
```

### 处理 fast-fail 场景

```
⚠️ 质量关卡未通过，返回 Phase 2 修复

失败项：
- [❌] test — 2 个测试失败: test_foo, test_bar
- [⏭️] coverage — SKIPPED (previous check failed)
- [⏭️] format — SKIPPED (previous check failed)
- [⏭️] static — SKIPPED (previous check failed)

修复建议：
1. 运行 `cargo test --workspace -- --nocapture` 查看失败详情
2. 修复失败的测试用例
3. 重新运行质量检查
```

---

## 注意事项

- **编排层不执行操作**：所有检查通过标准 CLI 命令执行
- **闸门不可跳过**：5 项检查必须全部通过才能进入交付阶段
- **fast-fail 策略**：遇到失败立即停止，节省时间
- **覆盖率阈值**：默认 80%，可通过环境变量 `COVERAGE_THRESHOLD` 自定义
- **非 Rust 项目**：检测项目语言后自动适配对应工具链
- **审计日志**：检查结果自动发布到关联 Issue（如果存在）
