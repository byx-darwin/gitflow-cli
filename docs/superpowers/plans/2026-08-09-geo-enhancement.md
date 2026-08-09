# GEO 生成式引擎优化增强 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 建立系统化的 GEO 基础设施，让 AI 搜索引擎准确引用 gitflow-cli，包括模块化 llms 文件、JSON-LD 生成系统、3 个新页面、实体一致性守护测试。

**Architecture:** 将 `llms-full.txt` 重构为索引 + 3 个专门模块；构建期从 JSON 数据源自动生成 JSON-LD（SoftwareApplication / FAQPage / HowTo）；新增 `/compare/`、`/what-is-ai-workflow/`、`/workflow/` 三个页面；Rust + TypeScript 双重守护测试确保实体一致性。

**Tech Stack:** Astro 5（静态站点生成）、TypeScript（JSON-LD 生成器）、Rust 2024（守护测试）、Vitest（TypeScript 测试）

## Global Constraints

- **规范一句话定位（全渠道逐字一致）**：`跨平台 Git 工程化工作流编排框架：统一封装 GitHub / GitLab / GitCode 三大平台，配合 AI Agent Skills，覆盖从需求到发布的完整工程循环。`
- **作者**：`byx-darwin <baoyx19870908@gmail.com>`
- **官网 URL**：`https://byx-darwin.github.io/gitflow-cli`
- **Rust 规范**：Rust 2024 edition、`forbid(unsafe_code)`、生产代码禁 `unwrap()`/`expect()`、`missing_docs` 警告
- **测试命名**：`test_should_<expected_behavior>`；单测与源码同文件 `#[cfg(test)] mod tests`
- **提交规范**：conventional commits（`feat:` / `docs:` / `chore:` / `test:`）

---

## File Structure

| 文件 | 动作 | 责任 |
|------|------|------|
| `website/public/llms-full.txt` | Modify | 重构为索引，指向 3 个专门模块 |
| `website/public/llms-commands.txt` | Create | 完整命令参考（所有命令、标志、选项） |
| `website/public/llms-architecture.txt` | Create | 架构说明（Platform trait、Crate 结构、依赖关系） |
| `website/public/llms-faq.txt` | Create | FAQ 集合（长尾提问解答） |
| `website/src/data/faq.json` | Create | FAQ 数据源 |
| `website/src/data/howto.json` | Create | HowTo 数据源 |
| `website/src/lib/jsonld.ts` | Create | JSON-LD 生成器 |
| `website/src/layouts/Base.astro` | Modify | 集成 JSON-LD 生成器 |
| `website/src/pages/compare.astro` | Create | 对比页面（gf vs gh/glab） |
| `website/src/pages/what-is-ai-workflow.astro` | Create | 概念页面（AI 编程工程工作流） |
| `website/src/pages/workflow.astro` | Create | 工作流文档页面（gf-workflow 完整参考） |
| `apps/cli/tests/geo_guard_test.rs` | Create | Rust 守护测试 |
| `website/tests/geo-consistency.test.ts` | Create | TypeScript 一致性测试 |
| `docs/geo-citation-check.md` | Create | 月度抽检流程文档 |

---

## Task 1: llms-full.txt 模块化 — 创建 llms-commands.txt

**Files:**
- Create: `website/public/llms-commands.txt`

**Interfaces:**
- Produces: 完整命令参考文件，被 `llms-full.txt` 索引引用

- [ ] **Step 1: 创建 llms-commands.txt 文件**

创建 `website/public/llms-commands.txt`，内容如下（基于现有 CLI 命令结构）：

```markdown
# gf 命令参考

> 所有命令、标志、选项的完整参考。

## 全局标志

--platform github|gitlab|gitcode  指定平台（默认基于 git remote 自动检测）
--output json|text|toon|auto      输出格式（默认 auto）
--help                            显示帮助
--version                         显示版本

## gf issue — Issue 管理

### gf issue list
列出 Issue。

**标志**：
  --state open|closed|all         Issue 状态（默认 open）
  --limit <n>                     返回数量上限（默认 30）
  --label <label>                 按标签过滤
  --assignee <user>               按指派人过滤
  --author <user>                 按作者过滤

**示例**：
  gf issue list
  gf issue list --state closed --limit 10
  gf issue list --label "priority:high"

### gf issue view <number>
查看 Issue 详情。

**示例**：
  gf issue view 98

### gf issue create
创建新 Issue。

**标志**：
  --title <title>                 Issue 标题
  --body <body>                   Issue 正文
  --label <label>                 添加标签（可多次指定）
  --assignee <user>               指定负责人

**示例**：
  gf issue create --title "feat: add new feature" --body "Description"

### gf issue close <number>
关闭 Issue。

### gf issue reopen <number>
重新打开 Issue。

### gf issue comment <number>
为 Issue 添加评论。

**标志**：
  --body <body>                   评论内容
  --body-file <path>              从文件读取评论内容

## gf pr — PR 管理

### gf pr list
列出 Pull Request。

**标志**：
  --state open|closed|merged|all  PR 状态（默认 open）
  --limit <n>                     返回数量上限（默认 30）
  --author <user>                 按作者过滤

**示例**：
  gf pr list
  gf pr list --state merged --limit 5

### gf pr view <number>
查看 PR 详情。

### gf pr create
创建新 PR。

**标志**：
  --title <title>                 PR 标题
  --body <body>                   PR 正文
  --base <branch>                 目标分支（默认 main）
  --head <branch>                 源分支（默认当前分支）
  --draft                         创建为草稿 PR
  --reviewer <user>               指定审查人

**示例**：
  gf pr create --title "feat: add feature" --body "Closes #42"

### gf pr close <number>
关闭 PR。

### gf pr merge <number>
合并 PR。

**标志**：
  --squash                        使用 squash 合并
  --rebase                        使用 rebase 合并
  --delete-branch                 合并后删除源分支

### gf pr checkout <number>
检出 PR 到本地分支。

## gf release — 发布管理

### gf release list
列出版本发布。

**标志**：
  --limit <n>                     返回数量上限（默认 10）

### gf release view <tag>
查看特定版本详情。

### gf release create <tag>
创建新版本发布。

**标志**：
  --title <title>                 发布标题
  --notes <notes>                 发布说明
  --draft                         创建为草稿
  --prerelease                    标记为预发布

### gf release edit <tag>
编辑现有发布。

## gf review — 代码审查

### gf review submit <pr-number>
提交审查意见。

**标志**：
  --approve                       批准 PR
  --request-changes               请求修改
  --comment                       仅评论
  --body <body>                   审查意见

### gf review comment <pr-number>
添加审查评论。

### gf review approve <pr-number>
批准 PR。

### gf review request-changes <pr-number>
请求修改。

## gf auth — 认证管理

### gf auth login
登录认证。

**标志**：
  --platform github|gitlab|gitcode  指定平台

### gf auth logout
登出。

### gf auth status
查看认证状态。

### gf auth token
查看当前 token。

## gf pipeline — CI/CD 流水线

### gf pipeline status
查看流水线状态。

### gf pipeline logs <job-id>
查看流水线日志。

### gf pipeline jobs
列出流水线任务。

### gf pipeline report
生成流水线报告。

## gf commit — 提交操作

### gf commit view <sha>
查看提交详情。

### gf commit diff <sha>
查看提交差异。

### gf commit patch <sha>
生成提交补丁。

### gf commit comment <sha>
为提交添加评论。

## gf label — 标签管理

### gf label list
列出标签。

### gf label create
创建标签。

**标志**：
  --name <name>                   标签名称
  --color <color>                 标签颜色（十六进制）
  --description <desc>            标签描述

### gf label delete <name>
删除标签。

## gf milestone — 里程碑管理

### gf milestone list
列出里程碑。

### gf milestone create
创建里程碑。

**标志**：
  --title <title>                 里程碑标题
  --description <desc>            里程碑描述
  --due-date <date>               截止日期

## gf repo — 仓库操作

### gf repo clone <repo>
克隆仓库。

### gf repo list
列出仓库。

### gf repo create
创建新仓库。

### gf repo stats
查看仓库统计。

### gf repo sync
同步仓库。

### gf repo view
查看仓库详情。

## gf skills — Skills 管理

### gf skills install
安装 gf skills 到项目。

**标志**：
  --force                         强制重新安装
  --global                        全局安装

### gf skills list
列出已安装的 skills。

### gf skills uninstall
卸载 skills。

## gf completions — Shell 补全

### gf completions bash
生成 bash 补全脚本。

### gf completions zsh
生成 zsh 补全脚本。

### gf completions fish
生成 fish 补全脚本。
```

