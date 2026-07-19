//! GitCode Issue 提供者实现。
//!
//! 通过 `gitcode` CLI 实现 [`IssueProvider`] trait。GitCode CLI
//! 使用 `-R` 指定仓库、`--json` 为布尔标志、`version` 子命令检测版本。
//! JSON 响应字段名与 GitHub/GitLab CLI 不同（`user` 而非 `author` 等），
//! 通过 [`IssueApiResponse`] 做字段映射后转换为 core 类型。

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use gitflow_cli_core::{
    CoreError, Result,
    issue::{CreateIssueArgs, IssueData, IssueProvider, ListIssueArgs},
    types::{CommentData, Label, State, UserSummary},
};
use serde::Deserialize;
use tracing::debug;

use crate::{
    error::parse_gitcode_error,
    runner::{CommandRunner, RealCommandRunner},
};

/// gitcode CLI `issue list --json` 的响应类型。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
struct IssueApiResponse {
    number: String,
    title: String,
    body: Option<String>,
    state: String,
    #[serde(default)]
    labels: Option<Vec<LabelApi>>,
    user: Option<UserApi>,
    #[serde(default)]
    assignees: Option<Vec<UserApi>>,
    created_at: Option<String>,
    updated_at: Option<String>,
    html_url: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
struct LabelApi {
    name: String,
    #[serde(default)]
    color: Option<String>,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct UserApi {
    login: String,
    #[serde(default)]
    id: Option<String>,
}

impl From<IssueApiResponse> for IssueData {
    fn from(api: IssueApiResponse) -> Self {
        Self {
            number: api.number.parse().unwrap_or(0),
            title: api.title,
            body: api.body,
            state: match api.state.as_str() {
                "closed" => State::Closed,
                _ => State::Open,
            },
            labels: api
                .labels
                .unwrap_or_default()
                .into_iter()
                .map(Label::from)
                .collect(),
            author: api.user.map_or(
                UserSummary {
                    login: "unknown".into(),
                    id: String::new(),
                },
                UserSummary::from,
            ),
            assignees: api
                .assignees
                .unwrap_or_default()
                .into_iter()
                .map(UserSummary::from)
                .collect(),
            created_at: api
                .created_at
                .and_then(|s| {
                    DateTime::parse_from_rfc3339(&s)
                        .ok()
                        .map(|d| d.with_timezone(&Utc))
                })
                .unwrap_or_else(Utc::now),
            updated_at: api
                .updated_at
                .and_then(|s| {
                    DateTime::parse_from_rfc3339(&s)
                        .ok()
                        .map(|d| d.with_timezone(&Utc))
                })
                .unwrap_or_else(Utc::now),
            url: api.html_url,
        }
    }
}

impl From<LabelApi> for Label {
    fn from(api: LabelApi) -> Self {
        Self {
            name: api.name,
            color: api.color,
            description: api.description,
        }
    }
}

impl From<UserApi> for UserSummary {
    fn from(api: UserApi) -> Self {
        Self {
            login: api.login,
            id: api.id.unwrap_or_default(),
        }
    }
}

/// gitcode CLI `issue comment --json` 的响应类型。
///
/// GitCode API 返回格式与 GitHub/GitLab 不同：
/// - `id` 为 JSON 字符串（如 `"178838115"`）
/// - `author` 为纯字符串（用户名），不是对象
/// - `created_at` 格式为 `"2026-07-07 10:40:20"`，不是 RFC3339
#[derive(Debug, Clone, Deserialize)]
struct CommentApiResponse {
    id: String,
    body: String,
    author: String,
    created_at: String,
}

impl From<CommentApiResponse> for CommentData {
    fn from(api: CommentApiResponse) -> Self {
        Self {
            id: api.id.parse().unwrap_or(0),
            body: api.body,
            author: UserSummary {
                login: api.author,
                id: String::new(),
            },
            created_at: chrono::NaiveDateTime::parse_from_str(&api.created_at, "%Y-%m-%d %H:%M:%S")
                .map_or_else(|_| Utc::now(), |ndt| ndt.and_utc()),
        }
    }
}

/// gitcode CLI `issue close/reopen --json` 的响应类型。
///
/// GitCode close/reopen 返回的字段比 list/view 少很多，
/// `number` 为 integer，没有 `title`/`body`/`labels` 等字段。
#[derive(Debug, Clone, Deserialize)]
struct CloseApiResponse {
    number: u64,
    state: String,
    url: String,
}

impl From<CloseApiResponse> for IssueData {
    fn from(api: CloseApiResponse) -> Self {
        Self {
            number: api.number,
            title: String::new(),
            body: None,
            state: match api.state.as_str() {
                "closed" => State::Closed,
                _ => State::Open,
            },
            labels: Vec::new(),
            author: UserSummary {
                login: "unknown".into(),
                id: String::new(),
            },
            assignees: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            url: api.url,
        }
    }
}

/// GitCode Issue 提供者，通过 `gitcode` CLI 操作。
///
/// 命令执行通过 [`CommandRunner`] 抽象，生产环境默认使用
/// [`RealCommandRunner`]，测试可注入自定义 runner 以模拟成功或失败场景。
#[derive(Debug, Clone)]
pub struct GitCodeIssueProvider<R: CommandRunner = RealCommandRunner> {
    /// GitCode `owner/repo`。
    repo: String,
    /// 用于执行 `gitcode` CLI 命令的 runner。
    runner: R,
}

impl GitCodeIssueProvider<RealCommandRunner> {
    /// 创建一个新的 `GitCodeIssueProvider`，使用真实的进程执行器。
    #[must_use]
    pub fn new(repo: impl Into<String>) -> Self {
        Self {
            repo: repo.into(),
            runner: RealCommandRunner,
        }
    }
}

impl<R: CommandRunner> GitCodeIssueProvider<R> {
    /// 使用自定义 [`CommandRunner`] 创建提供者。
    ///
    /// 主要用于测试，可注入模拟 runner 以控制 `gitcode` CLI 的输出。
    #[must_use]
    pub fn with_runner(repo: impl Into<String>, runner: R) -> Self {
        Self {
            repo: repo.into(),
            runner,
        }
    }

