# gitflow-cli 合并 ncgo-code-skills 与 gitcode-dev-workflow 设计

> 状态：已批准
> 日期：2026-07-03

## 背景

三个独立仓库承担着重叠的 AI 编程工作流职能：

- **gitflow-cli**（Rust CLI + 27 skills）：底层 CLI 引擎 + 完整 skill 集，支持 GitHub/GitLab/GitCode 三平台
- **gitcode-dev-workflow**（9 skills，纯 SKILL.md）：GitCode 专属，强项是**工作流编排**和**质量关卡**的严格执行（闸门、审计日志、TDD 强制）
- **ncgo-code-skills**（7 skills + hooks + bash provider）：纯 Bash 实现，强项是 brainstorm-from-issue、weekly-report、auto-report-bug hooks

gitcode-dev-workflow 的编排逻辑与 gitflow-cli 现有 `gitflow-workflow` / `gitflow-quality` 高度重叠（4 阶段闸门结构、Issue 审计日志、TDD 强制如出一辙）。ncgo-code-skills 的 auto-report-bug 实现比 gitflow-cli 现有版本更成熟（hooks 触发 + auth cache + JSON 验证）。

**决策：** 以 gitflow-cli 为唯一仓库，吸收两个源项目的增量能力，消除重复维护。

## 设计原则

1. **gitflow-cli 为唯一仓库** — 合并后所有 skill/hook 在 gitflow-cli 维护
2. **取增量不取存量** — 只吸收源项目中 gitflow-cli 没有或明显更好的部分
3. **保持实现习惯** — weekly-report 保持纯 bash，不重写为 Rust；workflow 保持 Markdown + CLI 指挥模式
4. **不破坏现有行为** — 变更对 gitflow-cli 现有 27 个 skill 无副作用

## 变更清单

### 变更 1：gitflow-workflow 强化

**文件：** `skills/gitflow-workflow/SKILL.md`

从 gitcode-dev-workflow 吸收三个增量点：

#### 1.1 合规检查清单

每个阶段结束时输出标准化 checklist，逐项打勾后才能进入下一阶段：

```
Phase N 合规检查:
  [ ] 步骤 1: <step> — <evidence>
  [ ] 步骤 2: <step> — <evidence>
  [ ] Issue 评论已发布: <comment_link>

  缺失项: <list or "无">
  是否允许进入下一阶段: [ ]
```

当前闸门校验只列"缺少则停止"的负面清单。合规清单改为**正面清单**，更适合 AI 代理执行。

#### 1.2 --body-file 强制规则

在"注意事项"章节追加规则：

> 长内容必须用 `--body-file`。阶段产出物（需求分析、任务清单、质量报告）通常 100-300 行，写入临时文件再传。短消息（PR 链接等）可继续用 `--body`。

受影响的方法：步骤 1.4、2.5、3.3、4.6 中所有 `gitflow-cli issue comment <number> --body "..."` 的长内容调用。

#### 1.3 强制执行规则头

在工作流顶部"启动条件"前增加：

```markdown
## 🚨 强制执行规则

以下规则不可跳过，任何一步不满足就不能进入下一步。
```

### 变更 2：gitflow-quality 强化

**文件：** `skills/gitflow-quality/SKILL.md`

从 gitcode-dev-quality 吸收增量：

#### 2.1 增加 Pre-commit 为第 6 步

在现有 5 步串行后追加第 6 步：

| # | 检查项 | 命令 | 通过标准 |
|---|--------|------|---------|
| 6 | pre-commit | `pre-commit run --all-files` 或读取 `.pre-commit-config.yaml` 检查配置 | 全部 hook 通过 |

fast-fail 兼容：pre-commit 在 static 之后运行。N/A 处理：无 `.pre-commit-config.yaml` 的项目标记为 `N/A`。

#### 2.2 Quality Report 更新

报告表格增加 pre-commit 行。

### 变更 3：gitflow-weekly-report 迁入

**新文件：** `skills/gitflow-weekly-report/SKILL.md`

从 ncgo-code-skills 迁入 `weekly-report` skill，保持纯 bash 实现。

#### 3.1 文件结构

```
skills/gitflow-weekly-report/
└── SKILL.md              # 从 ncgo-code-skills/weekly-report/SKILL.md 迁入（自包含，无外部脚本）
```

#### 3.2 命名适配

- skill name：`weekly-report` → `gitflow-weekly-report`
- description TRIGGER 描述追加 gitflow-cli 语境
- 脚本中的命令保持 `git` 原生命令（天然三平台兼容）

#### 3.3 安装方式

weekly-report 扫描多个 git 仓库，是典型的跨项目工具。SKILL.md 安装说明中应标注：

> 推荐使用用户级安装：`gitflow-cli skills install -g gitflow-weekly-report`
>
> 用户级安装后，无论从哪个项目目录调用，都能正常扫描指定的多个仓库路径。

gitflow-cli 现有安装系统已支持 `skills install -g`（安装到 `~/.claude/skills/`），不需要修改安装机制。在 SKILL.md 描述中明确推荐 `-g` 安装即可。

#### 3.4 不做的事

- 不替换为 gitflow-cli API 调用
- 不改变输出格式（plain-text, no tables）
- 不引入 platform provider 抽象

### 变更 4：gitflow-autoreport-bug 双源合并

**文件：** `skills/gitflow-autoreport-bug/SKILL.md`、`hooks/auto-report-bug.sh`

取两边优点合并为单一 skill + hook。

#### 4.1 能力取舍矩阵

