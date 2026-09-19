//! GitLab Label 和 Milestone 提供者实现。
//!
//! 通过 `glab label` 和 `glab milestone` CLI 命令实现 [`LabelProvider`] 和
//! [`MilestoneProvider`] trait，支持标签和里程碑的完整生命周期管理。

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use gitflow_core::{
    CoreError, DEFAULT_LIST_LIMIT, FetchStrategy, Paged, Result, fetch_capped,
    label::{
        CreateLabelArgs, CreateMilestoneArgs, LabelData, LabelProvider, MilestoneData,
        MilestoneProvider,
    },
    types::State,
};
use serde::Deserialize;
use tracing::debug;

use crate::{
    GITLAB_MAX_PER_PAGE,
    error::parse_glab_error,
    runner::{CommandRunner, RealCommandRunner},
};

/// 在写操作后的读回分页结果中按 `predicate` 定位刚写入的条目。
///
/// 写操作（create/edit/close/reopen）已经成功执行，紧接着的读回只是为了
/// 把写回的完整数据返回给调用方。若读回被截断，说明该条目**很可能已经
/// 写入成功**，只是不在读回抓到的前 N 条内——此时必须报告"截断"而不是
/// "未找到"，否则会让用户以为已经成功的写操作失败了。
///
/// # Errors
///
/// 读回被截断时返回 `truncated_msg`；未截断但 `predicate` 未命中任何条目
/// 时返回 `not_found_msg`（理论上不应发生，除非平台侧的读回与写回不一致）。
fn resolve_after_mutation<T>(
    paged: Paged<T>,
    predicate: impl Fn(&T) -> bool,
    truncated_msg: &str,
    not_found_msg: String,
) -> Result<T> {
    if paged.truncated {
        return Err(CoreError::Platform(truncated_msg.to_string()));
    }
    paged
        .items
        .into_iter()
        .find(predicate)
        .ok_or(CoreError::Platform(not_found_msg))
}

/// GitLab Label 提供者，通过 `glab` CLI 管理仓库标签。
///
/// # Examples
///
/// ```no_run
/// use gitflow_gitlab::GitLabLabelProvider;
///
/// let provider = GitLabLabelProvider::new("gitlab-org/gitlab");
/// ```
#[derive(Debug, Clone)]
pub struct GitLabLabelProvider<R: CommandRunner = RealCommandRunner> {
    /// GitLab `namespace/project`。
    repo: String,
    /// 传给 `glab label ...` 子命令 `--repo` 参数的目标字符串。默认等于
    /// `repo`；通过 [`with_remote_url`](GitLabLabelProvider::with_remote_url)
    /// 构造时为完整 git remote URL，用于在自建 GitLab 实例上显式锁定 host。
    repo_target: String,
    /// 用于执行 `glab` CLI 命令的 runner。
    runner: R,
}

impl GitLabLabelProvider<RealCommandRunner> {
    /// 创建新的 GitLab Label 提供者。
    ///
    /// `repo` 格式为 `namespace/project`。
    #[must_use]
    pub fn new(repo: impl Into<String>) -> GitLabLabelProvider<RealCommandRunner> {
        let repo = repo.into();
        GitLabLabelProvider {
            repo_target: repo.clone(),
            repo,
            runner: RealCommandRunner,
        }
    }

    /// Create a new provider from a shared [`Session`].
    ///
    /// This enables state reuse across multiple operations in workflow chains.
    #[must_use]
    pub fn with_session(session: &gitflow_core::Session) -> Self {
        let repo = session.repo.clone();
        Self {
            repo_target: repo.clone(),
            repo,
            runner: RealCommandRunner,
        }
    }

    /// 使用完整 git remote URL 作为 `glab label ...` 的 `--repo` 目标创建提供者。
    #[must_use]
    pub fn with_remote_url(repo: impl Into<String>, remote_url: impl Into<String>) -> Self {
        Self {
            repo: repo.into(),
            repo_target: remote_url.into(),
            runner: RealCommandRunner,
        }
    }
}

impl<R: CommandRunner> GitLabLabelProvider<R> {
    /// 使用自定义 [`CommandRunner`] 创建提供者。
    ///
    /// 主要用于测试，可注入模拟 runner 以控制 `glab` CLI 的输出。
    /// `repo` 格式为 `namespace/project`。
    #[must_use]
    pub fn with_runner(repo: impl Into<String>, runner: R) -> Self {
        let repo = repo.into();
        Self {
            repo_target: repo.clone(),
            repo,
            runner,
        }
    }

    /// 使用自定义 [`CommandRunner`] 并显式指定 `--repo` 目标创建提供者。
    ///
    /// 主要用于测试，验证 `repo_target`（如完整 remote URL）被正确传给 `glab`。
    #[must_use]
    pub fn with_runner_and_repo_target(
        repo: impl Into<String>,
        repo_target: impl Into<String>,
        runner: R,
    ) -> Self {
        Self {
            repo: repo.into(),
            repo_target: repo_target.into(),
            runner,
        }
    }
}