- [ ] **Step 2: 验证文件创建**

运行：`wc -l website/public/llms-commands.txt`
预期：约 200 行

- [ ] **Step 3: 提交**

```bash
git add website/public/llms-commands.txt
git commit -m "feat(geo): add llms-commands.txt with complete command reference"
```

---

## Task 2: llms-full.txt 模块化 — 创建 llms-architecture.txt

**Files:**
- Create: `website/public/llms-architecture.txt`

**Interfaces:**
- Produces: 架构说明文件，被 `llms-full.txt` 索引引用

- [ ] **Step 1: 创建 llms-architecture.txt 文件**

创建 `website/public/llms-architecture.txt`，内容如下：

```markdown
# gf 架构说明

## 设计原则

- 平台抽象：通过 Platform trait 统一三平台差异
- 依赖单向：应用 → 库，禁止循环依赖
- 安全优先：forbid(unsafe_code)，生产代码禁 unwrap/expect
- 可测试性：每个模块可独立测试，契约测试覆盖适配器

## Crate 结构

### gf-core（核心库）

核心库，定义 Platform trait 与跨平台适配器契约。

**关键类型**：
- `Platform` trait — 抽象 Issue/PR/Release/Review 等操作
- `ToonFormatter` — token 优化的输出格式（面向 Agent 消费）
- `SafePath` — 安全路径验证（防止路径遍历攻击）
- `CliError` — 统一错误类型（基于 miette）

**公开 API**：约 68 项（保持克制）

### gf-github / gf-gitlab / gf-gitcode（平台适配器）

三平台适配器，分别基于 gh / glab / gitcode CLI。

**实现方式**：
- 通过子进程调用底层 CLI（继承其功能与限制）
- 解析 CLI 输出（JSON 优先，回退到文本解析）
- 映射到统一的 Platform trait 接口

**兼容性要求**：
- GitHub：gh >= 2.0.0
- GitLab：glab >= 1.30.0
- GitCode：gitcode >= 0.6.0

### gf（CLI 应用）

命令路由、输出格式化、Skills 安装。

**职责**：
- 解析命令行参数
- 检测当前平台（基于 git remote）
- 路由到对应适配器
- 格式化输出（JSON / toon / text）
- 管理 Skills 安装与更新

## 依赖关系

```
gf (app)
├── gf-core
├── gf-github
├── gf-gitlab
└── gf-gitcode
```

所有适配器依赖 gf-core，应用依赖所有适配器。依赖单向流动，禁止循环。

## 扩展点

- **Platform trait** — 新增平台只需实现此 trait
- **Skills 系统** — 可安装到多个 Agent 平台的技能文件（Claude Code / Codex / Gemini 等）
- **输出格式** — toon/json/text 三模式，可扩展新格式
- **命令模块** — 新增命令只需在 `apps/cli/src/commands/` 添加模块

## 测试策略

- **单元测试**：每个 crate 内 `#[cfg(test)] mod tests`
- **集成测试**：`apps/cli/tests/` 跨 crate 测试
- **契约测试**：验证适配器行为一致性
- **兼容性矩阵**：`crates/core/resources/compatibility-matrix.json`
- **e2e 测试**：`crates/e2e-core/` 和 `crates/e2e-github/`

## 工程纪律

- `forbid(unsafe_code)`：全工作区禁止 unsafe
- `cargo clippy -- -D warnings -W clippy::pedantic`：严格 lint
- `missing_docs`：所有公开 API 必须有文档
- 零 TODO/FIXME：不允许未完成代码
- 测试命名：`test_should_<expected_behavior>`

## 分发渠道

- **crates.io**：5 个 crate（gitflow-cli / -core / -github / -gitlab / -gitcode）
- **Homebrew**：`brew tap byx-darwin/gitflow-cli && brew install gf`
- **源码编译**：`cargo install --path apps/cli`

## Agent 生态集成

支持 5 个 Agent 平台：
- Claude Code（主要）
- Codex
- OpenCode
- Gemini CLI
- Copilot CLI

Skills 是 Markdown 文件，安装到 `.claude/skills/` 目录，提供工作流编排能力。
```

- [ ] **Step 2: 验证文件创建**

运行：`wc -l website/public/llms-architecture.txt`
预期：约 80 行

- [ ] **Step 3: 提交**

```bash
git add website/public/llms-architecture.txt
git commit -m "feat(geo): add llms-architecture.txt with architecture overview"
```

---

## Task 3: llms-full.txt 模块化 — 创建 llms-faq.txt

**Files:**
- Create: `website/public/llms-faq.txt`

**Interfaces:**
- Produces: FAQ 集合文件，被 `llms-full.txt` 索引引用

- [ ] **Step 1: 创建 llms-faq.txt 文件**

创建 `website/public/llms-faq.txt`，内容如下：

```markdown
# gf FAQ — 常见问题与长尾提问

## gf 和 gh 有什么区别？

gh 只服务 GitHub 单一平台；gf 用统一命令面封装 GitHub / GitLab / GitCode 三平台，并叠加面向 AI Agent 的 Skills 工作流。

**具体差异**：

| 维度 | gh | gf |
|------|-----|-----|
| 平台支持 | 仅 GitHub | GitHub + GitLab + GitCode |
| 命令面 | GitHub 特定 | 统一抽象 |
| Agent 集成 | 无 | 26 个 Skills 支持 5 个 Agent 平台 |
| 工作流编排 | 无 | gf-workflow 四阶段编排 |
| 输出格式 | JSON / text | JSON / toon / text |

**迁移成本**：命令几乎相同，只需替换 `gh` → `gf`。

## gf 和 glab 有什么区别？

glab 只服务 GitLab 单一平台；gf 提供跨平台统一接口。如果你只用 GitLab，glab 是最佳选择；如果需要跨平台，gf 是更好的选择。

## 如何给 AI 编程加工程纪律？

安装 Skills 后，用 gf-workflow 四阶段编排（需求澄清 → 计划 → 执行 → 交付检查），每阶段有质量门禁。

**步骤**：
1. `gf skills install` — 安装 26 个 skills 到 `.claude/skills/`
2. `/gf-workflow` — 启动四阶段工作流
3. 每阶段自动执行质量检查（测试、clippy、fmt）