    /// 创建缺失的标签（使用 `--force` 保持幂等）。
    ///
    /// # Errors
    ///
    /// 当 `gitcode label create` 调用失败时返回错误。
    async fn ensure_label_exists(&self, name: &str) -> Result<()> {
        let binary = crate::gitcode_binary();
        debug!(repo = %self.repo, name, "auto-creating missing label via `gc label create`");

        let output = self
            .runner
            .run(
                &binary,
                &[
                    "label", "create", name, "--color", "ededed", "-R", &self.repo,
                ],
            )
            .await
            .map_err(|e| {
                CoreError::Platform(format!("Failed to spawn gitcode label create: {e}"))
            })?;

        if !output.status.success() {
            let gitcode_err = parse_gitcode_error(&output.stderr);
            return Err(CoreError::Platform(format!(
                "Failed to auto-create label '{name}': {gitcode_err}"
            )));
        }

        Ok(())
    }
}

#[async_trait]
impl<R: CommandRunner + 'static> IssueProvider for GitCodeIssueProvider<R> {
    async fn create(&self, args: CreateIssueArgs) -> Result<IssueData> {
        let binary = crate::gitcode_binary();
        let mut cmd_args: Vec<&str> = vec![
            "issue",
            "create",
            "-R",
            &self.repo,
            "--title",
            &args.title,
            "--json",
        ];

        if let Some(body) = &args.body {
            cmd_args.push("--body");
            cmd_args.push(body);
        }
        for label in &args.labels {
            cmd_args.push("--label");
            cmd_args.push(label);
        }
        for assignee in &args.assignees {
            cmd_args.push("--assignee");
            cmd_args.push(assignee);
        }

        debug!(repo = %self.repo, title = %args.title, "spawning gitcode issue create");
        let output = self
            .runner
            .run(&binary, &cmd_args)
            .await
            .map_err(|e| CoreError::Platform(format!("{e}")))?;
        if !output.status.success() {
            // gc issue create fails when a requested label doesn't exist.
            // Auto-create missing labels and retry once.
            let missing = extract_missing_labels_from_error(&output.stderr);
            if !missing.is_empty() {
                debug!(
                    repo = %self.repo,
                    missing_count = missing.len(),
                    "auto-creating missing label(s) before retrying issue create"
                );
                for label in &missing {
                    self.ensure_label_exists(label).await?;
                }

                let retry_output = self.runner.run(&binary, &cmd_args).await.map_err(|e| {
                    CoreError::Platform(format!("Failed to spawn gitcode on retry: {e}"))
                })?;
                if !retry_output.status.success() {
                    return Err(CoreError::Platform(
                        parse_gitcode_error(&retry_output.stderr).to_string(),
                    ));
                }
                return serde_json::from_slice::<IssueApiResponse>(&retry_output.stdout)
                    .map(IssueData::from)
                    .map_err(CoreError::Serialization);
            }

            return Err(CoreError::Platform(
                parse_gitcode_error(&output.stderr).to_string(),
            ));
        }
        serde_json::from_slice::<IssueApiResponse>(&output.stdout)
            .map(IssueData::from)
            .map_err(CoreError::Serialization)
    }

