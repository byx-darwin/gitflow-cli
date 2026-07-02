<!-- Issue: #3 -->
# Phase 2: GitHub 完整支持 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在 Phase 1 MVP 基础上，补齐 GitHub 平台的全部 Issue/PR 操作（close/reopen/comment/label/merge）、新增 Release/Review/Auth/Label/Milestone/Commit 全资源类型支持，并创建核心命令层和工作流层 Skills。

**Architecture:** 扩展 `crates/core` traits（新增方法 + 新 trait）→ 扩展 `crates/github`（完整 Provider 实现）→ 扩展 `apps/cli`（新子命令）→ 新建 `skills/` 目录（核心命令层 + 工作流层 Skills）。依赖流保持 `apps/cli → crates/github → crates/core`。

**Tech Stack:** Rust 2024, clap, tokio, serde_json, async-trait, chrono

## Global Constraints

- Rust 2024 edition，工具链 1.96.0
- `#![forbid(unsafe_code)]` 所有 crate
- 禁止 `unwrap()` / `expect()`，返回 `Result<T>`
- 公共项必须文档化（`#![warn(missing_docs)]`）
- 所有领域类型派生/实现 `Debug`
- 遵循 workspace lint policy
- Skills 文件采用 Markdown，遵循 Superpowers skill 规范

## GitHub Issue 规划

**Issue 标题:** feat: Phase 2 — GitHub 完整支持，补齐全资源类型操作 + 核心 Skills

**Issue 标签:** enhancement,core,cli,github,skills,phase-2

**Issue 描述:**
在 Phase 1 基础上，补齐 GitHub 平台的全部操作能力：扩展 Issue/PR trait（close/reopen/comment/label/merge/checkout），新增 Release/Review/Auth/Label/Milestone/Commit 六个资源类型的 trait 和 GitHub 实现，扩展 CLI 命令覆盖全部资源类型，并创建核心命令层和部分工作流层 Skills。

