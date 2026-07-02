# Phase 5: 完成度提升 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 补全剩余 Skills、增强 Shell 自动补全、输出 `--output text` 格式化、创建 Homebrew formula 和社区文档，使 gitflow-cli 达到社区项目的完成度标准。

**Architecture:** 剩余 Skills 遵循 Phase 2/4 已建立的模式。Shell 补全通过 clap 内置生成器增强。Homebrew formula 通过 GitHub Releases + Ruby 公式分发。社区文档补齐 CONTRIBUTING 等内容。

**Tech Stack:** Bash, Markdown (SKILL.md), Ruby (Homebrew formula), Rust (completions enhancement)

## Global Constraints

- Skills 遵循 Superpowers skill 规范
- Homebrew formula 遵循 Homebrew 官方指南
- `--output text` 格式化需与现有 JSON 输出共存（通过 `--output` flag 选择）
- 所有文档使用中文

## Issue 规划

**Issue 标题:** feat: Phase 5 — 剩余 Skills + Shell 补全增强 + Homebrew + 社区文档

**Issue 标签:** enhancement,skills,cli,docs,distribution,phase-5

**Issue 描述:**
补全设计规格中剩余的全部 Skills（gitflow-issue-review、gitflow-issue-triage、gitflow-pr-inline-review、gitflow-pr-apply-feedback、gitflow-release-helper、gitflow-pipeline-analyzer、gitflow-repo-onboarding、gitflow-repo、gitflow-precommit、gitflow-regression、gitflow-label-stats），增强 Shell 自动补全支持，实现 `--output text` 人类友好输出格式，创建 Homebrew formula 分发渠道，补齐社区文档。

**验收标准:**
- [ ] 所有任务完成
- [ ] 设计规格中全部 26 个 Skills 均已实现
- [ ] `gitflow completions bash/zsh/fish` 输出正确语法
- [ ] `gitflow issue list --output text` 输出人类友好格式
- [ ] Homebrew formula 语法通过 `brew audit` 检查
- [ ] 社区文档完整（CONTRIBUTING、CHANGELOG 模板等）

