# GitCode 适配器全面修复实施计划（Issue #90）

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 修复 Issue #90 报告的全部 5 个 GitCode 适配器缺陷，使 GitCode 平台的 issue 标签与 PR 工作流（create/list/view/merge/comment）达到与 GitHub/GitLab 同等的可用性。

**Architecture:** 根因是 `crates/gitcode/src/pr.rs` 与评论解析代码按 `gh` CLI 的输出假设编写（camelCase 字段、`author` 键、`--json <fields>` 字段选择器），而 gitcode CLI v0.6.1 的实际行为是：snake_case 字段、`user` 键、嵌套 `head/base` 对象、`html_url`、布尔 `--json` 标志（其后跟随的字段字符串会变成多余的位置参数）。修复采用同 crate `issue.rs` 已验证的模式：引入平台响应中间类型（`PrApiResponse` 等）+ `From` 映射到 core 类型；标签操作改用 gitcode 专用的 `issue label` 子命令；合并策略映射到 `--method`。

**Tech Stack:** Rust 2024、tokio、serde/serde_json、async-trait、chrono、cargo-nextest、clippy pedantic。

## Global Constraints

- Rust 2024 edition，工具链以 `rust-toolchain.toml` 为准；`#![forbid(unsafe_code)]` 已全局启用。
- 生产代码禁止 `unwrap()` / `expect()`；可失败操作返回 `Result<T>`，错误统一为 `CoreError`（`Platform` / `Serialization` 变体）。
- 所有新增类型与公开函数必须有文档注释；涉及错误的函数包含 `# Errors` 小节。
- 测试命名 `test_should_<expected_behavior>`；每个缺陷先写复现测试（RED）再修复（GREEN）。
- 提交信息遵循 conventional commits（`fix(gitcode):` / `test(gitcode):`）。
- gitcode CLI 目标版本：v0.6.1（本计划的全部 CLI 行为均已对该版本实测验证，二进制位于 `~/Library/Python/3.9/bin/gitcode`，`crate::gitcode_binary()` 可自动发现）。
- 实测确认的 gitcode v0.6.1 接口事实（修复依据，勿再猜测）：
  - `gitcode issue label <number> --add <a,b> -R <repo>` / `--remove <label>` / `--list`（专用子命令，存在）
  - `gitcode issue edit` **没有** `--add-label` / `--remove-label` flag（#90 根因之一）
  - `--json` 是**布尔标志**，不接受字段列表；`pr view` 只接受 1 个位置参数（`<number>`）
  - `gitcode pr merge <number> --method merge|squash|rebase --yes`（支持合并策略）
  - `gitcode pr close/reopen` 有 `--yes`（跳过确认提示）与布尔 `--json`
  - PR JSON（list/view 同构）顶层键：`additions, assignees, base, body, changed_files, closed_at, comments, commits, created_at, deletions, description, diff_url, draft, head, html_url, id, labels, mergeable, mergeable_state, merged, merged_at, milestone, number, patch_url, requested_reviewers, state, title, updated_at, user`
  - `user` 对象键：`avatar_url, created_at, email, html_url, id, login, name`（`id` 为字符串）
  - `head` / `base` 对象含 `label, ref, sha, repo`
  - `state` 取值含 `open` / `closed` / `merged`（core `State` 已有 `merged` → `Closed` 别名）
  - 时间戳为带偏移的 RFC3339（如 `2026-07-30T12:40:46+08:00`）
  - 评论 JSON（list 实测）键：`body, created_at, id, updated_at, user`（**无 `author`**）

---

## 文件结构

| 文件 | 动作 | 职责 |
|------|------|------|
| `crates/gitcode/src/runner.rs` | 修改 | 新增 `RecordingMockRunner`（#[cfg(test)]）：在返回预设输出的同时记录每次调用的参数，供回归测试断言 CLI 调用形态 |
| `crates/gitcode/src/pr.rs` | 修改 | 删除 `PR_FIELDS` 常量；新增 `PrApiResponse` / `PrUserApi` / `PrBranchApi` 中间类型与 `From<PrApiResponse> for PrData`；重写 create/list/view/close/reopen 的参数构造与解析；merge 映射 `--method`；pr comment 删除多余字段参数并改用容错评论映射 |
| `crates/gitcode/src/issue.rs` | 修改 | `add_labels` / `remove_label` 改用 `issue label` 子命令；`comment` 的 `CommentApiResponse` 增加 `user` 对象与多格式时间戳容错 |
| `crates/gitcode/tests/fixtures/pr_list_gitcode_v0.6.1.json` | 新建 | gitcode v0.6.1 `pr list --json` 真实捕获（精简），契约测试夹具 |
| `crates/gitcode/src/pr.rs` 内 `mod contract_tests` | 新建 | 契约测试：真实夹具 → `list()` → `PrData` 全链路反序列化回归（crate 内模块，因 mock runner 为 `#[cfg(test)]` 私有） |

---

### Task 1: 参数记录型 Mock Runner（测试基础设施）

**背景：** 现有 `MockCommandRunner` 忽略传入参数，无法断言"适配器到底向 gitcode CLI 传了什么"。#90 的 5 个缺陷中有 3 个（add-label flag、pr view 多余位置参数、merge strategy）属于**调用形态错误**，必须有参数级回归测试。

**Files:**
- Modify: `crates/gitcode/src/runner.rs:181`（`SequencedMockCommandRunner` 定义之前插入）
- Test: `crates/gitcode/src/runner.rs` 内 `mod tests`

**Interfaces:**
- Consumes: 无
- Produces: `RecordingMockRunner::success(stdout: &str) -> Self`、`RecordingMockRunner::failure(stderr: &str, code: i32) -> Self`、`RecordingMockRunner::calls(&self) -> Vec<Vec<String>>`（每次 `run` 调用的完整 argv 快照）

- [ ] **Step 1: 写失败测试**

在 `crates/gitcode/src/runner.rs` 的 `mod tests` 末尾追加：

