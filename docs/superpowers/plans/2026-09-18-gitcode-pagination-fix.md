# gitcode 列表命令分页修复 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 让 gitcode 的五个 list 子命令真正分页，消除「返回 100 条却报告 `truncated: false`」的静默丢数据。

**Architecture:** 把 `FetchStrategy::SingleShot` 换成 `Paged { per_page }`，`issue`/`pr`/`label`/`milestone` 四处显式传 `--per-page` / `--page` 并停止传 `--limit`；`release list` 因实测无分页旗标，改走 `gitcode api /repos/{repo}/releases?per_page&page`。配套补齐测试基建（让 `SequencedMockCommandRunner` 记录 argv）、argv 回归护栏与一条只读 e2e。

**Tech Stack:** Rust 2024 · `gitflow_core::{fetch_capped, FetchStrategy, Paged, DEFAULT_LIST_LIMIT}` · `tokio` · gitcode-cli 0.12.0

**Spec:** `docs/superpowers/specs/2026-09-18-gitcode-pagination-fix-design.md`

## Global Constraints

- 页大小公式全局统一：`let per_page = cap.saturating_add(1).min(crate::GITCODE_API_MAX_PER_PAGE);`
- `GITCODE_API_MAX_PER_PAGE = 100`，实测为 gitcode API 的 `per_page` **静默封顶值**（传 101 返回 100，不报错）
- 改造后的 list **一律不再传 `--limit`**：实测 `--per-page` 优先，单传 `--per-page 100` 不受 CLI 默认 `--limit 30` 影响
- 测试命名一律 `test_should_<expected_behavior>`
- 生产代码禁止 `unwrap()` / `expect()`；禁止 `println!` / `dbg!`，日志用 `tracing`
- 每个任务结束前必须跑 `cargo clippy -p gitflow-gitcode --all-targets --all-features -- -D warnings -W clippy::pedantic`
- **提交需用户明示许可**（CLAUDE.md）。各任务的 Commit 步骤在未获许可前不得执行；执行器应把改动累积在 feature 分支工作区，由编排器统一征询
- **分页测试的 `cap` 必须取 100**（Ruling 1）：`fetch_capped` 的 `want = cap + 1`，而
  `per_page = min(cap+1, 100)`。当 `cap < 100` 时 `per_page` 恰为 `cap+1`，首页一次就
  满足 `want`，循环**只发一次调用** —— 页号递增无从观测，断言 `calls[1]` 更会索引越界。
  `cap = 100` 时 `per_page = 100 < want = 101`，首页满页后必然发出第二页
- 任务顺序不可调换：Task 5 是 `GITCODE_DEFAULT_LIST_LIMIT` 的最后一个消费者，提前删除会让中间态出现 `dead_code` 告警而被 `-D warnings` 拒绝

## File Structure

| 文件 | 责任 | 本次动作 |
|---|---|---|
| `crates/gitcode/src/runner.rs` | 测试用 `CommandRunner` 替身 | 给 `SequencedMockCommandRunner` 加 argv 记录 |
| `crates/gitcode/src/issue.rs` | Issue provider | `list_impl` 改 `Paged`；更新既有 argv 断言 |
| `crates/gitcode/src/pr.rs` | PR provider | `list_impl` 改 `Paged`；更新既有 argv 断言 |
| `crates/gitcode/src/label.rs` | Label + Milestone provider | 两处 `list` 改 `Paged`；去掉死参数 `LABEL_FIELDS` |
| `crates/gitcode/src/release.rs` | Release provider | `list` 改走 `api` + `Paged`；其余方法不动 |
| `crates/gitcode/src/lib.rs` | crate 常量 | 删 `GITCODE_DEFAULT_LIST_LIMIT`，重写 `GITCODE_API_MAX_PER_PAGE` 注释 |
| `crates/e2e-gitcode/tests/pagination.rs` | 真实服务端只读验证 | 新建 |
| `docs/superpowers/specs/2026-09-17-list-pagination-design.md` | #360 设计文档 | §4.1 / §4.3 / §4.3.1 / §12 修订 |

---

### Task 0: 让 `e2e-gitcode::noauth` 对已登录环境密封

**Complexity:** simple（files=1，测试专属 → score 1）

**Files:**
- Modify: `crates/e2e-gitcode/tests/noauth.rs:1-16`

**Interfaces:**
- Consumes: 无
- Produces: 无

**Why:** Phase 2 的 `gf-quality` 闸门在本机跑出两例失败：
`test_should_fail_with_login_guidance_when_status_checked_unauthenticated` 与
`test_should_fail_with_login_guidance_when_listing_issues_unauthenticated`。

根因：该文件模块注释断言「`GitCodeAuthProvider` 是纯 env-var 短路 + 真实 `gc` 子进程读取，
**没有本地配置文件状态**，`env_remove` 单独即可保证确定性」—— 这一前提**已不成立**。
gitcode CLI 把登录态存在 `~/.config/gc/auth.json`，因此清除 `GITCODE_TOKEN` 之后
`gf auth status --platform gitcode` 仍返回 `loggedIn: true` 并退出 0，两条断言双双落空。
CI 无登录态故仍为绿，缺陷只在装了并登录了 gitcode CLI 的开发机上暴露 —— 而 Issue #365
的整个前提正是「现在可以装并登录 gitcode CLI 了」，AC#7 还要把 `e2e-gitcode` 纳入常规验证。
要常规跑，就不能在开发机上恒红。

**实测得到的密封手段**（gitcode-cli 0.12.0）：

| 手段 | 结果 |
|---|---|
| `XDG_CONFIG_HOME=<tmp>` | ❌ 被忽略，仍读 `~/.config/gc/auth.json` |
| `HOME=<tmp>` | ❌ pip wrapper 崩溃（`ModuleNotFoundError: gc_cli`），输出不含 "login"，断言仍不满足 |
| **`GC_TOKEN=<无效值>`** | ✅ 优先于配置文件**且会被校验**：exit=1，stderr 含 `gc auth login` |

- [ ] **Step 1: 跑测试确认当前失败**

Run: `cargo test -p e2e-gitcode --test noauth`
Expected: FAIL ×2，消息形如 `unauthenticated auth status must exit non-zero, stdout: {"success": true, ...}`

> 若本机 gitcode CLI 未登录，此步会直接 PASS —— 那说明复现前提不具备，
> 先运行 `gitcode auth status` 确认处于已登录态再继续。

- [ ] **Step 2: 修正模块注释与 runner 构造**

把 `crates/e2e-gitcode/tests/noauth.rs` 开头（第 1-16 行）改为：

```rust
//! 未认证错误路径 E2E 测试（无需凭据，前提是运行环境已安装 `gc`/`gitcode` CLI）。
//!
//! **不能只靠 `env_remove` 构造未认证环境**：gitcode CLI 把登录态存在
//! `~/.config/gc/auth.json`，清掉环境变量后它依然认为自己已登录，会让这两条
//! 测试在已登录的开发机上恒红（CI 无登录态故不暴露）。
//!
//! 实测（gitcode-cli 0.12.0）`GC_TOKEN` 优先于该配置文件**且会被服务端校验**，
//! 因此注入一个确定无效的值即可稳定构造「有凭据但认证失败」的确定性环境，
//! 无论本机是否登录，结果一致。
//!
//! 代价：该手段依赖一次真实的 token 校验请求，离线环境下报错形态可能不同。

#![allow(clippy::unwrap_used, reason = "Test code uses unwrap for simplicity")]

use e2e_core::{TtyMode, TtyRunner};

/// 语法合法但确定无效的 token，用于构造认证失败路径。
const INVALID_GC_TOKEN: &str = "invalid-token-for-e2e";

fn scrubbed_runner() -> TtyRunner {
    let mut runner = TtyRunner::new(TtyMode::NonInteractive);
    runner.env_remove("GITCODE_TOKEN");
    runner.env("GC_TOKEN", INVALID_GC_TOKEN);
    runner
}
```

