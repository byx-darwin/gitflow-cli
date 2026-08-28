//! GitCode Pull Request 提供者实现。
//!
//! 通过 `gitcode` CLI 实现 [`PrProvider`] trait，支持 Pull Request 的创建、列表、查看、
//! 关闭、合并、检出、草稿状态切换和分支同步。
//! 所有方法通过 [`CommandRunner`] 调用 `gitcode` CLI，捕获 stdout 并解析 JSON。
//! gitcode v0.6.x 的 JSON 架构（snake_case、`user` 键、嵌套 `head`/`base`）
//! 与 `gh` 不同，统一经 `PrApiResponse` 映射为 core 的 [`PrData`]。

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use gitflow_core::{
    CoreError, Result,
    pr::{CreatePrArgs, ListPrArgs, PrData, PrProvider},
    types::{CommentData, MergeResult, MergeStrategy, State, UserSummary},
};
use serde::Deserialize;
use tracing::debug;

use crate::{
    error::parse_gitcode_error,
    runner::{CommandRunner, RealCommandRunner},
};

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

/// gitcode CLI 评论响应类型，兼容两种已观测形态：
/// - v0.6.x：`user` 为对象、`created_at` 为带偏移 RFC3339
/// - 旧版本：`author` 为纯字符串、`created_at` 为 `YYYY-MM-DD HH:MM:SS`
#[derive(Debug, Clone, Deserialize)]
struct PrCommentApiResponse {
    #[serde(deserialize_with = "gitflow_core::types::deserialize_u64_or_string")]
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
                Some("closed" | "merged") => State::Closed,
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
            base_branch: api.base.map_or_else(String::new, |b| b.branch_ref),
            head_branch: api.head.map_or_else(String::new, |h| h.branch_ref),
            created_at: parse_time(api.created_at),
            updated_at: parse_time(api.updated_at),
            url: api.html_url.unwrap_or_default(),
        }
    }
}

/// GitCode Pull Request 提供者，通过 `gitcode` CLI 操作。
///
/// 该结构体通过调用 `gitcode` CLI 实现 [`PrProvider`] trait 的所有方法，
/// 使上层命令能够以统一的方式操作 GitCode Pull Request。
///
/// 命令执行通过 [`CommandRunner`] 抽象，生产环境默认使用
/// [`RealCommandRunner`]，测试可注入自定义 runner 以模拟成功或失败场景。
///
/// # Examples
///
/// ```no_run
/// use gitflow_gitcode::GitCodePrProvider;
///
/// let provider = GitCodePrProvider::new("octocat/hello-world");
/// ```
#[derive(Debug, Clone)]
pub struct GitCodePrProvider<R: CommandRunner = RealCommandRunner> {
    /// GitCode `owner/repo`，如 `"byx-darwin/gitflow-cli"`。
    repo: String,
    /// 用于执行 `gitcode` CLI 命令的 runner。
    runner: R,
}

impl GitCodePrProvider<RealCommandRunner> {
    /// 创建新的 GitCode Pull Request 提供者。
    ///
    /// `repo` 格式为 `owner/repo`。
    #[must_use]
    pub fn new(repo: impl Into<String>) -> Self {
        Self {
            repo: repo.into(),
            runner: RealCommandRunner,
        }
    }
}

impl<R: CommandRunner> GitCodePrProvider<R> {
    /// 使用自定义 [`CommandRunner`] 创建提供者。
    ///
    /// 主要用于测试，可注入模拟 runner 以控制 `gitcode` CLI 的输出。
    /// `repo` 格式为 `owner/repo`。
    #[must_use]
    pub fn with_runner(repo: impl Into<String>, runner: R) -> Self {
        Self {
            repo: repo.into(),
            runner,
        }
    }
}

