//! GitCode Label 和 Milestone 提供者实现。
//!
//! 通过 `gc label` 和 `gc api` CLI 命令实现 [`LabelProvider`] 和
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

use crate::error::parse_gc_error;

/// GitCode Label 提供者，通过 `gc` CLI 管理仓库标签。
///
/// # Examples
///
/// ```no_run
/// use gitflow_cli_gitcode::GitCodeLabelProvider;
///
/// let provider = GitCodeLabelProvider::new("octocat/hello-world");
/// ```
#[derive(Debug, Clone)]
pub struct GitCodeLabelProvider {
    /// GitCode `owner/repo`。
    repo: String,
}

impl GitCodeLabelProvider {
    /// 创建新的 GitCode Label 提供者。
    ///
    /// `repo` 格式为 `owner/repo`。
    #[must_use]
    pub fn new(repo: impl Into<String>) -> Self {
        Self { repo: repo.into() }
    }
}

/// `gc label list/create` 请求的 JSON 字段列表。
const LABEL_FIELDS: &str = "name,color,description";

#[async_trait]
impl LabelProvider for GitCodeLabelProvider {
    async fn create(&self, args: CreateLabelArgs) -> Result<LabelData> {
        debug!(
            repo = %self.repo,
            name = %args.name,
            color = %args.color,
            "spawning `gc label create`"
        );

        let mut cmd = tokio::process::Command::new(crate::gitcode_binary());
        cmd.args(["label", "create"])
            .arg(&args.name)
            .arg("--color")
            .arg(&args.color)
            .arg("--repo")
            .arg(&self.repo);

        if let Some(ref desc) = args.description {
            cmd.arg("--description").arg(desc);
        }

        let output = cmd
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gc label create: {e}")))?;

        if !output.status.success() {
            let gc_err = parse_gc_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gc_err}")));
        }

        let label: LabelData =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(label)
    }

    async fn list(&self) -> Result<Vec<LabelData>> {
        debug!(repo = %self.repo, "spawning `gc label list`");

        let output = tokio::process::Command::new(crate::gitcode_binary())
            .args(["label", "list"])
            .arg("--repo")
            .arg(&self.repo)
            .arg("--json")
            .arg(LABEL_FIELDS)
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gc label list: {e}")))?;

        if !output.status.success() {
            let gc_err = parse_gc_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gc_err}")));
        }

        let labels: Vec<LabelData> =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(labels)
    }

    async fn edit(&self, name: &str, args: CreateLabelArgs) -> Result<LabelData> {
        debug!(repo = %self.repo, name, "spawning `gc label edit`");

        let mut cmd = tokio::process::Command::new(crate::gitcode_binary());
        cmd.args(["label", "edit"])
            .arg(name)
            .arg("--repo")
            .arg(&self.repo)
            .arg("--color")
            .arg(&args.color);

        if let Some(ref desc) = args.description {
            cmd.arg("--description").arg(desc);
        }

        let output = cmd
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gc label edit: {e}")))?;

        if !output.status.success() {
            let gc_err = parse_gc_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gc_err}")));
        }

        // gc label edit 不返回 JSON，重新 fetch 获取最新数据
        self.fetch_label(name).await
    }

    async fn delete(&self, name: &str) -> Result<()> {
        debug!(repo = %self.repo, name, "spawning `gc label delete`");

        let output = tokio::process::Command::new(crate::gitcode_binary())
            .args(["label", "delete"])
            .arg(name)
            .arg("--yes")
            .arg("--repo")
            .arg(&self.repo)
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gc label delete: {e}")))?;

        if !output.status.success() {
            let gc_err = parse_gc_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gc_err}")));
        }

        Ok(())
    }
}

impl GitCodeLabelProvider {
    /// 获取指定名称的标签数据（内部辅助方法）。
    async fn fetch_label(&self, name: &str) -> Result<LabelData> {
        let output = tokio::process::Command::new(crate::gitcode_binary())
            .args(["label", "view"])
            .arg(name)
            .arg("--repo")
            .arg(&self.repo)
            .arg("--json")
            .arg(LABEL_FIELDS)
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gc label view: {e}")))?;

        if !output.status.success() {
            let gc_err = parse_gc_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gc_err}")));
        }