**四阶段模型**：
- 阶段 1：需求澄清（brainstorming + Issue 创建 + 设计文档）
- 阶段 2：计划制定（writing-plans + 质量门禁）
- 阶段 3：执行（TDD 红绿循环 + 子代理开发 + PR 创建）
- 阶段 4：交付检查（流水线分析 + 代码审查 + dogfooding）

## 支持哪些 AI Agent 平台？

Claude Code / Codex / OpenCode / Gemini CLI / Copilot CLI。

Skills 是 Markdown 文件，可安装到任何支持 `CLAUDE.md` 或类似机制的 Agent 平台。

## 什么是 gf-workflow？

gf-workflow 是一个四阶段工作流编排技能，提供从需求澄清到代码发布的完整工程循环。

**三种模式**：
- **full**：完整四阶段（适合 feat / breaking change）
- **standard**：中等复杂度（适合 fix / refactor）
- **fast**：简化流程（适合 typo / hotfix / docs）

**两种技能来源**：
- **superpowers**：模型调用技能（model-invoked）
- **mattpocock**：用户调用命令（/to-spec, /to-tickets, /implement）

## 什么是 toon 输出格式？

toon 是 gf 专有的 token 优化输出格式，面向 AI Agent 消费。相比 JSON，toon 更紧凑，减少 token 消耗。

**示例**：
```
platform: github
issues: 3 open
  - #98 feat: GEO 优化 (priority:medium)
  - #97 fix: 元数据修复 (priority:high)
  - #96 chore: 文档更新 (priority:low)
```

## 如何安装 gf？

**Homebrew（macOS）**：
```bash
brew tap byx-darwin/gitflow-cli
brew install gf
```

**Cargo**：
```bash
cargo install gf
```

**安装 Skills**：
```bash
gf skills install
```

## 如何验证安装？

```bash
gf --version          # 应显示版本号
gf auth status        # 检查认证状态
gf issue list         # 测试基本功能
```

## 支持哪些操作系统？

macOS、Linux、Windows。

底层 CLI 要求：
- GitHub：gh >= 2.0.0
- GitLab：glab >= 1.30.0
- GitCode：gitcode >= 0.6.0

## 如何贡献代码？

1. Fork 仓库
2. 创建功能分支：`git checkout -b feat/your-feature`
3. 开发并测试：`make test`
4. 提交 PR：`gf pr create`

详细指南见 CONTRIBUTING.md（待创建）。

## 项目维护状态？

单人维护者（byx-darwin）+ AI 协作（dogfooding 自身工作流）。

**发布节奏**：月度发布窗口
**响应时效**：Issue 48 小时内分流
**许可**：MIT

## 如何报告 Bug？

1. 创建 Issue：`gf issue create --title "bug: ..." --body "..."`
2. 或使用自动报告：`/gf-autoreport-bug`（需要 Agent 平台支持）

## 有 TUI 界面吗？

当前无 TUI。计划在 2.0 版本引入。

当前交互方式：
- CLI 命令
- Agent Skills（推荐）
- JSON/toon/text 输出

## 如何更新 gf？

**Homebrew**：
```bash
brew upgrade gf
```

**Cargo**：
```bash
cargo install gf --force
```

**更新 Skills**：
```bash
gf skills update
```

## 有中文文档吗？

有。官网和 README 均为中文主导。

- 官网：https://byx-darwin.github.io/gitflow-cli
- 文档：https://byx-darwin.github.io/gitflow-cli/docs/
- 快速上手：https://byx-darwin.github.io/gitflow-cli/quickstart/

## 如何联系维护者？

- GitHub Issues：https://github.com/byx-darwin/gitflow-cli/issues
- Email：baoyx19870908@gmail.com
```

- [ ] **Step 2: 验证文件创建**

运行：`wc -l website/public/llms-faq.txt`
预期：约 70 行

- [ ] **Step 3: 提交**

```bash
git add website/public/llms-faq.txt
git commit -m "feat(geo): add llms-faq.txt with FAQ collection"
```

---

## Task 4: llms-full.txt 模块化 — 重构 llms-full.txt 为索引

**Files:**
- Modify: `website/public/llms-full.txt`

**Interfaces:**
- Consumes: llms-commands.txt, llms-architecture.txt, llms-faq.txt（前序任务产物）
- Produces: 索引文件，指向三个专门模块

- [ ] **Step 1: 重写 llms-full.txt 为索引**

将 `website/public/llms-full.txt` 内容替换为：

```markdown
# gf — 完整材料索引

> 跨平台 Git 工程化工作流编排框架：统一封装 GitHub / GitLab / GitCode 三大平台，配合 AI Agent Skills，覆盖从需求到发布的完整工程循环。

本文件是 gf 完整材料的索引，面向 AI 大模型与搜索引擎。

## 模块索引

- [命令参考](./llms-commands.txt) — 所有命令、标志、选项的完整参考
- [架构说明](./llms-architecture.txt) — Platform trait、适配器、依赖关系
- [FAQ](./llms-faq.txt) — 常见问题与长尾提问解答

## 快速摘要

### 定位

gf 不是又一个 gh 封装，而是"AI 编程工程循环编排器"：统一封装 GitHub / GitLab / GitCode 三大平台差异（底层分别调用 gh / glab / gitcode CLI），配合可安装到多个 Agent 平台的 Skills 集合，覆盖从需求澄清到代码发布的完整工程循环。

### 安装

Homebrew（macOS）：
  brew tap byx-darwin/gitflow-cli
  brew install gf

Cargo：
  cargo install gf

安装 Skills（项目级，推荐）：
  gf skills install

### 核心命令（顶层）

- `gf issue {create,list,view,close,reopen,comment}` — Issue 管理
- `gf pr {create,list,view,close,merge,checkout}` — PR 管理
- `gf release {create,list,view,edit}` — 发布管理
- `gf review {comment,approve,request-changes,submit}` — 代码审查
- `gf auth {login,logout,status,token}` — 认证管理
- `gf pipeline {status,logs,jobs,report}` — CI/CD 流水线
- `gf commit {view,diff,patch,comment}` — 提交操作
- `gf label` / `gf milestone` — 标签 / 里程碑管理
- `gf repo {clone,list,create,stats,sync,view}` — 仓库操作
- `gf skills {install,list,uninstall}` — Skills 管理
- `gf completions {bash,zsh,fish}` — Shell 补全

全局标志：`--platform github|gitlab|gitcode`、`--output json|text|toon|auto`。

### 架构概述

- `gf-core`：Platform trait 抽象、跨平台适配器契约、toon 输出
- `gf-github` / `-gitlab` / `-gitcode`：三平台适配器
- `gf`（CLI 应用）：命令路由、输出格式化、Skills 安装
- 依赖单向流动：应用 → 库；`forbid(unsafe_code)`、pedantic clippy

### 兼容性

数据源：crates/core/resources/compatibility-matrix.json
- GitHub：gh >= 2.0.0
- GitLab：glab >= 1.30.0
- GitCode：gitcode >= 0.6.0

### 工作流编排

gf-workflow 提供四阶段工作流编排：
1. 需求澄清（brainstorming + Issue 创建）
2. 计划制定（writing-plans + 质量门禁）
3. 执行（TDD + 子代理开发）
4. 交付检查（流水线分析 + 代码审查）