**验收标准:**
- [ ] 所有任务完成
- [ ] `cargo build --workspace` 通过
- [ ] `cargo test --workspace` 通过
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` 通过
- [ ] `cargo +nightly fmt -- --check` 通过
- [ ] `gitflow issue {close,reopen,comment,label}` 可执行
- [ ] `gitflow pr {close,reopen,comment,merge,checkout,ready,wip}` 可执行
- [ ] `gitflow release {create,list,view,edit,upload,download,delete}` 可执行
- [ ] `gitflow review {comment,approve,request-changes,submit}` 可执行
- [ ] `gitflow auth {login,logout,status,token}` 可执行
- [ ] `gitflow label {create,list,edit,delete}` 可执行
- [ ] `gitflow milestone {create,list,edit,close,reopen}` 可执行
- [ ] `gitflow commit {view,diff,patch,comment}` 可执行
- [ ] 所有 Skills 文件语法正确、可被加载

**关联:**
- 计划文件: `docs/superpowers/plans/2026-07-01-phase2-github-full.md`
- 设计文档: `specs/gitflow-cli-design.md`
- 前置: Phase 1 (#1)
- 里程碑: GitHub 完整支持

## File Structure

```
gitflow-cli/
├── crates/
│   ├── core/src/
│   │   ├── issue.rs            # 修改：新增 close/reopen/comment/add_labels/remove_label 方法
│   │   ├── pr.rs               # 修改：新增 close/reopen/comment/merge/checkout/ready/wip/sync 方法
│   │   ├── release.rs          # 新增：ReleaseProvider trait + ReleaseData + args
│   │   ├── review.rs           # 新增：ReviewProvider trait + ReviewData + args
│   │   ├── auth.rs             # 新增：AuthProvider trait + AuthData + args
│   │   ├── label.rs            # 新增：LabelProvider + MilestoneProvider traits + args
│   │   ├── commit.rs           # 新增：CommitProvider trait + CommitData + args
│   │   ├── lib.rs              # 修改：pub mod 声明 + 重新导出
│   │   └── types.rs            # 修改：新增 CommentData, MergeStatus 等共享类型
│   └── github/src/
│       ├── issue.rs            # 修改：补齐 close/reopen/comment/label 实现
│       ├── pr.rs               # 修改：补齐 close/reopen/comment/merge/checkout 实现
│       ├── release.rs          # 新增：GitHubReleaseProvider
│       ├── review.rs           # 新增：GitHubReviewProvider
│       ├── auth.rs             # 新增：GitHubAuthProvider
│       ├── label.rs            # 新增：GitHubLabelProvider + GitHubMilestoneProvider
│       ├── commit.rs           # 新增：GitHubCommitProvider
│       └── lib.rs              # 修改：pub mod 声明 + 重新导出
├── apps/cli/src/commands/
│   ├── issue.rs                # 修改：新增 close/reopen/comment/label 子命令
│   ├── pr.rs                   # 修改：新增 close/reopen/comment/merge/checkout/ready/wip/sync 子命令
│   ├── release.rs              # 新增：release 命令模块
│   ├── review.rs               # 新增：review 命令模块
│   ├── auth.rs                 # 新增：auth 命令模块
│   ├── label.rs                # 新增：label/milestone 命令模块
│   ├── commit.rs               # 新增：commit 命令模块
│   └── mod.rs                  # 修改：新增模块声明
├── skills/
│   ├── gitflow-issue/SKILL.md          # 新增：Issue 核心命令层
│   ├── gitflow-pr/SKILL.md             # 新增：PR 核心命令层
│   ├── gitflow-release/SKILL.md        # 新增：Release 核心命令层
│   ├── gitflow-auth/SKILL.md           # 新增：Auth 核心命令层
│   ├── gitflow-review/SKILL.md         # 新增：Review 核心命令层
│   ├── gitflow-label-milestone/SKILL.md  # 新增：标签/里程碑核心命令层
│   ├── gitflow-commit/SKILL.md         # 新增：Commit 核心命令层
│   ├── gitflow-issue-create/SKILL.md   # 新增：引导创建 Issue
│   ├── gitflow-pr-create/SKILL.md      # 新增：引导创建 PR
│   └── gitflow-pr-review/SKILL.md      # 新增：PR 工程审查
└── scripts/
    └── smoke-test.sh           # 修改：扩展冒烟测试覆盖新增命令
```

## Tasks

### Task 1: 创建 Issue

**Description:** 从 "Issue 规划" 部分提取信息，创建 GitHub Issue 并保存编号。

- [ ] **Step 1: 运行 scripts/create-issue.sh**

```bash
bash /Users/byx/.claude/skills/writing-plans-with-issue/scripts/create-issue.sh docs/superpowers/plans/2026-07-01-phase2-github-full.md
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

### Task 3: crates/core — 扩展 IssueProvider/PrProvider trait

**Description:** 为 `IssueProvider` 新增 `close`/`reopen`/`comment`/`add_labels`/`remove_label` 方法；为 `PrProvider` 新增 `close`/`reopen`/`comment`/`merge`/`checkout`/`ready`/`wip`/`sync` 方法。新增 `CommentData`、`MergeStatus` 等 domain types。

**Dependencies:** 无（Task 1/2 之后第一个开发任务）

- [ ] **Step 1: 扩展 issue.rs — IssueProvider trait**
  - 新增 `close(&self, number: u64) -> Result<IssueData>`
  - 新增 `reopen(&self, number: u64) -> Result<IssueData>`
  - 新增 `comment(&self, number: u64, body: &str) -> Result<CommentData>`
  - 新增 `add_labels(&self, number: u64, labels: &[String]) -> Result<()>`
  - 新增 `remove_label(&self, number: u64, label: &str) -> Result<()>`

