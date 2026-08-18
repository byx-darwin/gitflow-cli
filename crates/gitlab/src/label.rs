//! GitLab Label 和 Milestone 提供者实现。
//!
//! 通过 `glab label` 和 `glab milestone` CLI 命令实现 [`LabelProvider`] 和
//! [`MilestoneProvider`] trait，支持标签和里程碑的完整生命周期管理。

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use gitflow_core::{
    CoreError, Result,
    label::{
        CreateLabelArgs, CreateMilestoneArgs, LabelData, LabelProvider, MilestoneData,
        MilestoneProvider,
    },
    types::State,
};
use serde::Deserialize;
use tracing::debug;

use crate::{
    error::parse_glab_error,
    runner::{CommandRunner, RealCommandRunner},
};

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
    /// 用于执行 `glab` CLI 命令的 runner。
    runner: R,
}

impl GitLabLabelProvider<RealCommandRunner> {
    /// 创建新的 GitLab Label 提供者。
    ///
    /// `repo` 格式为 `namespace/project`。
    #[must_use]
    pub fn new(repo: impl Into<String>) -> GitLabLabelProvider<RealCommandRunner> {
        GitLabLabelProvider {
            repo: repo.into(),
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
        Self {
            repo: repo.into(),
            runner,
        }
    }

    /// 通过 `glab label list --output json` 获取原始 label API 响应。
    ///
    /// # Errors
    ///
    /// 当 `glab` CLI 调用失败或响应解析失败时返回错误。
    async fn list_api(&self) -> Result<Vec<LabelApiResponse>> {
        let output = self
            .runner
            .run(
                "glab",
                &["label", "list", "--repo", &self.repo, "--output", "json"],
            )
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab label list: {e}")))?;
        if !output.status.success() {
            return Err(parse_glab_error(&output.stderr).into());
        }
        serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)
    }
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
        debug!(
            repo = %self.repo,
            name = %args.name,
            color = %args.color,
            "spawning `glab label create`"
        );

        let mut cmd_args: Vec<&str> = vec![
            "label",
            "create",
            "--name",
            &args.name,
            "--color",
            &args.color,
            "--repo",
            &self.repo,
        ];

        if let Some(ref desc) = args.description {
            cmd_args.push("--description");
            cmd_args.push(desc);
        }

        let output = self
            .runner
            .run("glab", &cmd_args)
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab label create: {e}")))?;

        if !output.status.success() {
            return Err(parse_glab_error(&output.stderr).into());
        }

        let labels = self.list().await?;
        labels
            .into_iter()
            .find(|l| l.name == args.name)
            .ok_or_else(|| {
                CoreError::Platform(format!("Label '{}' not found after create", args.name))
            })
    }

    async fn list(&self) -> Result<Vec<LabelData>> {
        let api_responses = self.list_api().await?;
        Ok(api_responses.into_iter().map(LabelData::from).collect())
    }

    async fn edit(&self, name: &str, args: CreateLabelArgs) -> Result<LabelData> {
        let api_labels = self.list_api().await?;
        let label_id = api_labels
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
            "label", "edit", "--label-id", &id_str, "--repo", &self.repo, "--new-name",
            &args.name, "--color", &args.color,
        ];
        if let Some(ref desc) = args.description {
            cmd_args.push("--description");
            cmd_args.push(desc);
        }
        let output = self.runner.run("glab", &cmd_args).await.map_err(|e| {
            CoreError::Platform(format!("Failed to spawn glab label edit: {e}"))
        })?;
        if !output.status.success() {
            return Err(parse_glab_error(&output.stderr).into());
        }

        let labels = self.list().await?;
        labels
            .into_iter()
            .find(|l| l.name == args.name)
            .ok_or_else(|| CoreError::Platform(format!("Label '{}' not found after edit", args.name)))
    }

    async fn delete(&self, name: &str) -> Result<()> {
        debug!(repo = %self.repo, name, "spawning `glab label delete`");

        let output = self
            .runner
            .run("glab", &["label", "delete", name, "--repo", &self.repo])
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
    /// 用于执行 `glab` CLI 命令的 runner。
    runner: R,
}

