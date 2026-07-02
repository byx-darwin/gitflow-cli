<!-- Issue: #5 -->
# Phase 4: 编排层 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 实现编排层三大核心 Skill（gitflow-workflow 全流程编排、gitflow-quality 质量关卡、gitflow-autoreport-bug 自动错误反馈完整版），配合 install.sh 一键安装脚本和 Superpowers 集成指南，使 gitflow-cli 成为完整的开发工作流指挥中心。

**Architecture:** 三个编排 Skill 通过 subprocess 调用 `gitflow` CLI，按阶段闸门驱动需求→开发→质量关卡→交付的完整流程。install.sh 负责 CLI 编译 + Skills 安装 + Hook 注册的一键部署。

**Tech Stack:** Bash, Markdown (SKILL.md), gitflow CLI (Rust)

## Global Constraints

- Skills 遵循 Superpowers skill 规范
- 编排层 Skill 只做指挥，不直接执行操作（通过 `gitflow` CLI 命令）
- 阶段闸门必须严格校验（不允许跳过）
- install.sh 支持 macOS、Linux、Windows (Git Bash)

## GitHub Issue 规划

**Issue 标题:** feat: Phase 4 — 编排层 Skills + 一键安装 + 集成指南

**Issue 标签:** enhancement,skills,orchestration,docs,phase-4

**Issue 描述:**
实现 gitflow-cli 的编排层三大核心 Skill：gitflow-workflow（全流程指挥，从需求分析到质量关卡再到交付）、gitflow-quality（5 项质量检查闸门：build/test/coverage/format/static）、gitflow-autoreport-bug（完整版，集成 Skills 侧 ERR trap + Claude 分析 + 去重创建 Issue）。同时提供 `scripts/install.sh` 一键安装脚本和 Superpowers 集成指南。

**验收标准:**
- [ ] 所有任务完成
- [ ] 三个编排层 Skill 文件语法正确
- [ ] `install.sh` 在 macOS 上测试通过
- [ ] `make install-skills` 将所有 Skills 复制到 `~/.claude/skills/`
- [ ] 集成指南完整、可执行
- [ ] 阶段闸门逻辑正确（需求→开发→质量→交付不可跳步）

