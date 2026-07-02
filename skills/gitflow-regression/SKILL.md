---
name: gitflow-regression
description: 冒烟测试工作流 — 运行 scripts/smoke-test.sh 执行端到端冒烟测试，解析测试结果，失败时自动调用 gitflow-autoreport-bug 上报
---

# gitflow-regression

运行项目中的冒烟测试脚本 `scripts/smoke-test.sh`，对 gitflow CLI 进行端到端验证。解析脚本输出，判断测试通过或失败状态，并在失败时自动调用 `gitflow-autoreport-bug` 工作流上报缺陷，确保回归问题被及时发现和跟踪。

## 前置条件

- 当前目录位于 git 仓库根目录
- `scripts/smoke-test.sh` 脚本存在且具有执行权限
- `gitflow` CLI 已构建或安装

**前置检查：**

```bash
# 确认在 git 仓库中
git rev-parse --show-toplevel

# 确认冒烟测试脚本存在
test -f scripts/smoke-test.sh && echo "✅ smoke-test.sh 存在" || echo "❌ smoke-test.sh 不存在"

# 确认脚本可执行
test -x scripts/smoke-test.sh && echo "✅ 脚本可执行" || chmod +x scripts/smoke-test.sh

# 确认 gitflow CLI 可用
if [[ -f "./target/release/gitflow-cli" ]]; then
    echo "使用本地 release 构建"
elif [[ -f "./target/debug/gitflow-cli" ]]; then
    echo "使用本地 debug 构建"
elif command -v gitflow-cli &>/dev/null; then
    echo "使用全局安装版本"
else
    echo "❌ gitflow-cli 未找到，请先构建：cargo build"
    exit 1
fi
```

## 工作流

### 步骤 1：确定测试参数

根据需求确定冒烟测试的参数：

```bash
# 目标平台（默认 github）
PLATFORM="github"

# 测试模式
MODE="read-only"  # 或 "write"

# 详细输出
VERBOSE=0  # 或 1
```

**平台选项：**

| 平台 | 参数 | 说明 |
|------|------|------|
| GitHub | `--platform github` | 默认平台 |
| GitLab | `--platform gitlab` | GitLab 平台 |
| GitCode | `--platform gitcode` | GitCode 平台 |

**测试模式：**

| 模式 | 参数 | 说明 |
|------|------|------|
| 只读 | `--read-only` | 默认模式，仅测试读取类命令 |
| 写入 | `--write` | 测试写入类命令（创建、修改等） |

### 步骤 2：运行冒烟测试

调用冒烟测试脚本：

```bash
# 运行只读模式的冒烟测试（默认）
bash scripts/smoke-test.sh --platform "$PLATFORM"

# 带详细输出
bash scripts/smoke-test.sh --platform "$PLATFORM" --verbose

# 运行写入模式的冒烟测试
bash scripts/smoke-test.sh --platform "$PLATFORM" --write

# 指定版本
bash scripts/smoke-test.sh --version
```

脚本会自动执行以下检查：

1. **帮助命令测试**：验证所有顶层资源和子命令的 `--help` 输出
2. **API 读取测试**（只读模式）：
   - `issue list --limit 3`
   - `pr list --limit 3`
   - `label list`
   - `milestone list`
   - `pipeline status`
   - 等等
3. **API 写入测试**（写入模式）：
   - `issue create --help`
   - `auth login --help`
   - 等等

### 步骤 3：解析测试结果

冒烟测试脚本的输出包含 PASS/FAIL/SKIP 计数：

```bash
# 运行脚本并捕获退出码和输出
OUTPUT=$(bash scripts/smoke-test.sh --platform "$PLATFORM" 2>&1)
EXIT_CODE=$?

echo "$OUTPUT"
```

**解析输出中的关键信息：**

```bash
# 提取统计数据
PASS_COUNT=$(echo "$OUTPUT" | grep -oP '\d+(?=\s+passed)' || echo "0")
FAIL_COUNT=$(echo "$OUTPUT" | grep -oP '\d+(?=\s+failed)' || echo "0")
SKIP_COUNT=$(echo "$OUTPUT" | grep -oP '\d+(?=\s+skipped)' || echo "0")

# 或者直接看脚本末尾的汇总
# PASS: X | FAIL: Y | SKIP: Z
```

**结果判断：**

| 退出码 | 含义 | 后续操作 |
|--------|------|----------|
| 0 | 所有测试通过 | 记录结果，完成 |
| 1 | 存在失败测试 | 进入步骤 4 上报 |

### 步骤 4：处理失败结果

当冒烟测试存在失败项时，收集失败信息并上报：

**4.1 提取失败详情**

```bash
# 从输出中提取所有 [FAIL] 行
FAIL_DETAILS=$(echo "$OUTPUT" | grep '\[FAIL\]')

echo "失败的测试项："
echo "$FAIL_DETAILS"
```

**4.2 分析失败原因**

对每个失败项进行分类：

| 失败类型 | 判断依据 | 严重程度 |
|----------|----------|----------|
| 命令未找到 | `command not found` | 🔴 关键 |
| 认证失败 | `auth`、`401`、`403` | 🔴 关键 |
| API 错误 | `4xx`、`5xx` 状态码 | 🟠 高 |
| 超时 | `timeout`、`timed out` | 🟠 高 |
| 输出不符 | `expected`、`mismatch` | 🟡 中 |
| 跳过异常 | 非预期的 SKIP | 🟡 中 |