```rust
#[tokio::test]
async fn test_should_record_arguments_in_recording_runner() {
    let runner = RecordingMockRunner::success("{}");
    let output = runner
        .run("gitcode", &["pr", "view", "20", "--json"])
        .await
        .expect("should succeed");
    assert!(output.status.success());

    let calls = runner.calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0], vec!["pr", "view", "20", "--json"]);
}

#[tokio::test]
async fn test_should_record_multiple_calls_in_order() {
    let runner = RecordingMockRunner::success("ok");
    runner.run("gitcode", &["issue", "label", "1", "--add", "bug"]).await.expect("first");
    runner.run("gitcode", &["issue", "label", "1", "--remove", "bug"]).await.expect("second");

    let calls = runner.calls();
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[1], vec!["issue", "label", "1", "--remove", "bug"]);
}
```

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo nextest run -p gf-gitcode runner::tests::test_should_record`
Expected: 编译错误 — `RecordingMockRunner` 未定义。

- [ ] **Step 3: 实现 RecordingMockRunner**

在 `crates/gitcode/src/runner.rs` 中 `SequencedMockCommandRunner` 定义之前插入：

```rust
/// Mock implementation that records every call's arguments while returning
/// a preconfigured result.
///
/// Used by regression tests that must assert the exact CLI invocation shape
/// (e.g. which flags the adapter passes to `gitcode`).
#[cfg(test)]
#[derive(Debug, Clone)]
pub struct RecordingMockRunner {
    inner: MockCommandRunner,
    calls: std::sync::Arc<std::sync::Mutex<Vec<Vec<String>>>>,
}

#[cfg(test)]
impl RecordingMockRunner {
    /// Create a recording runner that returns success with the given stdout.
    #[must_use]
    pub fn success(stdout: &str) -> Self {
        Self {
            inner: MockCommandRunner::success(stdout),
            calls: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    /// Create a recording runner that returns failure with the given stderr.
    #[must_use]
    pub fn failure(stderr: &str, code: i32) -> Self {
        Self {
            inner: MockCommandRunner::failure(stderr, code),
            calls: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }

    /// Snapshot of all recorded calls; each entry is the argv (without program).
    #[must_use]
    pub fn calls(&self) -> Vec<Vec<String>> {
        self.calls
            .lock()
            .expect("RecordingMockRunner mutex poisoned")
            .clone()
    }
}

#[cfg(test)]
#[async_trait::async_trait]
impl CommandRunner for RecordingMockRunner {
    async fn run(&self, program: &str, args: &[&str]) -> std::io::Result<CommandOutput> {
        self.calls
            .lock()
            .expect("RecordingMockRunner mutex poisoned")
            .push(args.iter().map(|s| (*s).to_owned()).collect());
        self.inner.run(program, args).await
    }

    async fn run_with_stdin(
        &self,
        program: &str,
        args: &[&str],
        stdin_data: &[u8],
    ) -> std::io::Result<CommandOutput> {
        self.calls
            .lock()
            .expect("RecordingMockRunner mutex poisoned")
            .push(args.iter().map(|s| (*s).to_owned()).collect());
        self.inner.run_with_stdin(program, args, stdin_data).await
    }
}
```

- [ ] **Step 4: 运行测试确认通过**

Run: `cargo nextest run -p gf-gitcode runner::tests::test_should_record`
Expected: 2 tests PASS。

- [ ] **Step 5: 提交**

```bash
git add crates/gitcode/src/runner.rs
git commit -m "test(gitcode): add argument-recording mock runner for CLI shape regression tests"
```

---

### Task 2: PR 响应中间类型与映射（#90 子问题 2、3 的根因修复）

**背景：** gitcode 的 PR JSON 使用 `user`（非 `author`）、嵌套 `head.ref`/`base.ref`、`html_url`、snake_case 时间戳。直接反序列化进 camelCase 的 core `PrData` 必然报 `missing field 'author'`。修复模式与 `issue.rs` 的 `IssueApiResponse` 完全一致。

**Files:**
- Modify: `crates/gitcode/src/pr.rs:1-25`（imports 与常量区）
- Test: `crates/gitcode/src/pr.rs` 内 `mod tests`

**Interfaces:**
- Consumes: `gitflow_cli_core::pr::PrData`、`gitflow_cli_core::types::{State, UserSummary}`
- Produces: `PrApiResponse`（私有，`serde::Deserialize`）、`impl From<PrApiResponse> for PrData`；后续 Task 3 的 `create/list/view/close/reopen` 将 `serde_json::from_slice::<PrApiResponse>` 后 `.into()`

- [ ] **Step 1: 写失败测试（真实捕获夹具）**

在 `pr.rs` 的 `mod tests` 顶部追加导入与夹具函数，并追加测试：

```rust
// mod tests 顶部追加：
use gitflow_cli_core::types::UserSummary;

/// gitcode CLI v0.6.1 `pr list/view --json` 的真实输出结构（2026-07-31 实测捕获，已精简）。
fn real_gitcode_pr_json() -> &'static str {
    r#"{
        "id": 8957463,
        "number": 52,
        "title": "test(badge): 引擎规则函数测试覆盖",
        "body": "## Summary\n\nCloses #88",
        "description": "## Summary\n\nCloses #88",
        "state": "merged",
        "html_url": "https://gitcode.com/byx-darwin/go-beniofit/merge_requests/52",
        "diff_url": "",
        "patch_url": "",
        "draft": false,
        "merged": true,
        "merged_at": "2026-07-30T13:23:13+08:00",
        "created_at": "2026-07-30T12:40:46+08:00",
        "updated_at": "2026-07-30T13:23:13+08:00",
        "user": {
            "id": "66767cd4096c81780c61bf07",
            "login": "byx-darwin",
            "name": "baoyx",
            "email": "",
            "avatar_url": "https://cdn-img.gitcode.com/avatar.png",
            "html_url": "https://gitcode.com/byx-darwin",
            "created_at": ""
        },
        "head": {
            "label": "test/88-engine-rule-coverage",
            "ref": "test/88-engine-rule-coverage",
            "sha": "8f1d3f31d7ee598a16f40fcac55b86154122c93c"
        },
        "base": {
            "label": "master",
            "ref": "master",
            "sha": "bba7d724c8c73531acf1dca5f639b2a273c26eae"
        },
        "labels": [],
        "assignees": [],
        "additions": 120,
        "deletions": 3,
        "changed_files": 1,
        "commits": 2,
        "comments": 0,
        "mergeable": true,
        "mergeable_state": "can_be_merged",
        "milestone": null,
        "closed_at": "2026-07-30T13:23:13+08:00",
        "requested_reviewers": []
    }"#
}

#[test]
fn test_should_map_real_gitcode_pr_response_to_pr_data() {
    let api: PrApiResponse =
        serde_json::from_str(real_gitcode_pr_json()).expect("valid gitcode v0.6.1 PR JSON");
    let pr: PrData = api.into();

    assert_eq!(pr.number, 52);
    assert_eq!(pr.title, "test(badge): 引擎规则函数测试覆盖");
    assert_eq!(pr.state, State::Closed, "merged 必须映射为 Closed");
    assert!(!pr.draft);
    assert_eq!(pr.author, UserSummary {
        login: "byx-darwin".into(),
        id: "66767cd4096c81780c61bf07".into(),
    });
    assert_eq!(pr.base_branch, "master");
    assert_eq!(pr.head_branch, "test/88-engine-rule-coverage");
    assert_eq!(pr.url, "https://gitcode.com/byx-darwin/go-beniofit/merge_requests/52");
    assert_eq!(pr.created_at.to_rfc3339(), "2026-07-30T04:40:46+00:00");
}

#[test]
fn test_should_map_open_pr_with_minimal_gitcode_fields() {
    let json = r#"{
        "id": 1,
        "number": 7,
        "title": "New work",
        "state": "open",
        "html_url": "https://gitcode.com/o/r/merge_requests/7",
        "draft": true,
        "user": {"id": "u1", "login": "dev"},
        "head": {"ref": "feature/x"},
        "base": {"ref": "main"}
    }"#;
    let api: PrApiResponse = serde_json::from_str(json).expect("minimal gitcode PR JSON");
    let pr: PrData = api.into();