两个测试函数体保持不变 —— 它们的断言（非零退出 + 输出含 `login`）在新环境下正确成立。

- [ ] **Step 3: 跑测试确认通过**

Run: `cargo test -p e2e-gitcode --test noauth`
Expected: 2 passed

- [ ] **Step 4: 静态检查**

Run: `cargo clippy -p e2e-gitcode --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: 无告警

- [ ] **Step 5: Commit（需用户许可后执行）**

```bash
git add crates/e2e-gitcode/tests/noauth.rs
git commit -m "test(e2e-gitcode): make noauth tests hermetic against config-file login"
```

---

### Task 1: 让 `SequencedMockCommandRunner` 记录 argv

**Complexity:** simple（files=1，不跨模块边界，不改公开 API，无迁移 → score 1）

**Files:**
- Modify: `crates/gitcode/src/runner.rs:210-270`

**Interfaces:**
- Consumes: 无
- Produces: `SequencedMockCommandRunner::recorded_calls() -> Vec<(String, Vec<String>)>`，供 Task 2-5 的翻页测试断言 argv

**Why:** 当前该 runner 的 `run` 签名是 `(_program, _args)`，**丢弃 argv**，无法支撑「断言各 list 的 argv 含 `--per-page` 与 `--page`」。GitLab 侧的同名结构体（`crates/gitlab/src/runner.rs:161`）已有此能力，本任务照搬其实现，使两侧一致。

- [ ] **Step 1: 写失败测试**

在 `crates/gitcode/src/runner.rs` 的 `mod tests` 中追加：

```rust
    #[tokio::test]
    async fn test_should_record_argv_for_each_sequenced_call() {
        let runner = SequencedMockCommandRunner::from_results(&[(true, "[]"), (true, "[]")]);

        runner
            .run("gitcode", &["issue", "list", "--page", "1"])
            .await
            .expect("first call should succeed");
        runner
            .run("gitcode", &["issue", "list", "--page", "2"])
            .await
            .expect("second call should succeed");

        let calls = runner.recorded_calls();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].0, "gitcode");
        assert!(calls[0].1.contains(&"1".to_string()));
        assert!(calls[1].1.contains(&"2".to_string()));
    }
```

- [ ] **Step 2: 跑测试确认失败**

Run: `cargo test -p gitflow-gitcode test_should_record_argv_for_each_sequenced_call`
Expected: 编译失败，`no method named 'recorded_calls' found for struct 'SequencedMockCommandRunner'`

- [ ] **Step 3: 实现**

把 `crates/gitcode/src/runner.rs:210` 起的结构体与 impl 改为：

```rust
#[cfg(test)]
#[derive(Debug, Clone)]
pub struct SequencedMockCommandRunner {
    responses: std::sync::Arc<std::sync::Mutex<std::collections::VecDeque<CommandOutput>>>,
    /// Recorded `(program, args)` sequences for every `run`/`run_with_stdin` call.
    recorded: std::sync::Arc<std::sync::Mutex<RecordedCalls>>,
}
```

`new` 里补上字段初始化：

```rust
    #[must_use]
    pub fn new(outputs: Vec<CommandOutput>) -> Self {
        Self {
            responses: std::sync::Arc::new(std::sync::Mutex::new(outputs.into())),
            recorded: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        }
    }
```

在 `impl SequencedMockCommandRunner` 中追加访问器：

```rust
    /// Return the recorded `(program, args)` sequences from every executed call.
    ///
    /// # Panics
    ///
    /// Panics if the internal recording mutex is poisoned (a prior panic while
    /// holding the lock).
    #[must_use]
    pub fn recorded_calls(&self) -> Vec<(String, Vec<String>)> {
        self.recorded.lock().expect("mock mutex poisoned").clone()
    }
```

把 `CommandRunner for SequencedMockCommandRunner` 的 `run` 改为：

```rust
    async fn run(&self, program: &str, args: &[&str]) -> std::io::Result<CommandOutput> {
        self.recorded.lock().expect("mock mutex poisoned").push((
            program.to_string(),
            args.iter().map(|s| (*s).to_string()).collect(),
        ));
        let mut guard = self
            .responses
            .lock()
            .expect("SequencedMockCommandRunner mutex poisoned");
        guard
            .pop_front()
            .ok_or_else(|| std::io::Error::other("no more responses"))
    }
```

`run_with_stdin` 已委托给 `run`，无需改动（委托后自动被记录一次）。

- [ ] **Step 4: 跑测试确认通过**

Run: `cargo test -p gitflow-gitcode --lib runner`
Expected: 全部 PASS

- [ ] **Step 5: 静态检查**

Run: `cargo clippy -p gitflow-gitcode --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: 无告警

- [ ] **Step 6: Commit（需用户许可后执行）**

```bash
git add crates/gitcode/src/runner.rs
git commit -m "test(gitcode): record argv in SequencedMockCommandRunner"
```

---

### Task 2: `issue list` 改用 `Paged` + `--per-page` / `--page`

**Complexity:** medium（files=2，不跨模块边界，不改公开 API → score 2；但含既有测试的语义改写，建议独立 subagent）

**Files:**
- Modify: `crates/gitcode/src/issue.rs:291-345`（`list_impl`）
- Modify: `crates/gitcode/src/issue.rs:1348-1360`（`test_should_request_default_cap_plus_one_on_gitcode`）
- Modify: `crates/gitcode/src/issue.rs:1385-1409`（完整 argv 断言）

**Interfaces:**
- Consumes: Task 1 的 `SequencedMockCommandRunner::recorded_calls()`
- Produces: 无新公开符号（`list_impl` 是私有关联函数）

- [ ] **Step 1: 写失败测试**

在 `crates/gitcode/src/issue.rs` 的 `mod tests` 中追加。注意**首个响应必须是满页**（`per_page` 条），否则翻页在第一次调用后即因短页终止，测试对「是否真的翻页」无判别力：

