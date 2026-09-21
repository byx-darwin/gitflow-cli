//! GitCode Issue 提供者实现。
//!
//! 通过 `gitcode` CLI 实现 [`IssueProvider`] trait。GitCode CLI
//! 使用 `-R` 指定仓库、`--json` 为布尔标志、`version` 子命令检测版本。
//! JSON 响应字段名与 GitHub/GitLab CLI 不同（`user` 而非 `author` 等），
//! 通过 [`IssueApiResponse`] 做字段映射后转换为 core 类型。

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use gitflow_core::{
    CoreError, DEFAULT_LIST_LIMIT, FetchStrategy, Paged, Result, Session, fetch_capped,
    issue::{CreateIssueArgs, EditIssueArgs, IssueData, IssueProvider, ListIssueArgs},
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
    #[serde(default)]
    milestone: Option<MilestoneRefApi>,
}

/// gitcode CLI 嵌入在 issue/PR 响应中的 `milestone` 对象的最小字段集。
///
/// 真实形状（2026-09-21 对 `byx-darwin/NexaTrade` issue #1 的实测捕获，
/// `gitcode issue view --json`）：
/// ```json
/// {"id": null, "number": 866493, "title": "gf-357-test-milestone",
///  "description": "", "state": "active", "due_on": "2026-10-21"}
/// ```
/// 只映射 `number`/`title`：与 `gitflow_core::types::MilestoneRef` 的契约一致，
/// `id`/`description`/`state`/`due_on` 由 `gf milestone view` 单独提供。
#[derive(Debug, Clone, Deserialize)]
struct MilestoneRefApi {
    number: u64,
    title: String,
}

impl From<MilestoneRefApi> for gitflow_core::types::MilestoneRef {
    fn from(api: MilestoneRefApi) -> Self {
        Self {
            number: api.number,
            title: api.title,
        }
    }
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
            milestone: api.milestone.map(Into::into),
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

/// gitcode CLI `issue comment --json` 的响应类型，兼容两种已观测形态：
/// - v0.6.x：`user` 为对象（含 `login`/`id`）、`id` 为数值、`created_at` 为带偏移 RFC3339
/// - 旧版本：`author` 为纯字符串（用户名）、`id` 为字符串、`created_at` 为 `YYYY-MM-DD HH:MM:SS`
#[derive(Debug, Clone, Deserialize)]
struct CommentApiResponse {
    #[serde(deserialize_with = "gitflow_core::types::deserialize_u64_or_string")]
    id: u64,
    #[serde(default)]
    body: String,
    #[serde(default)]
    author: Option<String>,
    #[serde(default)]
    user: Option<UserApi>,
    #[serde(default)]
    created_at: Option<String>,
}

impl From<CommentApiResponse> for CommentData {
    fn from(api: CommentApiResponse) -> Self {
        let author = api.user.map_or_else(
            || UserSummary {
                login: api.author.unwrap_or_else(|| "unknown".into()),
                id: String::new(),
            },
            UserSummary::from,
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

/// gitcode CLI `issue close/reopen --json` 的响应类型。
///
/// 只是一个精简确认对象——实测响应（2026-09-21，对 byx-darwin/NexaTrade 的
/// 真实 issue #3）为 `{"number":3,"state":"closed","owner":"...","repo":"...","url":"..."}`，
/// 没有 `title`/`body`/`created_at`/`milestone` 等字段。只需要 `number` 去调用
/// `view()` 拿完整数据，其余字段不建模，避免死代码。
#[derive(Debug, Clone, Deserialize)]
struct CloseApiResponse {
    number: u64,
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

    /// Create a new provider from a shared [`Session`].
    ///
    /// This enables state reuse across multiple operations in workflow chains.
    #[must_use]
    pub fn with_session(session: &Session) -> Self {
        Self {
            repo: session.repo.clone(),
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
            tracing::debug!(error = %gitcode_err, "Failed to auto-create label");
            return Err(CoreError::Platform(format!(
                "Failed to auto-create label '{name}': {}",
                gitcode_err.user_message
            )));
        }

        Ok(())
    }
}

impl<R: CommandRunner + Clone + 'static> GitCodeIssueProvider<R> {
    /// [`IssueProvider::list`] 的实际实现。
    ///
    /// 抽成普通（非 `async_trait` 装箱）的关联函数，避开泛型异步闭包在
    /// `async_trait` 装箱 Future 内部触发的 HRTB `Send` 检查缺陷。
    async fn list_impl(&self, args: ListIssueArgs) -> Result<Paged<IssueData>> {
        let binary = crate::gitcode_binary();
        let binary = &binary;
        let cap = args.limit.unwrap_or(DEFAULT_LIST_LIMIT);
        let repo = &self.repo;
        let runner = &self.runner;
        let state = args.state;
        let search = &args.search;
        let labels = &args.labels;
        let milestone = &args.milestone;
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
                let resolved_milestone_number_str;
                if let Some(identifier) = milestone {
                    let milestone_provider =
                        crate::GitCodeMilestoneProvider::with_runner(repo.as_str(), runner.clone());
                    let resolved = gitflow_core::label::resolve_milestone_identifier(
                        &milestone_provider,
                        identifier,
                    )
                    .await?;
                    resolved_milestone_number_str = resolved.number.to_string();
                    cmd_args.push("--milestone");
                    cmd_args.push(&resolved_milestone_number_str);
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
}

#[async_trait]
impl<R: CommandRunner + Clone + 'static> IssueProvider for GitCodeIssueProvider<R> {
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

        let resolved_milestone_number_str;
        if let Some(identifier) = &args.milestone {
            let milestone_provider = crate::GitCodeMilestoneProvider::with_runner(
                self.repo.as_str(),
                self.runner.clone(),
            );
            let resolved =
                gitflow_core::label::resolve_milestone_identifier(&milestone_provider, identifier)
                    .await?;
            resolved_milestone_number_str = resolved.number.to_string();
            cmd_args.push("--milestone");
            cmd_args.push(&resolved_milestone_number_str);
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
                    return Err(parse_gitcode_error(&retry_output.stderr).into());
                }
                return serde_json::from_slice::<IssueApiResponse>(&retry_output.stdout)
                    .map(IssueData::from)
                    .map_err(CoreError::Serialization);
            }

            return Err(parse_gitcode_error(&output.stderr).into());
        }
        serde_json::from_slice::<IssueApiResponse>(&output.stdout)
            .map(IssueData::from)
            .map_err(CoreError::Serialization)
    }