/// 拉取一页 `glab label list --output json` 的原始 API 响应。
///
/// 提取为自由函数（而非 `&self` 方法），以便在 [`fetch_capped`] 的闭包中
/// 通过借用的 `runner`/`repo_target` 调用，避免闭包捕获 `self` 触发
/// `async_trait` 装箱后的 `Send` 推导问题。
///
/// # Errors
///
/// 当 `glab` CLI 调用失败或响应解析失败时返回错误。
async fn fetch_label_page<R: CommandRunner>(
    runner: &R,
    repo_target: &str,
    page: u32,
    per_page: u32,
) -> Result<Vec<LabelApiResponse>> {
    let page_str = page.to_string();
    let per_page_str = per_page.to_string();

    let output = runner
        .run(
            "glab",
            &[
                "label",
                "list",
                "--repo",
                repo_target,
                "--output",
                "json",
                "--per-page",
                &per_page_str,
                "--page",
                &page_str,
            ],
        )
        .await
        .map_err(|e| CoreError::Platform(format!("Failed to spawn glab label list: {e}")))?;

    if !output.status.success() {
        return Err(parse_glab_error(&output.stderr).into());
    }

    serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)
}

/// `glab label --output json` 返回的 JSON 结构。
#[derive(Debug, Clone, Deserialize)]
struct LabelApiResponse {
    #[serde(default)]
    id: u64,
    #[serde(default)]
    name: String,
    #[serde(default)]
    color: Option<String>,
    #[serde(default)]
    description: Option<String>,
}

impl From<LabelApiResponse> for LabelData {
    fn from(api: LabelApiResponse) -> Self {
        Self {
            name: api.name,
            color: api.color,
            description: api.description,
        }
    }
}

#[async_trait]
impl<R: CommandRunner + 'static> LabelProvider for GitLabLabelProvider<R> {
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

    async fn list(&self, limit: Option<u32>) -> Result<Paged<LabelData>> {
        let cap = limit.unwrap_or(DEFAULT_LIST_LIMIT);
        let repo_target = &self.repo_target;
        let runner = &self.runner;
        let per_page = cap.saturating_add(1).min(GITLAB_MAX_PER_PAGE);

        debug!(repo = %self.repo, cap, "spawning `glab label list`");

        fetch_capped(
            FetchStrategy::Paged { per_page },
            cap,
            |page, per_page| async move {
                let api_responses = fetch_label_page(runner, repo_target, page, per_page).await?;
                Ok(api_responses.into_iter().map(LabelData::from).collect())
            },
        )
        .await
    }

    async fn edit(&self, name: &str, args: CreateLabelArgs) -> Result<LabelData> {
        // 名称 → id 解析必须看到完整标签集合，否则截断会导致把"存在但不在
        // 前 N 条内"的标签误判为不存在。因此这里固定请求
        // `DEFAULT_LIST_LIMIT` 的完整上限，而不是转发调用方的分页 `limit`，
        // 并在被截断时报错而非静默使用不完整集合。
        let cap = DEFAULT_LIST_LIMIT;
        let repo_target = &self.repo_target;
        let runner = &self.runner;
        let per_page = cap.saturating_add(1).min(GITLAB_MAX_PER_PAGE);

        let paged =
            fetch_capped(
                FetchStrategy::Paged { per_page },
                cap,
                |page, per_page| async move {
                    fetch_label_page(runner, repo_target, page, per_page).await
                },
            )
            .await?;

        if paged.truncated {
            return Err(CoreError::Platform(
                "label lookup truncated; too many labels to resolve by name".into(),
            ));
        }

        let label_id = paged
            .items
            .iter()
            .find(|l| l.name == name)
            .map(|l| l.id)
            .ok_or_else(|| CoreError::Platform(format!("Label '{name}' not found")))?;

        debug!(
            repo = %self.repo,
            name,
            label_id,
            new_name = %args.name,
            "spawning `glab label edit --label-id`"
        );

        let id_str = label_id.to_string();
        let mut cmd_args: Vec<&str> = vec![
            "label",
            "edit",
            "--label-id",
            &id_str,
            "--repo",
            &self.repo_target,
            "--new-name",
            &args.name,
            "--color",
            &args.color,
        ];
        if let Some(ref desc) = args.description {
            cmd_args.push("--description");
            cmd_args.push(desc);
        }
        let output =
            self.runner.run("glab", &cmd_args).await.map_err(|e| {
                CoreError::Platform(format!("Failed to spawn glab label edit: {e}"))
            })?;
        if !output.status.success() {
            return Err(parse_glab_error(&output.stderr).into());
        }

        let paged = self.list(None).await?;
        resolve_after_mutation(
            paged,
            |l| l.name == args.name,
            "label lookup truncated after edit; too many labels to confirm the write landed",
            format!("Label '{}' not found after edit", args.name),
        )
    }

    async fn delete(&self, name: &str) -> Result<()> {
        debug!(repo = %self.repo, name, "spawning `glab label delete`");

        let output = self
            .runner
            .run(
                "glab",
                &["label", "delete", name, "--repo", &self.repo_target],
            )
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab label delete: {e}")))?;

        if !output.status.success() {
            return Err(parse_glab_error(&output.stderr).into());
        }

        Ok(())
    }
}