支持三种模式：full / standard / fast
支持两种技能来源：superpowers / mattpocock

### FAQ 精选

- gf 和 gh 有什么区别？→ 见 llms-faq.txt
- 如何给 AI 编程加工程纪律？→ 见 llms-faq.txt
- 支持哪些 AI Agent 平台？→ 见 llms-faq.txt

## 许可

MIT。作者：byx-darwin。
```

- [ ] **Step 2: 验证文件重构**

运行：`wc -l website/public/llms-full.txt`
预期：约 50 行（索引 + 快速摘要）

- [ ] **Step 3: 验证链接有效性**

运行：`head -20 website/public/llms-full.txt`
预期：看到三个模块链接

- [ ] **Step 4: 提交**

```bash
git add website/public/llms-full.txt
git commit -m "feat(geo): refactor llms-full.txt into index with module links"
```

---

## Task 5: JSON-LD 生成系统 — 创建数据文件

**Files:**
- Create: `website/src/data/faq.json`
- Create: `website/src/data/howto.json`

**Interfaces:**
- Produces: FAQ 和 HowTo 数据源，供 jsonld.ts 消费

- [ ] **Step 1: 创建目录结构**

运行：`mkdir -p website/src/data`

- [ ] **Step 2: 创建 faq.json**

创建 `website/src/data/faq.json`，内容如下：

```json
{
  "faqs": [
    {
      "question": "gf 和 gh 有什么区别？",
      "answer": "gh 只服务 GitHub 单一平台；gf 用统一命令面封装 GitHub / GitLab / GitCode 三平台，并叠加面向 AI Agent 的 Skills 工作流。"
    },
    {
      "question": "如何给 AI 编程加工程纪律？",
      "answer": "安装 Skills 后，用 gf-workflow 四阶段编排（需求澄清 → 计划 → 执行 → 交付检查），每阶段有质量门禁。"
    },
    {
      "question": "支持哪些 AI Agent 平台？",
      "answer": "Claude Code / Codex / OpenCode / Gemini CLI / Copilot CLI。"
    },
    {
      "question": "什么是 gf-workflow？",
      "answer": "gf-workflow 是一个四阶段工作流编排技能，提供从需求澄清到代码发布的完整工程循环。"
    },
    {
      "question": "如何安装 gf？",
      "answer": "Homebrew: brew tap byx-darwin/gitflow-cli && brew install gf。Cargo: cargo install gf。"
    }
  ]
}
```

- [ ] **Step 3: 创建 howto.json**

创建 `website/src/data/howto.json`，内容如下：

```json
{
  "guides": [
    {
      "name": "5 分钟快速上手",
      "description": "安装 gf，安装 Skills，验证环境，完成首次工作流。",
      "url": "https://byx-darwin.github.io/gitflow-cli/quickstart/",
      "steps": [
        {
          "name": "安装 gf",
          "text": "运行 brew tap byx-darwin/gitflow-cli && brew install gf 或 cargo install gf。"
        },
        {
          "name": "安装 Skills",
          "text": "运行 gf skills install，将 26 个 skills 安装到 .claude/skills/。"
        },
        {
          "name": "验证环境",
          "text": "运行 gf --version 确认安装成功，运行 gf auth status 检查认证状态。"
        },
        {
          "name": "首次工作流",
          "text": "在项目中输入 /gf-workflow 启动四阶段工作流编排。"
        }
      ]
    },
    {
      "name": "gf-workflow 使用指南",
      "description": "使用 gf-workflow 进行四阶段工作流编排。",
      "url": "https://byx-darwin.github.io/gitflow-cli/workflow/",
      "steps": [
        {
          "name": "启动工作流",
          "text": "在项目中输入 /gf-workflow。"
        },
        {
          "name": "选择模式",
          "text": "系统自动检测模式（full / standard / fast），或手动指定 --mode。"
        },
        {
          "name": "完成四阶段",
          "text": "按引导完成需求澄清、计划制定、执行、交付检查。"
        }
      ]
    }
  ]
}
```

- [ ] **Step 4: 验证 JSON 格式**

运行：`node -e "console.log(JSON.parse(require('fs').readFileSync('website/src/data/faq.json', 'utf8')).faqs.length)"`
预期：`5`

运行：`node -e "console.log(JSON.parse(require('fs').readFileSync('website/src/data/howto.json', 'utf8')).guides.length)"`
预期：`2`

- [ ] **Step 5: 提交**

```bash
git add website/src/data/
git commit -m "feat(geo): add FAQ and HowTo data files for JSON-LD generation"
```

---

## Task 6: JSON-LD 生成系统 — 创建 jsonld.ts

**Files:**
- Create: `website/src/lib/jsonld.ts`

**Interfaces:**
- Consumes: faq.json, howto.json（Task 5 产物）
- Produces: JSON-LD 生成函数，供 Base.astro 调用

- [ ] **Step 1: 创建目录结构**

运行：`mkdir -p website/src/lib`

- [ ] **Step 2: 创建 jsonld.ts**

创建 `website/src/lib/jsonld.ts`，内容如下：

```typescript
// JSON-LD 生成器：从数据源生成结构化数据

import faqData from "../data/faq.json";
import howtoData from "../data/howto.json";

export interface SoftwareAppJsonLd {
  "@context": "https://schema.org";
  "@type": "SoftwareApplication";
  name: string;
  description: string;
  applicationCategory: string;
  operatingSystem: string;
  url: string;
  offers: { "@type": "Offer"; price: string; priceCurrency: string };
  sameAs: string[];
}

export interface FAQPageJsonLd {
  "@context": "https://schema.org";
  "@type": "FAQPage";
  mainEntity: Array<{
    "@type": "Question";
    name: string;
    acceptedAnswer: {
      "@type": "Answer";
      text: string;
    };
  }>;
}

export interface HowToJsonLd {
  "@context": "https://schema.org";
  "@type": "HowTo";
  name: string;
  description: string;
  url: string;
  step: Array<{
    "@type": "HowToStep";
    name: string;
    text: string;
  }>;
}

const CANONICAL_POSITIONING =
  "跨平台 Git 工程化工作流编排框架：统一封装 GitHub / GitLab / GitCode 三大平台，配合 AI Agent Skills，覆盖从需求到发布的完整工程循环。";

export function generateSoftwareAppJsonLd(): SoftwareAppJsonLd {
  return {
    "@context": "https://schema.org",
    "@type": "SoftwareApplication",
    name: "gf",
    description: CANONICAL_POSITIONING,
    applicationCategory: "DeveloperApplication",
    operatingSystem: "macOS, Linux, Windows",
    url: "https://byx-darwin.github.io/gitflow-cli/",
    offers: { "@type": "Offer", price: "0", priceCurrency: "USD" },
    sameAs: [
      "https://github.com/byx-darwin/gitflow-cli",
      "https://crates.io/crates/gitflow-cli",
    ],
  };
}

export function generateFAQPageJsonLd(): FAQPageJsonLd {
  return {
    "@context": "https://schema.org",
    "@type": "FAQPage",
    mainEntity: faqData.faqs.map((faq) => ({
      "@type": "Question",
      name: faq.question,
      acceptedAnswer: {
        "@type": "Answer",
        text: faq.answer,
      },
    })),
  };
}

