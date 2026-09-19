# GitLab 适配器批次1修复 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 修复三个独立的 GitLab 适配器缺陷：label create 的 color 前缀归一化、auth status 的 JSON 输出与多 host 折叠、issue create 无 body 时失败。

**Architecture:** 三处改动分别落在 `crates/gitlab/src/label.rs`、`crates/core/src/auth.rs` + `crates/gitlab/src/auth.rs`、`crates/gitlab/src/issue.rs`，互不重叠，按 TDD RED→GREEN→REFACTOR 顺序独立完成，每个任务结束即可独立提交。

**Tech Stack:** Rust 2024 / tokio / async-trait / `MockCommandRunner`（`crates/gitlab/src/runner.rs`，支持 `recorded_calls()` 断言实际传给 `glab` 的参数）

**Spec:** `docs/superpowers/specs/2026-09-19-gitlab-adapter-batch1-design.md`

## Global Constraints

- 禁止 `unwrap()`/`expect()` 于生产代码（测试代码可用 `.expect()`）
- `crates/core::auth::AuthStatus` 新增字段必须向后兼容：`#[serde(default, skip_serializing_if = "Vec::is_empty")]`
- 不引入 repo remote → host 的精确匹配（已与维护者确认范围：任一 host 已登录即整体判定已认证）
- 每个任务完成后运行 `cargo test -p gitflow-gitlab -p gitflow-core`，交付前运行 `make lint`

---

### Task 1: GitLab label color 归一化 + help 文案修正（#372）

**Files:**
- Modify: `crates/gitlab/src/label.rs:212-234`（`create()` 方法）
- Modify: `apps/cli/src/commands/label.rs:34`（help 文案）
- Test: `crates/gitlab/src/label.rs`（`#[cfg(test)] mod tests`）

**Interfaces:**
- Consumes: `CreateLabelArgs { name: String, color: String, description: Option<String> }`（`crates/core/src/label.rs:33-40`，不变）
- Produces: 不改变对外签名，仅改变传给 `glab` 的 `--color` 实际值

**背景核实（2026-09-19）：** 本地探测 `glab label create --color <no-#> --repo <fake>` 与带 `#` 的调用返回完全相同的 401 错误——说明 `glab` CLI 本身不校验 color 格式，是否需要 `#` 前缀取决于 GitLab API 侧。为避免依赖不确定的 API 行为，采用**始终归一化**策略：缺 `#` 就补上，带 `#` 就原样传递，无论 API 是否严格要求都不会更差。

- [ ] **Step 1: 写失败测试——无 `#` 前缀的 color 应被归一化**

在 `crates/gitlab/src/label.rs` 的 `mod tests` 中，紧邻现有 `test_should_create_label_without_output_json_and_refetch_via_list` 之后新增：

```rust
    #[tokio::test]
    async fn test_should_normalize_color_without_hash_prefix() {
        let runner = SequencedMockCommandRunner::from_results(&[
            (true, ""),
            (true, r##"[{"id":101,"name":"bug","color":"#d73a4a"}]"##),
        ]);
        let provider = GitLabLabelProvider::with_runner("owner/repo", runner.clone());

        let args = CreateLabelArgs {
            name: "bug".to_string(),
            color: "d73a4a".to_string(), // 无 # 前缀
            description: None,
        };

        provider.create(args).await.expect("should create");

        let calls = runner.recorded_calls();
        let create_call = &calls[0];
        let color_idx = create_call
            .1
            .iter()
            .position(|a| a == "--color")
            .expect("--color flag present");
        assert_eq!(create_call.1[color_idx + 1], "#d73a4a");
    }

    #[tokio::test]
    async fn test_should_pass_through_color_that_already_has_hash_prefix() {
        let runner = SequencedMockCommandRunner::from_results(&[
            (true, ""),
            (true, r##"[{"id":101,"name":"bug","color":"#d73a4a"}]"##),
        ]);
        let provider = GitLabLabelProvider::with_runner("owner/repo", runner.clone());

        let args = CreateLabelArgs {
            name: "bug".to_string(),
            color: "#d73a4a".to_string(), // 已带 #
            description: None,
        };

        provider.create(args).await.expect("should create");

        let calls = runner.recorded_calls();
        let create_call = &calls[0];
        let color_idx = create_call
            .1
            .iter()
            .position(|a| a == "--color")
            .expect("--color flag present");
        assert_eq!(create_call.1[color_idx + 1], "#d73a4a");
    }
```

