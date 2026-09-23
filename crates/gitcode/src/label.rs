//! GitCode Label 和 Milestone 提供者实现。
//!
//! 通过 `gc label` 和 `gc api` CLI 命令实现 [`LabelProvider`] 和
//! [`MilestoneProvider`] trait，支持标签和里程碑的完整生命周期管理。

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use gitflow_core::{
    CoreError, DEFAULT_LIST_LIMIT, FetchStrategy, Paged, Result, Session, fetch_capped,
    label::{
        CreateLabelArgs, CreateMilestoneArgs, LabelData, LabelProvider, MilestoneData,
        MilestoneProvider,
    },
    types::State,
};
use serde::Deserialize;
use tracing::debug;

use crate::{
    error::parse_gitcode_error,
    runner::{CommandRunner, RealCommandRunner},
};

/// GitCode Label 提供者，通过 `gitcode` CLI 管理仓库标签。
///
/// # Examples
///
/// ```no_run
/// use gitflow_gitcode::GitCodeLabelProvider;
///
/// let provider = GitCodeLabelProvider::new("octocat/hello-world");
/// ```
#[derive(Debug, Clone)]
pub struct GitCodeLabelProvider<R: CommandRunner = RealCommandRunner> {
    /// GitCode `owner/repo`。
    repo: String,
    /// 用于执行 `gitcode` CLI 命令的 runner。
    runner: R,
}

impl GitCodeLabelProvider<RealCommandRunner> {
    /// 创建新的 GitCode Label 提供者。
    ///
    /// `repo` 格式为 `owner/repo`。
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

impl<R: CommandRunner> GitCodeLabelProvider<R> {
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

/// `gc label list/create` 请求的 JSON 字段列表。
const LABEL_FIELDS: &str = "name,color,description";

#[async_trait]
impl<R: CommandRunner + 'static> LabelProvider for GitCodeLabelProvider<R> {
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
            .arg("-R")
            .arg(&self.repo)
            .arg("--json");

        if let Some(ref desc) = args.description {
            cmd.arg("--description").arg(desc);
        }

        let output = cmd.output().await.map_err(|e| {
            CoreError::Platform(format!("Failed to spawn gitcode label create: {e}"))
        })?;

        if !output.status.success() {
            return Err(parse_gitcode_error(&output.stderr).into());
        }

        let label: LabelData =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(label)
    }

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

    async fn edit(&self, name: &str, args: CreateLabelArgs) -> Result<LabelData> {
        debug!(repo = %self.repo, name, "spawning `gc label edit`");

        let mut cmd = tokio::process::Command::new(crate::gitcode_binary());
        cmd.args(["label", "edit"])
            .arg(name)
            .arg("-R")
            .arg(&self.repo)
            .arg("--color")
            .arg(&args.color)
            .arg("--json");

        if let Some(ref desc) = args.description {
            cmd.arg("--description").arg(desc);
        }

        let output = cmd
            .output()
            .await
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gitcode label edit: {e}")))?;

        if !output.status.success() {
            return Err(parse_gitcode_error(&output.stderr).into());
        }

        // Try to parse JSON response, fallback to fetch if not available
        if let Ok(label) = serde_json::from_slice::<LabelData>(&output.stdout) {
            Ok(label)
        } else {
            self.fetch_label(name).await
        }
    }

    async fn delete(&self, name: &str) -> Result<()> {
        debug!(repo = %self.repo, name, "spawning `gc label delete`");

        let output = tokio::process::Command::new(crate::gitcode_binary())
            .args(["label", "delete"])
            .arg(name)
            .arg("--yes")
            .arg("-R")
            .arg(&self.repo)
            .output()
            .await
            .map_err(|e| {
                CoreError::Platform(format!("Failed to spawn gitcode label delete: {e}"))
            })?;

        if !output.status.success() {
            return Err(parse_gitcode_error(&output.stderr).into());
        }

        Ok(())
    }
}

impl<R: CommandRunner> GitCodeLabelProvider<R> {
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
            .map_err(|e| CoreError::Platform(format!("Failed to spawn gitcode label view: {e}")))?;