- [ ] **Step 2: 扩展 pr.rs — PrProvider trait**
  - 新增 `close(&self, number: u64) -> Result<PrData>`
  - 新增 `reopen(&self, number: u64) -> Result<PrData>`
  - 新增 `comment(&self, number: u64, body: &str) -> Result<CommentData>`
  - 新增 `merge(&self, number: u64, strategy: Option<&str>) -> Result<MergeResult>`
  - 新增 `checkout(&self, number: u64) -> Result<()>`
  - 新增 `mark_ready(&self, number: u64) -> Result<PrData>`
  - 新增 `mark_wip(&self, number: u64) -> Result<PrData>`
  - 新增 `sync_branch(&self, number: u64) -> Result<()>`

- [ ] **Step 3: 扩展 types.rs 新增共享类型**
  - `CommentData { id: u64, body: String, author: UserSummary, created_at: DateTime<Utc> }`
  - `MergeResult { merged: bool, sha: Option<String>, message: Option<String> }`
  - `MergeStrategy { Merge, Squash, Rebase }` enum

- [ ] **Step 4: 写单元测试**（每个新增类型的序列化/反序列化）

- [ ] **Step 5: 更新 lib.rs 重新导出**

> **Commit:** `feat(core): extend IssueProvider and PrProvider with full operation set (#N)`

### Task 4: crates/core — 新增 ReleaseProvider/ReviewProvider/AuthProvider traits

**Description:** 新建 `release.rs`、`review.rs`、`auth.rs`，定义 release/review/auth 的 trait、data types、args types。

**Dependencies:** Task 3（共享 types 就绪）

- [ ] **Step 1: 新建 release.rs**
  - `ReleaseData { id, tag_name, name, body, draft, prerelease, author, created_at, published_at, url }`
  - `CreateReleaseArgs { tag_name, name, body, draft, prerelease, target_commitish }`
  - `ReleaseProvider trait: create, list, view, edit, upload_asset, download_asset, delete`

- [ ] **Step 2: 新建 review.rs**
  - `ReviewData { id, state, body, author, submitted_at }` + `ReviewState { Approved, ChangesRequested, Commented }`
  - `ReviewCommentData { id, path, body, line, diff_hunk, author, created_at }`
  - `ReviewProvider trait: comment(pr_number, body) -> Result<ReviewData>, approve(pr_number, body), request_changes(pr_number, body), submit_review(pr_number, event, body)`

- [ ] **Step 3: 新建 auth.rs**
  - `AuthProvider trait: login() -> Result<()>, logout() -> Result<()>, status() -> Result<AuthStatus>, token() -> Result<String>`
  - `AuthStatus { logged_in: bool, user: Option<String>, scopes: Vec<String> }`

- [ ] **Step 4: 写单元测试**

- [ ] **Step 5: 更新 lib.rs**

> **Commit:** `feat(core): add ReleaseProvider, ReviewProvider, and AuthProvider traits (#N)`

### Task 5: crates/core — 新增 LabelProvider/MilestoneProvider/CommitProvider traits

**Description:** 新建 `label.rs` 和 `commit.rs`，定义 label/milestone/commit 的 trait、data types、args types。

**Dependencies:** Task 3

- [ ] **Step 1: 新建 label.rs**
  - `LabelData { name, color, description }`（扩展 types.rs 中的 Label）
  - `MilestoneData { number, title, description, state, due_on, closed_issues, open_issues }`
  - `LabelProvider trait: create, list, edit, delete`
  - `MilestoneProvider trait: create, list, edit, close, reopen`

- [ ] **Step 2: 新建 commit.rs**
  - `CommitData { sha, message, author, committer, additions, deletions, files_changed }`
  - `CommitDetail { sha, message, author, committer, diff, files }` — 详细版本
  - `CommitProvider trait: view(sha) -> Result<CommitDetail>, diff(sha) -> Result<String>, patch(sha) -> Result<String>, comment(sha, body, path, line) -> Result<()>`

- [ ] **Step 3: 写单元测试**

- [ ] **Step 4: 更新 lib.rs**

> **Commit:** `feat(core): add LabelProvider, MilestoneProvider, and CommitProvider traits (#N)`

### Task 6: crates/github — 扩展 GitHubIssueProvider/GitHubPrProvider

**Description:** 在现有 `GitHubIssueProvider` 和 `GitHubPrProvider` 上实现 Task 3 中新增的 trait 方法。通过 `gh` CLI 调用对应子命令。