    assert_eq!(pr.state, State::Open);
    assert!(pr.draft);
    assert_eq!(pr.body, None);
    assert_eq!(pr.head_branch, "feature/x");
    assert_eq!(pr.base_branch, "main");
    assert_eq!(pr.author.login, "dev");
}
```

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo nextest run -p gf-gitcode pr::tests::test_should_map`
Expected: 编译错误 — `PrApiResponse` 未定义。

- [ ] **Step 3: 实现中间类型与映射**

在 `pr.rs` 的 imports 区追加 `use chrono::{DateTime, Utc};`、在 `use gitflow_cli_core::...` 中追加 `types::UserSummary`、追加 `use serde::Deserialize;`，然后在 `PR_FIELDS` 常量位置之后（Task 3 会删除该常量，本步骤先不动它）插入：

```rust
/// gitcode CLI v0.6.x `pr list/view/create --json` 的响应类型。
///
/// 字段命名与 `gh pr` 不同：snake_case、`user` 而非 `author`、
/// 分支信息嵌套在 `head`/`base` 对象的 `ref` 字段、URL 为 `html_url`。
/// 通过 [`From<PrApiResponse> for PrData`] 映射为 core 统一类型。
#[derive(Debug, Clone, Deserialize)]
struct PrApiResponse {
    number: u64,
    title: String,
    #[serde(default)]
    body: Option<String>,
    #[serde(default)]
    state: Option<String>,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    user: Option<PrUserApi>,
    #[serde(default)]
    head: Option<PrBranchApi>,
    #[serde(default)]
    base: Option<PrBranchApi>,
    #[serde(default)]
    created_at: Option<String>,
    #[serde(default)]
    updated_at: Option<String>,
    #[serde(default)]
    html_url: Option<String>,
}

/// gitcode PR JSON 中 `user` 对象的最小字段集。
#[derive(Debug, Clone, Deserialize)]
struct PrUserApi {
    #[serde(default)]
    login: String,
    #[serde(default)]
    id: Option<String>,
}

/// gitcode PR JSON 中 `head`/`base` 对象的最小字段集。
#[derive(Debug, Clone, Deserialize)]
struct PrBranchApi {
    #[serde(default, rename = "ref")]
    branch_ref: String,
}

impl From<PrApiResponse> for PrData {
    fn from(api: PrApiResponse) -> Self {
        let parse_time = |s: Option<String>| {
            s.and_then(|v| DateTime::parse_from_rfc3339(&v).ok())
                .map_or_else(Utc::now, |dt| dt.with_timezone(&Utc))
        };
        Self {
            number: api.number,
            title: api.title,
            body: api.body,
            state: match api.state.as_deref() {
                Some("closed") | Some("merged") => State::Closed,
                _ => State::Open,
            },
            draft: api.draft,
            author: api.user.map_or(
                UserSummary {
                    login: "unknown".into(),
                    id: String::new(),
                },
                |u| UserSummary {
                    login: u.login,
                    id: u.id.unwrap_or_default(),
                },
            ),
            base_branch: api
                .base
                .map_or_else(String::new, |b| b.branch_ref),
            head_branch: api
                .head
                .map_or_else(String::new, |h| h.branch_ref),
            created_at: parse_time(api.created_at),
            updated_at: parse_time(api.updated_at),
            url: api.html_url.unwrap_or_default(),
        }
    }
}
```

- [ ] **Step 4: 运行测试确认通过**

Run: `cargo nextest run -p gf-gitcode pr::tests::test_should_map`
Expected: 2 tests PASS。

- [ ] **Step 5: 提交**

```bash
git add crates/gitcode/src/pr.rs
git commit -m "fix(gitcode): add PrApiResponse mapping for real gitcode v0.6.x PR schema

gitcode PR JSON uses snake_case keys, 'user' instead of 'author',
nested head/base objects and 'html_url'. Directly deserializing into
camelCase PrData failed with 'missing field author' (Issue #90)."
```

---

### Task 3: 重写 PR 命令调用形态（#90 子问题 4：pr view 多余位置参数）

**背景：** `--json` 在 gitcode 是布尔标志。现有代码把 gh 风格的字段列表 `PR_FIELDS` 跟在 `--json` 后，`pr view` 将其当成第 2 个位置参数 → `accepts 1 arg(s), received 2`。`pr close/reopen` 同样携带多余参数且缺少 `--yes`（非交互环境会卡在确认提示）。

**Files:**
- Modify: `crates/gitcode/src/pr.rs:20-22`（删除 `PR_FIELDS`）、`76-277`（create/list/view/close/reopen 方法体）、`405-454`（mark_ready/mark_wip 依赖的 view 已随之修复，无需单独改）
- Test: `crates/gitcode/src/pr.rs` 内 `mod tests`

**Interfaces:**
- Consumes: `PrApiResponse`（Task 2）、`RecordingMockRunner`（Task 1）
- Produces: 修正后的 `create/list/view/close/reopen`；对外行为契约：view 的 argv 恰为 `["pr","view",<n>,"--repo",<repo>,"--json"]`；close/reopen 追加 `"--yes"`

- [ ] **Step 1: 写失败测试（调用形态回归）**

在 `mod tests` 追加：