// ── Milestone Provider ──────────────────────────────────────────────

/// GitLab 里程碑提供者，通过 `glab milestone` 管理仓库里程碑。
///
/// # Examples
///
/// ```no_run
/// use gitflow_gitlab::GitLabMilestoneProvider;
///
/// let provider = GitLabMilestoneProvider::new("gitlab-org/gitlab");
/// ```
#[derive(Debug, Clone)]
pub struct GitLabMilestoneProvider<R: CommandRunner = RealCommandRunner> {
    /// GitLab `namespace/project`。
    repo: String,
    /// 传给 `glab milestone ...` 子命令 `--project` 参数的目标字符串
    /// （`glab milestone` 用 `--project` 而非 `--repo`，语义与其余 provider
    /// 的 `repo_target` 一致，仅 flag 名不同）。
    project_target: String,
    /// 用于执行 `glab` CLI 命令的 runner。
    runner: R,
}

impl GitLabMilestoneProvider<RealCommandRunner> {
    /// 创建新的 GitLab Milestone 提供者。
    ///
    /// `repo` 格式为 `namespace/project`。
    #[must_use]
    pub fn new(repo: impl Into<String>) -> GitLabMilestoneProvider<RealCommandRunner> {
        let repo = repo.into();
        GitLabMilestoneProvider {
            project_target: repo.clone(),
            repo,
            runner: RealCommandRunner,
        }
    }

    /// Create a new provider from a shared [`Session`].
    ///
    /// This enables state reuse across multiple operations in workflow chains.
    #[must_use]
    pub fn with_session(session: &gitflow_core::Session) -> Self {
        let repo = session.repo.clone();
        Self {
            project_target: repo.clone(),
            repo,
            runner: RealCommandRunner,
        }
    }

    /// 使用完整 git remote URL 作为 `glab milestone ...` 的 `--project` 目标创建提供者。
    #[must_use]
    pub fn with_remote_url(repo: impl Into<String>, remote_url: impl Into<String>) -> Self {
        Self {
            repo: repo.into(),
            project_target: remote_url.into(),
            runner: RealCommandRunner,
        }
    }
}

impl<R: CommandRunner> GitLabMilestoneProvider<R> {
    /// 使用自定义 [`CommandRunner`] 创建提供者。
    ///
    /// 主要用于测试，可注入模拟 runner 以控制 `glab` CLI 的输出。
    /// `repo` 格式为 `namespace/project`。
    #[must_use]
    pub fn with_runner(repo: impl Into<String>, runner: R) -> Self {
        let repo = repo.into();
        Self {
            project_target: repo.clone(),
            repo,
            runner,
        }
    }

    /// 使用自定义 [`CommandRunner`] 并显式指定 `--project` 目标创建提供者。
    ///
    /// 主要用于测试，验证 `project_target`（如完整 remote URL）被正确传给 `glab`。
    #[must_use]
    pub fn with_runner_and_project_target(
        repo: impl Into<String>,
        project_target: impl Into<String>,
        runner: R,
    ) -> Self {
        Self {
            repo: repo.into(),
            project_target: project_target.into(),
            runner,
        }
    }
}

/// `glab milestone --output json` 返回的 JSON 结构。
#[derive(Debug, Clone, Deserialize)]
#[allow(
    dead_code,
    reason = "Used for deserialization; not all fields are read"
)]
struct MilestoneApiResponse {
    id: u64,
    #[serde(default)]
    iid: Option<u64>,
    title: String,
    #[serde(default)]
    description: Option<String>,
    state: String,
    #[serde(default)]
    due_date: Option<String>,
    #[serde(default)]
    start_date: Option<String>,
    #[serde(default)]
    created_at: Option<DateTime<Utc>>,
    #[serde(default)]
    updated_at: Option<DateTime<Utc>>,
}