**Dependencies:** Task 4（traits 定义完成）

- [ ] **Step 1: 扩展 GitHubIssueProvider**
  - `close` → `gh issue close <number> --repo <repo>`
  - `reopen` → `gh issue reopen <number> --repo <repo>`
  - `comment` → `gh issue comment <number> --body "<text>" --repo <repo>`
  - `add_labels` → `gh issue edit <number> --add-label "<labels>" --repo <repo>`
  - `remove_label` → `gh issue edit <number> --remove-label "<label>" --repo <repo>`

- [ ] **Step 2: 扩展 GitHubPrProvider**
  - `close` → `gh pr close <number> --repo <repo>`
  - `reopen` → `gh pr reopen <number> --repo <repo>`
  - `comment` → `gh pr comment <number> --body "<text>" --repo <repo>`
  - `merge` → `gh pr merge <number> [--squash|--rebase] --repo <repo>`
  - `checkout` → `gh pr checkout <number> --repo <repo>`
  - `mark_ready` → `gh pr ready <number> --repo <repo>`
  - `mark_wip` → （gh 无内置 WIP，将 `title` 加 `[WIP]` 前缀作为实现）
  - `sync_branch` → （gh 无内置 sync，通过 fetch + merge-base 实现）

- [ ] **Step 3: 写单元测试**（mock gh stderr/stdout + 错误解析）

> **Commit:** `feat(github): extend GitHubIssueProvider and GitHubPrProvider with full operations (#N)`

### Task 7: crates/github — 新增 GitHubReleaseProvider/GitHubReviewProvider

**Description:** 新建 `release.rs` 和 `review.rs`，通过 `gh` CLI 实现 release 和 review 操作。

**Dependencies:** Task 4, Task 6

- [ ] **Step 1: 新建 src/release.rs — GitHubReleaseProvider**
  - `create` → `gh release create <tag> [--title <t>] [--notes <n>] [--draft] [--prerelease]`
  - `list` → `gh release list --json <fields> [--limit N]`
  - `view` → `gh release view <tag> --json <fields>`
  - `edit` → `gh release edit <tag> [--title <t>] [--notes <n>]`
  - `upload_asset` → `gh release upload <tag> <file>`
  - `download_asset` → `gh release download <tag> [--pattern <p>]`
  - `delete` → `gh release delete <tag> --yes`

- [ ] **Step 2: 新建 src/review.rs — GitHubReviewProvider**
  - `comment` → `gh pr review <number> --comment --body "<text>"`
  - `approve` → `gh pr review <number> --approve --body "<text>"`
  - `request_changes` → `gh pr review <number> --request-changes --body "<text>"`
  - `submit_review` → `gh pr review <number> [--approve|--request-changes|--comment]`

- [ ] **Step 3: 更新 lib.rs**

- [ ] **Step 4: 写单元测试**

> **Commit:** `feat(github): add GitHubReleaseProvider and GitHubReviewProvider (#N)`

### Task 8: crates/github — 新增 GitHubAuthProvider/GitHubLabelProvider/GitHubMilestoneProvider/GitHubCommitProvider

**Description:** 新建 `auth.rs`、`label.rs`、`commit.rs`，实现剩余 4 个 Provider。

**Dependencies:** Task 5, Task 6

- [ ] **Step 1: 新建 src/auth.rs — GitHubAuthProvider**
  - `login` → `gh auth login`（交互式，subprocess 透传）
  - `logout` → `gh auth logout`
  - `status` → `gh auth status` + 解析输出
  - `token` → `gh auth token`

- [ ] **Step 2: 新建 src/label.rs — GitHubLabelProvider + GitHubMilestoneProvider**
  - `label create/list/edit/delete` → `gh label {create,list,edit,delete} --repo <repo>`
  - `milestone create/list/edit/close/reopen` → `gh api repos/<repo>/milestones`

