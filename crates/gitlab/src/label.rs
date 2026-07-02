//! GitLab Label 和 Milestone 提供者实现。
//!
//! 通过 `glab label` 和 `glab milestone` CLI 命令实现 [`LabelProvider`] 和
//! [`MilestoneProvider`] trait，支持标签和里程碑的完整生命周期管理。

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use gitflow_cli_core::{
    CoreError, Result,
    label::{
        CreateLabelArgs, CreateMilestoneArgs, LabelData, LabelProvider, MilestoneData,
        MilestoneProvider,
    },
    types::State,
};
use serde::Deserialize;
use tracing::debug;

use crate::error::parse_glab_error;

/// GitLab Label 提供者，通过 `glab` CLI 管理仓库标签。
///
/// # Examples
///
/// ```no_run
/// use gitflow_cli_gitlab::GitLabLabelProvider;
///
/// let provider = GitLabLabelProvider::new("gitlab-org/gitlab");
/// ```
#[derive(Debug, Clone)]
pub struct GitLabLabelProvider {
    /// GitLab `namespace/project`。
    repo: String,
}

impl GitLabLabelProvider {
    /// 创建新的 GitLab Label 提供者。
    ///
    /// `repo` 格式为 `namespace/project`。
    #[must_use]
    pub fn new(repo: impl Into<String>) -> Self {
        Self { repo: repo.into() }
    }
}

/// `glab label --output json` 返回的 JSON 结构。
#[derive(Debug, Clone, Deserialize)]
struct LabelApiResponse {
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
impl LabelProvider for GitLabLabelProvider {
    async fn create(&self, args: CreateLabelArgs) -> Result<LabelData> {
        debug!(
            repo = %self.repo,
            name = %args.name,
            color = %args.color,
            "spawning `glab label create`"
        );

        let mut cmd = tokio::process::Command::new("glab");
        cmd.args(["label", "create"])
            .arg("--name")
            .arg(&args.name)
            .arg("--color")
            .arg(&args.color)
            .arg("--repo")
            .arg(&self.repo)
            .arg("--output")
            .arg("json");

        if let Some(ref desc) = args.description {
            cmd.arg("--description").arg(desc);
        }

        let output = cmd
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab label create: {e}")))?;

        if !output.status.success() {
            let glab_err = parse_glab_error(&output.stderr);
            return Err(CoreError::Platform(format!("{glab_err}")));
        }

        let api_response: LabelApiResponse =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(api_response.into())
    }

    async fn list(&self) -> Result<Vec<LabelData>> {
        debug!(repo = %self.repo, "spawning `glab label list`");

        let output = tokio::process::Command::new("glab")
            .args(["label", "list"])
            .arg("--repo")
            .arg(&self.repo)
            .arg("--output")
            .arg("json")
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab label list: {e}")))?;

        if !output.status.success() {
            let glab_err = parse_glab_error(&output.stderr);
            return Err(CoreError::Platform(format!("{glab_err}")));
        }

        let api_responses: Vec<LabelApiResponse> =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(api_responses.into_iter().map(LabelData::from).collect())
    }

    async fn edit(&self, name: &str, args: CreateLabelArgs) -> Result<LabelData> {
        debug!(repo = %self.repo, name, "spawning `glab label edit`");

        let mut cmd = tokio::process::Command::new("glab");
        cmd.args(["label", "edit"])
            .arg("--name")
            .arg(name)
            .arg("--repo")
            .arg(&self.repo)
            .arg("--color")
            .arg(&args.color)
            .arg("--output")
            .arg("json");

        if let Some(ref desc) = args.description {
            cmd.arg("--description").arg(desc);
        }

        let output = cmd
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab label edit: {e}")))?;

        if !output.status.success() {
            let glab_err = parse_glab_error(&output.stderr);
            return Err(CoreError::Platform(format!("{glab_err}")));
        }

        let api_response: LabelApiResponse =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(api_response.into())
    }