        let label: LabelData =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(label)
    }
}

/// GitCode 里程碑提供者，通过 `gc api` 管理仓库里程碑。
///
/// # Examples
///
/// ```no_run
/// use gitflow_cli_gitcode::GitCodeMilestoneProvider;
///
/// let provider = GitCodeMilestoneProvider::new("octocat/hello-world");
/// ```
#[derive(Debug, Clone)]
pub struct GitCodeMilestoneProvider {
    /// GitCode `owner/repo`。
    repo: String,
}

impl GitCodeMilestoneProvider {
    /// 创建新的 GitCode Milestone 提供者。
    ///
    /// `repo` 格式为 `owner/repo`。
    #[must_use]
    pub fn new(repo: impl Into<String>) -> Self {
        Self { repo: repo.into() }
    }
}

/// `gc api milestones` 返回的 JSON 结构。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MilestoneApiResponse {
    number: u64,
    title: String,
    #[serde(default)]
    description: Option<String>,
    state: String,
    #[serde(default)]
    due_on: Option<String>,
    #[serde(default)]
    closed_issues: u64,
    #[serde(default)]
    open_issues: u64,
}

impl From<MilestoneApiResponse> for MilestoneData {
    fn from(api: MilestoneApiResponse) -> Self {
        Self {
            number: api.number,
            title: api.title,
            description: api.description,
            state: if api.state == "closed" {
                State::Closed
            } else {
                State::Open
            },
            due_on: api.due_on.and_then(|s| {
                DateTime::parse_from_rfc3339(&s).map_or(None, |dt| Some(dt.with_timezone(&Utc)))
            }),
            closed_issues: api.closed_issues,
            open_issues: api.open_issues,
        }
    }
}

#[async_trait]
impl MilestoneProvider for GitCodeMilestoneProvider {
    async fn create(&self, args: CreateMilestoneArgs) -> Result<MilestoneData> {
        debug!(repo = %self.repo, title = %args.title, "spawning `gc api milestones POST`");

        let api_path = format!("repos/{repo}/milestones", repo = self.repo);

        let mut cmd = tokio::process::Command::new(crate::gitcode_binary());
        cmd.args(["api", &api_path, "-X", "POST"])
            .arg("-f")
            .arg(format!("title={}", args.title));

        if let Some(ref desc) = args.description {
            cmd.arg("-f").arg(format!("description={desc}"));
        }

        if let Some(ref due) = args.due_on {
            cmd.arg("-f").arg(format!("due_on={}", due.to_rfc3339()));
        }

        let output = cmd
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gc api milestones: {e}")))?;

        if !output.status.success() {
            let gc_err = parse_gc_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gc_err}")));
        }