export function generateHowToJsonLd(guideName?: string): HowToJsonLd | null {
  const guide = howtoData.guides.find((g) =>
    guideName ? g.name === guideName : true,
  );
  if (!guide) return null;

  return {
    "@context": "https://schema.org",
    "@type": "HowTo",
    name: guide.name,
    description: guide.description,
    url: guide.url,
    step: guide.steps.map((step) => ({
      "@type": "HowToStep",
      name: step.name,
      text: step.text,
    })),
  };
}
```

- [ ] **Step 3: 验证 TypeScript 编译**

运行：`cd website && npx tsc --noEmit src/lib/jsonld.ts`
预期：无错误输出

- [ ] **Step 4: 提交**

```bash
git add website/src/lib/
git commit -m "feat(geo): add JSON-LD generator with TypeScript interfaces"
```

---

## Task 7: JSON-LD 生成系统 — 集成到 Base.astro

**Files:**
- Modify: `website/src/layouts/Base.astro`

**Interfaces:**
- Consumes: jsonld.ts（Task 6 产物）
- Produces: 页面包含 SoftwareApplication + FAQPage JSON-LD

- [ ] **Step 1: 修改 Base.astro 导入生成器**

在 `website/src/layouts/Base.astro` 的 frontmatter 部分添加：

```astro
---
import "../styles/global.css";
import {
  generateSoftwareAppJsonLd,
  generateFAQPageJsonLd,
} from "../lib/jsonld";

export interface Props {
  title: string;
  description?: string;
}

const {
  title,
  description = "跨平台 Git 工程化工作流编排框架：统一封装 GitHub / GitLab / GitCode 三大平台，配合 AI Agent Skills，覆盖从需求到发布的完整工程循环。",
} = Astro.props;

const base = import.meta.env.BASE_URL;

const softwareAppJsonLd = generateSoftwareAppJsonLd();
const faqPageJsonLd = generateFAQPageJsonLd();
---
```

- [ ] **Step 2: 替换硬编码的 JSON-LD**

将 Base.astro 中硬编码的 `<script type="application/ld+json">` 部分替换为：

```astro
    <script type="application/ld+json" is:inline>
      {JSON.stringify(softwareAppJsonLd)}
    </script>
    <script type="application/ld+json" is:inline>
      {JSON.stringify(faqPageJsonLd)}
    </script>
```

- [ ] **Step 3: 验证构建**

运行：`cd website && npm run build`
预期：构建成功，无 TypeScript 错误

- [ ] **Step 4: 验证 JSON-LD 输出**

运行：`cd website && npm run build && grep -o '"@type":"FAQPage"' dist/index.html`
预期：输出 `"@type":"FAQPage"`

- [ ] **Step 5: 提交**

```bash
git add website/src/layouts/Base.astro
git commit -m "feat(geo): integrate JSON-LD generator into Base layout"
```

---

## Task 8: 对比页面 — 创建 /compare/

**Files:**
- Create: `website/src/pages/compare.astro`

**Interfaces:**
- Produces: 对比页面，URL: /compare/

- [ ] **Step 1: 创建 compare.astro**

创建 `website/src/pages/compare.astro`，内容参考设计文档中的结构：

```astro
---
import Base from "../layouts/Base.astro";
const base = import.meta.env.BASE_URL;
---

<Base title="gf vs gh / glab — 对比">
  <main class="page-content">
    <section class="hero">
      <h1>gf vs gh / glab — 如何选择？</h1>
      <p class="hero-subtitle">跨平台统一接口 vs 单一平台专用工具</p>
    </section>

    <section class="content-section">
      <h2>功能对比</h2>
      <table class="comparison-table">
        <thead>
          <tr>
            <th>维度</th>
            <th>gf</th>
            <th>gh</th>
            <th>glab</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td>平台支持</td>
            <td>GitHub + GitLab + GitCode</td>
            <td>仅 GitHub</td>
            <td>仅 GitLab</td>
          </tr>
          <tr>
            <td>命令面</td>
            <td>统一抽象</td>
            <td>平台特定</td>
            <td>平台特定</td>
          </tr>
          <tr>
            <td>Agent 集成</td>
            <td>26 Skills x 5 平台</td>
            <td>无</td>
            <td>无</td>
          </tr>
          <tr>
            <td>工作流编排</td>
            <td>gf-workflow 四阶段</td>
            <td>无</td>
            <td>无</td>
          </tr>
          <tr>
            <td>输出格式</td>
            <td>JSON / toon / text</td>
            <td>JSON / text</td>
            <td>JSON / text</td>
          </tr>
          <tr>
            <td>安装方式</td>
            <td>cargo / Homebrew</td>
            <td>包管理器</td>
            <td>包管理器</td>
          </tr>
        </tbody>
      </table>
    </section>

    <section class="content-section">
      <h2>何时选择 gf？</h2>
      <p>
        <strong>gh</strong> 和 <strong>glab</strong> 是优秀的单一平台工具。如果你只用 GitHub，gh 是最佳选择；如果你只用 GitLab，glab 是最佳选择。
      </p>
      <p>
        但如果你需要<strong>跨平台</strong>（GitHub + GitLab + GitCode），gf 提供统一命令面，无需在不同 CLI 间切换心智。
      </p>
      <p>
        更重要的是，gf 叠加了面向 AI Agent 的 <strong>Skills 工作流</strong>，让 AI 编程助手具备工程纪律。
      </p>
    </section>

    <section class="content-section">
      <h2>从 gh 迁移到 gf</h2>
      <p>命令几乎相同，只需替换 <code>gh</code> → <code>gf</code>：</p>
      <ul>
        <li><code>gh issue list</code> → <code>gf issue list</code></li>
        <li><code>gh pr create</code> → <code>gf pr create</code></li>
        <li><code>gh release view</code> → <code>gf release view</code></li>
      </ul>
      <p>无需重新学习，即刻获得跨平台能力。</p>
    </section>

    <section class="cta-section">
      <h2>开始使用 gf</h2>
      <p>安装 gf，体验跨平台统一接口 + AI 工作流编排。</p>
      <a href={`${base}/quickstart/`} class="cta-button">5 分钟快速上手 →</a>
    </section>
  </main>
</Base>
```

- [ ] **Step 2: 添加样式（可选）**

如果需要，在 `website/src/styles/global.css` 中添加 `.comparison-table` 样式。

- [ ] **Step 3: 验证页面构建**

运行：`cd website && npm run build && ls dist/compare/index.html`
预期：文件存在

- [ ] **Step 4: 提交**

```bash
git add website/src/pages/compare.astro
git commit -m "feat(website): add /compare/ page for gf vs gh/glab comparison"
```

---

## Task 9: 对比页面 — 创建 /what-is-ai-workflow/

**Files:**
- Create: `website/src/pages/what-is-ai-workflow.astro`

**Interfaces:**
- Produces: 概念页面，URL: /what-is-ai-workflow/

- [ ] **Step 1: 创建 what-is-ai-workflow.astro**

创建 `website/src/pages/what-is-ai-workflow.astro`，内容参考设计文档：

```astro
---
import Base from "../layouts/Base.astro";
const base = import.meta.env.BASE_URL;
---