        if !output.status.success() {
            return Err(parse_gitcode_error(&output.stderr).into());
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
/// use gitflow_gitcode::GitCodeMilestoneProvider;
///
/// let provider = GitCodeMilestoneProvider::new("octocat/hello-world");
/// ```
#[derive(Debug, Clone)]
pub struct GitCodeMilestoneProvider<R: CommandRunner = RealCommandRunner> {
    /// GitCode `owner/repo`。
    repo: String,
    /// 用于执行 `gitcode` CLI 命令的 runner。
    runner: R,
}

impl GitCodeMilestoneProvider<RealCommandRunner> {
    /// 创建新的 GitCode Milestone 提供者。
    ///
    /// `repo` 格式为 `owner/repo`。
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

impl<R: CommandRunner> GitCodeMilestoneProvider<R> {
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

/// `gc api milestones` 返回的 JSON 结构。
#[derive(Debug, Clone, Deserialize)]
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
                // GitCode returns due_on as either full RFC3339 or a pure
                // "YYYY-MM-DD" date (issue #377); fall back to the latter,
                // anchored at midnight UTC, before giving up.
                if let Ok(dt) = DateTime::parse_from_rfc3339(&s) {
                    return Some(dt.with_timezone(&Utc));
                }
                chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d")
                    .ok()
                    .and_then(|d| d.and_hms_opt(0, 0, 0))
                    .map(|naive_dt| DateTime::<Utc>::from_naive_utc_and_offset(naive_dt, Utc))
            }),
            closed_issues: api.closed_issues,
            open_issues: api.open_issues,
        }
    }
}

