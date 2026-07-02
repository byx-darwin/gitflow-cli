<!-- Issue: #4 -->
# Phase 3: GitLab + GitCode 平台支持 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 新增 GitLab 和 GitCode 两个平台的完整实现，使 gitflow-cli 成为真正的跨平台工具。同时补充 Pipeline 通用接口和三平台冒烟测试。

**Architecture:** 新建 `crates/gitlab`（调用 `glab` CLI）和 `crates/gitcode`（调用 `gc` CLI），均实现 core 中定义的全部 trait。同时在 core 中新增 `PipelineProvider` trait，三个平台 crate 各自实现。依赖流保持 `apps/cli → crates/{github,gitlab,gitcode} → crates/core`。

**Tech Stack:** Rust 2024, clap, tokio, serde_json, async-trait, chrono

## Global Constraints

- Rust 2024 edition，工具链 1.96.0
- `#![forbid(unsafe_code)]` 所有 crate
- 禁止 `unwrap()` / `expect()`，返回 `Result<T>`
- 公共项必须文档化（`#![warn(missing_docs)]`）
- GitLab crate 调用 `glab` CLI，GitCode crate 调用 `gc` CLI
- 三个平台 crate 保持对称的代码结构和 API 风格

## GitHub Issue 规划

**Issue 标题:** feat: Phase 3 — GitLab + GitCode 平台支持 + Pipeline 通用接口

**Issue 标签:** enhancement,core,cli,gitlab,gitcode,pipeline,phase-3

**Issue 描述:**
新增 GitLab 和 GitCode 两个平台 crate，实现与 GitHub crate 对称的全部 Provider。同时在 core 新增 PipelineProvider 通用 trait，三平台各自实现流水线分析接口。扩展 CLI 的 resolve_platform() 和 handle() 函数以支持多平台动态分发，并编写覆盖三平台的冒烟测试脚本。

**验收标准:**
- [ ] 所有任务完成
- [ ] `cargo build --workspace` 通过
- [ ] `cargo test --workspace` 通过
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` 通过
- [ ] `cargo +nightly fmt -- --check` 通过
- [ ] `gitflow --platform gitlab issue list` 可执行（需 `glab` 环境）
- [ ] `gitflow --platform gitcode pr list` 可执行（需 `gc` 环境）
- [ ] `gitflow pipeline status` 三平台均可执行
- [ ] 三平台冒烟测试通过

**关联:**
- 计划文件: `docs/superpowers/plans/2026-07-01-phase3-gitlab-gitcode.md`
- 设计文档: `specs/gitflow-cli-design.md`
- 前置: Phase 1 (#1) + Phase 2
- 里程碑: 跨平台支持

## File Structure

```
gitflow-cli/
├── crates/
│   ├── core/src/
│   │   ├── pipeline.rs          # 新增：PipelineProvider trait + PipelineData + args
│   │   └── lib.rs               # 修改：pub mod pipeline + 重新导出
│   ├── github/src/
│   │   ├── pipeline.rs          # 新增：GitHubPipelineProvider（调用 gh run/workflow）
│   │   └── lib.rs               # 修改：pub mod pipeline
│   ├── gitlab/                  # 新增：完整的 GitLab crate
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── error.rs
│   │       ├── issue.rs
│   │       ├── mr.rs            # GitLab 用 MR (Merge Request) 而非 PR
│   │       ├── release.rs
│   │       ├── review.rs
│   │       ├── auth.rs
│   │       ├── label.rs
│   │       ├── commit.rs
│   │       └── pipeline.rs
│   └── gitcode/                 # 新增：完整的 GitCode crate
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── error.rs
│           ├── issue.rs
│           ├── pr.rs
│           ├── release.rs
│           ├── review.rs
│           ├── auth.rs
│           ├── label.rs
│           ├── commit.rs
│           └── pipeline.rs
├── apps/cli/src/
│   ├── main.rs                  # 修改：扩展 resolve_platform + 动态 Provider 注册
│   └── commands/
│       ├── pipeline.rs          # 新增：pipeline status/logs/jobs/report 命令
│       └── mod.rs               # 修改
└── scripts/
    └── smoke-test.sh            # 修改：扩展为三平台冒烟测试
```

## Tasks

### Task 1: 创建 Issue

**Description:** 从 "Issue 规划" 部分提取信息，创建 GitHub Issue 并保存编号。

- [ ] **Step 1: 运行 scripts/create-issue.sh**

```bash
bash /Users/byx/.claude/skills/writing-plans-with-issue/scripts/create-issue.sh docs/superpowers/plans/2026-07-01-phase3-gitlab-gitcode.md
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