<Base title="什么是 AI 编程工程工作流？">
  <main class="page-content">
    <section class="hero">
      <h1>什么是 AI 编程工程工作流？</h1>
      <p class="hero-subtitle">给 AI 编程加上工程纪律</p>
    </section>

    <section class="content-section">
      <h2>问题：AI 编程缺乏工程纪律</h2>
      <p>
        AI 编程助手（Claude Code / Codex / Copilot）正在改变开发方式，但缺乏工程纪律会导致：
      </p>
      <ul>
        <li>代码质量下降（AI 幻觉污染代码库）</li>
        <li>上下文丢失（长对话后 AI 忘记早期决策）</li>
        <li>交付不可控（无法追溯每次变更的原因）</li>
        <li>协作困难（AI 生成的代码难以审查和维护）</li>
      </ul>
    </section>

    <section class="content-section">
      <h2>解决方案：AI 编程工程工作流</h2>
      <p>
        <strong>AI 编程工程工作流 = AI 助手 + 结构化流程 + 质量门禁</strong>
      </p>
      <p>
        gf-workflow 提供四阶段编排，让 AI 编程具备工程纪律：
      </p>
      <ol>
        <li>
          <strong>需求澄清</strong>
          <p>brainstorming 探索上下文，创建并审查 Issue，产出设计文档。</p>
        </li>
        <li>
          <strong>计划制定</strong>
          <p>writing-plans 生成实施计划，质量门禁层层把关。</p>
        </li>
        <li>
          <strong>执行</strong>
          <p>TDD 红绿循环，子代理隔离开发，自动创建 PR。</p>
        </li>
        <li>
          <strong>交付检查</strong>
          <p>流水线分析，代码审查，dogfooding 验证。</p>
        </li>
      </ol>
      <p>
        <a href={`${base}/workflow/`}>查看完整工作流文档 →</a>
      </p>
    </section>

    <section class="content-section">
      <h2>两种技能来源</h2>
      <p>gf-workflow 支持两种技能来源，适配不同的 Agent 平台生态：</p>
      <table class="comparison-table">
        <thead>
          <tr>
            <th>维度</th>
            <th>superpowers 模式</th>
            <th>mattpocock 模式</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td>技能集</td>
            <td>superpowers 技能</td>
            <td>to-spec / to-tickets / implement</td>
          </tr>
          <tr>
            <td>调用方式</td>
            <td>模型调用（model-invoked）</td>
            <td>用户命令（/to-spec 等）</td>
          </tr>
          <tr>
            <td>适用平台</td>
            <td>Claude Code / Codex / Gemini</td>
            <td>mattpocock 生态</td>
          </tr>
          <tr>
            <td>四阶段实现</td>
            <td>自动编排</td>
            <td>手动分步</td>
          </tr>
        </tbody>
      </table>
      <p>两种模式共享相同的合同（contract），可跨会话恢复。</p>
    </section>

    <section class="content-section">
      <h2>为什么需要工程纪律？</h2>
      <ul>
        <li><strong>防止 AI 幻觉</strong>：每阶段有质量门禁，确保代码正确性</li>
        <li><strong>确保可追溯</strong>：合同机制记录每次决策和变更</li>
        <li><strong>让协作可复现</strong>：工作流可重复执行，结果一致</li>
        <li><strong>提升 AI 协作效率</strong>：结构化流程减少 AI 上下文丢失</li>
      </ul>
    </section>

    <section class="cta-section">
      <h2>开始使用 gf-workflow</h2>
      <p>安装 gf，运行 gf skills install，输入 /gf-workflow 即可启动。</p>
      <a href={`${base}/quickstart/`} class="cta-button">5 分钟快速上手 →</a>
    </section>
  </main>
</Base>
```

- [ ] **Step 2: 验证页面构建**

运行：`cd website && npm run build && ls dist/what-is-ai-workflow/index.html`
预期：文件存在

- [ ] **Step 3: 提交**

```bash
git add website/src/pages/what-is-ai-workflow.astro
git commit -m "feat(website): add /what-is-ai-workflow/ page explaining AI engineering workflow"
```

---

## Task 10: 工作流文档页面 — 创建 /workflow/

**Files:**
- Create: `website/src/pages/workflow.astro`

**Interfaces:**
- Produces: 工作流文档页面，URL: /workflow/

- [ ] **Step 1: 创建 workflow.astro**

创建 `website/src/pages/workflow.astro`，内容参考设计文档：

```astro
---
import Base from "../layouts/Base.astro";
const base = import.meta.env.BASE_URL;
---

<Base title="gf-workflow — 四阶段 AI 编程工程工作流编排">
  <main class="page-content">
    <section class="hero">
      <h1>gf-workflow</h1>
      <p class="hero-subtitle">从需求澄清到代码发布的完整工程循环</p>
    </section>

    <section class="content-section">
      <h2>四阶段模型</h2>
      <div class="workflow-phases">
        <div class="phase">
          <h3>阶段 1：需求澄清</h3>
          <ul>
            <li>brainstorming 探索上下文</li>
            <li>创建并审查 Issue</li>
            <li>产出设计文档</li>
          </ul>
        </div>
        <div class="phase">
          <h3>阶段 2：计划制定</h3>
          <ul>
            <li>writing-plans 生成实施计划</li>
            <li>质量门禁（build / test / fmt / clippy）</li>
            <li>用户审批计划</li>
          </ul>
        </div>
        <div class="phase">
          <h3>阶段 3：执行</h3>
          <ul>
            <li>TDD 红绿循环（RED → GREEN → REFACTOR）</li>
            <li>子代理隔离开发</li>
            <li>自动创建 PR</li>
          </ul>
        </div>
        <div class="phase">
          <h3>阶段 4：交付检查</h3>
          <ul>
            <li>流水线分析</li>
            <li>代码审查报告</li>
            <li>dogfooding 验证</li>
          </ul>
        </div>
      </div>
    </section>

    <section class="content-section">
      <h2>三种工作流模式</h2>
      <table class="comparison-table">
        <thead>
          <tr>
            <th>模式</th>
            <th>适用场景</th>
            <th>Phase 1</th>
            <th>Phase 2</th>
            <th>Phase 3</th>
            <th>Phase 4</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td><strong>full</strong></td>
            <td>feat / breaking change</td>
            <td>完整</td>
            <td>完整</td>
            <td>TDD + Review</td>
            <td>完整</td>
          </tr>
          <tr>
            <td><strong>standard</strong></td>
            <td>fix / refactor</td>
            <td>完整</td>
            <td>完整</td>
            <td>TDD + Review</td>
            <td>精简</td>
          </tr>
          <tr>
            <td><strong>fast</strong></td>
            <td>typo / hotfix / docs</td>
            <td>精简</td>
            <td>可跳过</td>
            <td>TDD + Review</td>
            <td>精简</td>
          </tr>
        </tbody>
      </table>
      <p><strong>自动检测规则</strong>：</p>
      <ul>
        <li><code>feat!</code> / breaking → full</li>
        <li><code>fix</code> / <code>refactor</code>（单模块）→ standard</li>
        <li><code>fix: typo</code> / <code>docs</code> / <code>chore</code> → fast</li>
        <li><code>good-first-issue</code> 标签 → fast</li>
      </ul>
    </section>

    <section class="content-section">
      <h2>两种技能来源</h2>
      <table class="comparison-table">
        <thead>
          <tr>
            <th>维度</th>
            <th>superpowers 模式</th>
            <th>mattpocock 模式</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td>技能集</td>
            <td>superpowers 技能</td>
            <td>to-spec / to-tickets / implement</td>
          </tr>
          <tr>
            <td>调用方式</td>
            <td>模型调用（model-invoked）</td>
            <td>用户命令（/to-spec 等）</td>
          </tr>
          <tr>
            <td>适用平台</td>
            <td>Claude Code / Codex / Gemini</td>
            <td>mattpocock 生态</td>
          </tr>
          <tr>
            <td>四阶段实现</td>
            <td>自动编排</td>
            <td>手动分步</td>
          </tr>
        </tbody>
      </table>
      <p>两种模式共享相同的合同（contract），可跨会话恢复。</p>
    </section>

    <section class="content-section">
      <h2>三种执行模式（Phase 3）</h2>
      <table class="comparison-table">
        <thead>
          <tr>
            <th>模式</th>
            <th>说明</th>
            <th>适用场景</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td><strong>background agent</strong></td>
            <td>后台子代理执行（默认）</td>
            <td>复杂任务，需要隔离</td>
          </tr>
          <tr>
            <td><strong>manual new window</strong></td>
            <td>新窗口手动执行</td>
            <td>需要人工介入</td>
          </tr>
          <tr>
            <td><strong>same-session</strong></td>
            <td>当前会话执行</td>
            <td>简单任务，需显式请求</td>
          </tr>
        </tbody>
      </table>
    </section>

    <section class="content-section">
      <h2>合同（Contract）机制</h2>
      <ul>
        <li><strong>JSON 状态文件</strong>：<code>.cache/workflows/active/&lt;workflow_id&gt;.json</code></li>
        <li><strong>记录内容</strong>：workflow_id / mode / current_phase / evidence</li>
        <li><strong>跨会话恢复</strong>：合同是 agent-agnostic 的，任何 agent 可恢复</li>
        <li><strong>闸门检查</strong>：每阶段进入前验证 evidence</li>
      </ul>
    </section>

    <section class="cta-section">
      <h2>开始使用 gf-workflow</h2>
      <p>在项目中输入 <code>/gf-workflow</code> 即可启动。系统会自动检测模式并引导完成四阶段流程。</p>
      <a href={`${base}/quickstart/`} class="cta-button">5 分钟快速上手 →</a>
    </section>
  </main>