    async fn list(&self, args: ListIssueArgs) -> Result<Vec<IssueData>> {
        let binary = crate::gitcode_binary();
        let mut cmd_args: Vec<&str> = vec!["issue", "list", "-R", &self.repo, "--json"];

        if let Some(ref state) = args.state {
            cmd_args.push("--state");
            cmd_args.push(match state {
                State::Open => "open",
                State::Closed => "closed",
            });
        }
        if let Some(ref search) = args.search {
            cmd_args.push("--search");
            cmd_args.push(search);
        }
        let limit_str = args.limit.map(|limit| limit.to_string());
        if let Some(ref limit) = limit_str {
            cmd_args.push("--limit");
            cmd_args.push(limit);
        }
        for label in &args.labels {
            cmd_args.push("--label");
            cmd_args.push(label);
        }

        debug!(repo = %self.repo, "spawning gitcode issue list");
        let output = self
            .runner
            .run(&binary, &cmd_args)
            .await
            .map_err(|e| CoreError::Platform(format!("{e}")))?;
        if !output.status.success() {
            return Err(CoreError::Platform(
                parse_gitcode_error(&output.stderr).to_string(),
            ));
        }
        let issues: Vec<IssueApiResponse> =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;
        Ok(issues.into_iter().map(IssueData::from).collect())
    }

    async fn view(&self, number: u64) -> Result<IssueData> {
        let binary = crate::gitcode_binary();
        let number_str = number.to_string();
        debug!(repo = %self.repo, number, "spawning gitcode issue view");
        let output = self
            .runner
            .run(
                &binary,
                &["issue", "view", &number_str, "-R", &self.repo, "--json"],
            )
            .await
            .map_err(|e| CoreError::Platform(format!("{e}")))?;

        if !output.status.success() {
            return Err(CoreError::Platform(
                parse_gitcode_error(&output.stderr).to_string(),
            ));
        }
        serde_json::from_slice::<IssueApiResponse>(&output.stdout)
            .map(IssueData::from)
            .map_err(CoreError::Serialization)
    }

    async fn close(&self, number: u64) -> Result<IssueData> {
        let binary = crate::gitcode_binary();
        let number_str = number.to_string();
        debug!(repo = %self.repo, number, "spawning gitcode issue close");
        let output = self
            .runner
            .run(
                &binary,
                &[
                    "issue",
                    "close",
                    &number_str,
                    "-R",
                    &self.repo,
                    "--yes",
                    "--json",
                ],
            )
            .await
            .map_err(|e| CoreError::Platform(format!("{e}")))?;

        if !output.status.success() {
            return Err(CoreError::Platform(
                parse_gitcode_error(&output.stderr).to_string(),
            ));
        }
        serde_json::from_slice::<CloseApiResponse>(&output.stdout)
            .map(IssueData::from)
            .map_err(CoreError::Serialization)
    }