```rust
use crate::runner::RecordingMockRunner;

#[tokio::test]
async fn test_should_not_pass_field_list_to_pr_view() {
    let runner = RecordingMockRunner::success(real_gitcode_pr_json());
    let provider = GitCodePrProvider::with_runner("octocat/hello-world", runner.clone());

    let pr = provider.view(20).await.expect("view should parse real schema");

    assert_eq!(pr.number, 52);
    let calls = runner.calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(
        calls[0],
        vec!["pr", "view", "20", "--repo", "octocat/hello-world", "--json"],
        "gitcode --json 是布尔标志，不得携带字段列表位置参数"
    );
}

#[tokio::test]
async fn test_should_pass_yes_flag_to_pr_close() {
    let runner = RecordingMockRunner::success(real_gitcode_pr_json());
    let provider = GitCodePrProvider::with_runner("o/r", runner.clone());

    provider.close(9).await.expect("close should succeed");

    let args = &runner.calls()[0];
    assert!(args.contains(&"--yes".to_string()), "close 必须跳过确认提示");
    assert!(!args.windows(2).any(|w| w[0] == "--json" && w[1] != "--yes".to_string() && !w[1].starts_with('-')),
        "--json 后不得跟随字段列表");
}

#[tokio::test]
async fn test_should_pass_limit_flag_to_pr_list() {
    let runner = RecordingMockRunner::success(&format!("[{}]", real_gitcode_pr_json()));
    let provider = GitCodePrProvider::with_runner("o/r", runner.clone());

    let prs = provider
        .list(ListPrArgs { state: Some(State::Open), limit: Some(5) })
        .await
        .expect("list should succeed");

    assert_eq!(prs.len(), 1);
    let args = &runner.calls()[0];
    assert!(args.contains(&"--limit".to_string()));
    assert!(args.contains(&"5".to_string()));
    assert!(args.contains(&"--state".to_string()));
    assert!(args.contains(&"open".to_string()));
}
```

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo nextest run -p gf-gitcode pr::tests::test_should_not_pass_field_list pr::tests::test_should_pass_yes pr::tests::test_should_pass_limit`
Expected: FAIL — view 断言失败（argv 含 `PR_FIELDS` 字符串）；解析失败（`missing field author`）。

- [ ] **Step 3: 重写五个方法并删除 PR_FIELDS**

删除 `pr.rs` 中的 `PR_FIELDS` 常量定义（原第 20-22 行），将方法体替换为：

```rust
    async fn create(&self, args: CreatePrArgs) -> Result<PrData> {
        let binary = crate::gitcode_binary();
        let mut cmd_args: Vec<&str> = vec![
            "pr",
            "create",
            "--repo",
            args.repo.as_deref().unwrap_or(&self.repo),
            "--title",
            &args.title,
            "--head",
            &args.head,
            "--base",
            &args.base,
            "--json",
        ];

        if let Some(body) = &args.body {
            cmd_args.push("--body");
            cmd_args.push(body);
        }

        if args.draft {
            cmd_args.push("--draft");
        }

        debug!(
            repo = %self.repo,
            title = %args.title,
            head = %args.head,
            base = %args.base,
            "spawning `gitcode pr create`"
        );

        let output = self
            .runner
            .run(&binary, &cmd_args)
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gitcode: {e}")))?;

        if !output.status.success() {
            let gitcode_err = parse_gitcode_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gitcode_err}")));
        }

        let api: PrApiResponse =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(api.into())
    }

    async fn list(&self, args: ListPrArgs) -> Result<Vec<PrData>> {
        let binary = crate::gitcode_binary();
        let mut cmd_args: Vec<&str> = vec!["pr", "list", "--repo", &self.repo, "--json"];

        if let Some(state) = &args.state {
            cmd_args.push("--state");
            cmd_args.push(match state {
                State::Open => "open",
                State::Closed => "closed",
            });
        }

        let limit_str = args.limit.map(|limit| limit.to_string());
        if let Some(ref limit) = limit_str {
            cmd_args.push("--limit");
            cmd_args.push(limit);
        }

        debug!(repo = %self.repo, "spawning `gitcode pr list`");

        let output = self
            .runner
            .run(&binary, &cmd_args)
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gitcode: {e}")))?;

        if !output.status.success() {
            let gitcode_err = parse_gitcode_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gitcode_err}")));
        }

        let apis: Vec<PrApiResponse> =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(apis.into_iter().map(PrData::from).collect())
    }

    async fn view(&self, number: u64) -> Result<PrData> {
        let binary = crate::gitcode_binary();
        let number_str = number.to_string();
        debug!(repo = %self.repo, number, "spawning `gitcode pr view`");

        let output = self
            .runner
            .run(
                &binary,
                &["pr", "view", &number_str, "--repo", &self.repo, "--json"],
            )
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gitcode: {e}")))?;

        if !output.status.success() {
            let gitcode_err = parse_gitcode_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gitcode_err}")));
        }

        let api: PrApiResponse =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(api.into())
    }
```

`close` 与 `reopen` 按同样模式重写：argv 为 `["pr", "close", &number_str, "--repo", &self.repo, "--yes", "--json"]`（reopen 将 `"close"` 换为 `"reopen"`），解析走 `PrApiResponse`，并同步更新两个方法的文档注释（将 `gc pr close <number> --repo <repo> --json <fields>` 改为 `gitcode pr close <number> --repo <repo> --yes --json`，删去 `<fields>` 描述）。

- [ ] **Step 4: 运行测试确认通过**

Run: `cargo nextest run -p gf-gitcode pr::tests`
Expected: 全部 PASS（旧测试中直接反序列化 gh 风格 JSON 到 `PrData` 的用例仍然有效——它们测的是 core 类型本身，不经过 `PrApiResponse`）。

- [ ] **Step 5: 提交**

```bash
git add crates/gitcode/src/pr.rs
git commit -m "fix(gitcode): drop gh-style field selector from pr commands (Issue #90)

gitcode --json is a boolean flag; the trailing field list became an
extra positional argument, breaking 'pr view' with 'accepts 1 arg(s),
received 2'. Also pass --yes to pr close/reopen for non-interactive use
and parse all responses through PrApiResponse."
```

---

### Task 4: 标签操作改用 `issue label` 子命令（#90 子问题 1）

**背景：** gitcode v0.6.1 的 `issue edit` 没有 `--add-label`/`--remove-label`；标签增删的正确接口是 `gitcode issue label <number> --add <a,b>` / `--remove <label>`。保留"缺失标签自动创建后重试"机制。

**Files:**
- Modify: `crates/gitcode/src/issue.rs:495-600`（`add_labels`、`remove_label` 及其文档注释）
- Test: `crates/gitcode/src/issue.rs` 内 `mod tests`

**Interfaces:**
- Consumes: `RecordingMockRunner`（Task 1）、`ensure_label_exists`（issue.rs:230 既有）、`extract_missing_labels_from_error`（issue.rs:607 既有，保留）
- Produces: `add_labels` 的 argv 恰为 `["issue","label",<n>,"--add",<逗号连接>,"-R",<repo>]`；`remove_label` 的 argv 恰为 `["issue","label",<n>,"--remove",<label>,"-R",<repo>]`

- [ ] **Step 1: 写失败测试**

在 `issue.rs` 的 `mod tests` 追加（导入 `RecordingMockRunner`）：

```rust
use crate::runner::RecordingMockRunner;