### Task 3: crates/core — 新增 PipelineProvider trait

**Description:** 新建 `pipeline.rs`，定义流水线分析的通用 trait 和 data types。Pipeline 是跨平台的 CI/CD 抽象。

**Dependencies:** 无（Task 1/2 之后第一个开发任务）

- [ ] **Step 1: 新建 pipeline.rs**
  - `PipelineStatus { id, ref_name, status, conclusion, created_at, updated_at, url }`
  - `PipelineStatusEnum { Running, Success, Failed, Cancelled, Pending }`
  - `JobData { id, name, status, conclusion, started_at, completed_at, url }`
  - `PipelineReport { total_runs, success_rate, avg_duration_secs, top_failures: Vec<String> }`
  - `PipelineProvider trait: status(branch) -> Result<Vec<PipelineStatus>>, logs(pipeline_id) -> Result<String>, jobs(pipeline_id) -> Result<Vec<JobData>>, report(branch, days) -> Result<PipelineReport>`

- [ ] **Step 2: 写单元测试**

- [ ] **Step 3: 更新 lib.rs**

> **Commit:** `feat(core): add PipelineProvider trait for CI/CD pipeline analysis (#N)`

### Task 4: crates/github — 新增 GitHubPipelineProvider

**Description:** 新建 `crates/github/src/pipeline.rs`，通过 `gh run list` / `gh run view` / `gh workflow` 实现 PipelineProvider。

**Dependencies:** Task 3

- [ ] **Step 1: 新建 pipeline.rs**
  - `status` → `gh run list --branch <branch> --json <fields>`
  - `logs` → `gh run view <id> --log`
  - `jobs` → `gh run view <id> --json jobs`
  - `report` → 组合多次 `gh run list` 调用 + 统计分析

- [ ] **Step 2: 更新 lib.rs**

- [ ] **Step 3: 写单元测试**

> **Commit:** `feat(github): add GitHubPipelineProvider (#N)`

### Task 5: crates/gitlab — crate 搭建 + 完整 Provider 实现

**Description:** 新建 `crates/gitlab` crate。通过 `glab` CLI 实现全部 Provider trait。GitLab 平台的关键差异：使用 MR (Merge Request) 而非 PR，`glab` CLI 命令略有不同。

**Dependencies:** Task 3, Task 4（参考 GitHub 实现模式）

- [ ] **Step 1: 新建 Cargo.toml**
  - `name = "gitflow-cli-gitlab"`，依赖 `gitflow-cli-core`、`tokio`、`serde_json`、`async-trait` 等

- [ ] **Step 2: 新建 error.rs**
  - `GlabError` 结构 + `parse_glab_error()` — 解析 `glab` CLI 的错误输出

- [ ] **Step 3: 新建 issue.rs — GitLabIssueProvider**
  - 实现 `IssueProvider` trait（全部 8 个方法），调用 `glab issue` 子命令

- [ ] **Step 4: 新建 mr.rs — GitLabMrProvider**
  - 注意：GitLab 用 `MergeRequestProvider` 做 trait 名（如果 core 中定义了别名 trait）
  - 实际方案：实现 `PrProvider` trait，内部调用 `glab mr` 子命令
  - MR 特有操作：`merge` → `glab mr merge`，`checkout` → `glab mr checkout`

- [ ] **Step 5: 新建 release.rs — GitLabReleaseProvider**
  - 调用 `glab release` 子命令

- [ ] **Step 6: 新建 review.rs — GitLabReviewProvider**
  - GitLab MR review → `glab mr approve` / `glab mr revoke`

- [ ] **Step 7: 新建 auth.rs — GitLabAuthProvider**
  - `login` → `glab auth login`，`status` → `glab auth status`

- [ ] **Step 8: 新建 label.rs — GitLabLabelProvider + GitLabMilestoneProvider**

- [ ] **Step 9: 新建 commit.rs — GitLabCommitProvider**

- [ ] **Step 10: 新建 pipeline.rs — GitLabPipelineProvider**
  - `status` → `glab ci list`，`logs` → `glab ci trace`，`jobs` → `glab ci list --json`

- [ ] **Step 11: 新建 lib.rs**（pub mod + 重新导出）

- [ ] **Step 12: 写单元测试**

> **Commit:** `feat(gitlab): add full GitLab platform support crate (#N)`

### Task 6: crates/gitcode — crate 搭建 + 完整 Provider 实现