检查 `SequencedMockCommandRunner` 是否已实现 `Clone`（`crates/gitlab/src/runner.rs`）；若未实现，在该 struct 上加 `#[derive(Clone)]`（其字段 `Arc<Mutex<..>>` 均可克隆）——这是本步骤的前置修复，不算独立任务。

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test -p gitflow-gitlab test_should_normalize_color_without_hash_prefix -- --nocapture`
Expected: FAIL（实际传入的是 `"d73a4a"` 而非 `"#d73a4a"`，assert_eq 失败）

- [ ] **Step 3: 实现归一化逻辑**

修改 `crates/gitlab/src/label.rs` 的 `create()`（第 212 行起）：

```rust
    async fn create(&self, args: CreateLabelArgs) -> Result<LabelData> {
        let color = if args.color.starts_with('#') {
            args.color.clone()
        } else {
            format!("#{}", args.color)
        };

        debug!(
            repo = %self.repo,
            name = %args.name,
            color = %color,
            "spawning `glab label create`"
        );

        let mut cmd_args: Vec<&str> = vec![
            "label",
            "create",
            "--name",
            &args.name,
            "--color",
            &color,
            "--repo",
            &self.repo_target,
        ];

        if let Some(ref desc) = args.description {
            cmd_args.push("--description");
            cmd_args.push(desc);
        }

        let output =
            self.runner.run("glab", &cmd_args).await.map_err(|e| {
                CoreError::Platform(format!("Failed to spawn glab label create: {e}"))
            })?;

        if !output.status.success() {
            return Err(parse_glab_error(&output.stderr).into());
        }

        let paged = self.list(None).await?;
        resolve_after_mutation(
            paged,
            |l| l.name == args.name,
            "label lookup truncated after create; too many labels to confirm the write landed",
            format!("Label '{}' not found after create", args.name),
        )
    }
```

- [ ] **Step 4: 运行测试确认通过**

Run: `cargo test -p gitflow-gitlab label -- --nocapture`
Expected: PASS（新增两条 + 既有 label 测试全部通过）

- [ ] **Step 5: 修正 CLI help 文案**

修改 `apps/cli/src/commands/label.rs:34`：

```rust
        /// 标签颜色（必填，十六进制格式，如 `d73a4a` 或 `#d73a4a`，均可）。
        #[arg(long)]
        color: String,
```

- [ ] **Step 6: 提交**

```bash
git add crates/gitlab/src/label.rs apps/cli/src/commands/label.rs
git commit -m "fix(gitlab): normalize label color to # prefix, fix help text (#372)"
```

---

### Task 2: `crates/core::auth::AuthStatus` 新增 `hosts` 字段（#362 基础）

**Files:**
- Modify: `crates/core/src/auth.rs:14-27`
- Modify: `crates/core/src/lib.rs:108`（re-export）

**Interfaces:**
- Produces: 新类型 `HostAuthStatus { host: String, logged_in: bool, user: Option<String> }`；`AuthStatus` 新增 `pub hosts: Vec<HostAuthStatus>`，默认空、序列化时省略空数组（向后兼容，GitHub/GitCode 不受影响）

- [ ] **Step 1: 写失败测试——序列化省略空 hosts，非空时正常输出**

在 `crates/core/src/auth.rs` 末尾新增 `#[cfg(test)] mod tests`（若已存在则追加）：

```rust
#[cfg(test)]
mod tests {
    use super::{AuthStatus, HostAuthStatus};

    #[test]
    fn test_should_omit_empty_hosts_field_when_serializing() {
        let status = AuthStatus {
            logged_in: true,
            user: Some("alice".to_string()),
            scopes: vec![],
            hosts: vec![],
        };
        let json = serde_json::to_string(&status).expect("serialize");
        assert!(!json.contains("hosts"));
    }

    #[test]
    fn test_should_include_hosts_field_when_non_empty() {
        let status = AuthStatus {
            logged_in: true,
            user: Some("alice".to_string()),
            scopes: vec![],
            hosts: vec![HostAuthStatus {
                host: "gitlab.com".to_string(),
                logged_in: true,
                user: Some("alice".to_string()),
            }],
        };
        let json = serde_json::to_string(&status).expect("serialize");
        assert!(json.contains("\"hosts\""));
        assert!(json.contains("\"gitlab.com\""));
    }
}
```

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test -p gitflow-core auth::tests -- --nocapture`
Expected: FAIL（编译错误：`AuthStatus` 无 `hosts` 字段，`HostAuthStatus` 不存在）

- [ ] **Step 3: 实现类型改动**

修改 `crates/core/src/auth.rs`（第 14-27 行起）：

```rust
/// 单个 host 的认证状态（GitLab 场景下可能同时配置多个 host）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HostAuthStatus {
    /// host 名称或地址（如 `gitlab.com`、`192.168.230.23`）。
    pub host: String,
    /// 该 host 是否已登录。
    pub logged_in: bool,
    /// 该 host 下的登录用户名（未登录时为 None）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