```rust
    /// 构造 `count` 条合法 issue JSON，编号从 `start` 递增。
    fn issue_page_json(start: u64, count: u64) -> String {
        let items: Vec<String> = (start..start + count)
            .map(|n| {
                format!(
                    r#"{{"number":{n},"title":"t{n}","state":"open","body":null,"labels":[],"created_at":"2026-07-30T12:00:00+08:00","updated_at":"2026-07-30T12:00:00+08:00"}}"#
                )
            })
            .collect();
        format!("[{}]", items.join(","))
    }

    #[tokio::test]
    async fn test_should_page_through_gitcode_issue_list_with_incrementing_page_numbers() {
        // cap=100 → per_page=min(101,100)=100，want=cap+1=101。
        // 首页满 100 条：既非短页，又未达 want ⇒ `fetch_capped` 必然发出第二页。
        // 这是唯一能观测到页号递增的取值区间 —— cap < 100 时 per_page 恰为
        // cap+1，首页一次就满足 want，循环只会发出一次调用。
        let page1 = issue_page_json(1, 100);
        let page2 = issue_page_json(101, 100);
        let runner = SequencedMockCommandRunner::from_results(&[(true, &page1), (true, &page2)]);
        let provider = GitCodeIssueProvider::with_runner("owner/repo", runner.clone());

        let paged = provider
            .list(ListIssueArgs {
                limit: Some(100),
                ..ListIssueArgs::default()
            })
            .await
            .expect("list should succeed");

        assert_eq!(paged.items.len(), 100, "返回条数必须被 cap 钳住");
        assert!(paged.truncated, "超过 cap 必须诚实报告截断");

        let calls = runner.recorded_calls();
        assert_eq!(calls.len(), 2, "必须真的翻到第二页，实际调用数: {}", calls.len());
        let first = &calls[0].1;
        assert!(
            first.windows(2).any(|w| w[0] == "--per-page" && w[1] == "100"),
            "首次调用必须显式传 --per-page，实际 argv: {first:?}"
        );
        assert!(
            first.windows(2).any(|w| w[0] == "--page" && w[1] == "1"),
            "首次调用必须显式传 --page=1，实际 argv: {first:?}"
        );
        assert!(
            !first.iter().any(|a| a == "--limit"),
            "--per-page 已决定单页大小，不得再传 --limit，实际 argv: {first:?}"
        );
        let second = &calls[1].1;
        assert!(
            second.windows(2).any(|w| w[0] == "--page" && w[1] == "2"),
            "页号必须在翻页中递增，实际 argv: {second:?}"
        );
    }

    #[tokio::test]
    async fn test_should_cap_gitcode_issue_per_page_at_api_maximum() {
        let runner = MockCommandRunner::success("[]");
        let provider = GitCodeIssueProvider::with_runner("owner/repo", runner.clone());

        provider
            .list(ListIssueArgs::default())
            .await
            .expect("list should succeed");

        let recorded = &runner.recorded_calls()[0].1;
        assert!(
            recorded
                .windows(2)
                .any(|w| w[0] == "--per-page" && w[1] == "100"),
            "默认 cap=1000 时 per_page 必须被 API 上限 100 钳住，实际 argv: {recorded:?}"
        );
    }
```

同时把既有的 `test_should_request_default_cap_plus_one_on_gitcode`（`issue.rs:1348`）整体删除 —— 它断言的 `--limit 101` 正是本次要消除的行为，已被上面 `test_should_cap_gitcode_issue_per_page_at_api_maximum` 取代。

- [ ] **Step 2: 跑测试确认失败**

Run: `cargo test -p gitflow-gitcode test_should_page_through_gitcode_issue_list_with_incrementing_page_numbers`
Expected: FAIL —— 当前实现只发一次 `--limit 4` 调用，`--per-page` 断言不成立

- [ ] **Step 3: 实现**

把 `crates/gitcode/src/issue.rs:291` 起的 `list_impl` 改为：

```rust
    async fn list_impl(&self, args: ListIssueArgs) -> Result<Paged<IssueData>> {
        let binary = crate::gitcode_binary();
        let binary = &binary;
        let cap = args.limit.unwrap_or(DEFAULT_LIST_LIMIT);
        let repo = &self.repo;
        let runner = &self.runner;
        let state = args.state;
        let search = &args.search;
        let labels = &args.labels;
        // 实测（gitcode-cli 0.12.0）：`--per-page` 优先于 `--limit`，且被 API
        // 静默封顶在 100；`--page` 真实翻页，页间编号不重叠。页大小不必超过
        // cap+1：N+1 探测只需多要一条即可判断截断。
        let per_page = cap.saturating_add(1).min(crate::GITCODE_API_MAX_PER_PAGE);

        debug!(repo = %self.repo, cap, per_page, "spawning gitcode issue list");

        fetch_capped(
            FetchStrategy::Paged { per_page },
            cap,
            |page, per_page| async move {
                let page_str = page.to_string();
                let per_page_str = per_page.to_string();
                let mut cmd_args: Vec<&str> = vec!["issue", "list", "-R", repo, "--json"];

                if let Some(state) = &state {
                    cmd_args.push("--state");
                    cmd_args.push(match state {
                        State::Open => "open",
                        State::Closed => "closed",
                        State::All => "all",
                    });
                }
                if let Some(search) = search {
                    cmd_args.push("--search");
                    cmd_args.push(search);
                }
                for label in labels {
                    cmd_args.push("--label");
                    cmd_args.push(label);
                }
                cmd_args.push("--per-page");
                cmd_args.push(&per_page_str);
                cmd_args.push("--page");
                cmd_args.push(&page_str);

                let output = runner
                    .run(binary, &cmd_args)
                    .await
                    .map_err(|e| CoreError::Platform(format!("{e}")))?;
                if !output.status.success() {
                    return Err(parse_gitcode_error(&output.stderr).into());
                }
                let issues: Vec<IssueApiResponse> =
                    serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;
                Ok(issues.into_iter().map(IssueData::from).collect())
            },
        )
        .await
    }
```

- [ ] **Step 4: 更新既有完整 argv 断言**

`crates/gitcode/src/issue.rs:1390-1408` 的期望 argv 改为（末尾两对旗标替换 `--limit 101`）：

```rust
        assert_eq!(
            runner.recorded_calls()[0].1,
            vec![
                "issue",
                "list",
                "-R",
                "owner/repo",
                "--json",
                "--state",
                "open",
                "--label",
                "bug",
                "--per-page",
                "100",
                "--page",
                "1"
            ]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>()
        );
```

- [ ] **Step 5: 跑测试确认通过**

Run: `cargo test -p gitflow-gitcode --lib issue`
Expected: 全部 PASS

- [ ] **Step 6: 静态检查**

Run: `cargo clippy -p gitflow-gitcode --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: 无告警

- [ ] **Step 7: Commit（需用户许可后执行）**

```bash
git add crates/gitcode/src/issue.rs
git commit -m "fix(gitcode): page through issue list instead of single-shot --limit"
```

---

### Task 3: `pr list` 改用 `Paged` + `--per-page` / `--page`

**Complexity:** medium（files=1，含既有测试改写 → score 1，但与 Task 2 同构、可独立评审）

**Files:**
- Modify: `crates/gitcode/src/pr.rs:11-15`（补 `DEFAULT_LIST_LIMIT` 导入）
- Modify: `crates/gitcode/src/pr.rs:225-265`（`list_impl`）
- Modify: `crates/gitcode/src/pr.rs:1015-1028`、`:1030-1048`、`:1280-1300`（既有断言）

**Interfaces:**
- Consumes: Task 1 的 `recorded_calls()`
- Produces: 无新公开符号

- [ ] **Step 1: 写失败测试**

在 `crates/gitcode/src/pr.rs` 的 `mod tests` 中追加：

```rust
    #[tokio::test]
    async fn test_should_page_through_gitcode_pr_list_with_incrementing_page_numbers() {
        // cap=100 → per_page=100，want=101。首页满 100 条 ⇒ 必然发出第二页。
        // cap 必须 ≥ 100：更小的 cap 会让 per_page 恰为 cap+1，首页一次满足
        // want，循环只发一次调用，页号递增无从观测。
        let one = real_gitcode_pr_json();
        let page = format!("[{}]", vec![one; 100].join(","));
        let runner = SequencedMockCommandRunner::from_results(&[(true, &page), (true, &page)]);
        let provider = GitCodePrProvider::with_runner("owner/repo", runner.clone());

        let paged = provider
            .list(ListPrArgs {
                state: Some(State::Open),
                limit: Some(100),
            })
            .await
            .expect("list should succeed");

        assert_eq!(paged.items.len(), 100);
        assert!(paged.truncated);

        let calls = runner.recorded_calls();
        assert_eq!(calls.len(), 2, "必须真的翻到第二页，实际调用数: {}", calls.len());
        let first = &calls[0].1;
        assert!(
            first.windows(2).any(|w| w[0] == "--per-page" && w[1] == "100"),
            "实际 argv: {first:?}"
        );
        assert!(
            first.windows(2).any(|w| w[0] == "--page" && w[1] == "1"),
            "实际 argv: {first:?}"
        );
        assert!(!first.iter().any(|a| a == "--limit"), "实际 argv: {first:?}");
        assert!(
            calls[1].1.windows(2).any(|w| w[0] == "--page" && w[1] == "2"),
            "页号必须递增，实际 argv: {:?}",
            calls[1].1
        );
    }