**关联:**
- 计划文件: `docs/superpowers/plans/2026-07-01-phase4-orchestration.md`
- 设计文档: `specs/gitflow-cli-design.md`
- 前置: Phase 1 (#1) + Phase 2 + Phase 3
- 里程碑: 编排能力就绪

## File Structure

```
gitflow-cli/
├── skills/
│   ├── gitflow-workflow/SKILL.md           # 新增：全流程编排
│   ├── gitflow-quality/SKILL.md            # 新增：质量关卡
│   ├── gitflow-autoreport-bug/SKILL.md     # 新增：自动错误反馈（完整版）
│   └── _common.sh                          # 新增：共享函数库（report_error + ERR trap）
├── scripts/
│   ├── install.sh                          # 新增：一键安装脚本
│   └── _common.sh                          # 新增：脚本公用函数（与 skills/_common.sh 共享）
├── hooks/
│   └── auto-report-bug.sh                  # 修改：增强 pending.json 分析
├── docs/
│   └── integration-guide.md                # 新增：Superpowers 集成指南
├── specs/
│   └── index.md                            # 修改：更新索引引用
└── Makefile                                # 修改：新增 install-skills/install-hooks targets
```

## Tasks

### Task 1: 创建 Issue

**Description:** 从 "Issue 规划" 部分提取信息，创建 GitHub Issue 并保存编号。

- [ ] **Step 1: 运行 scripts/create-issue.sh**

```bash
bash /Users/byx/.claude/skills/writing-plans-with-issue/scripts/create-issue.sh docs/superpowers/plans/2026-07-01-phase4-orchestration.md
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

### Task 3: skills/ — 共享函数库 _common.sh

**Description:** 新建 `skills/_common.sh`（同时复制到 `scripts/_common.sh`），为所有 Skill 脚本提供标准化的错误捕获、pending.json 写入、平台检测等共享函数。

**Dependencies:** 无（Task 1/2 之后第一个开发任务）

- [ ] **Step 1: 新建 skills/_common.sh**
  - `report_error(command, platform, error_code, error_message)`: 写入 `.cache/bug-reports/pending.json`
  - `ERR trap`: 在脚本异常退出时自动调用 `report_error`
  - `cd_to_git_root()`: 切换到仓库根目录
  - `detect_platform()`: 从 `git remote` 检测平台
  - `check_prerequisites()`: 检查所需 CLI 是否可用
  - `generate_error_id()`: 生成唯一错误 ID（时间戳+进程号 hash）

- [ ] **Step 2: 在 skills/_common.sh 中集成**
  - 添加 `set -euo pipefail`
  - 添加 `trap 'on_error $LINENO' ERR`
  - `on_error()` 调用 `report_error "skill-name" "$(detect_platform)" "SKILL_ERROR" "Script failed at line $1"`

- [ ] **Step 3: 复制到 scripts/_common.sh**（保持两份文件同步）

> **Commit:** `feat(skills): add shared shell function library with error reporting (#N)`

### Task 4: skills/ — gitflow-workflow 全流程编排

**Description:** 新建 `skills/gitflow-workflow/SKILL.md`。这是 gitflow-cli 的顶层指挥中心，按照设计规格的阶段闸门驱动完整开发流程。

**Dependencies:** Task 3

- [ ] **Step 1: 编写 SKILL.md**
  - **Phase 1: 需求澄清**
    - 触发 Superpowers brainstorming（如未执行）
    - 调用 `gitflow-issue-create` 引导创建 Issue
    - 调用 `gitflow-issue-review` 进行需求分析
    - 产出：Issue URL + 需求分析报告 → 评论到 Issue
  - **Phase 2: 开发实现**
    - 调用 Superpowers writing-plans 拆解为原子任务
    - 调用 superpowers:subagent-driven-development 执行
    - 每个原子任务通过 TDD 循环
    - 执行 requesting-code-review
    - 产出：全绿原子任务清单 → 评论到 Issue
  - **Phase 3: 质量关卡**
    - 调用 `gitflow-quality` 运行 5 项检查
    - 不通过 → 返回 Phase 2 修复
    - 产出：质量报告 → 评论到 Issue
  - **Phase 4: 交付**
    - 调用 `gitflow-pr-create` 创建 PR
    - 调用 `gitflow-pr-review` 请求审查
    - 调用 Superpowers finishing-a-development-branch
    - 调用 `gitflow-release-helper`（如需要 release）
    - 产出：Merged PR + Release（如适用）

- [ ] **Step 2: 定义阶段闸门**
  - 需求→开发：必须出示 Issue URL + 需求分析报告
  - 开发→质量：必须全部原子任务 done
  - 质量→交付：5 项检查全部绿色
  - 违规处理：输出明确阻断信息

- [ ] **Step 3: 添加使用示例**
  - 典型场景 1：从零开始一个新功能
  - 典型场景 2：修复一个 Bug
  - 典型场景 3：已有关联 Issue 的增量开发

> **Commit:** `feat(skills): add gitflow-workflow orchestration skill (#N)`

### Task 5: skills/ — gitflow-quality 质量关卡

**Description:** 新建 `skills/gitflow-quality/SKILL.md`。5 项质量检查闸门，全部通过才能进入交付阶段。

**Dependencies:** Task 3

- [ ] **Step 1: 编写 SKILL.md**
  - **5 项检查：**
    1. `build` — `cargo build --workspace`（无 error）
    2. `test` — `cargo test --workspace`（全部通过）
    3. `coverage` — `cargo tarpaulin --workspace`（> 80%）
    4. `format` — `cargo +nightly fmt -- --check`（无 diff）
    5. `static` — `cargo clippy --workspace --all-targets -- -D warnings`（无 lint）
  - **检查策略：**
    - 按顺序执行，失败即停（fast-fail）
    - 每项检查输出 ✅/❌ 状态
    - 最终汇总为 Quality Report
  - **非 Rust 项目的质量检查适配**
    - 检测项目语言 → 适配对应检查工具

- [ ] **Step 2: 编写 Quality Report 格式**
  ```
  ## Quality Report — 2026-07-01
  | Check    | Status | Details |
  |----------|--------|---------|
  | build    | ✅     | 0 errors, 2 warnings |
  | test     | ✅     | 47 passed, 0 failed |
  | coverage | ✅     | 85.3% (threshold: 80%) |
  | format   | ✅     | No diff |
  | static   | ✅     | No warnings |
  ```

- [ ] **Step 3: 配置自动发布到关联 Issue**
  - 检测 `.claude/gh-issue/current-issue.txt`
  - 存在 → `gitflow issue comment N --body-file quality-report.md`

> **Commit:** `feat(skills): add gitflow-quality gate skill (#N)`

### Task 6: skills/ — gitflow-autoreport-bug 完整版

**Description:** 新建 `skills/gitflow-autoreport-bug/SKILL.md`。实现五层架构的完整处理层：读取 pending.json → Claude 分析错误上下文 → 去重检查 → 创建 Issue → 清理。

**Dependencies:** Task 3

- [ ] **Step 1: 编写 SKILL.md**
  - **Step 1: 读取 pending.json**
    - 检查 `.cache/bug-reports/pending.json` 是否存在
    - 解析 JSON 提取 error_code、command、platform、error_message
  - **Step 2: Claude 分析**
    - 基于错误上下文生成：可能原因、建议修复方向、严重程度评估
    - 生成 Issue 标题和正文
  - **Step 3: 去重检查**
    - 搜索关键词：`[auto-report] {command} {error_code}`
    - 调用 `gitflow issue list --search "<keywords>" --state all`
    - 匹配 → 跳过并删除 pending.json；未匹配 → 继续
  - **Step 4: 创建 Issue**
    - 调用 `gitflow issue create --title "[auto-report] gitflow {command} — {error_code}" --body "<分析报告>" --label "bug,auto-reported,{platform},{error_code}"`
    - 删除 pending.json
  - **Step 5: 输出**
    - 输出 Issue 链接
    - 如果重复，输出 "已存在相似报告: {url}"

- [ ] **Step 2: 异常处理**
  - `gh auth` 失败 → 保留 pending.json + 记录到 failed.log
  - Issue 创建失败 → 保留 pending.json + 记录到 failed.log
  - pending.json 格式异常 → 重命名为 .invalid 后缀
  - 非 git 仓库 → 静默退出

- [ ] **Step 3: 增强 hooks/auto-report-bug.sh**
  - 从单纯的 "cat pending.json" 升级为：
    - 检查 pending.json 存在
    - 检查是否为交互式终端（交互式跳过）
    - 输出带格式的提示信息
    - 引导 Claude 加载 gitflow-autoreport-bug skill

> **Commit:** `feat(skills): add gitflow-autoreport-bug complete skill with deduplication (#N)`

### Task 7: scripts/install.sh — 一键安装脚本

**Description:** 新建 `scripts/install.sh`，实现 gitflow-cli 的一键安装：编译 CLI → 安装到 PATH → 复制 Skills → 注册 Hooks → 验证。

**Dependencies:** Task 3, Task 4, Task 5, Task 6

- [ ] **Step 1: 编写 install.sh**
  - **参数：** `--no-build`（跳过编译）, `--no-skills`（跳过 Skills 安装）, `--no-hooks`（跳过 Hook 注册）
  - **Step 1: 检查依赖**
    - Rust toolchain (`rustc --version` ≥ 1.96.0)
    - Git (`git --version`)
    - 原生 CLI（自动检测平台，提示安装）
  - **Step 2: 编译**
    - `cargo build --release`
    - 复制二进制到 `$CARGO_HOME/bin/` 或 `/usr/local/bin/`
  - **Step 3: 安装 Skills**
    - 复制 `skills/` 到 `~/.claude/skills/`
    - 检查是否与已有 skill 冲突（同名跳过 + warning）
  - **Step 4: 注册 Hooks**
    - 合并 `.claude/settings.json` 的 Stop Hook 配置
  - **Step 5: 验证**
    - `gitflow --version`
    - `gitflow auth status`
    - `ls ~/.claude/skills/ | grep gitflow | wc -l`

- [ ] **Step 2: 支持 Homebrew 安装路径**
  - `brew install gitflow-cli` 场景下的路径适配

- [ ] **Step 3: 支持 Windows (Git Bash)**
  - 路径分隔符适配
  - `which` vs `where` 命令适配

> **Commit:** `feat: add one-click install script (#N)`

### Task 8: docs/ — Superpowers 集成指南

**Description:** 新建 `docs/integration-guide.md`，说明如何将 gitflow-cli 的 Skills 与 Superpowers 的 SDD 工作流深度集成。

**Dependencies:** Task 4, Task 5, Task 6

- [ ] **Step 1: 编写 integration-guide.md**
  - **概览：** gitflow-cli 与 Superpowers 的关系图
  - **开发流程集成：**
    - brainstorming + gitflow-issue-create → Issue
    - writing-plans + gitflow-workflow → 原子任务
    - TDD + subagent-dev → 实现
    - gitflow-quality → 质量闸门
    - gitflow-pr-create + finishing-a-dev-branch → 交付
  - **错误反馈集成：**
    - gitflow CLI 错误 → pending.json → gitflow-autoreport-bug → Issue
  - **配置示例：**
    - `.claude/settings.json` Hook 配置示例
    - 个人化配置建议

- [ ] **Step 2: 更新 docs/index.md**
  - 添加 `integration-guide.md` 索引

> **Commit:** `docs: add Superpowers integration guide (#N)`

### Task 9: Makefile + specs 更新

**Description:** 更新 Makefile 添加 install 相关 targets，更新 specs/index.md 项目索引。

**Dependencies:** Task 7

- [ ] **Step 1: 更新 Makefile**
  - `make install-skills`: 复制 `skills/` 到 `~/.claude/skills/`
  - `make install-hooks`: 注册 `.claude/settings.json` Hook
  - `make install`: 完整安装（调用 install.sh）
  - `make list-skills`: 列出已安装 skills
  - `make uninstall-skills`: 卸载 gitflow skills

- [ ] **Step 2: 更新 specs/index.md**
  - 添加 Phase 4 计划文件索引
  - 更新设计文档引用

> **Commit:** `chore: update Makefile with install targets and specs index (#N)`

### Task 10: 终检 — 验证所有 Skills + install.sh

**Description:** 检查所有 SKILL.md 语法、install.sh 可执行性、Makefile targets 正确性。

**Dependencies:** Task 9

- [ ] **Step 1: 验证 Skill 文件**
  - 检查所有 `skills/*/SKILL.md` 存在且格式正确
  - 检查 skill 命名是否与设计规格一致

- [ ] **Step 2: 验证 install.sh**
  - 在 macOS 上执行 `bash scripts/install.sh --no-build`（验证脚本路径逻辑）

- [ ] **Step 3: 验证 Makefile**
  - `make install-skills` 输出确认
  - `make list-skills` 输出确认

- [ ] **Step 4: 修复所有问题**

> **Commit:** `chore: final validation pass for Phase 4 skills and scripts (#N)`

### Task 11: 收尾 — 本地合并后关闭 Issue

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