/// 当前认证状态。
///
/// 由 [`AuthProvider::status`] 返回，用于判断用户是否已登录、
/// 当前用户身份以及持有的权限范围。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthStatus {
    /// 用户是否已登录（任一已知 host 已登录即为 true）。
    pub logged_in: bool,
    /// 当前登录用户名（未登录时为 None）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    /// 已授权的权限范围列表。
    #[serde(default)]
    pub scopes: Vec<String>,
    /// 按 host 拆分的认证状态明细（单 host 平台如 GitHub/GitCode 始终为空数组）。
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hosts: Vec<HostAuthStatus>,
}
```

修改 `crates/core/src/lib.rs:108`：

```rust
pub use auth::{AuthStatus, HostAuthStatus};
```

修复所有既有的 `AuthStatus { .. }` 构造点（GitHub/GitCode/GitLab 三个 crate 的 `auth.rs`），补上 `hosts: vec![]`：

```bash
grep -rn "AuthStatus {" crates/github/src/auth.rs crates/gitcode/src/auth.rs crates/gitlab/src/auth.rs
```

对每一处命中，在字段列表末尾加 `hosts: vec![],`（GitLab 的构造点会在 Task 3 里整体重写，这里先保证编译通过即可）。

- [ ] **Step 4: 运行测试确认通过**

Run: `cargo build -p gitflow-core -p gitflow-github -p gitflow-gitcode -p gitflow-gitlab && cargo test -p gitflow-core auth::tests -- --nocapture`
Expected: PASS，且三个平台 crate 编译通过

- [ ] **Step 5: 提交**

```bash
git add crates/core/src/auth.rs crates/core/src/lib.rs crates/github/src/auth.rs crates/gitcode/src/auth.rs crates/gitlab/src/auth.rs
git commit -m "feat(core): add optional per-host breakdown to AuthStatus (#362)"
```

---

### Task 3: GitLab auth status 多 host 解析 + JSON 失败路径修复（#362）

**Files:**
- Modify: `crates/gitlab/src/auth.rs:139-174`（`status()` 方法），新增私有函数 `parse_hosts_from_status`
- Modify: `crates/github/src/auth.rs`、`crates/gitcode/src/auth.rs`（各自 1 个既有成功路径测试追加 `hosts.is_empty()` 断言，验证跨平台一致性）

**Interfaces:**
- Consumes: Task 2 产出的 `HostAuthStatus`（`crates/core::auth::HostAuthStatus`）
- Produces: `status()` 返回的 `AuthStatus.hosts` 在可解析出 host 结构时非空；`logged_in` 语义变为"任一 host 已登录"

- [ ] **Step 1: 写失败测试——混合 host 场景不应整体误报未认证**

在 `crates/gitlab/src/auth.rs` 的 `mod tests` 中，紧邻 `test_should_return_logged_in_status_when_authenticated` 之后新增：

```rust
    #[tokio::test]
    async fn test_should_report_authenticated_when_any_host_logged_in_in_mixed_status() {
        let combined = "gitlab.com\n  ! No token found (checked config file, keyring, and environment variables).\n192.168.230.23\n  ✓ Logged in to 192.168.230.23 as baoyuexing (keyring)\n";
        let runner = MockCommandRunner::failure(combined, 1);
        let provider = GitLabAuthProvider::with_runner(runner);

        let status = provider.status().await.expect("should not error on mixed host status");

        assert!(status.logged_in);
        assert_eq!(status.user, Some("baoyuexing".to_string()));
        assert_eq!(status.hosts.len(), 2);
        assert_eq!(status.hosts[0].host, "gitlab.com");
        assert!(!status.hosts[0].logged_in);
        assert_eq!(status.hosts[1].host, "192.168.230.23");
        assert!(status.hosts[1].logged_in);
        assert_eq!(status.hosts[1].user, Some("baoyuexing".to_string()));
    }