**4.3 调用 gitflow-autoreport-bug 上报**

为每个失败项创建错误报告，写入 `pending.json`：

```bash
# 创建错误报告目录
mkdir -p .cache/bug-reports

# 为每个失败项生成 pending.json
for FAIL_LINE in $FAIL_DETAILS; do
    # 提取失败信息
    COMMAND=$(echo "$FAIL_LINE" | extract_command)
    ERROR_MSG=$(echo "$FAIL_LINE" | extract_message)

    cat > .cache/bug-reports/pending.json << EOF
{
    "error_id": "smoke-$(date +%s)-$(echo $COMMAND | md5sum | head -c 8)",
    "command": "$COMMAND",
    "platform": "$PLATFORM",
    "error_code": "SMOKE_TEST_FAILED",
    "error_message": "$ERROR_MSG",
    "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF
done
```

然后调用 `gitflow-autoreport-bug` 工作流处理上报：

```
使用 gitflow-autoreport-bug 技能处理 pending.json 中的错误报告
```

### 步骤 5：生成测试报告

汇总测试结果，生成报告：

**全部通过时：**

```markdown
## 冒烟测试报告

**平台:** <platform>
**模式:** <read-only | write>
**时间:** <timestamp>

### 结果摘要

| 指标 | 数值 |
|------|------|
| ✅ 通过 | <pass-count> |
| ❌ 失败 | <fail-count> |
| ⏭️ 跳过 | <skip-count> |

### 结论

**✅ 冒烟测试全部通过** — CLI 功能正常，无回归问题
```

**存在失败时：**

```markdown
## 冒烟测试报告

**平台:** <platform>
**模式:** <read-only | write>
**时间:** <timestamp>

### 结果摘要

| 指标 | 数值 |
|------|------|
| ✅ 通过 | <pass-count> |
| ❌ 失败 | <fail-count> |
| ⏭️ 跳过 | <skip-count> |

### 失败详情

| # | 命令 | 错误信息 | 严重程度 |
|---|------|----------|----------|
| 1 | <command> | <error-msg> | 🔴/🟠/🟡 |

### 结论

**❌ 冒烟测试存在失败** — 已创建 <n> 个 Issue 上报

### 已上报 Issue

- [<error-id>] <command> — <url>
```

## 使用示例

### 运行基本冒烟测试（GitHub 平台）

```bash
# 运行只读冒烟测试
bash scripts/smoke-test.sh --platform github

# 预期输出类似：
# [INFO] 冒烟测试开始 — 平台: github, 模式: read-only
# [PASS] gitflow-cli issue --help
# [PASS] gitflow-cli pr --help
# [PASS] gitflow-cli release --help
# [PASS] gitflow-cli issue list --limit 3
# [PASS] gitflow-cli pr list --limit 3
# [PASS] gitflow-cli label list
# ...
# [INFO] 测试完成
# PASS: 25 | FAIL: 0 | SKIP: 2
```

### 运行 GitLab 平台的冒烟测试

```bash
bash scripts/smoke-test.sh --platform gitlab --verbose
```

### 运行带写入操作的冒烟测试

```bash
# 注意：写入模式会实际创建资源，谨慎使用
bash scripts/smoke-test.sh --platform github --write
```

### 集成到 CI 流程

```bash
# 在 CI 中运行冒烟测试
bash scripts/smoke-test.sh --platform github 2>&1 | tee smoke-test-output.log

# 检查退出码
if [ $? -ne 0 ]; then
    echo "❌ 冒烟测试失败，停止流水线"
    exit 1
fi
```

### 失败后自动上报

```bash
# 运行冒烟测试
bash scripts/smoke-test.sh --platform github > /tmp/smoke-output.txt 2>&1
EXIT_CODE=$?

if [ $EXIT_CODE -ne 0 ]; then
    echo "冒烟测试失败，生成错误报告..."

    # 解析失败信息并写入 pending.json
    mkdir -p .cache/bug-reports
    cat > .cache/bug-reports/pending.json << EOF
{
    "error_id": "smoke-$(date +%s)",
    "command": "smoke-test",
    "platform": "github",
    "error_code": "SMOKE_TEST_FAILED",
    "error_message": "冒烟测试存在失败项，详见输出日志",
    "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF

    # 调用 autoreport-bug 工作流
    echo "使用 gitflow-autoreport-bug 技能处理 pending.json"
fi
```

## 注意事项

- 冒烟测试脚本（`scripts/smoke-test.sh`）优先使用本地构建版本（`target/release/gitflow-cli` 或 `target/debug/gitflow-cli`），其次是全局安装的 `gitflow-cli`
- 只读模式（`--read-only`）是安全的默认模式，不会修改远程数据
- 写入模式（`--write`）会实际创建资源，应谨慎使用，避免在测试环境产生垃圾数据
- 冒烟测试的退出码为 0 表示全部通过，非 0 表示存在失败
- 上报 Issue 前应先确认失败不是由认证问题或网络问题导致的临时性错误
- 如果多次运行都出现相同的失败，可能是 CLI 的实际 bug，应优先排查
- 对于间歇性失败的测试，可以标记为 flaky 并单独跟踪，不应每次都创建新 Issue
- 冒烟测试应定期运行（如每次 release 前、每日 CI），确保 CLI 功能持续正常
- 失败上报后应跟踪 Issue 状态，确保问题被修复后关闭