</Base>
```

- [ ] **Step 2: 验证页面构建**

运行：`cd website && npm run build && ls dist/workflow/index.html`
预期：文件存在

- [ ] **Step 3: 提交**

```bash
git add website/src/pages/workflow.astro
git commit -m "feat(website): add /workflow/ page documenting gf-workflow system"
```

---

## Task 11: 实体一致性守护测试 — Rust 测试

**Files:**
- Create: `apps/cli/tests/geo_guard_test.rs`

**Interfaces:**
- Consumes: llms.txt, llms-full.txt, apps/cli/Cargo.toml, jsonld.ts
- Produces: Rust 守护测试，CI 执行

- [ ] **Step 1: 创建 geo_guard_test.rs**

创建 `apps/cli/tests/geo_guard_test.rs`，内容如下：

```rust
//! GEO 实体一致性守护测试
//!
//! 检查规范一句话定位在 Cargo.toml、llms.txt、llms-full.txt 中的逐字一致性。

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "守护测试读取已知存在的仓库内工件文件"
)]

use std::fs;
use std::path::PathBuf;

/// 全渠道逐字一致的规范一句话定位。
const CANONICAL_POSITIONING: &str =
    "跨平台 Git 工程化工作流编排框架：统一封装 GitHub / GitLab / GitCode 三大平台，配合 AI Agent Skills，覆盖从需求到发布的完整工程循环。";

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(rel: &str) -> String {
    fs::read_to_string(workspace_root().join(rel))
        .unwrap_or_else(|e| panic!("failed to read {rel}: {e}"))
}

#[test]
fn test_should_keep_canonical_positioning_in_llms_txt() {
    let content = read("website/public/llms.txt");
    assert!(
        content.contains(CANONICAL_POSITIONING),
        "llms.txt 必须包含规范一句话定位"
    );
}

#[test]
fn test_should_keep_canonical_positioning_in_llms_full_txt() {
    let content = read("website/public/llms-full.txt");
    assert!(
        content.contains(CANONICAL_POSITIONING),
        "llms-full.txt 必须包含规范一句话定位"
    );
}

#[test]
fn test_should_keep_canonical_positioning_in_cli_cargo_toml() {
    let content = read("apps/cli/Cargo.toml");
    assert!(
        content.contains(CANONICAL_POSITIONING),
        "apps/cli/Cargo.toml description 必须以规范一句话定位开头"
    );
}

#[test]
fn test_should_keep_canonical_positioning_in_jsonld_generator() {
    let content = read("website/src/lib/jsonld.ts");
    assert!(
        content.contains(CANONICAL_POSITIONING),
        "jsonld.ts 常量必须等于规范一句话定位"
    );
}

#[test]
fn test_should_not_contain_template_placeholders() {
    let files = [
        "Cargo.toml",
        "apps/cli/Cargo.toml",
        "crates/core/Cargo.toml",
        "website/public/llms.txt",
        "website/public/llms-full.txt",
    ];
    let placeholders = ["Your Name", "yourdomain", "TODO", "{{version}}"];

    for file in files {
        let content = read(file);
        for placeholder in &placeholders {
            assert!(
                !content.contains(placeholder),
                "{file} 包含模板占位符: {placeholder}"
            );
        }
    }
}
```

- [ ] **Step 2: 运行测试验证**

运行：`cargo test --test geo_guard_test`
预期：部分测试可能失败（因为 llms-full.txt 还未重构，jsonld.ts 还未创建），这是预期的 RED 阶段

- [ ] **Step 3: 提交**

```bash
git add apps/cli/tests/geo_guard_test.rs
git commit -m "test(geo): add Rust guard tests for entity consistency"
```

---

## Task 12: 实体一致性守护测试 — TypeScript 测试

**Files:**
- Create: `website/tests/geo-consistency.test.ts`

**Interfaces:**
- Consumes: jsonld.ts（Task 6 产物）
- Produces: TypeScript 一致性测试，vitest 执行

- [ ] **Step 1: 创建测试目录**

运行：`mkdir -p website/tests`

- [ ] **Step 2: 创建 geo-consistency.test.ts**

创建 `website/tests/geo-consistency.test.ts`，内容如下：

```typescript
import { describe, it, expect } from "vitest";
import {
  generateSoftwareAppJsonLd,
  generateFAQPageJsonLd,
  generateHowToJsonLd,
} from "../src/lib/jsonld";

const CANONICAL_POSITIONING =
  "跨平台 Git 工程化工作流编排框架：统一封装 GitHub / GitLab / GitCode 三大平台，配合 AI Agent Skills，覆盖从需求到发布的完整工程循环。";