```

删除既有的 `test_should_request_default_cap_plus_one_for_gitcode_pr_list`（`pr.rs:1015`），其 `--limit 101` 断言正是本次消除的行为。

- [ ] **Step 2: 跑测试确认失败**

Run: `cargo test -p gitflow-gitcode test_should_page_through_gitcode_pr_list_with_incrementing_page_numbers`
Expected: FAIL —— `--per-page` 断言不成立

- [ ] **Step 3: 实现**

`crates/gitcode/src/pr.rs:11-15` 的导入补上 `DEFAULT_LIST_LIMIT`：

```rust
use gitflow_core::{
    CoreError, DEFAULT_LIST_LIMIT, FetchStrategy, Paged, Result, Session, fetch_capped,
    pr::{CreatePrArgs, ListPrArgs, PrData, PrProvider},
    types::{CommentData, MergeResult, MergeStrategy, State, UserSummary},
};
```

`list_impl` 改为：

```rust
    async fn list_impl(&self, args: ListPrArgs) -> Result<Paged<PrData>> {
        let binary = crate::gitcode_binary();
        let binary = &binary;
        let cap = args.limit.unwrap_or(DEFAULT_LIST_LIMIT);
        let repo = &self.repo;
        let runner = &self.runner;
        let state = args.state;
        // 见 issue.rs 同处注释：`--per-page` 优先于 `--limit` 且被 API 封顶 100。
        // 不用 gitcode 的 `--paginate`：那会让 CLI 自行取完全部页，本适配器
        // 需要的是受控翻页以便 N+1 探测。
        let per_page = cap.saturating_add(1).min(crate::GITCODE_API_MAX_PER_PAGE);

        debug!(repo = %self.repo, cap, per_page, "spawning `gitcode pr list`");

        fetch_capped(
            FetchStrategy::Paged { per_page },
            cap,
            |page, per_page| async move {
                let page_str = page.to_string();
                let per_page_str = per_page.to_string();
                let mut cmd_args: Vec<&str> = vec!["pr", "list", "--repo", repo, "--json"];

                if let Some(state) = &state {
                    cmd_args.push("--state");
                    cmd_args.push(match state {
                        State::Open => "open",
                        State::Closed => "closed",
                        State::All => "all",
                    });
                }

                cmd_args.push("--per-page");
                cmd_args.push(&per_page_str);
                cmd_args.push("--page");
                cmd_args.push(&page_str);

                let output = runner
                    .run(binary, &cmd_args)
                    .await
                    .map_err(|e| CoreError::Platform(format!("Failed to spawn gitcode: {e}")))?;

                if !output.status.success() {
                    return Err(parse_gitcode_error(&output.stderr).into());
                }

                let apis: Vec<PrApiResponse> =
                    serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

                Ok(apis.into_iter().map(PrData::from).collect())
            },
        )
        .await
    }
```

- [ ] **Step 4: 更新两处既有断言**

`pr.rs:1030` 的 `test_should_produce_complete_argv_for_gitcode_pr_list_with_state` 期望 argv 末尾改为：

```rust
                "--per-page",
                "100",
                "--page",
                "1"
```

`pr.rs:1294-1297` 的 `test_should_pass_limit_flag_to_pr_list` 改为断言用户 limit 经由 `--per-page` 抵达底层（`cap=5` → `per_page=6`），并同步重命名：

```rust
    #[tokio::test]
    async fn test_should_pass_user_limit_to_pr_list_via_per_page() {
        let runner = RecordingMockRunner::success(&format!("[{}]", real_gitcode_pr_json()));
        let provider = GitCodePrProvider::with_runner("o/r", runner.clone());

        let prs = provider
            .list(ListPrArgs {
                state: Some(State::Open),
                limit: Some(5),
            })
            .await
            .expect("list should succeed");

        assert_eq!(prs.items.len(), 1);
        let args = &runner.calls()[0];
        // N+1 探测：cap=5 时页大小为 6，以便区分"恰好 5 条"与"还有更多"。
        assert!(args.windows(2).any(|w| w[0] == "--per-page" && w[1] == "6"));
        assert!(args.windows(2).any(|w| w[0] == "--page" && w[1] == "1"));
        assert!(args.contains(&"--state".to_string()));
        assert!(args.contains(&"open".to_string()));
    }
```

- [ ] **Step 5: 跑测试确认通过**

Run: `cargo test -p gitflow-gitcode --lib pr`
Expected: 全部 PASS

- [ ] **Step 6: 静态检查**

Run: `cargo clippy -p gitflow-gitcode --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: 无告警

- [ ] **Step 7: Commit（需用户许可后执行）**

```bash
git add crates/gitcode/src/pr.rs
git commit -m "fix(gitcode): page through pr list instead of single-shot --limit"
```

---

### Task 4: `label list` / `milestone list` 启用分页

**Complexity:** medium（files=1，两个 provider，含死参数清理 → score 2）

**Files:**
- Modify: `crates/gitcode/src/label.rs:119-154`（`GitCodeLabelProvider::list`）
- Modify: `crates/gitcode/src/label.rs:364-395`（`GitCodeMilestoneProvider::list`）
- Modify: `crates/gitcode/src/label.rs:668-690`（两处 runner-routing 断言）
- Modify: `crates/gitcode/src/label.rs:694-728`（两处完整 argv 断言）

**Interfaces:**
- Consumes: Task 1 的 `recorded_calls()`
- Produces: 无新公开符号

**Why:** #360 当时判断这两个子命令「是否支持 `--limit` 无从得知，故一律不传」。实测两者都支持 `-L/--limit`、`--page`（默认 1）、`--per-page`，且 `--per-page 2 --page 1/2` 返回不重叠 —— 分页能力完整，此前的保守处理让它们完全丧失截断检测能力。

- [ ] **Step 1: 写失败测试**

在 `crates/gitcode/src/label.rs` 的 `mod tests` 中追加：