impl From<MilestoneApiResponse> for MilestoneData {
    fn from(api: MilestoneApiResponse) -> Self {
        let state = if api.state == "closed" {
            State::Closed
        } else {
            State::Open
        };
        let due_on = api.due_date.and_then(|s| {
            // GitLab returns due_date as "YYYY-MM-DD" or ISO 8601
            if let Ok(dt) = DateTime::parse_from_rfc3339(&s) {
                return Some(dt.with_timezone(&Utc));
            }
            chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d")
                .ok()
                .map(|d| {
                    let naive_dt = d
                        .and_hms_opt(0, 0, 0)
                        .unwrap_or_else(|| d.and_hms_opt(12, 0, 0).unwrap_or_default());
                    DateTime::<Utc>::from_naive_utc_and_offset(naive_dt, Utc)
                })
        });

        Self {
            number: api.iid.unwrap_or(api.id),
            title: api.title,
            description: api.description,
            state,
            due_on,
            closed_issues: 0,
            open_issues: 0,
        }
    }
}

/// 拉取一页 `glab milestone list --output json` 的原始 API 响应。
///
/// 与 [`fetch_label_page`] 同理提取为自由函数，避免闭包捕获 `self`。
///
/// # Errors
///
/// 当 `glab` CLI 调用失败或响应解析失败时返回错误。
async fn fetch_milestone_page<R: CommandRunner>(
    runner: &R,
    project_target: &str,
    page: u32,
    per_page: u32,
) -> Result<Vec<MilestoneApiResponse>> {
    let page_str = page.to_string();
    let per_page_str = per_page.to_string();

    let output = runner
        .run(
            "glab",
            &[
                "milestone",
                "list",
                "--project",
                project_target,
                "--output",
                "json",
                "--per-page",
                &per_page_str,
                "--page",
                &page_str,
            ],
        )
        .await
        .map_err(|e| CoreError::Platform(format!("Failed to spawn glab milestone list: {e}")))?;

    if !output.status.success() {
        return Err(parse_glab_error(&output.stderr).into());
    }

    serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)
}