**关联:**
- 计划文件: `docs/superpowers/plans/2026-07-01-phase5-polish.md`
- 设计文档: `specs/gitflow-cli-design.md`
- 前置: Phase 1 (#1) + Phase 2 + Phase 3 + Phase 4
- 里程碑: RC (Release Candidate)

## File Structure

```
gitflow-cli/
├── skills/
│   ├── gitflow-issue-review/SKILL.md        # 新增：Issue 需求分析
│   ├── gitflow-issue-triage/SKILL.md        # 新增：Issue 分类分流
│   ├── gitflow-pr-inline-review/SKILL.md    # 新增：PR 行内评论
│   ├── gitflow-pr-apply-feedback/SKILL.md   # 新增：应用审查反馈
│   ├── gitflow-release-helper/SKILL.md      # 新增：发布助手
│   ├── gitflow-pipeline-analyzer/SKILL.md   # 新增：流水线分析
│   ├── gitflow-repo-onboarding/SKILL.md     # 新增：仓库入门指引
│   ├── gitflow-repo/SKILL.md                # 新增：仓库操作核心命令
│   ├── gitflow-precommit/SKILL.md           # 新增：Pre-commit 检查
│   ├── gitflow-regression/SKILL.md          # 新增：冒烟测试
│   └── gitflow-label-stats/SKILL.md         # 新增：标签统计分析
├── apps/cli/src/
│   ├── commands/
│   │   └── completions.rs                   # 修改：增强补全生成
│   ├── main.rs                              # 修改：新增 --output text 格式化逻辑
│   └── output.rs                            # 修改：新增 text 格式化器
├── scripts/
│   └── generate-completions.sh              # 新增：多 shell 补全生成
├── .github/
│   └── workflows/
│       └── release.yml                      # 新增：Release 工作流（含 Homebrew 更新）
├── docs/
│   ├── CONTRIBUTING.md                      # 增强：贡献指南
│   └── CHANGELOG.md                         # 已有，维护
├── HomebrewFormula/
│   └── gitflow-cli.rb                       # 新增：Homebrew formula
└── Makefile                                 # 修改：新增 completions/release targets
```

## Tasks

### Task 1: 创建 Issue

**Description:** 从 "Issue 规划" 部分提取信息，创建 GitHub Issue 并保存编号。

- [ ] **Step 1: 运行 scripts/create-issue.sh**

```bash
bash /Users/byx/.claude/skills/writing-plans-with-issue/scripts/create-issue.sh docs/superpowers/plans/2026-07-01-phase5-polish.md
```

- [ ] **Step 2: 验证 Issue 已创建**

```bash
cat .claude/gh-issue/current-issue.txt
gh issue view "$(cat .claude/gh-issue/current-issue.txt)"
```

### Task 2: 同步 Issue 状态为 in-progress

**Description:** 将 Issue 状态更新为 `status: in-progress`。

- [ ] **Step 1: 运行 scripts/sync-status.sh**

```bash
bash /Users/byx/.claude/skills/writing-plans-with-issue/scripts/sync-status.sh in-progress
```

- [ ] **Step 2: 确认**

```bash
echo "✅ Issue #$(cat .claude/gh-issue/current-issue.txt) 已标记为 in-progress"
```

### Task 3: skills/ — 补全工作流层 Skills（1/2）

**Description:** 新建 5 个工作流层 Skill：gitflow-issue-review、gitflow-issue-triage、gitflow-pr-inline-review、gitflow-pr-apply-feedback、gitflow-release-helper。

**Dependencies:** 无（Task 1/2 之后第一个开发任务）

- [ ] **Step 1: gitflow-issue-review/SKILL.md** — Issue 需求分析
  - 调用 `gitflow issue view N` 获取 Issue 详情
  - Claude 分析需求完整性（标题清晰度、描述充分度、验收标准明确度）
  - 输出分析报告（含改进建议）
  - 调用 `gitflow issue comment N --body-file analysis.md`

- [ ] **Step 2: gitflow-issue-triage/SKILL.md** — Issue 分类分流
  - 调用 `gitflow issue list` 获取所有 open issues
  - 按类型分类（bug/feature/enhancement/docs/question）
  - 按优先级评估（urgent/high/medium/low）
  - 调用 `gitflow issue label N --label "triage:done"` 标记已分类
  - 输出分类报告

- [ ] **Step 3: gitflow-pr-inline-review/SKILL.md** — PR 行内评论
  - 调用 `gitflow pr diff N` 获取 PR diff
  - 逐文件分析，针对具体代码行生成评论
  - 检查：逻辑错误、安全隐患、命名规范、边界条件
  - 调用 `gitflow commit comment <sha> --body "<comment>" --path <file> --line <N>`

- [ ] **Step 4: gitflow-pr-apply-feedback/SKILL.md** — 应用审查反馈
  - 调用 `gitflow pr view N` 获取 PR 详情和评论
  - 列出所有待处理的审查意见
  - 逐条在本地应用修改
  - 标记已处理的评论为 "resolved"

- [ ] **Step 5: gitflow-release-helper/SKILL.md** — 发布助手
  - 调用 `git log` 分析从上次 release 以来的变更
  - 生成 Release Note（conventional commits 分组）
  - 调用 `gitflow release create --tag <vX.Y.Z> --notes "<release_note>"`
  - 输出 release URL

> **Commit:** `feat(skills): add issue review, triage, inline review, feedback, and release helper skills (#N)`

### Task 4: skills/ — 补全工作流层 + 核心命令层 Skills（2/2）

**Description:** 新建剩余 6 个 Skill：gitflow-pipeline-analyzer、gitflow-repo-onboarding、gitflow-repo、gitflow-precommit、gitflow-regression、gitflow-label-stats。

**Dependencies:** Task 3

- [ ] **Step 1: gitflow-pipeline-analyzer/SKILL.md** — 流水线分析
  - 调用 `gitflow pipeline report --branch <current> --days 7`
  - 分析成功率趋势、失败模式、最长耗时
  - 输出分析报告 + 改进建议

- [ ] **Step 2: gitflow-repo-onboarding/SKILL.md** — 仓库入门指引
  - 分析仓库结构（语言、框架、测试框架、CI 配置）
  - 生成入门文档（如何构建、如何运行测试、项目约定）
  - 输出入门指南

- [ ] **Step 3: gitflow-repo/SKILL.md** — 仓库操作核心命令
  - 封装 `gitflow repo {clone,list,create,stats,sync,view}`
  - `stats` 调用 `gh repo view --json` 获取仓库统计数据
  - `sync` 执行 `git fetch upstream && git merge upstream/main`

- [ ] **Step 4: gitflow-precommit/SKILL.md** — Pre-commit 检查
  - 解析 `Cargo.toml` 和 `.pre-commit-config.yaml`
  - 运行 `cargo fmt -- --check` + `cargo clippy` + `cargo test`
  - 配置 `.git/hooks/pre-commit`

- [ ] **Step 5: gitflow-regression/SKILL.md** — 冒烟测试
  - 调用 `bash scripts/smoke-test.sh`
  - 解析输出，判断通过/失败
  - 失败时调用 gitflow-autoreport-bug 上报

- [ ] **Step 6: gitflow-label-stats/SKILL.md** — 标签统计分析
  - 调用 `gitflow issue list --label "<label>"` 多次获取
  - 统计分析：按标签分组计数、按优先级分布、未分类 Issue 识别
  - 输出统计报告

> **Commit:** `feat(skills): add pipeline analyzer, repo, precommit, regression, and label stats skills (#N)`

### Task 5: apps/cli — Shell 补全增强

**Description:** 增强 `gitflow completions` 命令，支持 bash/zsh/fish 三种 shell，生成完整参数补全（含动态值）。

**Dependencies:** 无（独立任务）

- [ ] **Step 1: 增强 completions.rs**
  - 使用 clap 的 `clap_complete` crate 生成多 shell 补全
  - 支持：`gitflow completions bash` / `zsh` / `fish`
  - 支持：`gitflow completions --install` 自动安装到对应 shell 配置
  - 支持：`gitflow completions --uninstall` 卸载

- [ ] **Step 2: 新增 scripts/generate-completions.sh**
  - 预生成三种 shell 的补全文件
  - 输出到 `completions/` 目录（供打包分发）

- [ ] **Step 3: 更新 Makefile**
  - `make completions`: 运行 generate-completions.sh
  - 在 `make install` 中集成补全安装步骤

- [ ] **Step 4: 写单元测试**
  - `test_should_generate_bash_completion`
  - `test_should_generate_zsh_completion`
  - `test_should_generate_fish_completion`

> **Commit:** `feat(cli): enhance shell completions for bash/zsh/fish (#N)`

### Task 6: apps/cli — `--output text` 格式化输出

**Description:** 实现 `--output text` 的人类友好格式化输出。与现有 JSON 输出（`--output json`，默认）共存。

**Dependencies:** 无（独立任务）

- [ ] **Step 1: 新增 text 格式化器**
  - 在 `apps/cli/src/` 下新建 `output.rs`（或扩展现有的）
  - 对每种数据类型实现 `Display` 或独立的 text 格式化函数
  - Issue 列表：表格格式（number | title | state | labels）
  - Issue 详情：键值对格式
  - PR 列表：表格格式（number | title | state | branch | draft）
  - Release 列表：简洁列表格式
  - Pipeline 状态：带颜色标记的列表

- [ ] **Step 2: 在 handle() 函数中集成**
  - 每个 `print_output()` 调用根据 `output_format` 选择 JSON 或 text 路径

- [ ] **Step 3: 写单元测试**
  - `test_should_format_issue_list_as_text_table`
  - `test_should_format_pr_view_as_key_value`
  - `test_should_handle_empty_list_gracefully`

> **Commit:** `feat(cli): add --output text human-friendly formatting (#N)`

### Task 7: Homebrew formula

**Description:** 创建 `HomebrewFormula/gitflow-cli.rb`，支持 macOS 用户通过 `brew install` 安装。配置 GitHub Release workflow 自动更新 formula。

**Dependencies:** 无（独立任务）

- [ ] **Step 1: 创建 HomebrewFormula/gitflow-cli.rb**
  - `class GitflowCli < Formula`
  - `desc`, `homepage`, `url`, `license`, `sha256`
  - `depends_on "rust" => :build`
  - `depends_on "gh"`（推荐但非强制）
  - `def install` → `system "cargo", "install", *std_cargo_args`
  - `test do` → `system "#{bin}/gitflow", "--version"`

- [ ] **Step 2: 创建 GitHub Release workflow**
  - `.github/workflows/release.yml`
  - 触发：tag push（`v*` 格式）
  - Step 1: `cargo build --release`
  - Step 2: 打包二进制（macOS/Linux/Windows）
  - Step 3: 计算 SHA256
  - Step 4: 创建 GitHub Release + 上传 assets
  - Step 5: 更新 Homebrew formula 中的 url + sha256

- [ ] **Step 3: 写 README 安装说明**
  - `brew install byx-darwin/gitflow-cli/gitflow-cli`（自定义 tap）
  - 或 `brew install gitflow-cli`（如合并到 homebrew-core）

> **Commit:** `feat: add Homebrew formula and GitHub Release workflow (#N)`

### Task 8: 社区文档 + CLI 收尾功能

**Description:** 增强 CONTRIBUTING.md、维护 CHANGELOG、实现 `gitflow skills install` CLI 命令、更新 README。

**Dependencies:** Task 4, Task 5, Task 6

- [ ] **Step 1: 增强 CONTRIBUTING.md**
  - 开发环境搭建指南（Rust toolchain、原生 CLI、IDE 配置）
  - 代码规范（参考 CLAUDE.md 的核心规范）
  - TDD 开发循环说明
  - PR 提交流程
  - Issue 分类标签说明

- [ ] **Step 2: 实现 `gitflow skills install` CLI 命令**
  - 在 `main.rs` 中已有 `SkillsInstall` 的占位，需实现：
    - 复制 `skills/` 目录到 `~/.claude/skills/`
    - 跳过同名文件（输出 warning）
    - 输出安装汇总（新增 N 个，跳过 M 个）
  - 新增 `gitflow skills list` 列出已安装 skills
  - 新增 `gitflow skills uninstall` 卸载

- [ ] **Step 3: 更新 README.md**
  - 完整的功能列表（所有命令一览）
  - 安装方式：brew / cargo / 源码编译
  - 快速开始 3 步示例
  - Skills 列表（26 个 skills 的分类表格）

- [ ] **Step 4: 维护 CHANGELOG.md**
  - 使用 cliff.toml 生成 conventional commits 风格的 changelog
  - 添加 Makefile target: `make changelog`

> **Commit:** `docs: complete community documentation and skills install CLI command (#N)`

### Task 9: 终检 — 完整验证

**Description:** 全面验证：Rust 代码质量检查、Skills 文件完整性、安装脚本端到端测试。

**Dependencies:** Task 8

- [ ] **Step 1: Rust 质量门**
  ```bash
  cargo build --workspace && cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings && cargo +nightly fmt -- --check && echo "✅ All Rust checks passed"
  ```

- [ ] **Step 2: Skills 完整性验证**
  ```bash
  echo "=== Skills 清单 ===" && ls -1 skills/ | grep gitflow && echo "" && echo "Total: $(ls -1 skills/ | grep gitflow | wc -l) skills"
  ```

- [ ] **Step 3: install.sh 干跑验证**
  ```bash
  bash scripts/install.sh --no-build 2>&1 | head -20
  ```

- [ ] **Step 4: 设计规格对照检查**
  - 对照 `specs/gitflow-cli-design.md` 的三层架构列表，逐一确认每个 Skill 已实现

> **Commit:** `chore: final quality gate for Phase 5 completion (#N)`

### Task 10: 收尾 — 本地合并后关闭 Issue

**Description:** 开发完成并本地合并到 base 分支后，push 并关闭 Issue。

- [ ] **Step 1: 确保已合并到 base 分支**

```bash
git branch --show-current
```

- [ ] **Step 2: 运行 scripts/finish-issue.sh**

```bash
bash /Users/byx/.claude/skills/writing-plans-with-issue/scripts/finish-issue.sh
```

- [ ] **Step 3: 确认 Issue 已关闭**

```bash
gh issue view "$(cat .claude/gh-issue/current-issue.txt 2>/dev/null || echo 'already cleaned')"
```