```rust
    use crate::runner::SequencedMockCommandRunner;

    #[tokio::test]
    async fn test_should_page_through_gitcode_label_list_with_incrementing_page_numbers() {
        // cap=100 → per_page=100，want=101；首页满 100 条 ⇒ 必然发出第二页。
        fn label_page_json(start: u32, count: u32) -> String {
            let items: Vec<String> = (start..start + count)
                .map(|n| format!(r#"{{"name":"l{n}","color":"#ffffff","description":""}}"#))
                .collect();
            format!("[{}]", items.join(","))
        }
        let page1 = label_page_json(1, 100);
        let page2 = label_page_json(101, 100);
        let runner = SequencedMockCommandRunner::from_results(&[(true, &page1), (true, &page2)]);
        let provider = GitCodeLabelProvider::with_runner("owner/repo", runner.clone());

        let paged = provider.list(Some(100)).await.expect("should list");

        assert_eq!(paged.items.len(), 100);
        assert!(paged.truncated);

        let calls = runner.recorded_calls();
        assert_eq!(calls.len(), 2, "必须真的翻到第二页，实际调用数: {}", calls.len());
        assert!(
            calls[0].1.windows(2).any(|w| w[0] == "--per-page" && w[1] == "100"),
            "实际 argv: {:?}",
            calls[0].1
        );
        assert!(
            calls[0].1.windows(2).any(|w| w[0] == "--page" && w[1] == "1"),
            "实际 argv: {:?}",
            calls[0].1
        );
        assert!(
            calls[1].1.windows(2).any(|w| w[0] == "--page" && w[1] == "2"),
            "页号必须递增，实际 argv: {:?}",
            calls[1].1
        );
    }

    #[tokio::test]
    async fn test_should_page_through_gitcode_milestone_list_with_incrementing_page_numbers() {
        // cap=100 → per_page=100，want=101；首页满 100 条 ⇒ 必然发出第二页。
        fn milestone_page_json(start: u64, count: u64) -> String {
            let items: Vec<String> = (start..start + count)
                .map(|n| {
                    format!(
                        r#"{{"number":{n},"title":"m{n}","description":null,"state":"open","due_on":null,"closed_issues":0,"open_issues":0}}"#
                    )
                })
                .collect();
            format!("[{}]", items.join(","))
        }
        let page1 = milestone_page_json(1, 100);
        let page2 = milestone_page_json(101, 100);
        let runner = SequencedMockCommandRunner::from_results(&[(true, &page1), (true, &page2)]);
        let provider = GitCodeMilestoneProvider::with_runner("owner/repo", runner.clone());

        let paged = provider.list(Some(100)).await.expect("should list");

        assert_eq!(paged.items.len(), 100);
        assert!(paged.truncated);

        let calls = runner.recorded_calls();
        assert_eq!(calls.len(), 2, "必须真的翻到第二页，实际调用数: {}", calls.len());
        assert!(
            calls[0].1.windows(2).any(|w| w[0] == "--per-page" && w[1] == "100"),
            "实际 argv: {:?}",
            calls[0].1
        );
        assert!(
            calls[0].1.windows(2).any(|w| w[0] == "--page" && w[1] == "1"),
            "实际 argv: {:?}",
            calls[0].1
        );
        assert!(
            calls[1].1.windows(2).any(|w| w[0] == "--page" && w[1] == "2"),
            "页号必须递增，实际 argv: {:?}",
            calls[1].1
        );
    }
```

> 若上述 milestone JSON 字段与 `MilestoneApiResponse` 的实际定义不符，以
> `crates/gitcode/src/label.rs` 中该结构体的字段为准调整 fixture，不要改结构体。

- [ ] **Step 2: 跑测试确认失败**

Run: `cargo test -p gitflow-gitcode test_should_page_through_gitcode_label_list_with_incrementing_page_numbers`
Expected: FAIL —— 当前实现不传任何分页旗标

- [ ] **Step 3: 实现 label list**

把 `crates/gitcode/src/label.rs:119` 起的 `list` 改为（注意删除 `LABEL_FIELDS` 位置参数：实测 gitcode 的 `--json` 是布尔旗标，该参数被静默忽略）：

```rust
    async fn list(&self, limit: Option<u32>) -> Result<Paged<LabelData>> {
        let binary = crate::gitcode_binary();
        let binary = &binary;
        let cap = limit.unwrap_or(DEFAULT_LIST_LIMIT);
        let repo = &self.repo;
        let runner = &self.runner;
        // 实测（gitcode-cli 0.12.0）：`label list` 支持 `--page`（默认 1）与
        // `--per-page`，`--per-page 2 --page 1/2` 返回不重叠 —— 分页真实生效。
        let per_page = cap.saturating_add(1).min(crate::GITCODE_API_MAX_PER_PAGE);

        debug!(repo = %self.repo, cap, per_page, "spawning `gitcode label list`");

        fetch_capped(
            FetchStrategy::Paged { per_page },
            cap,
            |page, per_page| async move {
                let page_str = page.to_string();
                let per_page_str = per_page.to_string();
                let output = runner
                    .run(
                        binary,
                        &[
                            "label",
                            "list",
                            "-R",
                            repo,
                            "--json",
                            "--per-page",
                            &per_page_str,
                            "--page",
                            &page_str,
                        ],
                    )
                    .await
                    .map_err(|e| {
                        CoreError::Platform(format!("Failed to spawn gitcode label list: {e}"))
                    })?;

                if !output.status.success() {
                    return Err(parse_gitcode_error(&output.stderr).into());
                }

                let labels: Vec<LabelData> =
                    serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

                Ok(labels)
            },
        )
        .await
    }
```

- [ ] **Step 4: 实现 milestone list**

把 `crates/gitcode/src/label.rs:364` 起的 `list` 改为：

```rust
    async fn list(&self, limit: Option<u32>) -> Result<Paged<MilestoneData>> {
        let binary = crate::gitcode_binary();
        let binary = &binary;
        let cap = limit.unwrap_or(DEFAULT_LIST_LIMIT);
        let repo = &self.repo;
        let runner = &self.runner;
        // 见 label list 同处注释：`--page` / `--per-page` 已实测可用。
        let per_page = cap.saturating_add(1).min(crate::GITCODE_API_MAX_PER_PAGE);

        debug!(repo = %self.repo, cap, per_page, "spawning `gitcode milestone list`");

        fetch_capped(
            FetchStrategy::Paged { per_page },
            cap,
            |page, per_page| async move {
                let page_str = page.to_string();
                let per_page_str = per_page.to_string();
                let output = runner
                    .run(
                        binary,
                        &[
                            "milestone",
                            "list",
                            "-R",
                            repo,
                            "--json",
                            "--per-page",
                            &per_page_str,
                            "--page",
                            &page_str,
                        ],
                    )
                    .await
                    .map_err(|e| {
                        CoreError::Platform(format!("Failed to spawn gitcode milestone list: {e}"))
                    })?;

                if !output.status.success() {
                    return Err(parse_gitcode_error(&output.stderr).into());
                }

                let milestones: Vec<MilestoneApiResponse> =
                    serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

                Ok(milestones.into_iter().map(MilestoneData::from).collect())
            },
        )
        .await
    }
```

- [ ] **Step 5: 更新既有四处断言**

`label.rs:668` 与 `:681` 两处 runner-routing 测试的 `assert!(!calls[0].1.iter().any(|a| a == "--limit"));` 保留（改造后确实不传 `--limit`），并在其后各追加一行确认分页旗标已就位：

```rust
        assert!(calls[0].1.iter().any(|a| a == "--per-page"));
        assert!(calls[0].1.iter().any(|a| a == "--page"));
```

`label.rs:694` 的 label 完整 argv 断言改为（去掉 `"name,color,description"`，补分页旗标）：

```rust
            vec![
                "label",
                "list",
                "-R",
                "owner/repo",
                "--json",
                "--per-page",
                "100",
                "--page",
                "1"
            ]
```

`label.rs:713` 的 milestone 完整 argv 断言改为：

```rust
            vec![
                "milestone",
                "list",
                "-R",
                "owner/repo",
                "--json",
                "--per-page",
                "100",
                "--page",
                "1"
            ]
```

- [ ] **Step 6: 处理 `LABEL_FIELDS` 的剩余引用**

`LABEL_FIELDS`（`label.rs:80`）在 `:221` 仍被 `label create` 使用，**保留常量本身**，不要删除。仅本任务重写的 `list` argv 不再引用它。删除后若出现 `dead_code` 告警，说明误删了 `:221` 的引用 —— 回退该处改动。

- [ ] **Step 7: 跑测试确认通过**

Run: `cargo test -p gitflow-gitcode --lib label`
Expected: 全部 PASS

- [ ] **Step 8: 静态检查**