#[tokio::test]
async fn test_should_invoke_issue_label_subcommand_for_add_labels() {
    let runner = RecordingMockRunner::success("");
    let provider = GitCodeIssueProvider::with_runner("o/r", runner.clone());

    provider
        .add_labels(54, &["type:bug".to_string(), "priority:high".to_string()])
        .await
        .expect("add_labels should succeed");

    let calls = runner.calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(
        calls[0],
        vec!["issue", "label", "54", "--add", "type:bug,priority:high", "-R", "o/r"],
        "gitcode v0.6.1 的 issue edit 没有 --add-label flag（Issue #90）"
    );
}

#[tokio::test]
async fn test_should_return_ok_without_any_call_for_empty_labels() {
    let runner = RecordingMockRunner::success("");
    let provider = GitCodeIssueProvider::with_runner("o/r", runner.clone());

    provider.add_labels(1, &[]).await.expect("empty labels is a no-op");

    assert!(runner.calls().is_empty());
}

#[tokio::test]
async fn test_should_invoke_issue_label_subcommand_for_remove_label() {
    let runner = RecordingMockRunner::success("");
    let provider = GitCodeIssueProvider::with_runner("o/r", runner.clone());

    provider.remove_label(54, "triage:done").await.expect("remove should succeed");

    assert_eq!(
        runner.calls()[0],
        vec!["issue", "label", "54", "--remove", "triage:done", "-R", "o/r"]
    );
}

#[tokio::test]
async fn test_should_auto_create_missing_label_and_retry_via_issue_label() {
    // 1. issue label --add 失败，报告标签缺失
    // 2. label create 成功（自动创建）
    // 3. issue label --add 重试成功
    let runner = SequencedMockCommandRunner::from_results(&[
        (false, "HTTP 404: 'type:new' not found"),
        (true, r#"{"name": "type:new", "color": "ededed"}"#),
        (true, ""),
    ]);
    let provider = GitCodeIssueProvider::with_runner("o/r", runner);

    provider
        .add_labels(18, &["type:new".to_string()])
        .await
        .expect("should recover by auto-creating the label");
}
```

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo nextest run -p gf-gitcode issue::tests::test_should_invoke_issue_label issue::tests::test_should_return_ok_without issue::tests::test_should_auto_create_missing_label_and_retry_via`
Expected: FAIL — 当前 argv 为 `["issue","edit","54","-R","o/r","--add-label",...]`。

- [ ] **Step 3: 重写 add_labels / remove_label**

替换 `issue.rs` 中 `add_labels`（原 495-563 行）与 `remove_label`（原 565-600 行）的实现与文档注释：

```rust
    /// 为指定 Issue 添加一个或多个标签。
    ///
    /// 调用 `gitcode issue label <number> --add <labels> -R <repo>` 添加标签
    ///（逗号分隔的 `--add` 是 gitcode v0.6.x 的专用标签子命令；`issue edit`
    /// 不支持 gh 风格的 `--add-label` flag）。`labels` 为空时不进行任何调用。
    ///
    /// # 自动创建缺失标签
    ///
    /// 当添加因标签不存在而失败时，本方法会自动调用 `gitcode label create`
    /// 创建缺失的标签（默认颜色 `ededed`），然后重试一次。
    ///
    /// # Errors
    ///
    /// 当 Issue 不存在、标签创建失败或 `gitcode` CLI 调用失败时返回错误。
    async fn add_labels(&self, number: u64, labels: &[String]) -> Result<()> {
        if labels.is_empty() {
            return Ok(());
        }

        let binary = crate::gitcode_binary();
        let number_str = number.to_string();
        let joined = labels.join(",");
        debug!(
            repo = %self.repo,
            number,
            label_count = labels.len(),
            "spawning `gitcode issue label --add`"
        );

        let cmd_args: Vec<&str> =
            vec!["issue", "label", &number_str, "--add", &joined, "-R", &self.repo];

        let output = self
            .runner
            .run(&binary, &cmd_args)
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gitcode: {e}")))?;

        if output.status.success() {
            return Ok(());
        }

        // Auto-create missing labels and retry once.
        let missing = extract_missing_labels_from_error(&output.stderr);
        if missing.is_empty() {
            let gitcode_err = parse_gitcode_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gitcode_err}")));
        }

        debug!(
            repo = %self.repo,
            missing_count = missing.len(),
            "auto-creating missing label(s) before retry"
        );

        for label in &missing {
            self.ensure_label_exists(label).await?;
        }

        let retry_output =
            self.runner.run(&binary, &cmd_args).await.map_err(|e| {
                CoreError::Platform(format!("Failed to spawn gitcode on retry: {e}"))
            })?;

        if !retry_output.status.success() {
            let gitcode_err = parse_gitcode_error(&retry_output.stderr);
            return Err(CoreError::Platform(format!("{gitcode_err}")));
        }

        Ok(())
    }

    /// 从指定 Issue 移除一个标签。
    ///
    /// 调用 `gitcode issue label <number> --remove <label> -R <repo>` 移除标签。
    ///
    /// # Errors
    ///
    /// 当 Issue 不存在、标签未附加到该 Issue 或 `gitcode` CLI 调用失败时返回错误。
    async fn remove_label(&self, number: u64, label: &str) -> Result<()> {
        let binary = crate::gitcode_binary();
        let number_str = number.to_string();
        debug!(repo = %self.repo, number, label, "spawning `gitcode issue label --remove`");

        let output = self
            .runner
            .run(
                &binary,
                &["issue", "label", &number_str, "--remove", label, "-R", &self.repo],
            )
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gitcode: {e}")))?;

        if !output.status.success() {
            let gitcode_err = parse_gitcode_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gitcode_err}")));
        }

        Ok(())
    }
```

- [ ] **Step 4: 运行测试确认通过**

Run: `cargo nextest run -p gf-gitcode issue::tests`
Expected: 全部 PASS。旧测试 `test_should_return_platform_error_when_gc_fails_for_add_labels` 与 `test_should_auto_create_label_and_retry_on_add_labels` 若不兼容新 argv（它们用 MockCommandRunner/SequencedMockCommandRunner，不校验 argv，应仍通过）；若失败则按新调用序列调整夹具，不改回旧实现。

- [ ] **Step 5: 提交**

```bash
git add crates/gitcode/src/issue.rs
git commit -m "fix(gitcode): use 'issue label' subcommand for add/remove labels (Issue #90)

gitcode v0.6.1 'issue edit' has no --add-label/--remove-label flags;
label mutation lives in the dedicated 'gitcode issue label' subcommand
(--add a,b / --remove x). Auto-create-missing-label retry is preserved."
```

---

### Task 5: 合并策略映射 `--method`（#90 子问题 5）

**背景：** gitcode v0.6.1 `pr merge` 支持 `--method merge|squash|rebase`。现有代码警告"不支持"并丢弃 strategy——与平台真实能力不符。

**Files:**
- Modify: `crates/gitcode/src/pr.rs:322-365`（`merge` 方法与文档注释）
- Test: `crates/gitcode/src/pr.rs` 内 `mod tests`

**Interfaces:**
- Consumes: `MergeStrategy::{Merge,Squash,Rebase}`（core::types）、`RecordingMockRunner`
- Produces: 传入 `Some(Squash)` 时 argv 含 `["--method","squash"]`；`None` 时不携带 `--method`

- [ ] **Step 1: 写失败测试**

```rust
#[tokio::test]
async fn test_should_map_squash_strategy_to_method_flag() {
    let runner = RecordingMockRunner::success("Merged pull request !52");
    let provider = GitCodePrProvider::with_runner("o/r", runner.clone());

    let result = provider.merge(52, Some(MergeStrategy::Squash)).await.expect("merge");

    assert!(result.merged);
    let args = &runner.calls()[0];
    let method_pos = args.iter().position(|a| a == "--method").expect("--method must be passed");
    assert_eq!(args[method_pos + 1], "squash");
}

#[tokio::test]
async fn test_should_map_all_merge_strategies() {
    for (strategy, expected) in [
        (MergeStrategy::Merge, "merge"),
        (MergeStrategy::Squash, "squash"),
        (MergeStrategy::Rebase, "rebase"),
    ] {
        let runner = RecordingMockRunner::success("done");
        let provider = GitCodePrProvider::with_runner("o/r", runner.clone());
        provider.merge(1, Some(strategy)).await.expect("merge");
        let args = &runner.calls()[0];
        let pos = args.iter().position(|a| a == "--method").expect("--method");
        assert_eq!(args[pos + 1], expected);
    }
}

#[tokio::test]
async fn test_should_omit_method_flag_when_no_strategy() {
    let runner = RecordingMockRunner::success("done");
    let provider = GitCodePrProvider::with_runner("o/r", runner.clone());

    provider.merge(1, None).await.expect("merge");

    assert!(!runner.calls()[0].contains(&"--method".to_string()));
}
```

测试顶部确保 `use gitflow_cli_core::types::MergeStrategy;`（已在文件顶层导入，`mod tests` 通过 `use super::*` 继承）。

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo nextest run -p gf-gitcode pr::tests::test_should_map_squash pr::tests::test_should_map_all pr::tests::test_should_omit_method`
Expected: FAIL — 当前实现不传 `--method`。

- [ ] **Step 3: 重写 merge**

替换 `merge` 方法及其文档注释：

```rust
    /// 合并指定编号的 PR。
    ///
    /// 调用 `gitcode pr merge <number> --repo <repo> --yes [--method <strategy>]`
    /// 合并 PR。`strategy` 映射到 gitcode 的 `--method` 参数
    ///（`merge` / `squash` / `rebase`）；未指定时使用平台默认策略。
    ///
    /// # Errors
    ///
    /// 当 PR 不存在、存在冲突无法合并或 `gitcode` CLI 调用失败时返回错误。
    async fn merge(&self, number: u64, strategy: Option<MergeStrategy>) -> Result<MergeResult> {
        let binary = crate::gitcode_binary();
        let number_str = number.to_string();
        let mut cmd_args: Vec<&str> =
            vec!["pr", "merge", &number_str, "--repo", &self.repo, "--yes"];

        let strategy_value;
        if let Some(strategy) = strategy {
            strategy_value = match strategy {
                MergeStrategy::Merge => "merge",
                MergeStrategy::Squash => "squash",
                MergeStrategy::Rebase => "rebase",
            };
            cmd_args.push("--method");
            cmd_args.push(strategy_value);
        }

        debug!(repo = %self.repo, number, ?strategy, "spawning `gitcode pr merge`");

        let output = self
            .runner
            .run(&binary, &cmd_args)
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gitcode: {e}")))?;

        if !output.status.success() {
            let gitcode_err = parse_gitcode_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gitcode_err}")));
        }

        // `gitcode pr merge` outputs a human-readable message, not JSON.
        let message = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(MergeResult {
            merged: true,
            sha: None,
            message: Some(message),
        })
    }