#[async_trait]
impl<R: CommandRunner + 'static> MilestoneProvider for GitLabMilestoneProvider<R> {
    async fn create(&self, args: CreateMilestoneArgs) -> Result<MilestoneData> {
        debug!(repo = %self.repo, title = %args.title, "spawning `glab milestone create`");

        let due_arg = args
            .due_on
            .as_ref()
            .map(|due| due.format("%Y-%m-%d").to_string());

        let mut cmd_args: Vec<&str> = vec![
            "milestone",
            "create",
            "--title",
            &args.title,
            "--project",
            &self.project_target,
        ];

        if let Some(ref desc) = args.description {
            cmd_args.push("--description");
            cmd_args.push(desc);
        }

        if let Some(ref due_str) = due_arg {
            cmd_args.push("--due-date");
            cmd_args.push(due_str);
        }

        let output = self.runner.run("glab", &cmd_args).await.map_err(|e| {
            CoreError::Platform(format!("Failed to spawn glab milestone create: {e}"))
        })?;

        if !output.status.success() {
            return Err(parse_glab_error(&output.stderr).into());
        }

        let paged = self.list(None).await?;
        resolve_after_mutation(
            paged,
            |m| m.title == args.title,
            "milestone lookup truncated after create; too many milestones to confirm the write \
             landed",
            "Milestone not found after create".into(),
        )
    }

    async fn list(&self, limit: Option<u32>) -> Result<Paged<MilestoneData>> {
        let cap = limit.unwrap_or(DEFAULT_LIST_LIMIT);
        let project_target = &self.project_target;
        let runner = &self.runner;
        let per_page = cap.saturating_add(1).min(GITLAB_MAX_PER_PAGE);

        debug!(repo = %self.repo, cap, "spawning `glab milestone list`");

        fetch_capped(
            FetchStrategy::Paged { per_page },
            cap,
            |page, per_page| async move {
                let api_responses =
                    fetch_milestone_page(runner, project_target, page, per_page).await?;
                Ok(api_responses.into_iter().map(MilestoneData::from).collect())
            },
        )
        .await
    }

    async fn edit(&self, number: u64, args: CreateMilestoneArgs) -> Result<MilestoneData> {
        debug!(repo = %self.repo, number, "spawning `glab milestone edit`");

        let due_arg = args
            .due_on
            .as_ref()
            .map(|due| due.format("%Y-%m-%d").to_string());
        let number_str = number.to_string();

        let mut cmd_args: Vec<&str> = vec![
            "milestone",
            "edit",
            &number_str,
            "--project",
            &self.project_target,
            "--title",
            &args.title,
        ];

        if let Some(ref desc) = args.description {
            cmd_args.push("--description");
            cmd_args.push(desc);
        }

        if let Some(ref due_str) = due_arg {
            cmd_args.push("--due-date");
            cmd_args.push(due_str);
        }

        let output = self.runner.run("glab", &cmd_args).await.map_err(|e| {
            CoreError::Platform(format!("Failed to spawn glab milestone edit: {e}"))
        })?;

        if !output.status.success() {
            return Err(parse_glab_error(&output.stderr).into());
        }

        let paged = self.list(None).await?;
        resolve_after_mutation(
            paged,
            |m| m.title == args.title || m.number == number,
            "milestone lookup truncated after edit; too many milestones to confirm the write \
             landed",
            "Milestone not found after edit".into(),
        )
    }

    async fn close(&self, number: u64) -> Result<MilestoneData> {
        debug!(
            repo = %self.repo,
            number,
            "spawning `glab milestone edit --state close`"
        );

        let number_str = number.to_string();
        let output = self
            .runner
            .run(
                "glab",
                &[
                    "milestone",
                    "edit",
                    &number_str,
                    "--state",
                    "close",
                    "--project",
                    &self.project_target,
                ],
            )
            .await
            .map_err(|e| {
                CoreError::Platform(format!("Failed to spawn glab milestone edit: {e}"))
            })?;

        if !output.status.success() {
            return Err(parse_glab_error(&output.stderr).into());
        }

        let paged = self.list(None).await?;
        resolve_after_mutation(
            paged,
            |m| m.number == number,
            "milestone lookup truncated after close; too many milestones to confirm the write \
             landed",
            "Milestone not found after close".into(),
        )
    }

    async fn reopen(&self, number: u64) -> Result<MilestoneData> {
        debug!(
            repo = %self.repo,
            number,
            "spawning `glab milestone edit --state activate`"
        );

        let number_str = number.to_string();
        let output = self
            .runner
            .run(
                "glab",
                &[
                    "milestone",
                    "edit",
                    &number_str,
                    "--state",
                    "activate",
                    "--project",
                    &self.project_target,
                ],
            )
            .await
            .map_err(|e| {
                CoreError::Platform(format!("Failed to spawn glab milestone edit: {e}"))
            })?;

        if !output.status.success() {
            return Err(parse_glab_error(&output.stderr).into());
        }

        let paged = self.list(None).await?;
        resolve_after_mutation(
            paged,
            |m| m.number == number,
            "milestone lookup truncated after reopen; too many milestones to confirm the write \
             landed",
            "Milestone not found after reopen".into(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runner::{MockCommandRunner, SequencedMockCommandRunner};

    // --- GitLabLabelProvider tests ---

    #[test]
    fn test_should_construct_gitlab_label_provider() {
        let provider = GitLabLabelProvider::new("gitlab-org/gitlab");
        assert_eq!(provider.repo, "gitlab-org/gitlab");
    }

    #[test]
    fn test_should_construct_gitlab_label_provider_from_string() {
        let repo = String::from("gitlab-org/gitlab");
        let provider = GitLabLabelProvider::new(repo);
        assert_eq!(provider.repo, "gitlab-org/gitlab");
    }

    #[tokio::test]
    async fn test_should_use_explicit_repo_target_for_delete() {
        let runner = MockCommandRunner::success("");
        let provider = GitLabLabelProvider::with_runner_and_repo_target(
            "owner/repo",
            "https://192.168.230.23/iproost/proxy/api-src.git",
            runner.clone(),
        );

        let result = provider.delete("bug").await;

        assert!(result.is_ok(), "expected Ok, got {result:?}");
        assert_eq!(
            runner.recorded_calls()[0].1,
            vec![
                "label",
                "delete",
                "bug",
                "--repo",
                "https://192.168.230.23/iproost/proxy/api-src.git",
            ]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_should_debug_format_label_provider() {
        let provider = GitLabLabelProvider::new("owner/repo");
        let debug = format!("{provider:?}");
        assert!(debug.contains("GitLabLabelProvider"));
        assert!(debug.contains("owner/repo"));
    }

    #[test]
    fn test_should_clone_gitlab_label_provider() {
        let original = GitLabLabelProvider::new("owner/repo");
        let cloned = original.clone();
        assert_eq!(original.repo, cloned.repo);
    }

    // --- LabelData deserialization tests ---

    #[test]
    fn test_should_deserialize_label_api_response() {
        let json = br##"{
            "name": "bug",
            "color": "#d73a4a",
            "description": "Something isn't working"
        }"##;

        let api: LabelApiResponse = serde_json::from_slice(json).expect("valid LabelApiResponse");
        let label: LabelData = api.into();
        assert_eq!(label.name, "bug");
        assert_eq!(label.color.as_deref(), Some("#d73a4a"));
        assert_eq!(
            label.description.as_deref(),
            Some("Something isn't working")
        );
    }

    #[test]
    fn test_should_deserialize_label_list() {
        let json = br##"[
            {"name": "bug", "color": "#d73a4a", "description": "Bug"},
            {"name": "feature", "color": "#0075ca", "description": null}
        ]"##;

        let list: Vec<LabelApiResponse> = serde_json::from_slice(json).expect("valid label list");
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].name, "bug");
        assert_eq!(list[1].name, "feature");
    }

    #[test]
    fn test_should_deserialize_empty_label_list() {
        let json = b"[]";
        let list: Vec<LabelApiResponse> = serde_json::from_slice(json).expect("valid empty list");
        assert!(list.is_empty());
    }

    // --- GitLabMilestoneProvider tests ---

    #[test]
    fn test_should_construct_gitlab_milestone_provider() {
        let provider = GitLabMilestoneProvider::new("gitlab-org/gitlab");
        assert_eq!(provider.repo, "gitlab-org/gitlab");
    }

    #[tokio::test]
    async fn test_should_use_explicit_project_target_for_close() {
        let runner = SequencedMockCommandRunner::from_results(&[
            (true, ""),
            (
                true,
                r#"[{"id":1,"iid":3,"title":"v1.0","description":null,"state":"closed","due_date":null,"created_at":"2026-01-01T00:00:00Z","updated_at":"2026-01-01T00:00:00Z"}]"#,
            ),
        ]);
        let provider = GitLabMilestoneProvider::with_runner_and_project_target(
            "owner/repo",
            "https://192.168.230.23/iproost/proxy/api-src.git",
            runner.clone(),
        );

        let ms = provider.close(3).await.expect("close should succeed");

        assert_eq!(ms.number, 3);
        assert_eq!(
            runner.recorded_calls()[0].1,
            vec![
                "milestone",
                "edit",
                "3",
                "--state",
                "close",
                "--project",
                "https://192.168.230.23/iproost/proxy/api-src.git",
            ]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_should_debug_format_milestone_provider() {
        let provider = GitLabMilestoneProvider::new("owner/repo");
        let debug = format!("{provider:?}");
        assert!(debug.contains("GitLabMilestoneProvider"));
        assert!(debug.contains("owner/repo"));
    }

    #[test]
    fn test_should_clone_gitlab_milestone_provider() {
        let original = GitLabMilestoneProvider::new("owner/repo");
        let cloned = original.clone();
        assert_eq!(original.repo, cloned.repo);
    }

    // --- Failure-path tests using an injected MockCommandRunner ---

    #[tokio::test]
    async fn test_should_fail_when_label_create_glab_fails() {
        let runner = MockCommandRunner::failure(r#"{"message": "Forbidden"}"#, 256);
        let provider = GitLabLabelProvider::with_runner("owner/repo", runner);
        let args = CreateLabelArgs {
            name: "bug".to_string(),
            color: "#d73a4a".to_string(),
            description: None,
        };
        let result = provider.create(args).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_should_fail_when_milestone_create_glab_fails() {
        let runner = MockCommandRunner::failure(r#"{"message": "Forbidden"}"#, 256);
        let provider = GitLabMilestoneProvider::with_runner("owner/repo", runner);
        let args = CreateMilestoneArgs {
            title: "v1.0".to_string(),
            description: None,
            due_on: None,
        };
        let result = provider.create(args).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_should_create_label_without_output_json_and_refetch_via_list() {
        let runner = SequencedMockCommandRunner::from_results(&[
            (true, ""), // label create 成功（stdout 为纯文本，忽略）
            (true, r##"[{"id":101,"name":"bug","color":"#d73a4a"}]"##), // list 找回
        ]);
        let provider = GitLabLabelProvider::with_runner("owner/repo", runner);

        let args = CreateLabelArgs {
            name: "bug".to_string(),
            color: "#d73a4a".to_string(),
            description: None,
        };

        let label = provider.create(args).await.expect("should create");

        assert_eq!(label.name, "bug");
    }

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

    #[tokio::test]
    async fn test_should_edit_label_with_label_id() {
        let list_json = r##"[{"id":101,"name":"bug","color":"#d73a4a"}]"##;
        let edited_json = r##"[{"id":101,"name":"critical","color":"#d73a4a"}]"##;
        let runner = SequencedMockCommandRunner::from_results(&[
            (true, list_json),   // list_api 解析 id
            (true, edited_json), // label edit 成功（stdout 为纯文本，忽略）
            (true, edited_json), // 再次 list 找回
        ]);
        let provider = GitLabLabelProvider::with_runner("owner/repo", runner);

        let args = CreateLabelArgs {
            name: "critical".to_string(),
            color: "#d73a4a".to_string(),
            description: None,
        };

        let label = provider.edit("bug", args).await.expect("should edit");

        assert_eq!(label.name, "critical");
    }

    #[tokio::test]
    async fn test_should_delete_label_without_yes_flag() {
        let runner = MockCommandRunner::success("");
        let provider = GitLabLabelProvider::with_runner("owner/repo", runner.clone());

        provider.delete("bug").await.expect("should delete");

        assert_eq!(
            runner.recorded_calls()[0].1,
            vec!["label", "delete", "bug", "--repo", "owner/repo"]
                .into_iter()
                .map(String::from)
                .collect::<Vec<_>>()
        );
    }

    #[tokio::test]
    async fn test_should_create_milestone_without_output_json_and_refetch() {
        let runner = SequencedMockCommandRunner::from_results(&[
            (true, ""), // milestone create 成功（stdout 为纯文本，忽略）
            (
                true,
                r#"[{"id":1,"iid":3,"title":"v1.0","description":null,"state":"active","due_date":null,"created_at":"2026-01-01T00:00:00Z","updated_at":"2026-01-01T00:00:00Z"}]"#,
            ),
        ]);
        let provider = GitLabMilestoneProvider::with_runner("owner/repo", runner);

        let args = CreateMilestoneArgs {
            title: "v1.0".to_string(),
            description: None,
            due_on: None,
        };

        let ms = provider.create(args).await.expect("should create");

        assert_eq!(ms.number, 3);
        assert_eq!(ms.title, "v1.0");
    }

    #[tokio::test]
    async fn test_should_edit_milestone_without_output_json_and_refetch() {
        let runner = SequencedMockCommandRunner::from_results(&[
            (true, ""), // milestone edit 成功
            (
                true,
                r#"[{"id":1,"iid":3,"title":"v1.1","description":null,"state":"active","due_date":null,"created_at":"2026-01-01T00:00:00Z","updated_at":"2026-01-01T00:00:00Z"}]"#,
            ),
        ]);
        let provider = GitLabMilestoneProvider::with_runner("owner/repo", runner);

        let args = CreateMilestoneArgs {
            title: "v1.1".to_string(),
            description: None,
            due_on: None,
        };

        let ms = provider.edit(3, args).await.expect("should edit");

        assert_eq!(ms.title, "v1.1");
    }

    #[tokio::test]
    async fn test_should_close_milestone_without_output_json_and_refetch() {
        let runner = SequencedMockCommandRunner::from_results(&[
            (true, ""),
            (
                true,
                r#"[{"id":1,"iid":3,"title":"v1.0","description":null,"state":"closed","due_date":null,"created_at":"2026-01-01T00:00:00Z","updated_at":"2026-01-01T00:00:00Z"}]"#,
            ),
        ]);
        let provider = GitLabMilestoneProvider::with_runner("owner/repo", runner.clone());

        let ms = provider.close(3).await.expect("should close");

        assert_eq!(ms.number, 3);
        assert_eq!(ms.state, State::Closed);
        // glab 1.113 has no `milestone close`; closing is `milestone edit --state close`.
        assert_eq!(
            runner.recorded_calls()[0].1,
            vec![
                "milestone",
                "edit",
                "3",
                "--state",
                "close",
                "--project",
                "owner/repo"
            ]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>()
        );
    }

    #[tokio::test]
    async fn test_should_reopen_milestone_without_output_json_and_refetch() {
        let runner = SequencedMockCommandRunner::from_results(&[
            (true, ""),
            (
                true,
                r#"[{"id":1,"iid":3,"title":"v1.0","description":null,"state":"active","due_date":null,"created_at":"2026-01-01T00:00:00Z","updated_at":"2026-01-01T00:00:00Z"}]"#,
            ),
        ]);
        let provider = GitLabMilestoneProvider::with_runner("owner/repo", runner.clone());

        let ms = provider.reopen(3).await.expect("should reopen");

        assert_eq!(ms.number, 3);
        assert_eq!(ms.state, State::Open);
        // glab 1.113 has no `milestone reopen`; reopening is `milestone edit --state activate`.
        assert_eq!(
            runner.recorded_calls()[0].1,
            vec![
                "milestone",
                "edit",
                "3",
                "--state",
                "activate",
                "--project",
                "owner/repo"
            ]
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>()
        );
    }

    // --- MilestoneData deserialization tests ---

    #[test]
    fn test_should_deserialize_milestone_api_response() {
        let json = br#"{
            "id": 1,
            "iid": 1,
            "title": "v1.0 Release",
            "description": "First stable release",
            "state": "active",
            "due_date": "2026-06-01",
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-01T00:00:00Z"
        }"#;

        let api: MilestoneApiResponse =
            serde_json::from_slice(json).expect("valid MilestoneApiResponse");
        let data: MilestoneData = api.into();

        assert_eq!(data.number, 1);
        assert_eq!(data.title, "v1.0 Release");
        assert_eq!(data.description, Some("First stable release".into()));
        assert_eq!(data.state, State::Open);
    }

    #[test]
    fn test_should_deserialize_closed_milestone() {
        let json = br#"{
            "id": 2,
            "iid": 2,
            "title": "v0.9 Beta",
            "description": null,
            "state": "closed",
            "due_date": null,
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-06-01T00:00:00Z"
        }"#;

        let api: MilestoneApiResponse =
            serde_json::from_slice(json).expect("valid MilestoneApiResponse");
        let data: MilestoneData = api.into();
        assert_eq!(data.state, State::Closed);
        assert!(data.description.is_none());
    }

    #[test]
    fn test_should_deserialize_milestone_list() {
        let json = br#"[
            {"id": 1, "iid": 1, "title": "v1.0", "description": null, "state": "active", "created_at": "2026-01-01T00:00:00Z", "updated_at": "2026-01-01T00:00:00Z"},
            {"id": 2, "iid": 2, "title": "v0.9", "description": "Beta", "state": "closed", "created_at": "2026-01-01T00:00:00Z", "updated_at": "2026-06-01T00:00:00Z"}
        ]"#;

        let milestones: Vec<MilestoneApiResponse> =
            serde_json::from_slice(json).expect("valid MilestoneApiResponse list");
        assert_eq!(milestones.len(), 2);
        assert_eq!(milestones[0].title, "v1.0");
        assert_eq!(milestones[1].title, "v0.9");
    }

    // --- Pagination tests ---

    #[tokio::test]
    async fn test_should_produce_complete_argv_for_gitlab_label_list_with_default_limit() {
        let runner = MockCommandRunner::success("[]");
        let provider = GitLabLabelProvider::with_runner("owner/repo", runner.clone());

        let paged = provider.list(None).await.expect("should list");

        assert!(paged.items.is_empty());
        assert_eq!(runner.recorded_calls()[0].0, "glab");
        assert_eq!(
            runner.recorded_calls()[0].1,
            vec![
                "label",
                "list",
                "--repo",
                "owner/repo",
                "--output",
                "json",
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

    #[tokio::test]
    async fn test_should_produce_complete_argv_for_gitlab_milestone_list_with_default_limit() {
        let runner = MockCommandRunner::success("[]");
        let provider = GitLabMilestoneProvider::with_runner("owner/repo", runner.clone());

        let paged = provider.list(None).await.expect("should list");

        assert!(paged.items.is_empty());
        assert_eq!(runner.recorded_calls()[0].0, "glab");
        assert_eq!(
            runner.recorded_calls()[0].1,
            vec![
                "milestone",
                "list",
                "--project",
                "owner/repo",
                "--output",
                "json",
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

    /// 验证名称→id 解析在被截断时报错而非静默使用不完整集合。
    ///
    /// 注：任务简报原文声称此截断防护应加在 `delete` 上（"delete 需要
    /// 全量查找以按名解析 id"），但实际代码中 `delete` 直接
    /// `glab label delete <name>`，从不做 id 解析、也从不调用
    /// `list_api`／分页查询；真正做名称→id 解析的是 `edit`
    /// （见本文件 `edit` 方法）。因此本测试针对 `edit`，而非简报所述的
    /// `delete`。
    #[tokio::test]
    async fn test_should_error_when_label_edit_lookup_is_truncated() {
        let labels: Vec<String> = (0..1001)
            .map(|i| format!(r##"{{"id":{i},"name":"label-{i}","color":"#ffffff"}}"##))
            .collect();
        let json = format!("[{}]", labels.join(","));
        let runner = MockCommandRunner::success(&json);
        let provider = GitLabLabelProvider::with_runner("owner/repo", runner);

        let args = CreateLabelArgs {
            name: "renamed".to_string(),
            color: "#ffffff".to_string(),
            description: None,
        };

        let err = provider
            .edit("label-999", args)
            .await
            .expect_err("edit should fail when lookup is truncated");
        assert!(err.to_string().contains("truncated"));
    }

    /// 验证写操作（`create`）成功后，若读回被截断，报告的是"截断"而非
    /// "未找到"——写操作本身已经成功，不能让用户误以为写入失败了。
    #[tokio::test]
    async fn test_should_report_truncation_not_missing_when_create_readback_is_truncated() {
        let labels: Vec<String> = (0..1001)
            .map(|i| format!(r##"{{"id":{i},"name":"label-{i}","color":"#ffffff"}}"##))
            .collect();
        let json = format!("[{}]", labels.join(","));
        // 同一份固定响应既充当 `label create` 的（被忽略的）输出，也充当
        // 紧接着 `list()` 读回的响应；mock 无视 page/per-page 参数，总是
        // 返回整个数组，因此读回一次即触发 N+1 探测下的截断。
        let runner = MockCommandRunner::success(&json);
        let provider = GitLabLabelProvider::with_runner("owner/repo", runner);

        let args = CreateLabelArgs {
            name: "brand-new-label".to_string(),
            color: "#00ff00".to_string(),
            description: None,
        };

        let err = provider.create(args).await.expect_err(
            "create should report truncated read-back rather than silently claiming success or \
             lying about not-found",
        );
        let msg = err.to_string();
        assert!(
            msg.contains("truncated"),
            "expected a truncation error, got: {msg}"
        );
        assert!(
            !msg.contains("not found after create"),
            "must not report the just-created label as missing when the read-back was merely \
             truncated: {msg}"
        );
    }
}