Run: `cargo clippy -p gitflow-gitcode --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: 无告警

- [ ] **Step 9: Commit（需用户许可后执行）**

```bash
git add crates/gitcode/src/label.rs
git commit -m "fix(gitcode): enable pagination for label and milestone list"
```

---

### Task 5: `release list` 改走 `gitcode api`，并清理 `lib.rs` 常量

**Complexity:** complex（files=2，跨模块边界改数据来源 +3，移除 crate 级常量 → score 5+；需额外评审）

**Files:**
- Modify: `crates/gitcode/src/release.rs:8-11`（补 `DEFAULT_LIST_LIMIT` 导入）
- Modify: `crates/gitcode/src/release.rs:145-181`（`list`）
- Modify: `crates/gitcode/src/release.rs:663-693`（三处既有断言）
- Modify: `crates/gitcode/src/lib.rs:57-75`（常量与注释）

**Interfaces:**
- Consumes: Task 1 的 `recorded_calls()`
- Produces: 无新公开符号；**移除** `crate::GITCODE_DEFAULT_LIST_LIMIT`

**Why:** 实测 `gitcode release list` **没有任何分页旗标**（只有 `-L/--limit`），无法像其余四处那样改造。改走 `gitcode api /repos/{repo}/releases?per_page&page`，与已交付且正确的 `issue comments` 路径（`issue.rs:594`）同构。

**关于类型：** 不新增中间类型。`ReleaseData` 的主字段名已是 snake_case（`tag_name` / `created_at` / `published_at` / `draft` / `prerelease`），`id` / `body` / `author` / `url` 均已 `#[serde(default)]` 或 `Option`，可直接反序列化 api 响应。

**本任务是 `GITCODE_DEFAULT_LIST_LIMIT` 的最后一个消费者**，必须在同一任务内删除该常量，否则中间态会出现 `dead_code` 告警而被 `-D warnings` 拒绝。

- [ ] **Step 1: 写失败测试**

在 `crates/gitcode/src/release.rs` 的 `mod tests` 中追加：

```rust
    #[tokio::test]
    async fn test_should_fetch_gitcode_releases_via_api_with_pagination() {
        // cap=100 → per_page=100，want=101；首页满 100 条 ⇒ 必然发出第二页。
        // cap 必须 ≥ 100，否则 per_page 恰为 cap+1，循环只发一次调用，
        // 断言 calls[1] 会直接索引越界。
        let one = valid_release_json();
        let page = format!("[{}]", vec![one; 100].join(","));
        let runner = SequencedMockCommandRunner::from_results(&[(true, &page), (true, &page)]);
        let provider = GitCodeReleaseProvider::with_runner("owner/repo", runner.clone());

        let paged = provider.list(Some(100)).await.expect("list should succeed");

        assert_eq!(paged.items.len(), 100);
        assert!(paged.truncated);

        let calls = runner.recorded_calls();
        assert_eq!(calls.len(), 2, "必须真的翻到第二页，实际调用数: {}", calls.len());
        assert_eq!(calls[0].1[0], "api");
        let first_path = &calls[0].1[1];
        assert!(
            first_path.starts_with("/repos/owner/repo/releases?"),
            "实际 api path: {first_path}"
        );
        assert!(
            first_path.contains("per_page=100"),
            "实际 api path: {first_path}"
        );
        assert!(first_path.contains("page=1"), "实际 api path: {first_path}");

        let second_path = &calls[1].1[1];
        assert!(
            second_path.contains("page=2"),
            "页号必须递增，实际 api path: {second_path}"
        );
    }

    #[tokio::test]
    async fn test_should_deserialize_release_from_gitcode_api_response() {
        // 钉住 api 响应 → ReleaseData 的字段映射（本机无带 release 的公开
        // gitcode 仓库可验，故以 fixture 覆盖）。
        let runner = MockCommandRunner::success(&format!("[{}]", valid_release_json()));
        let provider = GitCodeReleaseProvider::with_runner("owner/repo", runner);

        let paged = provider.list(None).await.expect("list should succeed");

        assert_eq!(paged.items.len(), 1);
        assert_eq!(paged.items[0].tag_name, "v1.0.0");
    }

    #[tokio::test]
    async fn test_should_cap_gitcode_release_per_page_at_api_maximum() {
        let runner = MockCommandRunner::success("[]");
        let provider = GitCodeReleaseProvider::with_runner("owner/repo", runner.clone());

        provider.list(None).await.expect("list should succeed");

        let path = &runner.recorded_calls()[0].1[1];
        assert!(
            path.contains("per_page=100"),
            "默认 cap=1000 时 per_page 必须被 API 上限 100 钳住，实际: {path}"
        );
    }
```

删除既有的三处 `--limit` 断言测试：`test_should_request_default_cap_plus_one_for_release_list`（`release.rs:663`）、`test_should_honour_user_release_limit`（`:677`）、`test_should_produce_complete_argv_for_release_list_with_default_limit`（`:692`）—— 它们断言的是被替换掉的 CLI 子命令路径。

- [ ] **Step 2: 跑测试确认失败**

Run: `cargo test -p gitflow-gitcode test_should_fetch_gitcode_releases_via_api_with_pagination`
Expected: FAIL —— 当前走的是 `release list --limit`，argv[0] 不是 `api`

- [ ] **Step 3: 实现**

`crates/gitcode/src/release.rs:8-11` 的导入补上 `DEFAULT_LIST_LIMIT`：

```rust
use gitflow_core::{
    CoreError, DEFAULT_LIST_LIMIT, FetchStrategy, Paged, Result, Session, fetch_capped,
    release::{CreateReleaseArgs, ReleaseData, ReleaseProvider},
};
```

把 `crates/gitcode/src/release.rs:145` 起的 `list` 改为：

```rust
    /// 列出 Release，按页抓取至多 `limit` 条。
    ///
    /// 走 `gitcode api` 而非 `release list` 子命令：实测（gitcode-cli 0.12.0）
    /// `release list` **没有任何分页旗标**（只有 `-L/--limit`），而其 API 层的
    /// `per_page` 被静默封顶在 100，因此 CLI 路径无法诚实报告截断。api 路径与
    /// 本 crate 的 `issue comments`（`issue.rs:594`）同构。
    ///
    /// `create` / `view` 等其余方法仍走 CLI 子命令，不受影响。
    ///
    /// # Errors
    ///
    /// 当 `gitcode` CLI 调用失败或响应无法反序列化时返回错误。
    async fn list(&self, limit: Option<u32>) -> Result<Paged<ReleaseData>> {
        let binary = crate::gitcode_binary();
        let binary = &binary;
        let cap = limit.unwrap_or(DEFAULT_LIST_LIMIT);
        let repo = &self.repo;
        let runner = &self.runner;
        let per_page = cap.saturating_add(1).min(crate::GITCODE_API_MAX_PER_PAGE);

        debug!(repo = %self.repo, cap, per_page, "spawning `gitcode api` GET releases");

        fetch_capped(
            FetchStrategy::Paged { per_page },
            cap,
            |page, per_page| async move {
                let api_path = format!("/repos/{repo}/releases?per_page={per_page}&page={page}");

                let output = runner
                    .run(binary, &["api", &api_path])
                    .await
                    .map_err(|e| {
                        CoreError::Platform(format!("Failed to spawn gitcode api: {e}"))
                    })?;

                if !output.status.success() {
                    return Err(parse_gitcode_error(&output.stderr).into());
                }

                let releases: Vec<ReleaseData> =
                    serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;
                Ok(releases)
            },
        )
        .await
    }
```

- [ ] **Step 4: 清理 `lib.rs` 常量**