    async fn reopen(&self, number: u64) -> Result<IssueData> {
        let binary = crate::gitcode_binary();
        let number_str = number.to_string();
        debug!(repo = %self.repo, number, "spawning gitcode issue reopen");
        let output = self
            .runner
            .run(
                &binary,
                &[
                    "issue",
                    "reopen",
                    &number_str,
                    "-R",
                    &self.repo,
                    "--yes",
                    "--json",
                ],
            )
            .await
            .map_err(|e| CoreError::Platform(format!("{e}")))?;

        if !output.status.success() {
            return Err(CoreError::Platform(
                parse_gitcode_error(&output.stderr).to_string(),
            ));
        }
        serde_json::from_slice::<CloseApiResponse>(&output.stdout)
            .map(IssueData::from)
            .map_err(CoreError::Serialization)
    }

    /// 在指定 Issue 上添加评论。
    ///
    /// 调用 `gc issue comment <number> --repo <repo> --body "<body>" --json
    /// id,body,author,createdAt` 发布评论，并返回新建评论的数据。
    ///
    /// # Errors
    ///
    /// 当 Issue 不存在、`body` 为空或 `gitcode` CLI 调用失败时返回错误。
    async fn comment(&self, number: u64, body: &str) -> Result<CommentData> {
        let binary = crate::gitcode_binary();
        let number_str = number.to_string();
        debug!(repo = %self.repo, number, "spawning `gc issue comment`");

        let output = self
            .runner
            .run(
                &binary,
                &[
                    "issue",
                    "comment",
                    &number_str,
                    "-R",
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

        let api: CommentApiResponse =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(CommentData::from(api))
    }

    /// 为指定 Issue 添加一个或多个标签。
    ///
    /// 调用 `gc issue edit <number> -R <repo> --add-label <label>` 逐个添加标签。
    /// 如果 `labels` 为空，不进行任何调用并返回成功。
    ///
    /// # 自动创建缺失标签
    ///
    /// 当 `gc issue edit --add-label` 因标签不存在而失败时，本方法会自动调用
    /// `gc label create` 创建缺失的标签（使用默认颜色 `ededed`），然后重试原操作。
    ///
    /// # Errors
    ///
    /// 当 Issue 不存在、标签创建失败或 `gitcode` CLI 调用失败时返回错误。
    async fn add_labels(&self, number: u64, labels: &[String]) -> Result<()> {
        let binary = crate::gitcode_binary();
        let number_str = number.to_string();
        debug!(
            repo = %self.repo,
            number,
            label_count = labels.len(),
            "spawning `gc issue edit --add-label`"
        );

        let mut cmd_args: Vec<&str> = vec!["issue", "edit", &number_str, "-R", &self.repo];
        for label in labels {
            cmd_args.push("--add-label");
            cmd_args.push(label);
        }

        let output = self
            .runner
            .run(&binary, &cmd_args)
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gitcode: {e}")))?;

        if output.status.success() {
            return Ok(());
        }

        // gc issue edit --add-label fails when a label doesn't exist.
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
    /// 调用 `gc issue edit <number> --repo <repo> --remove-label <label>` 移除标签。
    ///
    /// # Errors
    ///
    /// 当 Issue 不存在、标签未附加到该 Issue 或 `gitcode` CLI 调用失败时返回错误。
    async fn remove_label(&self, number: u64, label: &str) -> Result<()> {
        let binary = crate::gitcode_binary();
        let number_str = number.to_string();
        debug!(repo = %self.repo, number, label, "spawning `gc issue edit --remove-label`");

        let output = self
            .runner
            .run(
                &binary,
                &[
                    "issue",
                    "edit",
                    &number_str,
                    "-R",
                    &self.repo,
                    "--remove-label",
                    label,
                ],
            )
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gitcode: {e}")))?;

        if !output.status.success() {
            let gitcode_err = parse_gitcode_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gitcode_err}")));
        }

        Ok(())
    }
}

/// 从 `gc issue edit --add-label` 的 stderr 中提取缺失的标签名。
///
/// GitCode CLI 是 `gh` 的分支，错误格式与 `gh` 一致：
/// `'<label>' not found`。
fn extract_missing_labels_from_error(stderr: &[u8]) -> Vec<String> {
    let text = String::from_utf8_lossy(stderr);
    let mut labels = Vec::new();
    let mut search_from: usize = 0;

    while let Some(rel_open) = text[search_from..].find('\'') {
        let open_pos = search_from + rel_open + 1;
        let Some(rel_close) = text[open_pos..].find('\'') else {
            break;
        };
        let close_pos = open_pos + rel_close;
        let after_close = &text[close_pos + 1..];

        if after_close.starts_with(" not found") {
            let label = text[open_pos..close_pos].to_string();
            if !label.is_empty() {
                labels.push(label);
            }
        }
        search_from = close_pos + 1;
    }

    labels
}

#[cfg(test)]
mod tests {
    use gitflow_cli_core::types::UserSummary;

    use super::*;
    use crate::runner::{MockCommandRunner, SequencedMockCommandRunner};

    #[test]
    fn test_should_construct_gitcode_issue_provider() {
        let provider = GitCodeIssueProvider::new("octocat/hello-world");
        assert_eq!(provider.repo, "octocat/hello-world");
    }

    #[test]
    fn test_should_construct_gitcode_issue_provider_from_string() {
        let repo = String::from("octocat/hello-world");
        let provider = GitCodeIssueProvider::new(repo);
        assert_eq!(provider.repo, "octocat/hello-world");
    }

    #[test]
    fn test_should_deserialize_issue_data_from_gc_output() {
        let gc_json = br#"{
            "number": 42,
            "title": "Fix login bug",
            "body": "Reproduced on v1.2.3",
            "state": "open",
            "labels": [
                {"name": "bug", "color": "d73a4a", "description": "Something isn't working"}
            ],
            "author": {"login": "octocat", "id": "1"},
            "assignees": [{"login": "alice", "id": "7"}],
            "createdAt": "2026-01-15T09:30:00Z",
            "updatedAt": "2026-01-16T11:00:00Z",
            "url": "https://gitcode.com/octocat/hello-world/issues/42"
        }"#;

        let issue: IssueData = serde_json::from_slice(gc_json).expect("valid IssueData JSON");
        assert_eq!(issue.number, 42);
        assert_eq!(issue.title, "Fix login bug");
        assert_eq!(issue.state, State::Open);
        assert_eq!(issue.labels.len(), 1);
        assert_eq!(issue.author.login, "octocat");
        assert_eq!(issue.assignees.len(), 1);
        assert_eq!(
            issue.url,
            "https://gitcode.com/octocat/hello-world/issues/42"
        );
    }

    #[test]
    fn test_should_deserialize_empty_issue_list_from_gc_output() {
        let gc_json = b"[]";
        let issues: Vec<IssueData> = serde_json::from_slice(gc_json).expect("valid IssueData list");
        assert!(issues.is_empty());
    }

    #[test]
    fn test_should_debug_format_provider() {
        let provider = GitCodeIssueProvider::new("octocat/hello-world");
        let debug = format!("{provider:?}");
        assert!(debug.contains("GitCodeIssueProvider"));
        assert!(debug.contains("octocat/hello-world"));
    }

    #[test]
    fn test_should_deserialize_closed_issue_from_gc_close_output() {
        let gc_json = br#"{
            "number": 10,
            "title": "Fixed typo",
            "body": null,
            "state": "closed",
            "labels": [],
            "author": {"login": "dev", "id": "5"},
            "assignees": [],
            "createdAt": "2026-06-01T08:00:00Z",
            "updatedAt": "2026-06-02T12:00:00Z",
            "url": "https://gitcode.com/octocat/hello-world/issues/10"
        }"#;

        let issue: IssueData = serde_json::from_slice(gc_json).expect("valid closed IssueData");
        assert_eq!(issue.number, 10);
        assert_eq!(issue.state, State::Closed);
    }