```

（`MockCommandRunner::failure` 把传入文本写入 stderr——与真实 `glab` 在该场景下非零退出、状态块打印到 stderr 的行为一致。）

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test -p gitflow-gitlab test_should_report_authenticated_when_any_host_logged_in_in_mixed_status -- --nocapture`
Expected: FAIL（当前实现命中 `!output.status.success()` 分支，文本不含 "not logged in"/"no active account"/"not authenticated"，返回 `Err`）

- [ ] **Step 3: 实现多 host 解析**

在 `crates/gitlab/src/auth.rs` 顶部 `use` 之后新增：

```rust
use gitflow_core::auth::HostAuthStatus;
```

在 `parse_user_from_status` 函数之前（约第 289 行）新增：

```rust
/// 把 `glab auth status` 的 stdout+stderr 合并文本解析为按 host 分组的状态。
///
/// `glab` 对每个已配置 host 输出一个不含空白的裸行作为 host 标识
/// （如 `gitlab.com`、`192.168.230.23`），紧随其后是若干缩进的状态行。
/// 无法识别出任何 host 行时返回空 `Vec`，由调用方回退到旧的整段文本判断。
fn parse_hosts_from_status(output: &str) -> Vec<HostAuthStatus> {
    let mut hosts = Vec::new();
    let mut current: Option<HostAuthStatus> = None;

    for line in output.lines() {
        let is_header = !line.starts_with(' ') && !line.starts_with('\t');
        let trimmed = line.trim();

        if is_header && !trimmed.is_empty() && !trimmed.contains(' ') {
            if let Some(host) = current.take() {
                hosts.push(host);
            }
            current = Some(HostAuthStatus {
                host: trimmed.to_string(),
                logged_in: false,
                user: None,
            });
            continue;
        }

        if let Some(ref mut host) = current
            && trimmed.contains("Logged in to")
        {
            host.logged_in = true;
            host.user = parse_user_from_status(trimmed);
        }
    }

    if let Some(host) = current.take() {
        hosts.push(host);
    }

    hosts
}
```

重写 `status()`（第 139-174 行）：

```rust
    async fn status(&self) -> Result<AuthStatus> {
        debug!("spawning `glab auth status`");

        let output = self
            .runner
            .run("glab", &["auth", "status"])
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab auth status: {e}")))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{stdout}{stderr}");

        let hosts = parse_hosts_from_status(&combined);

        if !hosts.is_empty() {
            let logged_in = hosts.iter().any(|h| h.logged_in);
            let user = hosts
                .iter()
                .find(|h| h.logged_in)
                .and_then(|h| h.user.clone());

            return Ok(AuthStatus {
                logged_in,
                user,
                scopes: vec![],
                hosts,
            });
        }

        if !output.status.success() {
            let text = combined.to_lowercase();
            if text.contains("not logged in")
                || text.contains("no active account")
                || text.contains("not authenticated")
            {
                return Ok(AuthStatus {
                    logged_in: false,
                    user: None,
                    scopes: vec![],
                    hosts: vec![],
                });
            }

            return Err(parse_glab_error(&output.stderr).into());
        }

        Ok(AuthStatus {
            logged_in: false,
            user: None,
            scopes: vec![],
            hosts: vec![],
        })
    }
```

- [ ] **Step 4: 运行测试确认通过**

Run: `cargo test -p gitflow-gitlab auth:: -- --nocapture`
Expected: PASS（新增测试 + 全部既有 `auth.rs` 测试，包括 `test_should_return_logged_out_status_when_glab_reports_not_logged_in` 与 `test_should_return_platform_error_when_status_fails_unexpectedly` 均不受影响）

- [ ] **Step 5: 补跨平台一致性测试**

在 `crates/github/src/auth.rs` 的 `test_should_return_logged_in_status_when_authenticated`（或等价的成功路径测试）末尾追加一行：

```rust
        assert!(status.hosts.is_empty());
```

在 `crates/gitcode/src/auth.rs` 对应测试同样追加一行 `assert!(status.hosts.is_empty());`。

- [ ] **Step 6: 运行完整回归**

Run: `cargo test -p gitflow-core -p gitflow-github -p gitflow-gitcode -p gitflow-gitlab`
Expected: PASS

- [ ] **Step 7: 提交**

```bash
git add crates/gitlab/src/auth.rs crates/github/src/auth.rs crates/gitcode/src/auth.rs
git commit -m "fix(gitlab): parse per-host auth status, stop collapsing mixed hosts to unauthenticated (#362)"
```

---

### Task 4: GitLab issue create 无 body 时始终传 `--description`（#375）