    async fn delete(&self, name: &str) -> Result<()> {
        debug!(repo = %self.repo, name, "spawning `glab label delete`");

        let output = tokio::process::Command::new("glab")
            .args(["label", "delete"])
            .arg(name)
            .arg("--yes")
            .arg("--repo")
            .arg(&self.repo)
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab label delete: {e}")))?;

        if !output.status.success() {
            let glab_err = parse_glab_error(&output.stderr);
            return Err(CoreError::Platform(format!("{glab_err}")));
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
/// use gitflow_cli_gitlab::GitLabMilestoneProvider;
///
/// let provider = GitLabMilestoneProvider::new("gitlab-org/gitlab");
/// ```
#[derive(Debug, Clone)]
pub struct GitLabMilestoneProvider {
    /// GitLab `namespace/project`。
    repo: String,
}

impl GitLabMilestoneProvider {
    /// 创建新的 GitLab Milestone 提供者。
    ///
    /// `repo` 格式为 `namespace/project`。
    #[must_use]
    pub fn new(repo: impl Into<String>) -> Self {
        Self { repo: repo.into() }
    }
}

/// `glab milestone --output json` 返回的 JSON 结构。
#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code, reason = "Used for deserialization; not all fields are read")]
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
            chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok().map(|d| {
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
impl MilestoneProvider for GitLabMilestoneProvider {
    async fn create(&self, args: CreateMilestoneArgs) -> Result<MilestoneData> {
        debug!(repo = %self.repo, title = %args.title, "spawning `glab milestone create`");

        let mut cmd = tokio::process::Command::new("glab");
        cmd.args(["milestone", "create"])
            .arg("--title")
            .arg(&args.title)
            .arg("--repo")
            .arg(&self.repo)
            .arg("--output")
            .arg("json");

        if let Some(ref desc) = args.description {
            cmd.arg("--description").arg(desc);
        }

        if let Some(ref due) = args.due_on {
            cmd.arg("--due-date").arg(due.format("%Y-%m-%d").to_string());
        }

        let output = cmd
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab milestone create: {e}")))?;

        if !output.status.success() {
            let glab_err = parse_glab_error(&output.stderr);
            return Err(CoreError::Platform(format!("{glab_err}")));
        }

        let api_response: MilestoneApiResponse =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(api_response.into())
    }

    async fn list(&self) -> Result<Vec<MilestoneData>> {
        debug!(repo = %self.repo, "spawning `glab milestone list`");

        let output = tokio::process::Command::new("glab")
            .args(["milestone", "list"])
            .arg("--repo")
            .arg(&self.repo)
            .arg("--output")
            .arg("json")
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn glab milestone list: {e}")))?;

        if !output.status.success() {
            let glab_err = parse_glab_error(&output.stderr);
            return Err(CoreError::Platform(format!("{glab_err}")));
        }

        let api_responses: Vec<MilestoneApiResponse> =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(api_responses.into_iter().map(MilestoneData::from).collect())
    }

    async fn edit(&self, number: u64, args: CreateMilestoneArgs) -> Result<MilestoneData> {
        debug!(repo = %self.repo, number, "spawning `glab milestone edit`");

        let mut cmd = tokio::process::Command::new("glab");
        cmd.args(["milestone", "edit"])
            .arg(number.to_string())
            .arg("--repo")
            .arg(&self.repo)
            .arg("--title")
            .arg(&args.title)
            .arg("--output")
            .arg("json");

        if let Some(ref desc) = args.description {
            cmd.arg("--description").arg(desc);
        }

        if let Some(ref due) = args.due_on {
            cmd.arg("--due-date").arg(due.format("%Y-%m-%d").to_string());
        }

        let output = cmd
            .output()
            .await
            .map_err(|e| {
                CoreError::Platform(format!("Failed to spawn glab milestone edit: {e}"))
            })?;

        if !output.status.success() {
            let glab_err = parse_glab_error(&output.stderr);
            return Err(CoreError::Platform(format!("{glab_err}")));
        }

        let api_response: MilestoneApiResponse =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(api_response.into())
    }

    async fn close(&self, number: u64) -> Result<MilestoneData> {
        debug!(repo = %self.repo, number, "spawning `glab milestone close`");

        let output = tokio::process::Command::new("glab")
            .args(["milestone", "close"])
            .arg(number.to_string())
            .arg("--repo")
            .arg(&self.repo)
            .arg("--output")
            .arg("json")
            .output()
            .await
            .map_err(|e| {
                CoreError::Platform(format!("Failed to spawn glab milestone close: {e}"))
            })?;

        if !output.status.success() {
            let glab_err = parse_glab_error(&output.stderr);
            return Err(CoreError::Platform(format!("{glab_err}")));
        }

        let api_response: MilestoneApiResponse =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(api_response.into())
    }

    async fn reopen(&self, number: u64) -> Result<MilestoneData> {
        debug!(repo = %self.repo, number, "spawning `glab milestone reopen`");

        let output = tokio::process::Command::new("glab")
            .args(["milestone", "reopen"])
            .arg(number.to_string())
            .arg("--repo")
            .arg(&self.repo)
            .arg("--output")
            .arg("json")
            .output()
            .await
            .map_err(|e| {
                CoreError::Platform(format!("Failed to spawn glab milestone reopen: {e}"))
            })?;

        if !output.status.success() {
            let glab_err = parse_glab_error(&output.stderr);
            return Err(CoreError::Platform(format!("{glab_err}")));
        }

        let api_response: MilestoneApiResponse =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(api_response.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

        let api: LabelApiResponse =
            serde_json::from_slice(json).expect("valid LabelApiResponse");
        let label: LabelData = api.into();
        assert_eq!(label.name, "bug");
        assert_eq!(label.color.as_deref(), Some("#d73a4a"));
        assert_eq!(label.description.as_deref(), Some("Something isn't working"));
    }

    #[test]
    fn test_should_deserialize_label_list() {
        let json = br##"[
            {"name": "bug", "color": "#d73a4a", "description": "Bug"},
            {"name": "feature", "color": "#0075ca", "description": null}
        ]"##;

        let list: Vec<LabelApiResponse> =
            serde_json::from_slice(json).expect("valid label list");
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].name, "bug");
        assert_eq!(list[1].name, "feature");
    }

    #[test]
    fn test_should_deserialize_empty_label_list() {
        let json = b"[]";
        let list: Vec<LabelApiResponse> =
            serde_json::from_slice(json).expect("valid empty list");
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