    /// 编辑 Issue 的标题和/或正文。
    ///
    /// 调用 `<gitcode_binary> issue edit <number> -R <repo> [--title T] [--body B]`，
    /// 成功后通过 [`view`](Self::view) 重新拉取最新数据并返回（不解析 `edit` 自身的
    /// stdout，避免依赖未经验证的响应结构）。
    ///
    /// # Errors
    ///
    /// 当 Issue 不存在或 `gitcode` CLI 调用失败时返回错误。
    async fn edit(&self, number: u64, args: EditIssueArgs) -> Result<IssueData> {
        let binary = crate::gitcode_binary();
        let number_str = number.to_string();
        debug!(repo = %self.repo, number, "spawning gitcode issue edit");

        let mut cmd_args: Vec<&str> = vec!["issue", "edit", &number_str, "-R", &self.repo];
        if let Some(title) = &args.title {
            cmd_args.push("--title");
            cmd_args.push(title);
        }
        if let Some(body) = &args.body {
            cmd_args.push("--body");
            cmd_args.push(body);
        }

        let resolved_milestone_number_str;
        match &args.milestone {
            None => {}
            Some(None) => {
                // 实测（2026-09-21 对 byx-darwin/NexaTrade issue #1）：`gitcode
                // issue edit --milestone 0` 被 CLI 解析层当作"未设置"直接拒绝
                // （"at least one edit option is required"）；`--milestone -1`
                // 会被后端静默丢弃而不是清除里程碑——`edit` 响应本身回显
                // milestone: null，但紧随其后的 `issue view` 仍显示原里程碑未变。
                // GitCode CLI 没有任何取消关联里程碑的手段，诚实报错而非假装成功。
                return Err(gitflow_core::CoreError::Platform(
                    "GitCode CLI does not support unassigning a milestone from an issue".into(),
                ));
            }
            Some(Some(identifier)) => {
                let milestone_provider = crate::GitCodeMilestoneProvider::with_runner(
                    self.repo.as_str(),
                    self.runner.clone(),
                );
                let resolved = gitflow_core::label::resolve_milestone_identifier(
                    &milestone_provider,
                    identifier,
                )
                .await?;
                resolved_milestone_number_str = resolved.number.to_string();
                cmd_args.push("--milestone");
                cmd_args.push(&resolved_milestone_number_str);
            }
        }

        let output = self
            .runner
            .run(&binary, &cmd_args)
            .await
            .map_err(|e| CoreError::Platform(format!("{e}")))?;

        if !output.status.success() {
            return Err(parse_gitcode_error(&output.stderr).into());
        }

        self.view(number).await
    }