#[async_trait]
impl<R: CommandRunner + 'static> MilestoneProvider for GitCodeMilestoneProvider<R> {
    async fn create(&self, args: CreateMilestoneArgs) -> Result<MilestoneData> {
        debug!(repo = %self.repo, title = %args.title, "spawning `gc milestone create`");

        let mut cmd = tokio::process::Command::new(crate::gitcode_binary());
        cmd.args(["milestone", "create"])
            .arg(&args.title)
            .arg("-R")
            .arg(&self.repo)
            .arg("--json");

        if let Some(ref desc) = args.description {
            cmd.arg("--description").arg(desc);
        }

        if let Some(ref due) = args.due_on {
            cmd.arg("--due-date")
                .arg(due.format("%Y-%m-%d").to_string());
        }

        let output = cmd.output().await.map_err(|e| {
            CoreError::Platform(format!("Failed to spawn gitcode milestone create: {e}"))
        })?;

        if !output.status.success() {
            return Err(parse_gitcode_error(&output.stderr).into());
        }

        let api_response: MilestoneApiResponse =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(api_response.into())
    }

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

    async fn edit(&self, number: u64, args: CreateMilestoneArgs) -> Result<MilestoneData> {
        debug!(repo = %self.repo, number, "spawning `gc milestone edit`");

        let mut cmd = tokio::process::Command::new(crate::gitcode_binary());
        cmd.args(["milestone", "edit"])
            .arg(number.to_string())
            .arg("-R")
            .arg(&self.repo)
            .arg("--title")
            .arg(&args.title)
            .arg("--json");

        if let Some(ref desc) = args.description {
            cmd.arg("--description").arg(desc);
        }

        if let Some(ref due) = args.due_on {
            cmd.arg("--due-date")
                .arg(due.format("%Y-%m-%d").to_string());
        }

        let output = cmd.output().await.map_err(|e| {
            CoreError::Platform(format!("Failed to spawn gitcode milestone edit: {e}"))
        })?;

        if !output.status.success() {
            return Err(parse_gitcode_error(&output.stderr).into());
        }

        let api_response: MilestoneApiResponse =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(api_response.into())
    }

    async fn close(&self, number: u64) -> Result<MilestoneData> {
        debug!(repo = %self.repo, number, "spawning `gc milestone close`");

        let output = tokio::process::Command::new(crate::gitcode_binary())
            .args(["milestone", "close"])
            .arg(number.to_string())
            .arg("-R")
            .arg(&self.repo)
            .arg("--json")
            .output()
            .await
            .map_err(|e| {
                CoreError::Platform(format!("Failed to spawn gitcode milestone close: {e}"))
            })?;

        if !output.status.success() {
            return Err(parse_gitcode_error(&output.stderr).into());
        }

        let api_response: MilestoneApiResponse =
            serde_json::from_slice(&output.stdout).map_err(CoreError::Serialization)?;

        Ok(api_response.into())
    }

    async fn reopen(&self, number: u64) -> Result<MilestoneData> {
        debug!(repo = %self.repo, number, "spawning `gc milestone reopen`");

        let output = tokio::process::Command::new(crate::gitcode_binary())
            .args(["milestone", "reopen"])
            .arg(number.to_string())
            .arg("-R")
            .arg(&self.repo)
            .arg("--json")
            .output()
            .await
            .map_err(|e| {
                CoreError::Platform(format!("Failed to spawn gitcode milestone reopen: {e}"))
            })?;

        if !output.status.success() {
            return Err(parse_gitcode_error(&output.stderr).into());
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
            "due_on": "2026-06-01T00:00:00Z",
            "closed_issues": 10,
            "open_issues": 5
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
            "due_on": null,
            "closed_issues": 20,
            "open_issues": 0
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
            {"number": 1, "title": "v1.0", "description": null, "state": "open", "due_on": null, "closed_issues": 0, "open_issues": 3},
            {"number": 2, "title": "v0.9", "description": "Beta", "state": "closed", "due_on": "2026-01-01T00:00:00Z", "closed_issues": 15, "open_issues": 0}
        ]"#;

        let milestones: Vec<MilestoneApiResponse> =
            serde_json::from_slice(json).expect("valid MilestoneApiResponse list");
        assert_eq!(milestones.len(), 2);
        assert_eq!(milestones[0].title, "v1.0");
        assert_eq!(milestones[1].title, "v0.9");
    }

    #[test]
    fn test_should_default_issue_counts_when_absent_from_real_list_response() {
        // 真实响应形状（2026-09-19 对 gitcode milestone list --repo openharmony/docs
        // 的实测）：closed_issues/open_issues 键完全不存在，不是"值为 0"。
        //
        // 该真实响应的 due_on 是纯日期格式（"2026-08-31"，无时间/时区部分）。见
        // Issue #377。
        let json = br#"{
            "id": null,
            "number": 733070,
            "title": "IT26_OpenHarmony 7.0(Release)",
            "description": "",
            "state": "active",
            "due_on": "2026-08-31"
        }"#;

        let api: MilestoneApiResponse =
            serde_json::from_slice(json).expect("valid MilestoneApiResponse");
        let data: MilestoneData = api.into();

        assert_eq!(data.number, 733_070);
        assert_eq!(
            data.closed_issues, 0,
            "键缺失时应靠 #[serde(default)] 落到 0，而不是反序列化失败"
        );
        assert_eq!(
            data.due_on,
            Some(
                chrono::DateTime::parse_from_rfc3339("2026-08-31T00:00:00Z")
                    .expect("valid rfc3339")
                    .with_timezone(&Utc)
            ),
            "纯日期格式的 due_on 应解析为当天 UTC 零点，而不是静默丢弃为 None（#377）"
        );
        assert_eq!(data.open_issues, 0);
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

    // --- Runner-routing regression tests ---
    //
    // `list` 曾直接 `tokio::process::Command::new(...)`，绕过可注入的
    // `CommandRunner`，导致其 argv 无法被测试观测。以下测试确认两者
    // 均已改为通过 `self.runner` 派发。

    use crate::runner::{MockCommandRunner, SequencedMockCommandRunner};

    #[tokio::test]
    #[allow(
        clippy::similar_names,
        reason = "page1/page2 fixtures vs paged result read clearly in test context"
    )]
    async fn test_should_page_through_gitcode_label_list_with_incrementing_page_numbers() {
        // cap=100 → per_page=100，want=101；首页满 100 条 ⇒ 必然发出第二页。
        fn label_page_json(start: u32, count: u32) -> String {
            let items: Vec<String> = (start..start + count)
                .map(|n| format!(r##"{{"name":"l{n}","color":"#ffffff","description":""}}"##))
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
        assert_eq!(
            calls.len(),
            2,
            "必须真的翻到第二页，实际调用数: {}",
            calls.len()
        );
        assert!(
            calls[0]
                .1
                .windows(2)
                .any(|w| w[0] == "--per-page" && w[1] == "100"),
            "实际 argv: {:?}",
            calls[0].1
        );
        assert!(
            calls[0]
                .1
                .windows(2)
                .any(|w| w[0] == "--page" && w[1] == "1"),
            "实际 argv: {:?}",
            calls[0].1
        );
        assert!(
            calls[1]
                .1
                .windows(2)
                .any(|w| w[0] == "--page" && w[1] == "2"),
            "页号必须递增，实际 argv: {:?}",
            calls[1].1
        );
    }

    #[tokio::test]
    #[allow(
        clippy::similar_names,
        reason = "page1/page2 fixtures vs paged result read clearly in test context"
    )]
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
        assert_eq!(
            calls.len(),
            2,
            "必须真的翻到第二页，实际调用数: {}",
            calls.len()
        );
        assert!(
            calls[0]
                .1
                .windows(2)
                .any(|w| w[0] == "--per-page" && w[1] == "100"),
            "实际 argv: {:?}",
            calls[0].1
        );
        assert!(
            calls[0]
                .1
                .windows(2)
                .any(|w| w[0] == "--page" && w[1] == "1"),
            "实际 argv: {:?}",
            calls[0].1
        );
        assert!(
            calls[1]
                .1
                .windows(2)
                .any(|w| w[0] == "--page" && w[1] == "2"),
            "页号必须递增，实际 argv: {:?}",
            calls[1].1
        );
    }

    #[tokio::test]
    async fn test_should_route_label_list_through_runner() {
        let runner = MockCommandRunner::success("[]");
        let provider = GitCodeLabelProvider::with_runner("owner/repo", runner.clone());

        let paged = provider.list(None).await.expect("should list");

        assert!(paged.items.is_empty());
        let calls = runner.recorded_calls();
        assert_eq!(calls.len(), 1);
        assert!(!calls[0].1.iter().any(|a| a == "--limit"));
        assert!(calls[0].1.iter().any(|a| a == "--per-page"));
        assert!(calls[0].1.iter().any(|a| a == "--page"));
    }

    #[tokio::test]
    async fn test_should_route_milestone_list_through_runner() {
        let runner = MockCommandRunner::success("[]");
        let provider = GitCodeMilestoneProvider::with_runner("owner/repo", runner.clone());

        let paged = provider.list(None).await.expect("should list");

        assert!(paged.items.is_empty());
        let calls = runner.recorded_calls();
        assert_eq!(calls.len(), 1);
        assert!(!calls[0].1.iter().any(|a| a == "--limit"));
        assert!(calls[0].1.iter().any(|a| a == "--per-page"));
        assert!(calls[0].1.iter().any(|a| a == "--page"));
    }

    #[tokio::test]
    async fn test_should_produce_complete_argv_for_gitcode_label_list_with_default_limit() {
        let runner = MockCommandRunner::success("[]");
        let provider = GitCodeLabelProvider::with_runner("owner/repo", runner.clone());

        let paged = provider.list(None).await.expect("should list");

        assert!(paged.items.is_empty());
        assert_eq!(runner.recorded_calls()[0].0, crate::gitcode_binary());
        assert_eq!(
            runner.recorded_calls()[0].1,
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
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>()
        );
    }

    #[tokio::test]
    async fn test_should_produce_complete_argv_for_gitcode_milestone_list_with_default_limit() {
        let runner = MockCommandRunner::success("[]");
        let provider = GitCodeMilestoneProvider::with_runner("owner/repo", runner.clone());

        let paged = provider.list(None).await.expect("should list");

        assert!(paged.items.is_empty());
        assert_eq!(runner.recorded_calls()[0].0, crate::gitcode_binary());
        assert_eq!(
            runner.recorded_calls()[0].1,
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
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>()
        );
    }
}