```

- [ ] **Step 4: 运行测试确认通过**

Run: `cargo nextest run -p gf-gitcode pr::tests`
Expected: 全部 PASS。

- [ ] **Step 5: 提交**

```bash
git add crates/gitcode/src/pr.rs
git commit -m "fix(gitcode): map merge strategy to 'pr merge --method' (Issue #90)

gitcode v0.6.1 supports --method merge|squash|rebase; the adapter no
longer warns and ignores the requested strategy."
```

---

### Task 6: 评论解析容错（pr comment 多余位置参数 + `user` 架构）

**背景：** `pr comment` 同样把字段列表跟在布尔 `--json` 后（多余位置参数）。且实测 gitcode 评论 JSON 使用 `user` 对象而非 `author`；`issue.rs` 现有 `CommentApiResponse` 假定 `author` 为纯字符串——两种形态在不同 CLI 版本/端点都出现过，需双形态容错。

**Files:**
- Modify: `crates/gitcode/src/pr.rs:279-320`（`comment` 方法）、`crates/gitcode/src/issue.rs:127-155`（`CommentApiResponse` 与其 `From` 实现）
- Test: `crates/gitcode/src/pr.rs` 与 `crates/gitcode/src/issue.rs` 各自 `mod tests`

**Interfaces:**
- Consumes: `gitflow_cli_core::types::deserialize_u64_or_string`（core::types，已 pub）
- Produces: `PrCommentApiResponse`（pr.rs 私有）；issue.rs 的 `CommentApiResponse` 升级为 `author: Option<String>` + `user: Option<UserApi>` 双形态

- [ ] **Step 1: 写失败测试**

在 `pr.rs` 的 `mod tests` 追加：

```rust
#[tokio::test]
async fn test_should_not_pass_field_list_to_pr_comment() {
    let comment_json = r#"{"id": "9001", "body": "LGTM", "user": {"login": "rev", "id": "u9"}, "created_at": "2026-07-30T12:00:00+08:00"}"#;
    let runner = RecordingMockRunner::success(comment_json);
    let provider = GitCodePrProvider::with_runner("o/r", runner.clone());

    let comment = provider.comment(52, "LGTM").await.expect("comment should parse");

    assert_eq!(comment.id, 9001);
    assert_eq!(comment.author.login, "rev");
    assert_eq!(
        runner.calls()[0],
        vec!["pr", "comment", "52", "--repo", "o/r", "--body", "LGTM", "--json"]
    );
}