把 `crates/gitcode/src/lib.rs:57-75` 改为（删除 `GITCODE_DEFAULT_LIST_LIMIT` 整块，重写 `GITCODE_API_MAX_PER_PAGE` 的文档注释）：

```rust
/// gitcode 分页端点的单页最大条目数。
///
/// **已实测**（gitcode-cli 0.12.0，样本 `openharmony/docs`）：`per_page` 不报错，
/// 而是被服务端**静默封顶**在 100 —— `per_page=101` 与 `per_page=1001` 均实回
/// 100 条。`--page` 真实翻页，`--per-page 3 --page 1/2` 返回的编号不重叠。
///
/// `issue` / `pr` / `label` / `milestone` 的 list 子命令与 `api` 端点共用这一上限，
/// 因此本 crate 的所有分页路径都用它钳住页大小：
/// `cap.saturating_add(1).min(GITCODE_API_MAX_PER_PAGE)`。
pub(crate) const GITCODE_API_MAX_PER_PAGE: u32 = 100;
```

- [ ] **Step 5: 跑测试确认通过**

Run: `cargo test -p gitflow-gitcode`
Expected: 全部 PASS。若出现 `cannot find value GITCODE_DEFAULT_LIST_LIMIT`，说明 Task 2/3 未完成或有遗漏引用，用 `grep -rn GITCODE_DEFAULT_LIST_LIMIT crates/` 定位并改为 `DEFAULT_LIST_LIMIT`

- [ ] **Step 6: 全量构建与静态检查**

Run: `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: 无告警（全工作区，因为改了 crate 级常量）

- [ ] **Step 7: Commit（需用户许可后执行）**

```bash
git add crates/gitcode/src/release.rs crates/gitcode/src/lib.rs
git commit -m "fix(gitcode): fetch releases via api with pagination; drop GITCODE_DEFAULT_LIST_LIMIT"
```

---

### Task 6: 端到端只读验证

**Complexity:** simple（files=1，新建独立测试文件 → score 1）

**Files:**
- Create: `crates/e2e-gitcode/tests/pagination.rs`

**Interfaces:**
- Consumes: `e2e_core::{TestConfig, TestMode, TtyMode, TtyRunner, scratch_repo_dir}`（与 `crates/e2e-gitcode/tests/issue.rs` 完全一致的用法）
- Produces: 无

**Why:** 回答 Issue #365 的 AC#5 与 AC#7。`scratch_repo_dir` 只做 `git init` + `git remote add`，不做 clone，因此指向大仓库 `openharmony/docs` 成本极低。该仓库 issue 数 >200（已实测），能真正暴露本缺陷。

- [ ] **Step 1: 写测试**

创建 `crates/e2e-gitcode/tests/pagination.rs`：

```rust
//! `GitCode` 列表分页 E2E 实测（真实服务端，只读）。
//!
//! 默认打公开仓库 `openharmony/docs`（issue 数 >200，已实测），可用
//! `E2E_TEST_REPO_GITCODE` 覆盖。无 `E2E_GITCODE_TOKEN` 时自动 skip。
//!
//! **本测试在 Issue #365 修复前必然失败**：彼时 `gf issue list --limit 150`
//! 返回 100 条却报告 `truncated: false`。

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "Test code uses unwrap/expect/indexing for simplicity"
)]

use e2e_core::{TestConfig, TestMode, TtyMode, TtyRunner, scratch_repo_dir};

/// 默认样本仓库：公开、只读、issue 数 >200。
const DEFAULT_PAGINATION_REPO: &str = "openharmony/docs";

#[tokio::test]
async fn test_should_not_silently_truncate_gitcode_issue_list_beyond_one_page() {
    let config = TestConfig::from_env_lenient();
    if config.gitcode_mode() != TestMode::Authenticated {
        eprintln!("skipped: E2E_GITCODE_TOKEN not set");
        return;
    }
    let repo = config
        .gitcode_test_repo
        .clone()
        .unwrap_or_else(|| DEFAULT_PAGINATION_REPO.to_string());

    let scratch = scratch_repo_dir(&format!("https://gitcode.com/{repo}.git"))
        .await
        .expect("scratch repo setup must succeed");

    let mut runner = TtyRunner::new(TtyMode::NonInteractive);
    runner.dir(scratch.path().to_path_buf());
    for (key, value) in config.gitcode_env() {
        runner.env(key, value);
    }

    let output = runner
        .run(&[
            "issue",
            "list",
            "--platform",
            "gitcode",
            "--state",
            "all",
            "--limit",
            "150",
            "--output",
            "json",
        ])
        .await
        .unwrap();

    assert!(output.status.success(), "stderr: {}", output.stderr);
    let parsed: serde_json::Value =
        serde_json::from_str(&output.stdout).expect("stdout must be a JSON envelope");
    assert_eq!(parsed["success"], serde_json::json!(true));

    let items = parsed["data"]
        .as_array()
        .expect("data must be an array of issues");
    let truncated = parsed["pagination"]["truncated"]
        .as_bool()
        .expect("envelope must carry pagination.truncated");

    // 核心断言：要么真的翻过了单页 100 的坎，要么诚实报告截断。
    // 修复前二者皆不成立 —— 返回 100 条且 truncated=false。
    assert!(
        items.len() > 100 || truncated,
        "静默截断：返回 {} 条且 truncated={truncated}（仓库 {repo} issue 数应 >100）",
        items.len()
    );
}
```

> **字段名核对**：若 `parsed["pagination"]["truncated"]` 取不到，以
> `crates/core/src/paging.rs` 中 `Paged<T>` 的序列化形状与
> `crates/e2e-gitcode/tests/issue.rs` 既有断言为准修正路径，**不要改生产代码的信封结构**。

- [ ] **Step 2: 跑测试**

Run: `E2E_GITCODE_TOKEN=<token> cargo test -p e2e-gitcode --test pagination -- --nocapture`
Expected: PASS（修复已完成）。未设 token 时应打印 `skipped:` 并 PASS

- [ ] **Step 3: 确认该测试对缺陷有判别力**

Run: `git stash && E2E_GITCODE_TOKEN=<token> cargo test -p e2e-gitcode --test pagination; git stash pop`
Expected: 修复前 FAIL，消息形如「静默截断：返回 100 条且 truncated=false」。这一步是 AC#5 的硬性要求 —— **必须实际执行并记录输出**，不得仅凭推理声称

- [ ] **Step 4: 静态检查**

Run: `cargo clippy -p e2e-gitcode --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: 无告警

- [ ] **Step 5: Commit（需用户许可后执行）**

```bash
git add crates/e2e-gitcode/tests/pagination.rs
git commit -m "test(e2e-gitcode): assert issue list pages past the 100-item API cap"
```

---

### Task 7: 修订 #360 设计文档

**Complexity:** simple（files=1，纯文档 → score 1）

**Files:**
- Modify: `docs/superpowers/specs/2026-09-17-list-pagination-design.md` §4.1 / §4.3 / §4.3.1 / §12

**Interfaces:**
- Consumes: Task 2-5 的最终实现形态
- Produces: 无

**Why:** AC#6。该文档现有的 §4.3 / §4.3.1 通篇建立在「gitcode CLI 不可获得」之上，其保守结论已被实测推翻；§4.3.1 更是整节围绕 `GITCODE_DEFAULT_LIST_LIMIT` 展开，而该常量已在 Task 5 删除。

- [ ] **Step 1: 更新 §4.1 策略矩阵的 gitcode 列**

把该表 gitcode 列改为：