    #[test]
    fn test_should_deserialize_reopened_issue_from_gc_reopen_output() {
        let gc_json = br#"{
            "number": 10,
            "title": "Fixed typo",
            "body": null,
            "state": "open",
            "labels": [],
            "author": {"login": "dev", "id": "5"},
            "assignees": [],
            "createdAt": "2026-06-01T08:00:00Z",
            "updatedAt": "2026-06-03T09:00:00Z",
            "url": "https://gitcode.com/octocat/hello-world/issues/10"
        }"#;

        let issue: IssueData = serde_json::from_slice(gc_json).expect("valid reopened IssueData");
        assert_eq!(issue.number, 10);
        assert_eq!(issue.state, State::Open);
    }

    #[test]
    fn test_should_deserialize_comment_data_from_gc_comment_output() {
        let gc_json = br#"{
            "id": "1001",
            "body": "Thanks for reporting, looking into it.",
            "author": "maintainer",
            "created_at": "2026-06-15 14:00:00"
        }"#;

        let api: CommentApiResponse =
            serde_json::from_slice(gc_json).expect("valid CommentApiResponse");
        let comment = CommentData::from(api);
        assert_eq!(comment.id, 1001);
        assert_eq!(comment.body, "Thanks for reporting, looking into it.");
        assert_eq!(comment.author.login, "maintainer");
    }

    #[test]
    fn test_should_roundtrip_comment_data_via_serde() {
        let comment = CommentData {
            id: 77,
            body: "reviewed".into(),
            author: UserSummary {
                login: "alice".into(),
                id: "3".to_string(),
            },
            created_at: "2026-05-01T00:00:00Z".parse().expect("valid date"),
        };
        let json = serde_json::to_string(&comment).expect("serialize");
        let round_tripped: CommentData = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(round_tripped.id, comment.id);
        assert_eq!(round_tripped.body, comment.body);
        assert_eq!(round_tripped.author.login, comment.author.login);
    }

    #[test]
    fn test_should_create_provider_with_different_repos() {
        let r1 = GitCodeIssueProvider::new("org/repo-a");
        let r2 = GitCodeIssueProvider::new("org/repo-b");
        assert_eq!(r1.repo, "org/repo-a");
        assert_eq!(r2.repo, "org/repo-b");
    }

    #[test]
    fn test_should_clone_gitcode_issue_provider() {
        let original = GitCodeIssueProvider::new("owner/repo");
        let cloned = original.clone();
        assert_eq!(original.repo, cloned.repo);
    }

    // --- Failure-path tests using an injected MockCommandRunner ---

    fn sample_create_args() -> CreateIssueArgs {
        CreateIssueArgs {
            title: "Bug report".to_string(),
            body: Some("Steps to reproduce".to_string()),
            labels: vec!["bug".to_string()],
            assignees: vec!["alice".to_string()],
        }
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gc_fails_for_view() {
        let runner = MockCommandRunner::failure("issue not found", 256);
        let provider = GitCodeIssueProvider::with_runner("owner/repo", runner);

        let result = provider.view(999).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Platform(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_serialization_error_on_invalid_json_for_view() {
        let runner = MockCommandRunner::success("not valid json");
        let provider = GitCodeIssueProvider::with_runner("owner/repo", runner);

        let result = provider.view(1).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Serialization(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gc_fails_for_list() {
        let runner = MockCommandRunner::failure("forbidden", 256);
        let provider = GitCodeIssueProvider::with_runner("owner/repo", runner);

        let result = provider.list(ListIssueArgs::default()).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Platform(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_serialization_error_on_invalid_json_for_list() {
        let runner = MockCommandRunner::success("invalid");
        let provider = GitCodeIssueProvider::with_runner("owner/repo", runner);

        let result = provider.list(ListIssueArgs::default()).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Serialization(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gc_fails_for_create() {
        let runner = MockCommandRunner::failure("validation failed", 256);
        let provider = GitCodeIssueProvider::with_runner("owner/repo", runner);

        let result = provider.create(sample_create_args()).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Platform(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_serialization_error_on_invalid_json_for_create() {
        let runner = MockCommandRunner::success("not valid json");
        let provider = GitCodeIssueProvider::with_runner("owner/repo", runner);

        let result = provider.create(sample_create_args()).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Serialization(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gc_fails_for_close() {
        let runner = MockCommandRunner::failure("not found", 256);
        let provider = GitCodeIssueProvider::with_runner("owner/repo", runner);

        let result = provider.close(42).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Platform(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gc_fails_for_reopen() {
        let runner = MockCommandRunner::failure("not found", 256);
        let provider = GitCodeIssueProvider::with_runner("owner/repo", runner);

        let result = provider.reopen(42).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Platform(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gc_fails_for_comment() {
        let runner = MockCommandRunner::failure("not found", 256);
        let provider = GitCodeIssueProvider::with_runner("owner/repo", runner);

        let result = provider.comment(42, "a comment").await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Platform(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gc_fails_for_add_labels() {
        let runner = MockCommandRunner::failure("not found", 256);
        let provider = GitCodeIssueProvider::with_runner("owner/repo", runner);

        let result = provider.add_labels(42, &["bug".to_string()]).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Platform(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gc_fails_for_remove_label() {
        let runner = MockCommandRunner::failure("not found", 256);
        let provider = GitCodeIssueProvider::with_runner("owner/repo", runner);

        let result = provider.remove_label(42, "bug").await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_cli_core::CoreError::Platform(_)
        ));
    }

    // --- extract_missing_labels_from_error: pure-function tests ---

    #[test]
    fn test_should_extract_single_missing_label_from_gc_stderr() {
        let stderr =
            b"failed to update https://gitcode.com/owner/repo/issues/18: 'type:enhancement' not found";
        let missing = extract_missing_labels_from_error(stderr);
        assert_eq!(missing, vec!["type:enhancement".to_string()]);
    }

    #[test]
    fn test_should_return_empty_when_no_label_not_found_in_gc_stderr() {
        let stderr = b"gitcode: Not logged in";
        let missing = extract_missing_labels_from_error(stderr);
        assert!(missing.is_empty());
    }

    // --- add_labels: auto-create missing labels ---

    #[tokio::test]
    async fn test_should_auto_create_label_and_retry_on_add_labels() {
        // Sequence:
        // 1. `gc issue edit 18 --add-label type:enhancement` → fails
        // 2. `gc label create type:enhancement --color ededed -R owner/repo` → succeeds
        // 3. `gc issue edit 18 --add-label type:enhancement` → succeeds (retry)
        let runner = SequencedMockCommandRunner::from_results(&[
            (
                false,
                "failed to update https://gitcode.com/owner/repo/issues/18: 'type:enhancement' \
                 not found",
            ),
            (true, ""),
            (true, ""),
        ]);
        let provider = GitCodeIssueProvider::with_runner("owner/repo", runner);

        let result = provider
            .add_labels(18, &["type:enhancement".to_string()])
            .await;

        assert!(result.is_ok(), "expected Ok, got {result:?}");
    }

    #[tokio::test]
    async fn test_should_propagate_error_when_gc_label_create_fails() {
        let runner = SequencedMockCommandRunner::from_results(&[
            (false, "failed to update ...: 'ghost' not found"),
            (false, "gitcode: 403 Forbidden"),
        ]);
        let provider = GitCodeIssueProvider::with_runner("o/r", runner);

        let result = provider.add_labels(1, &["ghost".to_string()]).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_should_not_retry_on_non_label_error_gc() {
        let runner = SequencedMockCommandRunner::from_results(&[(false, "gitcode: Not logged in")]);
        let provider = GitCodeIssueProvider::with_runner("o/r", runner);

        let result = provider.add_labels(1, &["bug".to_string()]).await;

        assert!(result.is_err());
    }
}