**Files:**
- Modify: `crates/gitlab/src/issue.rs:360-375`（`create()` 方法）
- Test: `crates/gitlab/src/issue.rs`（`#[cfg(test)] mod tests`）

**Interfaces:**
- Consumes: `CreateIssueArgs`（不变）
- Produces: 不改变对外签名，仅改变传给 `glab` 的参数列表（新增无条件 `--description`）

- [ ] **Step 1: 写失败测试——`body: None` 时应仍然传 `--description`**

在 `crates/gitlab/src/issue.rs` 的 `mod tests` 中，紧邻 `test_should_return_platform_error_when_glab_fails_for_create` 之前新增：

```rust
    #[tokio::test]
    async fn test_should_pass_empty_description_when_body_is_none() {
        let runner = MockCommandRunner::success(
            r#"{"iid":1,"title":"t","description":null,"state":"opened","web_url":"https://gitlab.com/o/r/-/issues/1","labels":[],"assignees":[],"created_at":"2026-01-01T00:00:00Z","updated_at":"2026-01-01T00:00:00Z"}"#,
        );
        let provider = GitLabIssueProvider::with_runner("owner/repo", runner.clone());

        let args = CreateIssueArgs {
            title: "t".to_string(),
            body: None,
            labels: vec![],
            assignees: vec![],
        };

        provider.create(args).await.expect("should create");

        let calls = runner.recorded_calls();
        let create_call = &calls[0];
        let desc_idx = create_call
            .1
            .iter()
            .position(|a| a == "--description")
            .expect("--description flag must be present even when body is None");
        assert_eq!(create_call.1[desc_idx + 1], "");
    }
```

若 `MockCommandRunner`（`crates/gitlab/src/runner.rs`）未实现 `Clone`，参照 Task 1 Step 1 的说明先补上 `#[derive(Clone)]`（两个任务若并行执行，此步骤只需做一次）。若 `CreateIssueArgs` 字段与上面不完全一致，以 `crates/core/src/issue.rs` 的实际定义为准调整字段名。

- [ ] **Step 2: 运行测试确认失败**

Run: `cargo test -p gitflow-gitlab test_should_pass_empty_description_when_body_is_none -- --nocapture`
Expected: FAIL（`--description` 不在 recorded args 中）

- [ ] **Step 3: 实现无条件传参**

修改 `crates/gitlab/src/issue.rs` 的 `create()`（第 360-375 行起）：

```rust
    async fn create(&self, args: CreateIssueArgs) -> Result<IssueData> {
        let labels_joined = args.labels.join(",");
        let assignees_joined = args.assignees.join(",");
        let description = args.body.clone().unwrap_or_default();

        let mut cmd_args: Vec<&str> = vec![
            "issue",
            "create",
            "--repo",
            &self.repo_target,
            "--title",
            &args.title,
            "--description",
            &description,
        ];

        if !args.labels.is_empty() {
            cmd_args.push("--label");
            cmd_args.push(&labels_joined);
        }

        if !args.assignees.is_empty() {
            cmd_args.push("--assignee");
            cmd_args.push(&assignees_joined);
        }

        debug!(repo = %self.repo, title = %args.title, "spawning `glab issue create`");

        let output = self
```

（保留原方法剩余部分不变；上面只替换了参数构建段落，原先 `if let Some(body) = &args.body { ... }` 分支整体删除。）

- [ ] **Step 4: 运行测试确认通过**

Run: `cargo test -p gitflow-gitlab issue:: -- --nocapture`
Expected: PASS（新增测试 + 既有的 `Some(body)` 路径测试均通过——`description` 变量对 `Some` 情形取到原值，对 `None` 情形取到空字符串）

- [ ] **Step 5: 提交**

```bash
git add crates/gitlab/src/issue.rs
git commit -m "fix(gitlab): always pass --description to glab issue create, fixing no-body 100% failure (#375)"
```

---

## Final Verification

- [ ] **Step 1: 全量测试**

Run: `cargo test -p gitflow-core -p gitflow-github -p gitflow-gitcode -p gitflow-gitlab`
Expected: PASS

- [ ] **Step 2: Lint**

Run: `make lint`
Expected: 0 warnings（`cargo fmt` + `cargo clippy --all-targets --all-features -- -D warnings`）

- [ ] **Step 3: 针对性 clippy pedantic（本次改动的三个文件 + core/auth.rs）**

Run: `cargo clippy -p gitflow-core -p gitflow-gitlab --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: 0 warnings，或对新增代码逐条修正