| 能力 | 采用方 | 原因 |
|------|--------|------|
| Issue 创建 / 去重搜索 | gitflow-cli 版 | `gitflow-cli` CLI 天然三平台统一 |
| pending.json schema | gitflow-cli 版 | 已有完整字段定义（error_id / command / platform / error_code / error_message / timestamp） |
| hooks 触发机制 | ncgo-code 版 | Stop Hook 检测 pending.json 更可靠 |
| auth cache（24h TTL） | ncgo-code 版 | 避免每次上报都调用 `gitflow-cli auth status` |
| JSON 验证 + 容错 | ncgo-code 版 | 无效 JSON → rename .invalid + 告警 |
| failed.log 重试记录 | ncgo-code 版 | 认证失败时记录并保留 pending.json |

#### 4.2 合并后执行流程

```
Stop Hook (auto-report-bug.sh) 触发
    ↓
检测 .cache/bug-reports/pending.json 是否存在
    ↓ 是
读取 pending.json
    ↓
JSON 格式有效？ → 否 → rename .invalid + 告警 + 结束
    ↓ 是
auth cache 命中（24h 内检查过）？ → 是 → 跳过认证检查
    ↓ 否
gitflow-cli auth status --platform {platform}
    ↓ 失败
追加 failed.log + 保留 pending.json + 结束
    ↓ 成功
更新 auth cache timestamp
    ↓
gitflow-cli issue list --search "[auto-report] {command} {error_code}" --state all
    ↓ 已存在
去重命中 → 清理 pending.json + 结束
    ↓ 不存在
Claude 分析错误上下文 → 生成 Issue 标题/正文
    ↓
gitflow-cli issue create --title "..." --body "..." --label "auto-report"
    ↓ 成功
清理 pending.json
```

#### 4.3 auth cache 实现

沿用 ncgo-code 方案：`.cache/auth-cache/{platform}.ttl` 存储 Unix 时间戳。ttl 默认 86400（24h），可通过 pending.json 中可选字段 `auth_cache_ttl` 覆盖。

#### 4.4 pending.json schema

保持 gitflow-cli 现有字段，追加可选字段：

```json
{
  "id": "uuid",
  "command": "gitflow issue create",
  "platform": "github",
  "error_code": "401",
  "error_message": "Unauthorized",
  "timestamp": "2026-07-03T10:00:00Z",
  "auth_cache_ttl": 86400
}
```

`auth_cache_ttl` 可选，缺省 86400。其余字段为必填，缺失则视为无效 JSON。

### 变更 5：Hooks 注册

**新文件：** `hooks/auto-report-bug.sh`、`hooks/sync-readme-check.sh`

从 ncgo-code-skills 迁入 2 个 Stop Hook，注册到 `.claude/settings.json`。

#### 5.1 auto-report-bug.sh

```bash
#!/usr/bin/env bash
set -euo pipefail

PENDING_FILE=".cache/bug-reports/pending.json"

if [ -f "$PENDING_FILE" ]; then
    cat "$PENDING_FILE"
    echo ""
    echo "⚠️ Detected pending bug report. Running gitflow-autoreport-bug..."
fi
```

#### 5.2 sync-readme-check.sh

```bash
#!/usr/bin/env bash
set -euo pipefail

# 比较实际 skills/ 目录与 README 中 "Structure" 章节
# 不一致时输出提醒
```

#### 5.3 注册方式

在 `.claude/settings.json` 中追加：

```json
{
  "hooks": {
    "Stop": [
      "bash hooks/auto-report-bug.sh",
      "bash hooks/sync-readme-check.sh"
    ]
  }
}
```

#### 5.4 不迁入 auto-smoke-test.sh 的原因

ncgo-code 的 `auto-smoke-test.sh` 监控其 bash `_provider.sh` 脚本变更。gitflow-cli 没有 provider 抽象层（由 Rust core 替代），该 hook 不适用。

## 不做的事

- 不迁入 ncgo-code-skills 其余 6 个 skill（brainstorm-from-issue / check-status / issue-status / sync-readme / writing-plans-with-issue / auto-smoke-test）
- 不迁入 gitcode-dev-workflow 的 7 个非编排 skill（issue-create / issue-review / issue-triage / pr-create / pr-review / release-helper / repo-onboarding / security-check，这些 gateway-cli 自有版本）
- 不为 weekly-report 引入 gitflow-cli API
- 不删除或归档源仓库（保留现状，未来自然废弃）

## 文件变更清单

| 动作 | 文件路径 |
|------|---------|
| 修改 | `skills/gitflow-workflow/SKILL.md` |
| 修改 | `skills/gitflow-quality/SKILL.md` |
| 新增 | `skills/gitflow-weekly-report/SKILL.md` |
| 修改 | `skills/gitflow-autoreport-bug/SKILL.md` |
| 新增 | `hooks/auto-report-bug.sh` |
| 新增 | `hooks/sync-readme-check.sh` |
| 修改 | `.claude/settings.json`（追加 hooks 注册） |

## 风险与缓解

| 风险 | 缓解 |
|------|------|
| auto-report-bug 合并后行为变化可能遗漏原有场景 | SKILL.md 中明确列出能力取舍矩阵，review 时对照两边原文件 |
| hooks 增加 Stop Hook 延迟 | 两个 hook 都是轻量文件检测 + 简单 diff，耗时 <100ms |
| weekly-report 的 bash 与 gitflow-cli Rust 风格不一致 | weekly-report 是独立 skill，bash 是刻意选择（零依赖），不影响其他 skill |
| sync-readme-check.sh 的目录检测逻辑需适配 gitflow-cli 结构 | 实现时以 gitflow-cli 当前 skills/ 目录结构为准 |