describe("GEO entity consistency", () => {
  it("should use canonical positioning in SoftwareApplication JSON-LD", () => {
    const jsonLd = generateSoftwareAppJsonLd();
    expect(jsonLd.description).toBe(CANONICAL_POSITIONING);
  });

  it("should reference GitHub and crates.io in sameAs", () => {
    const jsonLd = generateSoftwareAppJsonLd();
    expect(jsonLd.sameAs).toContain(
      "https://github.com/byx-darwin/gitflow-cli",
    );
    expect(jsonLd.sameAs).toContain("https://crates.io/crates/gitflow-cli");
  });

  it("should generate valid FAQPage JSON-LD", () => {
    const jsonLd = generateFAQPageJsonLd();
    expect(jsonLd["@type"]).toBe("FAQPage");
    expect(jsonLd.mainEntity.length).toBeGreaterThan(0);
    for (const entity of jsonLd.mainEntity) {
      expect(entity["@type"]).toBe("Question");
      expect(entity.name.length).toBeGreaterThan(0);
      expect(entity.acceptedAnswer.text.length).toBeGreaterThan(0);
    }
  });

  it("should generate valid HowTo JSON-LD", () => {
    const jsonLd = generateHowToJsonLd();
    expect(jsonLd).not.toBeNull();
    if (jsonLd) {
      expect(jsonLd["@type"]).toBe("HowTo");
      expect(jsonLd.step.length).toBeGreaterThan(0);
      for (const step of jsonLd.step) {
        expect(step["@type"]).toBe("HowToStep");
        expect(step.name.length).toBeGreaterThan(0);
        expect(step.text.length).toBeGreaterThan(0);
      }
    }
  });
});
```

- [ ] **Step 3: 安装 vitest（如果未安装）**

运行：`cd website && npm install -D vitest`

- [ ] **Step 4: 运行测试验证**

运行：`cd website && npx vitest run tests/geo-consistency.test.ts`
预期：所有测试通过（前提是 jsonld.ts 已正确实现）

- [ ] **Step 5: 提交**

```bash
git add website/tests/ website/package.json website/package-lock.json
git commit -m "test(geo): add TypeScript consistency tests for JSON-LD generator"
```

---

## Task 13: GitHub description 更新 + 月度抽检文档

**Files:**
- Update: GitHub 仓库 description
- Create: `docs/geo-citation-check.md`

**Interfaces:**
- Produces: GitHub description 已更新，抽检文档已创建

- [ ] **Step 1: 更新 GitHub description**

运行：

```bash
gh repo edit --description "跨平台 Git 工程化工作流编排框架：统一封装 GitHub / GitLab / GitCode 三大平台，配合 AI Agent Skills，覆盖从需求到发布的完整工程循环。"
```

- [ ] **Step 2: 验证更新**

运行：`gh repo view --json description`
预期：输出包含规范一句话定位

- [ ] **Step 3: 创建 geo-citation-check.md**

创建 `docs/geo-citation-check.md`，内容如下：

```markdown
# GEO 月度 AI 引用抽检流程

## 目的

跟踪 AI 搜索引擎对 gf 的引用准确性，建立基线并持续改进 GEO 效果。

## 目标关键词

### 中文关键词
1. "有哪些 Git 工作流工具"
2. "AI 编程工程化"
3. "跨平台 Git CLI"
4. "gf vs gh"
5. "如何给 AI 编程加工程纪律"

### 英文关键词
1. "Git workflow tools"
2. "AI coding engineering"
3. "cross-platform Git CLI"
4. "gf vs gh vs glab"

## 抽检平台

| 平台 | URL | 频率 |
|------|-----|------|
| Perplexity | https://www.perplexity.ai | 月度 |
| ChatGPT | https://chat.openai.com | 月度 |
| Kimi | https://kimi.moonshot.cn | 月度 |
| 豆包 | https://www.doubao.com | 月度 |

## 抽检步骤

1. 打开目标平台
2. 输入目标关键词
3. 记录是否引用 gf
4. 评估引用准确性（1-5 分）：
   - 5 分：准确引用完整定位 + 推荐
   - 4 分：准确提及但未推荐
   - 3 分：部分信息正确
   - 2 分：信息模糊或有误
   - 1 分：未引用或完全错误

## 记录模板

| 日期 | 平台 | 关键词 | 是否引用 | 准确性评分 | 备注 |
|------|------|--------|---------|-----------|------|
| 2026-08-XX | Perplexity | "有哪些 Git 工作流工具" | 是/否 | 1-5 | ... |

## 执行频率

- **月度抽检**：每月第一个工作日执行
- **季度回顾**：每季度末汇总分析趋势
- **年度基线**：每年初建立新的引用基线

## 负责人

- 执行：项目维护者
- 审核：产品负责人（如有）
```

- [ ] **Step 4: 提交**

```bash
git add docs/geo-citation-check.md
git commit -m "docs(geo): add monthly AI citation spot-check process"
```

- [ ] **Step 5: 提交 GitHub description 更新记录**

```bash
git commit --allow-empty -m "chore(geo): update GitHub repo description to canonical positioning"
```

---

## Task 14: 集成测试 + 文档更新

**Files:**
- Verify: 所有测试通过
- Update: `docs/index.md`（如有必要）

**Interfaces:**
- Consumes: 前序所有任务产物
- Produces: 完整的 GEO 增强系统

- [ ] **Step 1: 运行所有 Rust 测试**

运行：`make test`
预期：所有测试通过（包括 geo_guard_test）

- [ ] **Step 2: 运行所有 TypeScript 测试**

运行：`cd website && npm run test`（或 `npx vitest run`）
预期：所有测试通过（包括 geo-consistency.test.ts）

- [ ] **Step 3: 验证网站构建**

运行：`cd website && npm run build`
预期：构建成功，无错误

- [ ] **Step 4: 验证新页面可访问**

运行：

```bash
ls website/dist/compare/index.html
ls website/dist/what-is-ai-workflow/index.html
ls website/dist/workflow/index.html
```

预期：三个文件均存在

- [ ] **Step 5: 验证 llms 文件**

运行：

```bash
wc -l website/public/llms-full.txt
wc -l website/public/llms-commands.txt
wc -l website/public/llms-architecture.txt
wc -l website/public/llms-faq.txt
```

预期：
- llms-full.txt: ~50 行
- llms-commands.txt: ~200 行
- llms-architecture.txt: ~80 行
- llms-faq.txt: ~70 行

- [ ] **Step 6: 验证 JSON-LD 输出**

运行：`grep -o '"@type":"FAQPage"' website/dist/index.html`
预期：输出 `"@type":"FAQPage"`

- [ ] **Step 7: 更新 docs/index.md（如有必要）**

如果 `docs/index.md` 需要添加 GEO 相关文档链接，更新之。

- [ ] **Step 8: 运行 clippy**

运行：`cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic`
预期：无警告

- [ ] **Step 9: 最终提交**

```bash
git add -A
git commit -m "chore(geo): complete GEO enhancement integration"
```

---

## Summary

本实施计划包含 **14 个任务**，覆盖设计文档中的所有交付物：

1. ✅ llms-commands.txt 创建
2. ✅ llms-architecture.txt 创建
3. ✅ llms-faq.txt 创建
4. ✅ llms-full.txt 重构为索引
5. ✅ FAQ + HowTo 数据文件创建
6. ✅ jsonld.ts 生成器创建
7. ✅ Base.astro 集成 JSON-LD
8. ✅ /compare/ 页面对比
9. ✅ /what-is-ai-workflow/ 页面创建
10. ✅ /workflow/ 页面创建
11. ✅ Rust 守护测试创建
12. ✅ TypeScript 一致性测试创建
13. ✅ GitHub description 更新 + 抽检文档
14. ✅ 集成测试 + 文档更新

**预计总工作量**：~10.5 小时

**退出标准验证**：
- ✅ llms-full.txt 模块化完成
- ✅ 实体一致性修复（GitHub description 已更新）
- ✅ JSON-LD 生成系统工作
- ✅ 3 个新页面发布
- ✅ 守护测试通过
- ✅ 抽检文档完成