        let api_response: MilestoneApiResponse =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(api_response.into())
    }

    async fn list(&self) -> Result<Vec<MilestoneData>> {
        debug!(repo = %self.repo, "spawning `gc api milestones list`");

        let api_path = format!("repos/{repo}/milestones", repo = self.repo);

        let output = tokio::process::Command::new(crate::gitcode_binary())
            .args(["api", &api_path])
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gc api milestones: {e}")))?;

        if !output.status.success() {
            let gc_err = parse_gc_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gc_err}")));
        }

        let milestones: Vec<MilestoneApiResponse> =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(milestones.into_iter().map(MilestoneData::from).collect())
    }

    async fn edit(&self, number: u64, args: CreateMilestoneArgs) -> Result<MilestoneData> {
        debug!(repo = %self.repo, number, "spawning `gc api milestones PATCH`");

        let api_path = format!("repos/{repo}/milestones/{number}", repo = self.repo);

        let mut cmd = tokio::process::Command::new(crate::gitcode_binary());
        cmd.args(["api", &api_path, "-X", "PATCH"]);

        cmd.arg("-f").arg(format!("title={}", args.title));

        if let Some(ref desc) = args.description {
            cmd.arg("-f").arg(format!("description={desc}"));
        }

        if let Some(ref due) = args.due_on {
            cmd.arg("-f").arg(format!("due_on={}", due.to_rfc3339()));
        }

        let output = cmd.output().await.map_err(|e| {
            CoreError::Platform(format!("Failed to spawn gc api milestone edit: {e}"))
        })?;

        if !output.status.success() {
            let gc_err = parse_gc_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gc_err}")));
        }

        let api_response: MilestoneApiResponse =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(api_response.into())
    }

    async fn close(&self, number: u64) -> Result<MilestoneData> {
        debug!(repo = %self.repo, number, "spawning `gc api milestones close`");

        let api_path = format!("repos/{repo}/milestones/{number}", repo = self.repo);

        let output = tokio::process::Command::new(crate::gitcode_binary())
            .args(["api", &api_path, "-X", "PATCH", "-f", "state=closed"])
            .output()
            .await
            .map_err(|e| {
                CoreError::Platform(format!("Failed to spawn gc api milestone close: {e}"))
            })?;

        if !output.status.success() {
            let gc_err = parse_gc_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gc_err}")));
        }

        let api_response: MilestoneApiResponse =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(api_response.into())
    }

    async fn reopen(&self, number: u64) -> Result<MilestoneData> {
        debug!(repo = %self.repo, number, "spawning `gc api milestones reopen`");

        let api_path = format!("repos/{repo}/milestones/{number}", repo = self.repo);

        let output = tokio::process::Command::new(crate::gitcode_binary())
            .args(["api", &api_path, "-X", "PATCH", "-f", "state=open"])
            .output()
            .await
            .map_err(|e| {
                CoreError::Platform(format!("Failed to spawn gc api milestone reopen: {e}"))
            })?;

        if !output.status.success() {
            let gc_err = parse_gc_error(&output.stderr);
            return Err(CoreError::Platform(format!("{gc_err}")));
        }

        let api_response: MilestoneApiResponse =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(api_response.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- GitCodeLabelProvider tests ---

    #[test]
    fn test_should_construct_gitcode_label_provider() {
        let provider = GitCodeLabelProvider::new("octocat/hello-world");
        assert_eq!(provider.repo, "octocat/hello-world");
    }

    #[test]
    fn test_should_construct_gitcode_label_provider_from_string() {
        let repo = String::from("octocat/hello-world");
        let provider = GitCodeLabelProvider::new(repo);
        assert_eq!(provider.repo, "octocat/hello-world");
    }

    #[test]
    fn test_should_debug_format_label_provider() {
        let provider = GitCodeLabelProvider::new("owner/repo");
        let debug = format!("{provider:?}");
        assert!(debug.contains("GitCodeLabelProvider"));
        assert!(debug.contains("owner/repo"));
    }

    #[test]
    fn test_should_clone_gitcode_label_provider() {
        let original = GitCodeLabelProvider::new("owner/repo");
        let cloned = original.clone();
        assert_eq!(original.repo, cloned.repo);
    }

    // --- GitCodeMilestoneProvider tests ---

    #[test]
    fn test_should_construct_gitcode_milestone_provider() {
        let provider = GitCodeMilestoneProvider::new("octocat/hello-world");
        assert_eq!(provider.repo, "octocat/hello-world");
    }

    #[test]
    fn test_should_debug_format_milestone_provider() {
        let provider = GitCodeMilestoneProvider::new("owner/repo");
        let debug = format!("{provider:?}");
        assert!(debug.contains("GitCodeMilestoneProvider"));
        assert!(debug.contains("owner/repo"));
    }

    #[test]
    fn test_should_clone_gitcode_milestone_provider() {
        let original = GitCodeMilestoneProvider::new("owner/repo");
        let cloned = original.clone();
        assert_eq!(original.repo, cloned.repo);
    }

    // --- LabelData deserialization tests ---

    #[test]
    fn test_should_deserialize_label_data_from_gc_output() {
        let json = br#"[
            {"name": "bug", "color": "d73a4a", "description": "Something isn't working"},
            {"name": "enhancement", "color": "a2eeef", "description": null}
        ]"#;

        let labels: Vec<LabelData> = serde_json::from_slice(json).expect("valid LabelData list");
        assert_eq!(labels.len(), 2);
        assert_eq!(labels[0].name, "bug");
        assert_eq!(labels[0].color.as_deref(), Some("d73a4a"));
        assert_eq!(labels[1].description, None);
    }

    #[test]
    fn test_should_deserialize_single_label_from_gc_output() {
        let json = br#"{"name": "wip", "color": "ffff00", "description": "Work in progress"}"#;

        let label: LabelData = serde_json::from_slice(json).expect("valid LabelData");
        assert_eq!(label.name, "wip");
        assert_eq!(label.color.as_deref(), Some("ffff00"));
        assert_eq!(label.description.as_deref(), Some("Work in progress"));
    }

    #[test]
    fn test_should_deserialize_empty_label_list() {
        let json = b"[]";
        let labels: Vec<LabelData> =
            serde_json::from_slice(json).expect("valid empty LabelData list");
        assert!(labels.is_empty());
    }

    // --- MilestoneData deserialization tests ---

    #[test]
    fn test_should_deserialize_milestone_api_response() {
        let json = br#"{
            "number": 1,
            "title": "v1.0 Release",
            "description": "First stable release",
            "state": "open",
            "dueOn": "2026-06-01T00:00:00Z",
            "closedIssues": 10,
            "openIssues": 5
        }"#;

        let api: MilestoneApiResponse =
            serde_json::from_slice(json).expect("valid MilestoneApiResponse");
        let data: MilestoneData = api.into();

        assert_eq!(data.number, 1);
        assert_eq!(data.title, "v1.0 Release");
        assert_eq!(data.description, Some("First stable release".into()));
        assert_eq!(data.state, State::Open);
        assert!(data.due_on.is_some());
        assert_eq!(data.closed_issues, 10);
        assert_eq!(data.open_issues, 5);
    }

    #[test]
    fn test_should_deserialize_closed_milestone() {
        let json = br#"{
            "number": 2,
            "title": "v0.9 Beta",
            "description": null,
            "state": "closed",
            "dueOn": null,
            "closedIssues": 20,
            "openIssues": 0
        }"#;

        let api: MilestoneApiResponse =
            serde_json::from_slice(json).expect("valid MilestoneApiResponse");
        let data: MilestoneData = api.into();

        assert_eq!(data.state, State::Closed);
        assert!(data.description.is_none());
        assert!(data.due_on.is_none());
    }

    #[test]
    fn test_should_deserialize_milestone_list() {
        let json = br#"[
            {"number": 1, "title": "v1.0", "description": null, "state": "open", "dueOn": null, "closedIssues": 0, "openIssues": 3},
            {"number": 2, "title": "v0.9", "description": "Beta", "state": "closed", "dueOn": "2026-01-01T00:00:00Z", "closedIssues": 15, "openIssues": 0}
        ]"#;

        let milestones: Vec<MilestoneApiResponse> =
            serde_json::from_slice(json).expect("valid MilestoneApiResponse list");
        assert_eq!(milestones.len(), 2);
        assert_eq!(milestones[0].title, "v1.0");
        assert_eq!(milestones[1].title, "v0.9");
    }

    #[test]
    fn test_should_convert_milestone_api_to_data() {
        let api = MilestoneApiResponse {
            number: 42,
            title: "Test Milestone".into(),
            description: Some("A test".into()),
            state: "open".into(),
            due_on: Some("2026-12-01T00:00:00Z".into()),
            closed_issues: 5,
            open_issues: 10,
        };

        let data: MilestoneData = api.clone().into();
        assert_eq!(data.number, api.number);
        assert_eq!(data.title, api.title);
    }
}