**Description:** 新建 `crates/gitcode` crate。通过 `gc` CLI 实现全部 Provider trait。GitCode 使用 `gc` 作为原生 CLI。

**Dependencies:** Task 5（参考 GitLab 实现模式，保持三平台对称）

- [ ] **Step 1: 新建 Cargo.toml**
  - `name = "gitflow-cli-gitcode"`

- [ ] **Step 2: 新建 error.rs**
  - `GcError` 结构 + `parse_gc_error()`

- [ ] **Step 3-11: 实现全部 Provider**
  - `issue.rs` → GitCodeIssueProvider（调用 `gc issue`）
  - `pr.rs` → GitCodePrProvider（调用 `gc pr`）
  - `release.rs` → GitCodeReleaseProvider（调用 `gc release`）
  - `review.rs` → GitCodeReviewProvider
  - `auth.rs` → GitCodeAuthProvider（调用 `gc auth`）
  - `label.rs` → GitCodeLabelProvider + GitCodeMilestoneProvider
  - `commit.rs` → GitCodeCommitProvider
  - `pipeline.rs` → GitCodePipelineProvider（调用 `gc pipeline`）
  - `lib.rs` → pub mod + 重新导出

- [ ] **Step 12: 写单元测试**

> **Commit:** `feat(gitcode): add full GitCode platform support crate (#N)`

### Task 7: apps/cli — 多平台动态分发 + pipeline 命令

**Description:** 扩展 `main.rs` 的 `async_main`，实现根据 platform 字符串动态选择对应的 Provider。新建 `commands/pipeline.rs`。

**Dependencies:** Task 4, Task 5, Task 6

- [ ] **Step 1: 新建 Provider 注册中心**
  - 在 `main.rs` 新增 `create_provider<T>(platform, repo) -> Box<dyn T>` 模式
  - 或采用 match 分发：`match platform { "github" => GitHubXxxProvider::new(&repo).do_something(), "gitlab" => GitLabXxxProvider::new(&repo).do_something(), ... }`

- [ ] **Step 2: 扩展每个 commands/*.rs 的 handle() 函数**
  - 将硬编码的 `GitHubXxxProvider` 改为动态分发
  - 每个 `handle()` 增加 `gitlab` 和 `gitcode` 分支

- [ ] **Step 3: 新建 commands/pipeline.rs**
  - `PipelineCommand` enum: `Status`, `Logs`, `Jobs`, `Report`
  - `handle()` 函数 → 三平台动态分发 PipelineProvider

- [ ] **Step 4: 更新 main.rs**
  - 新增 `Commands::Pipeline(PipelineCommand)` 子命令
  - 新增 `gitflow pipeline` 帮助文档

- [ ] **Step 5: 写在 CI 环境中的集成测试**（使用 mock 或 dry-run）

> **Commit:** `feat(cli): add multi-platform dispatch and pipeline commands (#N)`

### Task 8: 扩展三平台冒烟测试

**Description:** 扩展 `scripts/smoke-test.sh`，支持 `--platform` 参数覆盖三平台。新建 `.github/workflows/smoke-test.yml` 矩阵构建。

**Dependencies:** Task 7

- [ ] **Step 1: 重写 smoke-test.sh 为多平台版本**
  - 参数：`--platform <github|gitlab|gitcode>`
  - 默认自动检测，可强制指定
  - 只读模式（`--read-only`）覆盖所有资源类型
  - 写操作需要 `--write` flag 才执行

- [ ] **Step 2: 新建 smoke-test CI workflow**
  - `.github/workflows/smoke-test.yml`: 矩阵构建 `[github, gitlab, gitcode]`
  - 每个平台使用对应的原生 CLI token/secrets

- [ ] **Step 3: 新增 Makefile target**
  - `make smoke-test`: 自动检测平台执行只读冒烟测试

> **Commit:** `test: add multi-platform smoke test and CI matrix (#N)`

### Task 9: 终检 — clippy + fmt + build + test

**Description:** 运行全 workspace 的构建、测试、lint、格式化检查，确保三个平台 crate 均编译通过。

**Dependencies:** Task 8

- [ ] **Step 1: `cargo build --workspace`**
- [ ] **Step 2: `cargo test --workspace`**
- [ ] **Step 3: `cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic`**
- [ ] **Step 4: `cargo +nightly fmt -- --check`**
- [ ] **Step 5: 修复所有错误和 warning**
- [ ] **Step 6: 最终确认全部通过**

> **Commit:** `chore: final lint and formatting pass for Phase 3 (#N)`

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