#[test]
fn test_should_parse_comment_with_legacy_string_author() {
    let json = r#"{"id": "7", "body": "old format", "author": "alice", "created_at": "2026-07-07 10:40:20"}"#;
    let api: PrCommentApiResponse = serde_json::from_str(json).expect("legacy shape");
    let comment: CommentData = api.into();
    assert_eq!(comment.author.login, "alice");
    assert_eq!(comment.created_at.to_rfc3339(), "2026-07-07T10:40:20+00:00");
}
```

在 `issue.rs` 的 `mod tests` 追加（验证 issue 评论同样兼容 `user` 对象形态）：

```rust
#[test]
fn test_should_parse_issue_comment_with_user_object() {
    let json = r#"{"id": 12, "body": "hi", "user": {"login": "bob", "id": "u2"}, "created_at": "2026-07-30T12:00:00+08:00"}"#;
    let api: CommentApiResponse = serde_json::from_str(json).expect("user-object shape");
    let comment: CommentData = api.into();
    assert_eq!(comment.id, 12);
    assert_eq!(comment.author.login, "bob");
    assert_eq!(comment.author.id, "u2");
}
```

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo nextest run -p gf-gitcode pr::tests::test_should_not_pass_field_list_to_pr_comment pr::tests::test_should_parse_comment_with_legacy issue::tests::test_should_parse_issue_comment_with_user_object`
Expected: FAIL — `PrCommentApiResponse` 未定义；issue `CommentApiResponse` 无法解析 `user` 对象。

- [ ] **Step 3: 实现双形态评论映射**

在 `pr.rs` 中 `PrBranchApi` 之后插入：

```rust
/// gitcode CLI 评论响应类型，兼容两种已观测形态：
/// - v0.6.x：`user` 为对象、`created_at` 为带偏移 RFC3339
/// - 旧版本：`author` 为纯字符串、`created_at` 为 `YYYY-MM-DD HH:MM:SS`
#[derive(Debug, Clone, Deserialize)]
struct PrCommentApiResponse {
    #[serde(deserialize_with = "gitflow_cli_core::types::deserialize_u64_or_string")]
    id: u64,
    #[serde(default)]
    body: String,
    #[serde(default)]
    author: Option<String>,
    #[serde(default)]
    user: Option<PrUserApi>,
    #[serde(default)]
    created_at: Option<String>,
}

impl From<PrCommentApiResponse> for CommentData {
    fn from(api: PrCommentApiResponse) -> Self {
        let author = api.user.map_or_else(
            || UserSummary {
                login: api.author.unwrap_or_else(|| "unknown".into()),
                id: String::new(),
            },
            |u| UserSummary {
                login: u.login,
                id: u.id.unwrap_or_default(),
            },
        );
        let created_at = api.created_at.as_deref().map_or_else(Utc::now, |s| {
            DateTime::parse_from_rfc3339(s)
                .map(|dt| dt.with_timezone(&Utc))
                .or_else(|_| {
                    chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
                        .map(|ndt| ndt.and_utc())
                })
                .unwrap_or_else(|_| Utc::now())
        });
        Self {
            id: api.id,
            body: api.body,
            author,
            created_at,
        }
    }
}
```

重写 `pr.rs` 的 `comment` 方法 argv（删除字段列表，改用 `PrCommentApiResponse`）：

```rust
    async fn comment(&self, number: u64, body: &str) -> Result<CommentData> {
        let binary = crate::gitcode_binary();
        let number_str = number.to_string();
        debug!(repo = %self.repo, number, "spawning `gitcode pr comment`");

        let output = self
            .runner
            .run(
                &binary,
                &[
                    "pr",
                    "comment",
                    &number_str,
                    "--repo",
                    &self.repo,
                    "--body",
                    body,
                    "--json",
                ],
            )
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gitcode: {e}")))?;

        if !output.status.success() {
            let gitcode_err = parse_gitcode_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gitcode_err}")));
        }

        let api: PrCommentApiResponse =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(api.into())
    }
```

并将该方法文档注释中的 `--json id,body,author,createdAt` 描述改为 `--json`。

将 `issue.rs` 的 `CommentApiResponse`（原 127-155 行）替换为同样的双形态结构（字段：`id`（`deserialize_u64_or_string`）、`body`、`author: Option<String>`、`user: Option<UserApi>`（复用 issue.rs 既有 `UserApi`）、`created_at: Option<String>`），其 `From<CommentApiResponse> for CommentData` 逻辑与上面 `PrCommentApiResponse` 的 `From` 完全相同（`UserApi` → `UserSummary` 走既有 `impl From<UserApi>`）；同步更新该结构体上方说明两种形态的文档注释。

- [ ] **Step 4: 运行测试确认通过**

Run: `cargo nextest run -p gf-gitcode pr::tests issue::tests`
Expected: 全部 PASS（issue.rs 旧的字符串 `author` 夹具测试仍通过——双形态保持后向兼容）。

- [ ] **Step 5: 提交**

```bash
git add crates/gitcode/src/pr.rs crates/gitcode/src/issue.rs
git commit -m "fix(gitcode): tolerate both comment schemas and drop pr comment field arg (Issue #90)

gitcode comment JSON uses a 'user' object in v0.6.x while older builds
return a plain-string 'author'; parse both. Also remove the gh-style
field list after boolean --json in pr comment."
```

---

### Task 7: 契约测试与真实夹具（防止回归的长效机制）

**背景：** 本 crate 原有部分单测使用"想象中的 gitcode 输出"（gh 风格 camelCase 夹具），正是 #90 长期未被发现的原因。用真实捕获的夹具建立契约测试，并清理误导性旧夹具。

**Files:**
- Create: `crates/gitcode/tests/fixtures/pr_list_gitcode_v0.6.1.json`
- Create: `crates/gitcode/tests/gitcode_schema_contract.rs`
- Modify: `crates/gitcode/src/pr.rs` 内 `mod tests`（删除 gh 风格误导性夹具测试：`test_should_deserialize_pr_data_from_gc_output`、`test_should_deserialize_draft_pr_from_gc_output`、`test_should_deserialize_closed_pr_from_gc_close_output`、`test_should_deserialize_reopened_pr_from_gc_reopen_output` —— 其断言已被 Task 2/3 的真实架构测试覆盖）

**Interfaces:**
- Consumes: `gitflow_cli_gitcode` 的公开 API；`PrApiResponse` 为私有，契约测试通过**公开行为**验证——即构造 `GitCodePrProvider::with_runner` + 夹具输出 → 调用 `list/view` → 断言 `PrData`
- Produces: `tests/gitcode_schema_contract.rs` 中的契约测试集

- [ ] **Step 1: 写入真实夹具文件**