- [ ] **Step 3: 新建 src/commit.rs — GitHubCommitProvider**
  - `view` → `gh api repos/<repo>/commits/<sha>`
  - `diff` → `gh api repos/<repo>/commits/<sha>`（通过 diff URL）
  - `patch` → `gh api repos/<repo>/commits/<sha>`（通过 patch URL）
  - `comment` → `gh api repos/<repo>/commits/<sha>/comments`

- [ ] **Step 4: 更新 lib.rs**

- [ ] **Step 5: 写单元测试**

> **Commit:** `feat(github): add Auth, Label, Milestone, and Commit providers (#N)`

### Task 9: apps/cli — 扩展 issue/pr 命令

**Description:** 扩展 `commands/issue.rs` 和 `commands/pr.rs`，新增 Phase 1 中不包含的操作。

**Dependencies:** Task 6（GitHub Provider 实现就绪）

- [ ] **Step 1: 扩展 commands/issue.rs**
  - 新增子命令: `Close { number }`, `Reopen { number }`, `Comment { number, body, body_file }`, `AddLabel { number, labels: Vec<String> }`, `RemoveLabel { number, label }`
  - `handle()` 中新增 match 分支，调用对应 provider 方法

- [ ] **Step 2: 扩展 commands/pr.rs**
  - 新增子命令: `Close { number }`, `Reopen { number }`, `Comment { number, body }`, `Merge { number, strategy }`, `Checkout { number }`, `Ready { number }`, `Wip { number }`, `Sync { number }`
  - `handle()` 中新增 match 分支

- [ ] **Step 3: 更新 resolve_body() 支持 --body-file**（Phase 1 中的 TODO）

- [ ] **Step 4: 写集成测试**
  - 每个新增子命令至少一个 help 输出测试

> **Commit:** `feat(cli): extend issue and pr commands with full operation set (#N)`

### Task 10: apps/cli — 新增 release/review/auth 命令

**Description:** 新建 `commands/release.rs`、`commands/review.rs`、`commands/auth.rs`。

**Dependencies:** Task 7, Task 9

- [ ] **Step 1: 新建 commands/release.rs**
  - `ReleaseCommand` enum: `Create`, `List`, `View`, `Edit`, `Upload`, `Download`, `Delete`
  - `handle()` 函数 → GitHubReleaseProvider

- [ ] **Step 2: 新建 commands/review.rs**
  - `ReviewCommand` enum: `Comment`, `Approve`, `RequestChanges`, `Submit`
  - `handle()` 函数 → GitHubReviewProvider

- [ ] **Step 3: 新建 commands/auth.rs**
  - `AuthCommand` enum: `Login`, `Logout`, `Status`, `Token`
  - `handle()` 函数 → GitHubAuthProvider（login/logout 交互式透传）

- [ ] **Step 4: 更新 commands/mod.rs + main.rs Commands enum**

- [ ] **Step 5: 写集成测试**

> **Commit:** `feat(cli): add release, review, and auth commands (#N)`

### Task 11: apps/cli — 新增 label/milestone/commit 命令

**Description:** 新建 `commands/label.rs` 和 `commands/commit.rs`（milestone 作为 label.rs 的子模块）。

**Dependencies:** Task 8, Task 10

- [ ] **Step 1: 新建 commands/label.rs**
  - `LabelCommand` enum + `MilestoneCommand` enum
  - `handle_label()` / `handle_milestone()` 函数

- [ ] **Step 2: 新建 commands/commit.rs**
  - `CommitCommand` enum: `View`, `Diff`, `Patch`, `Comment`
  - `handle()` 函数

- [ ] **Step 3: 更新 commands/mod.rs + main.rs Commands enum**

- [ ] **Step 4: 写集成测试**

> **Commit:** `feat(cli): add label, milestone, and commit commands (#N)`

### Task 12: skills/ — 创建核心命令层 Skills

**Description:** 新建 7 个核心命令层 Skill 文件。每个 SKILL.md 封装对应 CLI 命令的调用方式、参数说明和常见场景示例。

**Dependencies:** Task 11（所有 CLI 命令就绪）