impl GitLabMilestoneProvider<RealCommandRunner> {
    /// 创建新的 GitLab Milestone 提供者。
    ///
    /// `repo` 格式为 `namespace/project`。
    #[must_use]
    pub fn new(repo: impl Into<String>) -> GitLabMilestoneProvider<RealCommandRunner> {
        GitLabMilestoneProvider {
            repo: repo.into(),
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
        Self {
            repo: repo.into(),
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
            &self.repo,
            "--output",
            "json",
        ];

        if let Some(ref desc) = args.description {
            cmd_args.push("--description");
            cmd_args.push(desc);
        }

        if let Some(ref due_str) = due_arg {
            cmd_args.push("--due-date");
            cmd_args.push(due_str);
        }

        let output = self
            .runner
            .run("glab", &cmd_args)
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab milestone create: {e}")))?;

        if !output.status.success() {
            return Err(parse_glab_error(&output.stderr).into());
        }

        let api_response: MilestoneApiResponse =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(api_response.into())
    }

    async fn list(&self) -> Result<Vec<MilestoneData>> {
        debug!(repo = %self.repo, "spawning `glab milestone list`");

        let output = self
            .runner
            .run(
                "glab",
                &["milestone", "list", "--project", &self.repo, "--output", "json"],
            )
            .await
            .map_err(|e| {
                CoreError::Platform(format!("Failed to spawn glab milestone list: {e}"))
            })?;

        if !output.status.success() {
            return Err(parse_glab_error(&output.stderr).into());
        }

        let api_responses: Vec<MilestoneApiResponse> =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(api_responses.into_iter().map(MilestoneData::from).collect())
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
            &self.repo,
            "--title",
            &args.title,
            "--output",
            "json",
        ];

        if let Some(ref desc) = args.description {
            cmd_args.push("--description");
            cmd_args.push(desc);
        }

        if let Some(ref due_str) = due_arg {
            cmd_args.push("--due-date");
            cmd_args.push(due_str);
        }

        let output = self
            .runner
            .run("glab", &cmd_args)
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab milestone edit: {e}")))?;

        if !output.status.success() {
            return Err(parse_glab_error(&output.stderr).into());
        }

        let api_response: MilestoneApiResponse =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(api_response.into())
    }

    async fn close(&self, number: u64) -> Result<MilestoneData> {
        debug!(repo = %self.repo, number, "spawning `glab milestone close`");

        let output = self
            .runner
            .run(
                "glab",
                &[
                    "milestone",
                    "close",
                    &number.to_string(),
                    "--project",
                    &self.repo,
                    "--output",
                    "json",
                ],
            )
            .await
            .map_err(|e| {
                CoreError::Platform(format!("Failed to spawn glab milestone close: {e}"))
            })?;

        if !output.status.success() {
            return Err(parse_glab_error(&output.stderr).into());
        }

        let api_response: MilestoneApiResponse =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(api_response.into())
    }

    async fn reopen(&self, number: u64) -> Result<MilestoneData> {
        debug!(repo = %self.repo, number, "spawning `glab milestone reopen`");

        let output = self
            .runner
            .run(
                "glab",
                &[
                    "milestone",
                    "reopen",
                    &number.to_string(),
                    "--project",
                    &self.repo,
                    "--output",
                    "json",
                ],
            )
            .await
            .map_err(|e| {
                CoreError::Platform(format!("Failed to spawn glab milestone reopen: {e}"))
            })?;

        if !output.status.success() {
            return Err(parse_glab_error(&output.stderr).into());
        }

        let api_response: MilestoneApiResponse =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(api_response.into())
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
            (
                true,
                r##"[{"id":101,"name":"bug","color":"#d73a4a"}]"##,
            ), // list 找回
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
}