#[async_trait]
impl<R: CommandRunner + 'static> PrProvider for GitCodePrProvider<R> {
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

        let final_body = gitflow_core::pr::format_closing_body(
            &args.body,
            &args.closes_issues,
            "Closes",
        );

        if let Some(body) = &final_body {
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
            return Err(parse_gitcode_error(&output.stderr).into());
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
                State::All => "all",
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
            return Err(parse_gitcode_error(&output.stderr).into());
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
            return Err(parse_gitcode_error(&output.stderr).into());
        }

        let api: PrApiResponse =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(api.into())
    }

    /// 关闭指定编号的 PR。
    ///
    /// 调用 `gitcode pr close <number> --repo <repo> --yes --json` 关闭 PR，
    /// 并返回更新后的完整 PR 数据。
    ///
    /// # Errors
    ///
    /// 当 PR 不存在、已关闭或 `gitcode` CLI 调用失败时返回错误。
    async fn close(&self, number: u64) -> Result<PrData> {
        let binary = crate::gitcode_binary();
        let number_str = number.to_string();
        debug!(repo = %self.repo, number, "spawning `gitcode pr close`");

        let output = self
            .runner
            .run(
                &binary,
                &[
                    "pr",
                    "close",
                    &number_str,
                    "--repo",
                    &self.repo,
                    "--yes",
                    "--json",
                ],
            )
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gitcode: {e}")))?;

        if !output.status.success() {
            return Err(parse_gitcode_error(&output.stderr).into());
        }

        let api: PrApiResponse =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(api.into())
    }

    /// 重新打开指定编号的 PR。
    ///
    /// 调用 `gitcode pr reopen <number> --repo <repo> --yes --json` 重新打开已关闭的 PR，
    /// 并返回更新后的完整 PR 数据。
    ///
    /// # Errors
    ///
    /// 当 PR 不存在、未关闭或 `gitcode` CLI 调用失败时返回错误。
    async fn reopen(&self, number: u64) -> Result<PrData> {
        let binary = crate::gitcode_binary();
        let number_str = number.to_string();
        debug!(repo = %self.repo, number, "spawning `gitcode pr reopen`");

        let output = self
            .runner
            .run(
                &binary,
                &[
                    "pr",
                    "reopen",
                    &number_str,
                    "--repo",
                    &self.repo,
                    "--yes",
                    "--json",
                ],
            )
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gitcode: {e}")))?;

        if !output.status.success() {
            return Err(parse_gitcode_error(&output.stderr).into());
        }

        let api: PrApiResponse =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(api.into())
    }

    /// 在指定 PR 上添加评论。
    ///
    /// 调用 `gitcode pr comment <number> --repo <repo> --body "<body>" --json`
    /// 发布评论，并返回新建评论的数据。
    ///
    /// # Errors
    ///
    /// 当 PR 不存在、`body` 为空或 `gitcode` CLI 调用失败时返回错误。
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
            return Err(parse_gitcode_error(&output.stderr).into());
        }

        let api: PrCommentApiResponse =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(api.into())
    }

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
            return Err(parse_gitcode_error(&output.stderr).into());
        }

        // `gitcode pr merge` outputs a human-readable message, not JSON.
        let message = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(MergeResult {
            merged: true,
            sha: None,
            message: Some(message),
        })
    }

    /// 在本地检出指定 PR 的分支。
    ///
    /// 调用 `gc pr checkout <number> --repo <repo>` 在本地工作区创建并切换到
    /// PR 的来源分支。如果本地已存在该分支，则尝试更新它。
    ///
    /// # Errors
    ///
    /// 当 PR 不存在、本地 git 操作失败或 `gitcode` CLI 调用失败时返回错误。
    async fn checkout(&self, number: u64) -> Result<()> {
        let binary = crate::gitcode_binary();
        let number_str = number.to_string();
        debug!(repo = %self.repo, number, "spawning `gc pr checkout`");

        let output = self
            .runner
            .run(
                &binary,
                &["pr", "checkout", &number_str, "--repo", &self.repo],
            )
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gitcode: {e}")))?;

        if !output.status.success() {
            return Err(parse_gitcode_error(&output.stderr).into());
        }

        Ok(())
    }

    /// 将草稿 PR 标记为可审查状态（ready for review）。
    ///
    /// 调用 `gc pr ready <number> --repo <repo>` 将草稿 PR 转为可审查状态，
    /// 并通过 `gc pr view` 重新获取更新后的 PR 数据。
    ///
    /// # Errors
    ///
    /// 当 PR 不存在、不是草稿状态或 `gitcode` CLI 调用失败时返回错误。
    async fn mark_ready(&self, number: u64) -> Result<PrData> {
        let binary = crate::gitcode_binary();
        let number_str = number.to_string();
        debug!(repo = %self.repo, number, "spawning `gc pr ready`");

        let output = self
            .runner
            .run(&binary, &["pr", "ready", &number_str, "--repo", &self.repo])
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gitcode: {e}")))?;

        if !output.status.success() {
            return Err(parse_gitcode_error(&output.stderr).into());
        }

        // `gc pr ready` does not return JSON; re-fetch the PR to get updated data.
        self.view(number).await
    }

    /// 将 PR 标记为草稿状态（work in progress）。
    ///
    /// 调用 `gc pr convert-to-draft <number> --repo <repo>` 将可审查的 PR 转为草稿，
    /// 并通过 `gc pr view` 重新获取更新后的 PR 数据。
    ///
    /// # Errors
    ///
    /// 当 PR 不存在、已是草稿状态或 `gitcode` CLI 调用失败时返回错误。
    async fn mark_wip(&self, number: u64) -> Result<PrData> {
        let binary = crate::gitcode_binary();
        let number_str = number.to_string();
        debug!(repo = %self.repo, number, "spawning `gc pr convert-to-draft`");

        let output = self
            .runner
            .run(
                &binary,
                &["pr", "convert-to-draft", &number_str, "--repo", &self.repo],
            )
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gitcode: {e}")))?;

        if !output.status.success() {
            return Err(parse_gitcode_error(&output.stderr).into());
        }

        // `gc pr convert-to-draft` does not return JSON; re-fetch the PR.
        self.view(number).await
    }

    /// 同步 PR 的分支（将 base 分支的最新变更合入 head 分支）。
    ///
    /// 调用 `gc pr update-branch <number> --repo <repo>` 将 PR 的来源分支
    /// 更新到与目标分支的最新状态同步，解决分支过时问题。
    ///
    /// # Errors
    ///
    /// 当 PR 不存在、同步存在冲突或 `gitcode` CLI 调用失败时返回错误。
    async fn sync_branch(&self, number: u64) -> Result<()> {
        let binary = crate::gitcode_binary();
        let number_str = number.to_string();
        debug!(repo = %self.repo, number, "spawning `gc pr update-branch`");

        let output = self
            .runner
            .run(
                &binary,
                &["pr", "update-branch", &number_str, "--repo", &self.repo],
            )
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gitcode: {e}")))?;

        if !output.status.success() {
            return Err(parse_gitcode_error(&output.stderr).into());
        }

        Ok(())
    }

    /// 获取指定 PR 的统一差异格式（unified diff）文本。
    ///
    /// 调用 `gc mr diff <number> -R <repo>` 获取平台原生的 diff 输出，
    /// 可直接用于 `git apply`。
    ///
    /// # Errors
    ///
    /// 当 PR 不存在或 `gitcode` CLI 调用失败时返回错误。
    async fn diff(&self, number: u64) -> Result<String> {
        let binary = crate::gitcode_binary();
        let number_str = number.to_string();
        debug!(repo = %self.repo, number, "spawning `gc mr diff`");

        let output = self
            .runner
            .run(&binary, &["mr", "diff", &number_str, "-R", &self.repo])
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gitcode mr diff: {e}")))?;

        if !output.status.success() {
            return Err(parse_gitcode_error(&output.stderr).into());
        }

        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }

    /// 获取指定 PR 的 patch 格式文本（含邮件头信息）。
    ///
    /// 调用 `gc mr patch <number> -R <repo>` 获取包含 commit 元数据的
    /// patch 格式输出，可用于 `git am`。
    ///
    /// # Errors
    ///
    /// 当 PR 不存在或 `gitcode` CLI 调用失败时返回错误。
    async fn patch(&self, number: u64) -> Result<String> {
        let binary = crate::gitcode_binary();
        let number_str = number.to_string();
        debug!(repo = %self.repo, number, "spawning `gc mr patch`");

        let output = self
            .runner
            .run(&binary, &["mr", "patch", &number_str, "-R", &self.repo])
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gitcode mr patch: {e}")))?;

        if !output.status.success() {
            return Err(parse_gitcode_error(&output.stderr).into());
        }

        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runner::MockCommandRunner;

    /// gitcode CLI v0.6.1 `pr list/view --json` 的真实输出结构（2026-07-31 实测捕获，已精简）。
    fn real_gitcode_pr_json() -> &'static str {
        r###"{
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
        }"###
    }

    #[test]
    fn test_should_map_real_gitcode_pr_response_to_pr_data() {
        let api: PrApiResponse =
            serde_json::from_str(real_gitcode_pr_json()).expect(r"valid gitcode v0.6.1 PR JSON");
        let pr: PrData = api.into();

        assert_eq!(pr.number, 52);
        assert_eq!(pr.title, r"test(badge): 引擎规则函数测试覆盖");
        assert_eq!(pr.state, State::Closed, r"merged 必须映射为 Closed");
        assert!(!pr.draft);
        assert_eq!(pr.author.login, "byx-darwin");
        assert_eq!(pr.author.id, "66767cd4096c81780c61bf07");
        assert_eq!(pr.base_branch, "master");
        assert_eq!(pr.head_branch, "test/88-engine-rule-coverage");
        assert_eq!(
            pr.url,
            "https://gitcode.com/byx-darwin/go-beniofit/merge_requests/52"
        );
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
        let api: PrApiResponse = serde_json::from_str(json).expect(r"minimal gitcode PR JSON");
        let pr: PrData = api.into();

        assert_eq!(pr.state, State::Open);
        assert!(pr.draft);
        assert_eq!(pr.body, None);
        assert_eq!(pr.head_branch, "feature/x");
        assert_eq!(pr.base_branch, "main");
        assert_eq!(pr.author.login, "dev");
    }

    #[test]
    fn test_should_construct_gitcode_pr_provider() {
        let provider = GitCodePrProvider::new("octocat/hello-world");
        assert_eq!(provider.repo, "octocat/hello-world");
    }

    #[test]
    fn test_should_construct_gitcode_pr_provider_from_string() {
        let repo = String::from("octocat/hello-world");
        let provider = GitCodePrProvider::new(repo);
        assert_eq!(provider.repo, "octocat/hello-world");
    }

    #[test]
    fn test_should_deserialize_empty_pr_list_from_gc_output() {
        let gc_json = b"[]";
        let prs: Vec<PrData> = serde_json::from_slice(gc_json).expect("valid PrData list");
        assert!(prs.is_empty());
    }

    #[test]
    fn test_should_debug_format_provider() {
        let provider = GitCodePrProvider::new("octocat/hello-world");
        let debug = format!("{provider:?}");
        assert!(debug.contains("GitCodePrProvider"));
        assert!(debug.contains("octocat/hello-world"));
    }

    #[test]
    fn test_should_deserialize_comment_data_from_gc_pr_comment_output() {
        let gc_json = br#"{
            "id": 2002,
            "body": "Approved, merging now.",
            "author": {"login": "reviewer", "id": "88"},
            "createdAt": "2026-06-20T16:00:00Z"
        }"#;

        let comment: CommentData = serde_json::from_slice(gc_json).expect("valid CommentData");
        assert_eq!(comment.id, 2002);
        assert_eq!(comment.body, "Approved, merging now.");
        assert_eq!(comment.author.login, "reviewer");
        assert_eq!(comment.author.id, "88");
    }

    #[test]
    fn test_should_deserialize_merge_result_from_gc_merge_output() {
        let gc_text = b"Pull request #123 was successfully merged.\n";
        let message = String::from_utf8_lossy(gc_text).trim().to_string();
        let result = MergeResult {
            merged: true,
            sha: None,
            message: Some(message),
        };

        assert!(result.merged);
        assert!(result.message.as_deref().is_some());
        assert_eq!(
            result.message.as_deref(),
            Some("Pull request #123 was successfully merged.")
        );
    }

    #[test]
    fn test_should_roundtrip_merge_result_via_serde() {
        let result = MergeResult {
            merged: true,
            sha: Some("deadbeef1234".into()),
            message: Some("Squash merged".into()),
        };
        let json = serde_json::to_string(&result).expect("serialize");
        let round_tripped: MergeResult = serde_json::from_str(&json).expect("deserialize");
        assert!(round_tripped.merged);
        assert_eq!(round_tripped.sha, result.sha);
        assert_eq!(round_tripped.message, result.message);
    }

    #[test]
    fn test_should_serialize_merge_result_skips_null_fields() {
        let result = MergeResult {
            merged: false,
            sha: None,
            message: None,
        };
        let json = serde_json::to_string(&result).expect("serialize");
        assert!(!json.contains("null"));
        assert_eq!(json, r#"{"merged":false}"#);
    }

    #[test]
    fn test_should_create_provider_with_different_repos() {
        let r1 = GitCodePrProvider::new("org/repo-a");
        let r2 = GitCodePrProvider::new("org/repo-b");
        assert_eq!(r1.repo, "org/repo-a");
        assert_eq!(r2.repo, "org/repo-b");
    }

    #[test]
    fn test_should_clone_gitcode_pr_provider() {
        let original = GitCodePrProvider::new("owner/repo");
        let cloned = original.clone();
        assert_eq!(original.repo, cloned.repo);
    }

    // --- Failure-path tests using an injected MockCommandRunner ---

    fn sample_create_args() -> CreatePrArgs {
        CreatePrArgs {
            title: "New feature".to_string(),
            body: Some("Adds a feature".to_string()),
            head: "feature/x".to_string(),
            base: "main".to_string(),
            draft: false,
            repo: None,
            closes_issues: vec![],
        }
    }

    #[test]
    fn test_should_format_gitcode_body_with_closing_issues() {
        use gitflow_core::pr::format_closing_body;

        let body = Some("PR description".to_string());
        let issues = vec![10u64, 11];
        let result = format_closing_body(&body, &issues, "Closes");
        assert_eq!(
            result,
            Some("PR description\n\nCloses #10\nCloses #11".to_string())
        );
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gc_fails_for_create() {
        let runner = MockCommandRunner::failure("validation failed", 256);
        let provider = GitCodePrProvider::with_runner("owner/repo", runner);

        let result = provider.create(sample_create_args()).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_serialization_error_on_invalid_json_for_create() {
        let runner = MockCommandRunner::success("not valid json");
        let provider = GitCodePrProvider::with_runner("owner/repo", runner);

        let result = provider.create(sample_create_args()).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Serialization(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gc_fails_for_list() {
        let runner = MockCommandRunner::failure("forbidden", 256);
        let provider = GitCodePrProvider::with_runner("owner/repo", runner);

        let result = provider.list(ListPrArgs::default()).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_serialization_error_on_invalid_json_for_list() {
        let runner = MockCommandRunner::success("invalid");
        let provider = GitCodePrProvider::with_runner("owner/repo", runner);

        let result = provider.list(ListPrArgs::default()).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Serialization(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gc_fails_for_view() {
        let runner = MockCommandRunner::failure("pr not found", 256);
        let provider = GitCodePrProvider::with_runner("owner/repo", runner);

        let result = provider.view(999).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_serialization_error_on_invalid_json_for_view() {
        let runner = MockCommandRunner::success("not valid json");
        let provider = GitCodePrProvider::with_runner("owner/repo", runner);

        let result = provider.view(1).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Serialization(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gc_fails_for_close() {
        let runner = MockCommandRunner::failure("not found", 256);
        let provider = GitCodePrProvider::with_runner("owner/repo", runner);

        let result = provider.close(42).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gc_fails_for_reopen() {
        let runner = MockCommandRunner::failure("not found", 256);
        let provider = GitCodePrProvider::with_runner("owner/repo", runner);

        let result = provider.reopen(42).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gc_fails_for_merge() {
        let runner = MockCommandRunner::failure("merge conflict", 256);
        let provider = GitCodePrProvider::with_runner("owner/repo", runner);

        let result = provider.merge(42, None).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gc_fails_for_comment() {
        let runner = MockCommandRunner::failure("not found", 256);
        let provider = GitCodePrProvider::with_runner("owner/repo", runner);

        let result = provider.comment(42, "a comment").await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gc_fails_for_checkout() {
        let runner = MockCommandRunner::failure("git error", 256);
        let provider = GitCodePrProvider::with_runner("owner/repo", runner);

        let result = provider.checkout(42).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gc_fails_for_sync_branch() {
        let runner = MockCommandRunner::failure("sync conflict", 256);
        let provider = GitCodePrProvider::with_runner("owner/repo", runner);

        let result = provider.sync_branch(42).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gc_fails_for_mark_ready() {
        let runner = MockCommandRunner::failure("not a draft", 256);
        let provider = GitCodePrProvider::with_runner("owner/repo", runner);

        let result = provider.mark_ready(42).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gc_fails_for_mark_wip() {
        let runner = MockCommandRunner::failure("already a draft", 256);
        let provider = GitCodePrProvider::with_runner("owner/repo", runner);

        let result = provider.mark_wip(42).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    // --- diff() tests ---

    #[tokio::test]
    async fn test_should_fetch_mr_diff() {
        use gitflow_core::pr::PrProvider;

        let diff_output = "diff --git a/src/main.rs b/src/main.rs\nindex 1234567..abcdefg \
                           100644\n--- a/src/main.rs\n+++ b/src/main.rs\n@@ -1 +1 @@\n-old\n+new\n";
        let runner = MockCommandRunner::success(diff_output);
        let provider = GitCodePrProvider::with_runner("owner/repo", runner);

        let result = provider.diff(42).await.expect("diff should succeed");
        assert_eq!(result, diff_output);
    }

    #[tokio::test]
    async fn test_should_return_error_when_mr_diff_fails() {
        use gitflow_core::pr::PrProvider;

        let runner = MockCommandRunner::failure("404 not found", 256);
        let provider = GitCodePrProvider::with_runner("owner/repo", runner);

        let result = provider.diff(999).await;
        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    // --- patch() tests ---

    #[tokio::test]
    async fn test_should_fetch_mr_patch() {
        use gitflow_core::pr::PrProvider;

        let patch_output =
            "From abc123\nSubject: [PATCH] Update file\n\ndiff --git a/src/main.rs b/src/main.rs\n";
        let runner = MockCommandRunner::success(patch_output);
        let provider = GitCodePrProvider::with_runner("owner/repo", runner);

        let result = provider.patch(42).await.expect("patch should succeed");
        assert_eq!(result, patch_output);
    }

    #[tokio::test]
    async fn test_should_return_error_when_mr_patch_fails() {
        use gitflow_core::pr::PrProvider;

        let runner = MockCommandRunner::failure("404 not found", 256);
        let provider = GitCodePrProvider::with_runner("owner/repo", runner);

        let result = provider.patch(999).await;
        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    // --- Call-shape regression tests (Issue #90) ---

    use crate::runner::RecordingMockRunner;

    #[tokio::test]
    async fn test_should_not_pass_field_list_to_pr_view() {
        let runner = RecordingMockRunner::success(real_gitcode_pr_json());
        let provider = GitCodePrProvider::with_runner("octocat/hello-world", runner.clone());

        let pr = provider
            .view(20)
            .await
            .expect("view should parse real schema");

        assert_eq!(pr.number, 52);
        let calls = runner.calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(
            calls[0],
            vec![
                "pr",
                "view",
                "20",
                "--repo",
                "octocat/hello-world",
                "--json"
            ],
            "gitcode --json 是布尔标志，不得携带字段列表位置参数"
        );
    }

    #[tokio::test]
    async fn test_should_pass_yes_flag_to_pr_close() {
        let runner = RecordingMockRunner::success(real_gitcode_pr_json());
        let provider = GitCodePrProvider::with_runner("o/r", runner.clone());

        provider.close(9).await.expect("close should succeed");

        let args = &runner.calls()[0];
        assert!(
            args.contains(&"--yes".to_string()),
            "close 必须跳过确认提示"
        );
        assert!(
            !args
                .windows(2)
                .any(|w| { w[0] == "--json" && w[1] != "--yes" && !w[1].starts_with('-') }),
            "--json 后不得跟随字段列表"
        );
    }

    #[tokio::test]
    async fn test_should_pass_limit_flag_to_pr_list() {
        let runner = RecordingMockRunner::success(&format!("[{}]", real_gitcode_pr_json()));
        let provider = GitCodePrProvider::with_runner("o/r", runner.clone());

        let prs = provider
            .list(ListPrArgs {
                state: Some(State::Open),
                limit: Some(5),
            })
            .await
            .expect("list should succeed");

        assert_eq!(prs.len(), 1);
        let args = &runner.calls()[0];
        assert!(args.contains(&"--limit".to_string()));
        assert!(args.contains(&"5".to_string()));
        assert!(args.contains(&"--state".to_string()));
        assert!(args.contains(&"open".to_string()));
    }

    #[tokio::test]
    async fn test_should_map_squash_strategy_to_method_flag() {
        let runner = RecordingMockRunner::success("Merged pull request !52");
        let provider = GitCodePrProvider::with_runner("o/r", runner.clone());

        let result = provider
            .merge(52, Some(MergeStrategy::Squash))
            .await
            .expect("merge");

        assert!(result.merged);
        let args = &runner.calls()[0];
        let method_pos = args
            .iter()
            .position(|a| a == "--method")
            .expect("--method must be passed");
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

    #[tokio::test]
    async fn test_should_not_pass_field_list_to_pr_comment() {
        let comment_json = r#"{"id": "9001", "body": "LGTM", "user": {"login": "rev", "id": "u9"}, "created_at": "2026-07-30T12:00:00+08:00"}"#;
        let runner = RecordingMockRunner::success(comment_json);
        let provider = GitCodePrProvider::with_runner("o/r", runner.clone());

        let comment = provider
            .comment(52, "LGTM")
            .await
            .expect("comment should parse");

        assert_eq!(comment.id, 9001);
        assert_eq!(comment.author.login, "rev");
        assert_eq!(
            runner.calls()[0],
            vec![
                "pr", "comment", "52", "--repo", "o/r", "--body", "LGTM", "--json"
            ]
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
}

#[cfg(test)]
mod contract_tests {
    //! gitcode CLI v0.6.1 JSON 架构契约测试。
    //!
    //! 夹具来源：2026-07-31 对 gitcode CLI v0.6.1
    //!（commit c20f71f67ead1d748e78391cd9e470c2ea51b887, built 2026-06-05）
    //! `pr list -R byx-darwin/go-beniofit --json --state all` 的真实捕获。
    //! 若 gitcode CLI 升级导致这些测试失败，说明上游架构变更，需要更新
    //! 适配器映射并重新捕获夹具（参见路线图"契约测试 + 兼容性矩阵"单元）。

    use gitflow_core::pr::ListPrArgs;

    use super::*;
    use crate::runner::MockCommandRunner;

    const PR_LIST_FIXTURE: &str = include_str!("../tests/fixtures/pr_list_gitcode_v0.6.json");

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