    async fn list(&self, args: ListIssueArgs) -> Result<Paged<IssueData>> {
        self.list_impl(args).await
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
            return Err(parse_gitcode_error(&output.stderr).into());
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
            return Err(parse_gitcode_error(&output.stderr).into());
        }
        let close_result: CloseApiResponse =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;
        self.view(close_result.number).await
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
            return Err(parse_gitcode_error(&output.stderr).into());
        }
        let reopen_result: CloseApiResponse =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;
        self.view(reopen_result.number).await
    }

    /// 在指定 Issue 上添加评论。
    ///
    /// 调用 `gitcode issue comment <number> -R <repo> --body "<body>" --json`
    /// 发布评论，并返回新建评论的数据。
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
            return Err(parse_gitcode_error(&output.stderr).into());
        }

        let api: CommentApiResponse =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(CommentData::from(api))
    }

    /// 列出指定 Issue 的评论。
    ///
    /// 调用 `gitcode api /repos/{owner}/{repo}/issues/{number}/comments` 获取评论列表，
    /// 通过 `per_page`/`page` 查询参数逐页取到 `limit`。
    ///
    /// GitCode CLI 的 `api` 子命令是否支持这两个查询参数**未经实测**（本环境
    /// 无法获取 GitCode CLI 二进制）。若平台忽略它们，首页会短于 `per_page`，
    /// 翻页循环在第一次调用后即因短页而终止——退化为今天「只取首页」的行为，
    /// 不会死循环也不会丢数据。
    ///
    /// # Errors
    ///
    /// 当 Issue 不存在或 `gitcode` CLI 调用失败时返回错误。
    async fn list_comments(&self, number: u64, limit: Option<u32>) -> Result<Paged<CommentData>> {
        let cap = limit.unwrap_or(DEFAULT_LIST_LIMIT);
        let binary = crate::gitcode_binary();
        let binary = &binary;
        let repo = &self.repo;
        let runner = &self.runner;
        let per_page = cap.saturating_add(1).min(crate::GITCODE_API_MAX_PER_PAGE);

        debug!(repo = %self.repo, number, cap, "spawning `gitcode api` GET issue comments");

        fetch_capped(
            FetchStrategy::Paged { per_page },
            cap,
            |page, per_page| async move {
                let api_path = format!(
                    "/repos/{repo}/issues/{number}/comments?per_page={per_page}&page={page}"
                );

                let output = runner.run(binary, &["api", &api_path]).await.map_err(|e| {
                    CoreError::Platform(format!("Failed to spawn gitcode api: {e}"))
                })?;

                if !output.status.success() {
                    return Err(parse_gitcode_error(&output.stderr).into());
                }

                let comments: Vec<CommentApiResponse> =
                    serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;
                Ok(comments.into_iter().map(CommentData::from).collect())
            },
        )
        .await
    }

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

        let cmd_args: Vec<&str> = vec![
            "issue",
            "label",
            &number_str,
            "--add",
            &joined,
            "-R",
            &self.repo,
        ];

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
            return Err(parse_gitcode_error(&output.stderr).into());
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
            return Err(parse_gitcode_error(&retry_output.stderr).into());
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
                &[
                    "issue",
                    "label",
                    &number_str,
                    "--remove",
                    label,
                    "-R",
                    &self.repo,
                ],
            )
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gitcode: {e}")))?;

        if !output.status.success() {
            return Err(parse_gitcode_error(&output.stderr).into());
        }

        Ok(())
    }
}