```markdown
| 命令 | github | gitlab | gitcode |
|---|---|---|---|
| `issue list` | `SingleShot` — `--limit N` | `Paged{100}` — `--per-page/--page` | `Paged{100}` — `--per-page/--page` |
| `pr list` | `SingleShot` — `--limit N` | `Paged{100}` — `--per-page/--page` | `Paged{100}` — `--per-page/--page` |
| `release list` | `SingleShot` — `--limit N` | `Paged{100}` — `--per-page/--page` | `Paged{100}` — `api ?per_page&page`（CLI 无分页旗标） |
| `label list` | `SingleShot` — `--limit N` | `Paged{100}` — `--per-page/--page` | `Paged{100}` — `--per-page/--page` |
| `milestone list` | `Paged{100}` — `api ?per_page&page` | `Paged{100}` — `--per-page/--page` | `Paged{100}` — `--per-page/--page` |
| `issue comments` | `Paged{100}` — `api ?per_page&page` | `Paged{100}` — `api ?per_page&page` | `Paged{100}` — `api ?per_page&page` |
```

并删除 `issue comments` 行 gitcode 列原有的「（未实测）」字样。

- [ ] **Step 2: 重写 §4.3**

把整节替换为：

```markdown
### 4.3 gitcode 的 label / milestone：已实测，分页可用

**本节已由 Issue #365 用实测结论替换。** 原记载「其分页旗标无从实测，故不传
`--limit`，截断无法探测也无法报告」是在 gitcode CLI 不可获得的前提下写的，该
前提已不成立。

实测（gitcode-cli 0.12.0，样本 `openharmony/docs`）：`label list` 与
`milestone list` **都支持** `-L/--limit`、`--page`（默认 1）、`--per-page`，且
`--per-page 2 --page 1/2` 返回的条目不重叠 —— 分页真实生效。两者已随 #365 改为
`Paged{per_page}` 并显式传 `--per-page` / `--page`，截断检测能力完整。

两处此前已改走 `runner`，argv 可被测试观测，该部分结论不变。

详见 `docs/superpowers/specs/2026-09-18-gitcode-pagination-fix-design.md`。
```

- [ ] **Step 3: 重写 §4.3.1**

把整节替换为（该节原本围绕 `GITCODE_DEFAULT_LIST_LIMIT` 展开，常量已删除）：

```markdown
### 4.3.1 gitcode 的默认 limit：常量已移除

**本节已由 Issue #365 作废。** 原设计为 gitcode 单列
`GITCODE_DEFAULT_LIST_LIMIT = 100`，理由是「`--limit` 的合法取值范围未经实测，
传 `DEFAULT_LIST_LIMIT + 1 = 1001` 可能越界报错」，并记录了「实际传出 `--limit 101`，
仍比假定上限多 1，风险未归零」的残留风险。

实测推翻了这一整条推理链：gitcode 的 `per_page` **不报错**，而是静默封顶在 100
（`per_page=101` 与 `per_page=1001` 均实回 100 条）。「越界报错」的风险从不存在；
真正存在的是**反方向的缺陷** —— 静默封顶叠加 `SingleShot` 的 N+1 探测，令
`truncated = 100 > 100 = false`，即本设计要消灭的静默丢数据，只是阈值从 30 挪到了 100。

因此 #365 删除了 `GITCODE_DEFAULT_LIST_LIMIT`：改用 `Paged` 之后，`cap` 不再出现在
任何 argv 中，页大小由 `cap.saturating_add(1).min(GITCODE_API_MAX_PER_PAGE)` 钳住，
gitcode 与 github / gitlab 一样回落 `DEFAULT_LIST_LIMIT`。
```

- [ ] **Step 4: 更新 §12 已知遗留**

删除第 2 条（label/milestone 截断无法探测）、第 3 条（gitcode 适配器全程未经实测）、
第 5 条（`--limit` 取值范围未验证），保留第 1 条与第 4 条并重新编号，然后追加：

```markdown
3. gitcode `release list` 子命令无任何分页旗标 —— 实测（#365）只有 `-L/--limit`，
   故该路径改走 `gitcode api /repos/{repo}/releases?per_page&page`。api 非空响应的
   字段形状未经真实服务端确认（本机未找到带 release 的公开 gitcode 仓库），由
   fixture 单测覆盖
```

- [ ] **Step 5: 核对文档内交叉引用**

Run: `grep -n "GITCODE_DEFAULT_LIST_LIMIT\|4.3.1\|未实测\|无法获得" docs/superpowers/specs/2026-09-17-list-pagination-design.md`
Expected: 不再有指向已删除常量的断言性描述；残留的历史叙述必须带有「已由 #365 推翻」的限定语

- [ ] **Step 6: Commit（需用户许可后执行）**

```bash
git add docs/superpowers/specs/2026-09-17-list-pagination-design.md
git commit -m "docs(specs): replace gitcode assumptions with measured findings from #365"
```

---

## Self-Review

**1. Spec coverage**

| 设计文档章节 | 对应任务 |
|---|---|
| §3.1 策略矩阵（issue / pr） | Task 2、Task 3 |
| §3.1 策略矩阵（label / milestone） | Task 4 |
| §3.2 release 走 api + 类型复用 | Task 5 Step 1/3 |
| §3.3 删除 `GITCODE_DEFAULT_LIST_LIMIT` + 重写 MAX_PER_PAGE 注释 | Task 5 Step 4 |
| §3.3 清理死参数 `LABEL_FIELDS` / `RELEASE_FIELDS` | Task 4 Step 3（label）、Task 5 Step 3（release 走 api 后自然不再引用） |
| §3.4 测试基建 | Task 1 |
| （Phase 2 闸门发现）noauth 测试非密封 | Task 0 |
| §4 测试策略（满页、argv、页号递增、truncated） | Task 2/3/4/5 各自 Step 1 |
| §5 端到端验证 | Task 6 |
| §6 前序文档修订 | Task 7 |
| §7 已知遗留 | Task 7 Step 4 |

无遗漏。

**2. Placeholder scan**

无 TBD / TODO / 「类似 Task N」/ 「添加适当的错误处理」。每个代码步骤都给出完整可粘贴代码。两处标注了「若字段名不符，以既有结构体为准调整 fixture」—— 这是对执行器的**边界约束**（不许改生产结构体去迁就测试），非占位符。

**3. Type consistency**

- `recorded_calls() -> Vec<(String, Vec<String>)>` 在 Task 1 定义，Task 2/3/4/5 的用法（`calls[0].0` 取程序名、`calls[0].1` 取 argv）一致
- `per_page` 公式五处逐字一致：`cap.saturating_add(1).min(crate::GITCODE_API_MAX_PER_PAGE)`
- `DEFAULT_LIST_LIMIT` 的导入在 Task 3（pr.rs）、Task 5（release.rs）显式补齐；issue.rs、label.rs 已有
- `FetchStrategy::Paged { per_page }` 与 `fetch_capped(strategy, cap, |page, per_page| ...)` 的闭包签名各处一致
- Task 5 明确标注为 `GITCODE_DEFAULT_LIST_LIMIT` 的最后消费者，与 Global Constraints 的顺序约束呼应

**4. 任务顺序依赖**

Task 0 → Task 1 → (Task 2 ∥ Task 3 ∥ Task 4) → Task 5 → Task 6 → Task 7。
Task 0 独立于其余任务，但必须最先做：它是 Phase 2 质量闸门的未决失败，先恢复绿基线才能判断后续改动是否引入回归。
Task 5 必须在 2/3 之后（常量删除时机）；Task 4 与 2/3 相互独立；Task 6 需全部实现就位；Task 7 需最终形态确定。