创建 `crates/gitcode/tests/fixtures/pr_list_gitcode_v0.6.1.json`（内容为 Task 2 `real_gitcode_pr_json()` 的同一 JSON，外层包成数组 `[{...}]`，文件头不加注释——JSON 不支持注释；来源信息写入契约测试的文档注释）。

- [ ] **Step 2: 写契约测试**

`runner` 模块的 mock 类型为 `#[cfg(test)]` 且 crate 私有，外部集成测试无法导入——因此契约测试写成 crate 内测试模块。在 `crates/gitcode/src/pr.rs` 的 `mod tests` 之后追加独立的契约模块：

```rust
#[cfg(test)]
mod contract_tests {
    //! gitcode CLI v0.6.1 JSON 架构契约测试。
    //!
    //! 夹具来源：2026-07-31 对 gitcode CLI v0.6.1
    //!（commit c20f71f67ead1d748e78391cd9e470c2ea51b887, built 2026-06-05）
    //! `pr list -R byx-darwin/go-beniofit --json --state all` 的真实捕获。
    //! 若 gitcode CLI 升级导致这些测试失败，说明上游架构变更，需要更新
    //! 适配器映射并重新捕获夹具（参见路线图"契约测试 + 兼容性矩阵"单元）。

    use gitflow_cli_core::pr::ListPrArgs;

    use super::*;
    use crate::runner::MockCommandRunner;

    const PR_LIST_FIXTURE: &str = include_str!("../tests/fixtures/pr_list_gitcode_v0.6.1.json");

    #[tokio::test]
    async fn test_should_parse_real_gitcode_v061_pr_list_output() {
        let provider = GitCodePrProvider::with_runner(
            "byx-darwin/go-beniofit",
            MockCommandRunner::success(PR_LIST_FIXTURE),
        );

        let prs = provider
            .list(ListPrArgs::default())
            .await
            .expect("contract fixture must parse");

        assert_eq!(prs.len(), 1);
        let pr = &prs[0];
        assert_eq!(pr.number, 52);
        assert_eq!(pr.state, State::Closed);
        assert_eq!(pr.author.login, "byx-darwin");
        assert_eq!(pr.head_branch, "test/88-engine-rule-coverage");
        assert_eq!(pr.base_branch, "master");
        assert!(pr.url.starts_with("https://gitcode.com/"));
    }
}
```

- [ ] **Step 3: 运行契约测试确认通过**

Run: `cargo nextest run -p gf-gitcode contract`
Expected: PASS（依赖 Task 2/3 的修复；若 Task 2/3 被回退，此测试必须以 `missing field` 类错误失败）。

- [ ] **Step 4: 删除误导性旧夹具测试**

从 `pr.rs` 的 `mod tests` 中删除 Step 0（Task 7 Files 节列出的）四个 gh 风格夹具测试。运行全量测试确认无其他依赖：

Run: `cargo nextest run -p gf-gitcode`
Expected: 全部 PASS，测试总数净增（新增 > 删除）。

- [ ] **Step 5: 提交**

```bash
git add crates/gitcode/tests crates/gitcode/src/pr.rs
git commit -m "test(gitcode): add v0.6.1 contract fixtures captured from real CLI output

Replace imagined gh-style fixtures with real gitcode v0.6.1 captures so
schema drift in the upstream CLI is caught by tests (Issue #90 root
cause was invisible to the old fixtures)."
```

---

### Task 8: 全量验证与文档收尾

**Files:**
- Modify: `crates/gitcode/src/pr.rs:1-5`（模块文档注释：将"捕获 stdout 并解析 JSON"补充为"通过 `PrApiResponse` 映射 v0.6.x 架构"）

- [ ] **Step 1: 更新模块文档注释**

将 `pr.rs` 顶部文档注释第 5 行：

```rust
//! 所有方法通过 `tokio::process::Command` 调用 `gc`，捕获 stdout 并解析 JSON。
```

改为：

```rust
//! 所有方法通过 [`CommandRunner`] 调用 `gitcode` CLI，捕获 stdout 并解析 JSON。
//! gitcode v0.6.x 的 JSON 架构（snake_case、`user` 键、嵌套 `head`/`base`）
//! 与 `gh` 不同，统一经 `PrApiResponse` 映射为 core 的 [`PrData`]。
```

- [ ] **Step 2: 格式检查**

Run: `cargo +nightly fmt --check`
Expected: 无差异（有差异则 `cargo +nightly fmt` 后重跑）。

- [ ] **Step 3: 静态检查（pedantic）**

Run: `cargo clippy -p gf-gitcode --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: 零警告。出现 `similar_names` / `module_name_repetitions` 之外的新警告时就地修复（工作区已 allow 这两个）。

- [ ] **Step 4: 全量测试**

Run: `make test`
Expected: 全部 PASS（含其他 crate 的既有用例，确认无回归）。

- [ ] **Step 5: 实测冒烟（只读，安全）**

Run: `make smoke-test-gitcode`
Expected: 全部只读探测通过。随后手动执行实测核对（本机 gitcode 位于 `~/Library/Python/3.9/bin/gitcode`，`GITCODE_TOKEN` 已配置）：

```bash
cargo run --quiet -- pr list --platform gitcode --repo byx-darwin/go-beniofit --output json | head -20
cargo run --quiet -- pr view 52 --platform gitcode --repo byx-darwin/go-beniofit
```

Expected: 两条命令成功返回 PR 数据（修复前分别报 `missing field 'author'` 与 `accepts 1 arg(s), received 2`）。若顶层 CLI 参数名有出入（如 `--repo` 不被接受），以 `cargo run -- pr view --help` 输出为准调整命令——这不影响适配器修复本身。

- [ ] **Step 6: 提交文档收尾**

```bash
git add crates/gitcode/src/pr.rs
git commit -m "docs(gitcode): document PrApiResponse schema mapping in module header"
```

---

## 完成定义（Definition of Done）

- [ ] #90 的 5 个子问题各有对应回归测试（Task 4→add-label；Task 2/3→pr create/list；Task 3→pr view；Task 5→merge strategy；Task 6→pr comment）
- [ ] `cargo nextest run -p gf-gitcode` 全绿，新增测试 ≥ 12 个
- [ ] `cargo clippy -p gf-gitcode --all-targets -- -D warnings -W clippy::pedantic` 零警告
- [ ] `make smoke-test-gitcode` 通过；`pr list` / `pr view` 对真实 GitCode 仓库的只读实测成功
- [ ] 契约夹具 `pr_list_gitcode_v0.6.1.json` 来自真实捕获并在测试注释中标注 CLI 版本与 commit
- [ ] 关闭 #90 的动作**不在本计划内**——由 gf-workflow 编排器在交付阶段（PR 合并后）执行