/// 从 `gc issue label --add` 的 stderr 中提取缺失的标签名。
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
    use gitflow_core::types::UserSummary;

    use super::*;
    use crate::runner::{MockCommandRunner, RecordingMockRunner, SequencedMockCommandRunner};

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
            milestone: None,
        }
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gc_fails_for_view() {
        let runner = MockCommandRunner::failure("issue not found", 256);
        let provider = GitCodeIssueProvider::with_runner("owner/repo", runner);

        let result = provider.view(999).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_serialization_error_on_invalid_json_for_view() {
        let runner = MockCommandRunner::success("not valid json");
        let provider = GitCodeIssueProvider::with_runner("owner/repo", runner);

        let result = provider.view(1).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Serialization(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gc_fails_for_list() {
        let runner = MockCommandRunner::failure("forbidden", 256);
        let provider = GitCodeIssueProvider::with_runner("owner/repo", runner);

        let result = provider.list(ListIssueArgs::default()).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_serialization_error_on_invalid_json_for_list() {
        let runner = MockCommandRunner::success("invalid");
        let provider = GitCodeIssueProvider::with_runner("owner/repo", runner);

        let result = provider.list(ListIssueArgs::default()).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Serialization(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gc_fails_for_create() {
        let runner = MockCommandRunner::failure("validation failed", 256);
        let provider = GitCodeIssueProvider::with_runner("owner/repo", runner);

        let result = provider.create(sample_create_args()).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_serialization_error_on_invalid_json_for_create() {
        let runner = MockCommandRunner::success("not valid json");
        let provider = GitCodeIssueProvider::with_runner("owner/repo", runner);

        let result = provider.create(sample_create_args()).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Serialization(_)
        ));
    }

    #[tokio::test]
    async fn test_should_close_then_view_to_get_full_issue_with_milestone() {
        // 实测响应（2026-09-21，对 byx-darwin/NexaTrade 的真实 issue #3）：
        // `gitcode issue close --json` 只返回精简确认对象（number/state/owner/repo/url），
        // 没有 title/created_at/updated_at/milestone。close() 必须用这个精简形状拿到
        // number，再调用 view() 取回完整、正确的 IssueData（而不是伪造 title=""、
        // created_at=now() 并丢弃 milestone）。
        let runner = SequencedMockCommandRunner::from_results(&[
            (
                true,
                r#"{"number":3,"state":"closed","owner":"byx-darwin","repo":"NexaTrade","url":"https://gitcode.com/byx-darwin/NexaTrade/issues/3"}"#,
            ),
            (
                true,
                r#"{"number":"3","title":"Real title","body":"real body","state":"closed","labels":[],"user":{"login":"octocat","id":"1"},"assignees":[],"created_at":"2026-01-01T00:00:00Z","updated_at":"2026-01-02T00:00:00Z","html_url":"https://gitcode.com/owner/repo/issues/3","milestone":{"number":7,"title":"v3.0"}}"#,
            ),
        ]);
        let provider = GitCodeIssueProvider::with_runner("owner/repo", runner.clone());

        let issue = provider.close(3).await.expect("close should succeed");

        // 标题/时间戳来自第二次（view）响应，证明确实走了 view 而非直接转换精简响应
        assert_eq!(issue.number, 3);
        assert_eq!(issue.title, "Real title");
        assert_eq!(issue.body.as_deref(), Some("real body"));
        assert_eq!(issue.created_at.to_rfc3339(), "2026-01-01T00:00:00+00:00");
        assert_eq!(
            issue.milestone,
            Some(gitflow_core::types::MilestoneRef {
                number: 7,
                title: "v3.0".into(),
            })
        );
        let calls = runner.recorded_calls();
        assert_eq!(calls.len(), 2, "close 必须先 close 再 view 两次调用");
        assert!(calls[0].1.contains(&"close".to_string()));
        assert!(calls[1].1.contains(&"view".to_string()));
    }

    #[tokio::test]
    async fn test_should_reopen_then_view_to_get_full_issue_with_milestone() {
        let runner = SequencedMockCommandRunner::from_results(&[
            (
                true,
                r#"{"number":3,"state":"open","owner":"byx-darwin","repo":"NexaTrade","url":"https://gitcode.com/byx-darwin/NexaTrade/issues/3"}"#,
            ),
            (
                true,
                r#"{"number":"3","title":"Real title","body":null,"state":"open","labels":[],"user":{"login":"octocat","id":"1"},"assignees":[],"created_at":"2026-01-01T00:00:00Z","updated_at":"2026-01-02T00:00:00Z","html_url":"https://gitcode.com/owner/repo/issues/3","milestone":{"number":7,"title":"v3.0"}}"#,
            ),
        ]);
        let provider = GitCodeIssueProvider::with_runner("owner/repo", runner.clone());

        let issue = provider.reopen(3).await.expect("reopen should succeed");

        assert_eq!(issue.number, 3);
        assert_eq!(issue.title, "Real title");
        assert_eq!(
            issue.milestone,
            Some(gitflow_core::types::MilestoneRef {
                number: 7,
                title: "v3.0".into(),
            })
        );
        let calls = runner.recorded_calls();
        assert_eq!(calls.len(), 2, "reopen 必须先 reopen 再 view 两次调用");
        assert!(calls[0].1.contains(&"reopen".to_string()));
        assert!(calls[1].1.contains(&"view".to_string()));
    }

    #[tokio::test]
    async fn test_should_propagate_view_error_after_issue_close_succeeds() {
        let runner = SequencedMockCommandRunner::from_results(&[
            (true, r#"{"number":3,"state":"closed"}"#),
            (false, "gitcode: issue not found"),
        ]);
        let provider = GitCodeIssueProvider::with_runner("owner/repo", runner);

        let err = provider
            .close(3)
            .await
            .expect_err("view failure must propagate");

        assert!(matches!(err, gitflow_core::CoreError::Cli(_)));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gc_fails_for_close() {
        let runner = MockCommandRunner::failure("not found", 256);
        let provider = GitCodeIssueProvider::with_runner("owner/repo", runner);

        let result = provider.close(42).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gc_fails_for_reopen() {
        let runner = MockCommandRunner::failure("not found", 256);
        let provider = GitCodeIssueProvider::with_runner("owner/repo", runner);

        let result = provider.reopen(42).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gc_fails_for_comment() {
        let runner = MockCommandRunner::failure("not found", 256);
        let provider = GitCodeIssueProvider::with_runner("owner/repo", runner);

        let result = provider.comment(42, "a comment").await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_produce_complete_argv_for_list_comments_with_default_limit() {
        let runner = MockCommandRunner::success("[]");
        let provider = GitCodeIssueProvider::with_runner("owner/repo", runner.clone());

        let paged = provider
            .list_comments(359, None)
            .await
            .expect("list_comments should succeed");

        assert!(paged.items.is_empty());
        let calls = runner.recorded_calls();
        assert_eq!(calls[0].0, crate::gitcode_binary());
        assert_eq!(
            calls[0].1,
            vec![
                "api",
                "/repos/owner/repo/issues/359/comments?per_page=100&page=1"
            ]
        );
    }

    #[tokio::test]
    async fn test_should_build_well_formed_query_string_for_list_comments() {
        let runner = MockCommandRunner::success("[]");
        let provider = GitCodeIssueProvider::with_runner("owner/repo", runner.clone());

        provider
            .list_comments(359, None)
            .await
            .expect("list_comments should succeed");

        let calls = runner.recorded_calls();
        let api_path = &calls[0].1[1];
        assert_eq!(
            api_path.matches('?').count(),
            1,
            "must have exactly one '?', got: {api_path}"
        );
        assert_eq!(
            api_path.matches('&').count(),
            1,
            "must have exactly one '&', got: {api_path}"
        );
        // 整串相等，而非 `contains`：`"per_page=1001".contains("per_page=100")`
        // 与 `"per_page=100".contains("page=1")` 都为真，子串断言无法检测出
        // 页大小钳位失效或页号错误——正是本测试得名的那个缺陷。
        assert_eq!(
            api_path, "/repos/owner/repo/issues/359/comments?per_page=100&page=1",
            "got: {api_path}"
        );
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gc_fails_for_add_labels() {
        let runner = MockCommandRunner::failure("not found", 256);
        let provider = GitCodeIssueProvider::with_runner("owner/repo", runner);

        let result = provider.add_labels(42, &["bug".to_string()]).await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gc_fails_for_remove_label() {
        let runner = MockCommandRunner::failure("not found", 256);
        let provider = GitCodeIssueProvider::with_runner("owner/repo", runner);

        let result = provider.remove_label(42, "bug").await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    #[tokio::test]
    async fn test_should_edit_issue_and_view_result() {
        // Sequence: 1. `gitcode issue edit` → succeeds
        //           2. `gitcode issue view --json` → returns updated issue JSON
        let runner = SequencedMockCommandRunner::from_results(&[
            (true, ""),
            (
                true,
                r#"{"number":"42","title":"New title","body":"orig","state":"open","labels":[],"user":{"login":"octocat","id":"1"},"assignees":[],"created_at":"2026-01-01T00:00:00Z","updated_at":"2026-01-02T00:00:00Z","html_url":"https://gitcode.com/owner/repo/issues/42"}"#,
            ),
        ]);
        let provider = GitCodeIssueProvider::with_runner("owner/repo", runner);

        let issue = provider
            .edit(
                42,
                gitflow_core::issue::EditIssueArgs {
                    title: Some("New title".to_string()),
                    body: None,
                    milestone: None,
                },
            )
            .await
            .expect("edit should succeed");

        assert_eq!(issue.number, 42);
        assert_eq!(issue.title, "New title");
    }

    #[tokio::test]
    async fn test_should_invoke_issue_edit_subcommand_with_provided_fields() {
        let runner = RecordingMockRunner::success("");
        let provider = GitCodeIssueProvider::with_runner("o/r", runner.clone());

        // view() call afterward will fail on empty stdout — irrelevant to this test,
        // which only asserts the first (edit) call's argv.
        let _ = provider
            .edit(
                54,
                gitflow_core::issue::EditIssueArgs {
                    title: Some("T".to_string()),
                    body: Some("B".to_string()),
                    milestone: None,
                },
            )
            .await;

        assert_eq!(
            runner.calls()[0],
            vec![
                "issue", "edit", "54", "-R", "o/r", "--title", "T", "--body", "B"
            ]
        );
    }

    #[tokio::test]
    async fn test_should_return_platform_error_when_gc_fails_for_edit() {
        let runner = MockCommandRunner::failure("not found", 256);
        let provider = GitCodeIssueProvider::with_runner("owner/repo", runner);

        let result = provider
            .edit(42, gitflow_core::issue::EditIssueArgs::default())
            .await;

        assert!(matches!(
            result.unwrap_err(),
            gitflow_core::CoreError::Cli(_)
        ));
    }

    // --- milestone wiring: create/edit/list ---

    #[test]
    fn test_should_deserialize_issue_with_milestone() {
        // Real shape captured from `gitcode issue view --json` on
        // byx-darwin/NexaTrade issue #1 (2026-09-21), trimmed to the fields
        // this module reads (`id`/`description`/`state`/`due_on` ignored).
        let json = br#"{
            "number": "1",
            "title": "gf-357 milestone test issue",
            "body": "temp test issue for milestone shape investigation",
            "state": "open",
            "html_url": "https://gitcode.com/byx-darwin/NexaTrade/issues/1",
            "user": {"login": "byx-darwin", "id": "66767cd4096c81780c61bf07"},
            "assignees": [],
            "labels": [],
            "milestone": {
                "id": null,
                "number": 866493,
                "title": "gf-357-test-milestone",
                "description": "",
                "state": "active",
                "due_on": "2026-10-21"
            },
            "created_at": "2026-09-21T10:18:16+08:00",
            "updated_at": "2026-09-21T10:18:16+08:00"
        }"#;
        let api: IssueApiResponse = serde_json::from_slice(json).expect("deserialize");
        let issue: IssueData = api.into();
        assert_eq!(
            issue.milestone,
            Some(gitflow_core::types::MilestoneRef {
                number: 866_493,
                title: "gf-357-test-milestone".into()
            })
        );
    }

    #[test]
    fn test_should_deserialize_issue_with_no_milestone() {
        let json = br#"{
            "number": "1",
            "title": "t",
            "body": null,
            "state": "open",
            "html_url": "https://gitcode.com/o/r/issues/1",
            "user": null,
            "assignees": [],
            "labels": [],
            "created_at": "2026-09-21T10:18:16+08:00",
            "updated_at": "2026-09-21T10:18:16+08:00"
        }"#;
        let api: IssueApiResponse = serde_json::from_slice(json).expect("deserialize");
        let issue: IssueData = api.into();
        assert!(issue.milestone.is_none());
    }

    /// A single-item `gitcode milestone list --json` response resolving to
    /// number 3 / title "v2.0", matching `resolve_milestone_identifier`'s contract.
    fn milestone_list_fixture() -> String {
        r#"[{"number": 3, "title": "v2.0", "description": null, "state": "open", "due_on": null, "closed_issues": 0, "open_issues": 0}]"#.to_string()
    }

    #[tokio::test]
    async fn test_should_wire_resolved_milestone_number_into_issue_create() {
        // Sequence: 1. milestone resolution: `gitcode milestone list -R owner/repo --json ...`
        //           2. `gitcode issue create ... --milestone 3`
        let runner = SequencedMockCommandRunner::from_results(&[
            (true, &milestone_list_fixture()),
            (
                true,
                r#"{"number":"42","title":"New feature","body":"Description","state":"open","labels":[],"user":{"login":"octocat","id":"1"},"assignees":[],"created_at":"2026-01-01T00:00:00Z","updated_at":"2026-01-01T00:00:00Z","html_url":"https://gitcode.com/owner/repo/issues/42","milestone":{"number":3,"title":"v2.0"}}"#,
            ),
        ]);
        let provider = GitCodeIssueProvider::with_runner("owner/repo", runner.clone());

        let mut args = sample_create_args();
        args.milestone = Some("3".to_string()); // resolve by number

        let issue = provider.create(args).await.expect("create should succeed");

        assert_eq!(
            issue.milestone,
            Some(gitflow_core::types::MilestoneRef {
                number: 3,
                title: "v2.0".into()
            })
        );

        let calls = runner.recorded_calls();
        assert_eq!(calls.len(), 2);
        assert_eq!(
            calls[0].1.first().map(String::as_str),
            Some("milestone"),
            "first call must resolve the milestone via `gitcode milestone list`, got: {:?}",
            calls[0].1
        );
        let create_call = &calls[1].1;
        assert!(
            create_call
                .windows(2)
                .any(|w| w[0] == "--milestone" && w[1] == "3"),
            "issue create argv must carry the resolved milestone NUMBER (not title), got: \
             {create_call:?}"
        );
    }

    #[tokio::test]
    async fn test_should_propagate_error_when_milestone_identifier_not_found_on_issue_create() {
        let runner = MockCommandRunner::success("[]");
        let provider = GitCodeIssueProvider::with_runner("owner/repo", runner);

        let mut args = sample_create_args();
        args.milestone = Some("does-not-exist".to_string());

        let result = provider.create(args).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_should_wire_resolved_milestone_number_into_issue_edit() {
        // Sequence: 1. milestone resolution
        //           2. `gitcode issue edit <number> --milestone 3`
        //           3. `gitcode issue view --json` (edit() re-fetches via view)
        let runner = SequencedMockCommandRunner::from_results(&[
            (true, &milestone_list_fixture()),
            (true, ""),
            (
                true,
                r#"{"number":"42","title":"T","body":null,"state":"open","labels":[],"user":{"login":"octocat","id":"1"},"assignees":[],"created_at":"2026-01-01T00:00:00Z","updated_at":"2026-01-01T00:00:00Z","html_url":"https://gitcode.com/owner/repo/issues/42","milestone":{"number":3,"title":"v2.0"}}"#,
            ),
        ]);
        let provider = GitCodeIssueProvider::with_runner("owner/repo", runner.clone());

        let issue = provider
            .edit(
                42,
                gitflow_core::issue::EditIssueArgs {
                    title: None,
                    body: None,
                    milestone: Some(Some("3".to_string())),
                },
            )
            .await
            .expect("edit should succeed");

        assert_eq!(issue.milestone.map(|m| m.number), Some(3));

        let calls = runner.recorded_calls();
        let edit_call = &calls[1].1;
        assert!(
            edit_call
                .windows(2)
                .any(|w| w[0] == "--milestone" && w[1] == "3"),
            "issue edit argv must carry the resolved milestone NUMBER, got: {edit_call:?}"
        );
    }

    #[tokio::test]
    async fn test_should_error_on_unassign_milestone_without_spawning_cli() {
        // Confirmed 2026-09-21 against byx-darwin/NexaTrade issue #1: neither
        // `--milestone 0` (rejected by the CLI's own flag parsing as "no edit
        // option provided") nor `--milestone -1` (silently ignored by the
        // backend — `issue view` right after still shows the original
        // milestone attached) actually unassigns. GitCode's CLI has no
        // unassign capability, so `Some(None)` must error before spawning
        // anything, not fake success.
        let runner = RecordingMockRunner::success("should not be called");
        let provider = GitCodeIssueProvider::with_runner("owner/repo", runner.clone());

        let result = provider
            .edit(
                42,
                gitflow_core::issue::EditIssueArgs {
                    title: None,
                    body: None,
                    milestone: Some(None),
                },
            )
            .await;

        let err = result.expect_err("unassign must error, not silently succeed");
        assert!(
            matches!(err, gitflow_core::CoreError::Platform(_)),
            "expected CoreError::Platform, got {err:?}"
        );
        assert!(
            runner.calls().is_empty(),
            "must reject before spawning the CLI, got calls: {:?}",
            runner.calls()
        );
    }

    #[tokio::test]
    async fn test_should_wire_resolved_milestone_number_into_issue_list_filter() {
        // Sequence: 1. milestone resolution
        //           2. `gitcode issue list ... --milestone 3`
        let runner = SequencedMockCommandRunner::from_results(&[
            (true, &milestone_list_fixture()),
            (true, "[]"),
        ]);
        let provider = GitCodeIssueProvider::with_runner("owner/repo", runner.clone());

        let _ = provider
            .list(ListIssueArgs {
                milestone: Some("v2.0".to_string()),
                ..ListIssueArgs::default()
            })
            .await;

        let calls = runner.recorded_calls();
        let list_call = &calls[1].1;
        assert!(
            list_call
                .windows(2)
                .any(|w| w[0] == "--milestone" && w[1] == "3"),
            "issue list argv must carry the resolved milestone NUMBER (not title), got: \
             {list_call:?}"
        );
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
            vec![
                "issue",
                "label",
                "54",
                "--add",
                "type:bug,priority:high",
                "-R",
                "o/r"
            ],
            r"gitcode v0.6.1 的 issue edit 没有 --add-label flag（Issue #90）"
        );
    }

    #[tokio::test]
    async fn test_should_return_ok_without_any_call_for_empty_labels() {
        let runner = RecordingMockRunner::success("");
        let provider = GitCodeIssueProvider::with_runner("o/r", runner.clone());

        provider
            .add_labels(1, &[])
            .await
            .expect("empty labels is a no-op");

        assert!(runner.calls().is_empty());
    }

    #[tokio::test]
    async fn test_should_invoke_issue_label_subcommand_for_remove_label() {
        let runner = RecordingMockRunner::success("");
        let provider = GitCodeIssueProvider::with_runner("o/r", runner.clone());

        provider
            .remove_label(54, "triage:done")
            .await
            .expect("remove should succeed");

        assert_eq!(
            runner.calls()[0],
            vec![
                "issue",
                "label",
                "54",
                "--remove",
                "triage:done",
                "-R",
                "o/r"
            ]
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

    #[test]
    fn test_should_parse_issue_comment_with_user_object() {
        let json = r#"{"id": 12, "body": "hi", "user": {"login": "bob", "id": "u2"}, "created_at": "2026-07-30T12:00:00+08:00"}"#;
        let api: CommentApiResponse = serde_json::from_str(json).expect("user-object shape");
        let comment: CommentData = api.into();
        assert_eq!(comment.id, 12);
        assert_eq!(comment.author.login, "bob");
        assert_eq!(comment.author.id, "u2");
    }

    /// 构造 `count` 条合法 issue JSON，编号从 `start` 递增。
    fn issue_page_json(start: u64, count: u64) -> String {
        let items: Vec<String> = (start..start + count)
            .map(|n| {
                format!(
                    r#"{{"number":"{n}","title":"t{n}","state":"open","body":null,"labels":[],"created_at":"2026-07-30T12:00:00+08:00","updated_at":"2026-07-30T12:00:00+08:00","html_url":"https://gitcode.com/owner/repo/issues/{n}"}}"#
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

        let result = provider
            .list(ListIssueArgs {
                limit: Some(100),
                ..ListIssueArgs::default()
            })
            .await
            .expect("list should succeed");

        assert_eq!(result.items.len(), 100, "返回条数必须被 cap 钳住");
        assert!(result.truncated, "超过 cap 必须诚实报告截断");

        let calls = runner.recorded_calls();
        assert_eq!(
            calls.len(),
            2,
            "必须真的翻到第二页，实际调用数: {}",
            calls.len()
        );
        let first = &calls[0].1;
        assert!(
            first
                .windows(2)
                .any(|w| w[0] == "--per-page" && w[1] == "100"),
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
    async fn test_should_report_no_truncation_when_gitcode_issue_list_ends_on_short_page() {
        // cap=150 → per_page=min(151,100)=100，want=cap+1=151。
        // 首页满 100 条（非短页，且未达 want）⇒ 循环必须继续翻到第二页；
        // 第二页返回 20 条（短页）⇒ 循环必须就此停止，不再探第三页。
        // 总条目 120 < cap=150，因此必须诚实报告 truncated=false。
        let page1 = issue_page_json(1, 100);
        let page2 = issue_page_json(101, 20);
        let runner = SequencedMockCommandRunner::from_results(&[(true, &page1), (true, &page2)]);
        let provider = GitCodeIssueProvider::with_runner("owner/repo", runner.clone());

        let result = provider
            .list(ListIssueArgs {
                limit: Some(150),
                ..ListIssueArgs::default()
            })
            .await
            .expect("list should succeed");

        assert_eq!(result.items.len(), 120, "两页条目必须全部保留，不得丢数据");
        assert!(!result.truncated, "总数未达 cap，必须诚实报告未截断");

        let calls = runner.recorded_calls();
        assert_eq!(
            calls.len(),
            2,
            "短页应在第二页停止，不得再探第三页，实际调用数: {}",
            calls.len()
        );
        let first = &calls[0].1;
        assert!(
            first.windows(2).any(|w| w[0] == "--page" && w[1] == "1"),
            "首次调用必须显式传 --page=1，实际 argv: {first:?}"
        );
        let second = &calls[1].1;
        assert!(
            second.windows(2).any(|w| w[0] == "--page" && w[1] == "2"),
            "第二次调用必须显式传 --page=2，实际 argv: {second:?}"
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

    #[tokio::test]
    async fn test_should_keep_forwarding_label_filter_on_gitcode() {
        let runner = MockCommandRunner::success("[]");
        let provider = GitCodeIssueProvider::with_runner("owner/repo", runner.clone());
        let args = ListIssueArgs {
            labels: vec!["bug".to_string()],
            ..ListIssueArgs::default()
        };
        provider.list(args).await.expect("list should succeed");
        let recorded = &runner.recorded_calls()[0].1;
        assert!(
            recorded
                .windows(2)
                .any(|w| w[0] == "--label" && w[1] == "bug")
        );
    }

    #[tokio::test]
    async fn test_should_list_issues_with_state_and_label_using_full_argv() {
        let runner = MockCommandRunner::success("[]");
        let provider = GitCodeIssueProvider::with_runner("owner/repo", runner.clone());

        let _ = provider
            .list(ListIssueArgs {
                state: Some(State::Open),
                labels: vec!["bug".to_string()],
                ..ListIssueArgs::default()
            })
            .await;

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
    }
}

#[cfg(test)]
mod contract_tests {
    use super::*;
    use crate::runner::MockCommandRunner;

    /// 契约测试：验证 gitcode issue list JSON 输出与反序列化一致。
    ///
    /// 夹具来源：gitcode v0.6.x 真实 CLI 输出。
    #[tokio::test]
    async fn test_contract_issue_list_gitcode_v0_6() {
        let fixture = include_str!("../tests/fixtures/issue_list_gitcode_v0.6.json");
        let runner = MockCommandRunner::success(fixture);
        let provider = GitCodeIssueProvider::with_runner("owner/repo", runner);

        let issues = provider
            .list(ListIssueArgs::default())
            .await
            .expect("contract fixture must parse");
        let issues = issues.items;

        assert_eq!(issues.len(), 1);
        let issue = &issues[0];
        assert_eq!(issue.number, 15);
        assert!(!issue.title.is_empty());
        assert_eq!(issue.state, gitflow_core::types::State::Open);
    }
}