- [ ] **Step 1: 新建 gitflow-issue/SKILL.md**
  - 封装 `gitflow issue {create,list,view,close,reopen,comment,label}` 的调用方式
  - 包含参数说明和 2-3 个典型场景

- [ ] **Step 2: 新建 gitflow-pr/SKILL.md**
  - 封装 `gitflow pr {create,list,view,close,reopen,comment,merge,checkout,ready,wip,sync}`

- [ ] **Step 3: 新建 gitflow-release/SKILL.md**
  - 封装 `gitflow release {create,list,view,edit,upload,download,delete}`

- [ ] **Step 4: 新建 gitflow-auth/SKILL.md**
  - 封装 `gitflow auth {login,logout,status,token}`

- [ ] **Step 5: 新建 gitflow-review/SKILL.md**
  - 封装 `gitflow review {comment,approve,request-changes,submit}`

- [ ] **Step 6: 新建 gitflow-label-milestone/SKILL.md**
  - 封装 `gitflow label` 和 `gitflow milestone` 命令

- [ ] **Step 7: 新建 gitflow-commit/SKILL.md**
  - 封装 `gitflow commit {view,diff,patch,comment}`

> **Commit:** `feat(skills): add core command layer skills for all resource types (#N)`

### Task 13: skills/ — 创建工作流层 Skills

**Description:** 新建 3 个工作流层 Skill 文件（gitflow-issue-create, gitflow-pr-create, gitflow-pr-review）和 gitflow-security-check。

**Dependencies:** Task 12

- [ ] **Step 1: 新建 gitflow-issue-create/SKILL.md**
  - 引导用户填写 Issue 模板（标题、描述、标签、里程碑）
  - 调用 `gitflow issue create` 创建
  - 输出 Issue URL

- [ ] **Step 2: 新建 gitflow-pr-create/SKILL.md**
  - 检查当前分支 + 变更
  - 引导填写 PR 标题和描述（conventional commits 格式）
  - 检查 base branch 是否最新
  - 调用 `gitflow pr create` 创建

- [ ] **Step 3: 新建 gitflow-pr-review/SKILL.md**
  - 6 维度审查清单（代码正确性、安全性、性能、可维护性、测试覆盖、文档）
  - 调用 `gitflow pr view` 获取 PR 详情
  - 调用 `gitflow review {approve,request-changes,comment}` 提交审查结果

- [ ] **Step 4: 新建 gitflow-security-check/SKILL.md**
  - 安全审计 checklist（密钥硬编码检查、依赖漏洞、输入验证）
  - 调用 `cargo audit` 进行检查

> **Commit:** `feat(skills): add workflow layer skills (#N)`

### Task 14: 扩展冒烟测试 + scripts

**Description:** 扩展 `scripts/smoke-test.sh`，覆盖新增命令的只读操作。更新 Makefile 添加 skills 安装 target。

**Dependencies:** Task 11

- [ ] **Step 1: 扩展 smoke-test.sh**
  - 新增: `issue comment view`、`release list`、`label list`、`milestone list`
  - 对写操作添加 `--dry-run` 或 `--help` 验证

- [ ] **Step 2: 新增 Makefile target**
  - `make install-skills`: 复制 `skills/` 到 `~/.claude/skills/`

- [ ] **Step 3: 更新 specs/index.md**

> **Commit:** `test: extend smoke test for Phase 2 commands (#N)`

### Task 15: 终检 — clippy + fmt + build + test

**Description:** 运行全 workspace 的构建、测试、lint、格式化检查。

**Dependencies:** Task 14

- [ ] **Step 1: `cargo build --workspace`**
- [ ] **Step 2: `cargo test --workspace`**
- [ ] **Step 3: `cargo clippy --workspace --all-targets -- -D warnings`**
- [ ] **Step 4: `cargo +nightly fmt -- --check`**
- [ ] **Step 5: 修复所有错误和 warning**
- [ ] **Step 6: 最终确认全部通过**

> **Commit:** `chore: final lint and formatting pass for Phase 2 (#N)`

### Task 16: 收尾 — 本地合并后关闭 Issue

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
