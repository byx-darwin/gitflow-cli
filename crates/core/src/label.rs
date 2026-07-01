//! Label and Milestone resource types with platform abstraction.
//!
//! Defines data types for managing repository labels and milestones,
//! along with [`LabelProvider`] and [`MilestoneProvider`] traits for
//! cross-platform implementations (GitHub, GitLab, `GitCode`, etc.).

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{Result, types::State};

/// A standalone label resource definition.
///
/// Unlike [`crate::types::Label`] which represents a label *association*
/// on an Issue or PR, this type represents the label resource itself —
/// its name, display color, and description as managed in the repository.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LabelData {
    /// The label name.
    pub name: String,
    /// The label color as a hex string (e.g. `"d73a4a"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// A human-readable description of the label.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Arguments for creating a new label.
#[derive(Debug, Clone)]
pub struct CreateLabelArgs {
    /// The label name.
    pub name: String,
    /// The label color as a hex string (e.g. `"d73a4a"`).
    pub color: String,
    /// A human-readable description of the label.
    pub description: Option<String>,
}

/// A milestone resource definition.
///
/// Milestones group issues and PRs under a shared goal or release
/// target with optional due dates and progress tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MilestoneData {
    /// The milestone's numeric identifier.
    pub number: u64,
    /// The milestone title.
    pub title: String,
    /// The milestone description (Markdown).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The milestone's current state.
    pub state: State,
    /// The optional due date (UTC).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_on: Option<DateTime<Utc>>,
    /// The number of closed issues under this milestone.
    pub closed_issues: u64,
    /// The number of open issues under this milestone.
    pub open_issues: u64,
}

/// Arguments for creating a new milestone.
#[derive(Debug, Clone)]
pub struct CreateMilestoneArgs {
    /// The milestone title.
    pub title: String,
    /// The milestone description (Markdown).
    pub description: Option<String>,
    /// The optional due date (UTC).
    pub due_on: Option<DateTime<Utc>>,
}

/// Platform abstraction for label management.
///
/// All platform implementations (GitHub/GitLab/GitCode) must implement
/// this trait to provide unified label CRUD operations.
///
/// # Errors
///
/// All methods return [`CoreError`](crate::CoreError) on platform API
/// failure, deserialization errors, or authentication failures.
#[async_trait]
pub trait LabelProvider: std::fmt::Debug + Send + Sync {
    /// Create a new label in the repository.
    ///
    /// # Errors
    ///
    /// Returns an error if the platform API call fails, the label name
    /// already exists, or the parameters are invalid.
    async fn create(&self, args: CreateLabelArgs) -> Result<LabelData>;

    /// List all labels in the repository.
    ///
    /// # Errors
    ///
    /// Returns an error if the platform API call fails.
    async fn list(&self) -> Result<Vec<LabelData>>;

    /// Edit an existing label by name.
    ///
    /// # Errors
    ///
    /// Returns an error if the label does not exist or the platform API call fails.
    async fn edit(&self, name: &str, args: CreateLabelArgs) -> Result<LabelData>;

    /// Delete a label by name.
    ///
    /// # Errors
    ///
    /// Returns an error if the label does not exist or the platform API call fails.
    async fn delete(&self, name: &str) -> Result<()>;
}

/// Platform abstraction for milestone management.
///
/// All platform implementations (GitHub/GitLab/GitCode) must implement
/// this trait to provide unified milestone lifecycle operations.
///
/// # Errors
///
/// All methods return [`CoreError`](crate::CoreError) on platform API
/// failure, deserialization errors, or authentication failures.
#[async_trait]
pub trait MilestoneProvider: std::fmt::Debug + Send + Sync {
    /// Create a new milestone.
    ///
    /// # Errors
    ///
    /// Returns an error if the platform API call fails or the parameters are invalid.
    async fn create(&self, args: CreateMilestoneArgs) -> Result<MilestoneData>;

    /// List all milestones in the repository.
    ///
    /// # Errors
    ///
    /// Returns an error if the platform API call fails.
    async fn list(&self) -> Result<Vec<MilestoneData>>;

    /// Edit an existing milestone by number.
    ///
    /// # Errors
    ///
    /// Returns an error if the milestone does not exist or the platform API call fails.
    async fn edit(&self, number: u64, args: CreateMilestoneArgs) -> Result<MilestoneData>;

    /// Close a milestone by number.
    ///
    /// # Errors
    ///
    /// Returns an error if the milestone does not exist or the platform API call fails.
    async fn close(&self, number: u64) -> Result<MilestoneData>;

    /// Reopen a closed milestone by number.
    ///
    /// # Errors
    ///
    /// Returns an error if the milestone does not exist or the platform API call fails.
    async fn reopen(&self, number: u64) -> Result<MilestoneData>;
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- LabelData tests ---

    #[test]
    fn test_should_serialize_label_data_to_camel_case_json() {
        let label = LabelData {
            name: "bug".into(),
            color: Some("d73a4a".into()),
            description: Some("Something isn't working".into()),
        };
        let json = serde_json::to_string(&label).expect("serialize LabelData");
        assert!(json.contains("\"name\":\"bug\""));
        assert!(json.contains("\"color\":\"d73a4a\""));
        assert!(json.contains("\"description\""));
    }

    #[test]
    fn test_should_deserialize_label_data_from_camel_case_json() {
        let json = r#"{"name":"enhancement","color":"a2eeef","description":"New feature"}"#;
        let label: LabelData = serde_json::from_str(json).expect("valid LabelData");
        assert_eq!(label.name, "enhancement");
        assert_eq!(label.color, Some("a2eeef".into()));
        assert_eq!(label.description, Some("New feature".into()));
    }

    #[test]
    fn test_should_deserialize_label_data_with_missing_optional_fields() {
        let json = r#"{"name":"wip"}"#;
        let label: LabelData = serde_json::from_str(json).expect("valid LabelData");
        assert_eq!(label.name, "wip");
        assert!(label.color.is_none());
        assert!(label.description.is_none());
    }

    #[test]
    fn test_should_serialize_label_data_skips_null_fields() {
        let label = LabelData {
            name: "wip".into(),
            color: None,
            description: None,
        };
        let json = serde_json::to_string(&label).expect("serialize LabelData");
        assert_eq!(json, r#"{"name":"wip"}"#);
        assert!(!json.contains("null"));
    }

    #[test]
    fn test_should_roundtrip_label_data_via_serde() {
        let label = LabelData {
            name: "bug".into(),
            color: Some("ff0000".into()),
            description: Some("A defect".into()),
        };
        let json = serde_json::to_string(&label).expect("serialize");
        let round_tripped: LabelData = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(round_tripped.name, label.name);
        assert_eq!(round_tripped.color, label.color);
        assert_eq!(round_tripped.description, label.description);
    }

    #[test]
    fn test_should_derive_debug_for_label_data() {
        let label = LabelData {
            name: "test".into(),
            color: None,
            description: None,
        };
        let debug = format!("{label:?}");
        assert!(debug.contains("LabelData"));
        assert!(debug.contains("test"));
    }

    // --- CreateLabelArgs tests ---

    #[test]
    fn test_should_create_label_args_with_description() {
        let args = CreateLabelArgs {
            name: "bug".into(),
            color: "d73a4a".into(),
            description: Some("Defect report".into()),
        };
        assert_eq!(args.name, "bug");
        assert_eq!(args.color, "d73a4a");
        assert_eq!(args.description, Some("Defect report".into()));
    }

    #[test]
    fn test_should_create_label_args_without_description() {
        let args = CreateLabelArgs {
            name: "wip".into(),
            color: "ffff00".into(),
            description: None,
        };
        assert_eq!(args.name, "wip");
        assert!(args.description.is_none());
    }

    // --- MilestoneData tests ---

    fn sample_milestone_json() -> &'static str {
        r#"{
            "number": 1,
            "title": "v1.0 Release",
            "description": "First stable release",
            "state": "open",
            "dueOn": "2026-06-01T00:00:00Z",
            "closedIssues": 10,
            "openIssues": 5
        }"#
    }

    #[test]
    fn test_should_deserialize_milestone_data_from_camel_case_json() {
        let json = sample_milestone_json();
        let milestone: MilestoneData = serde_json::from_str(json).expect("valid MilestoneData");
        assert_eq!(milestone.number, 1);
        assert_eq!(milestone.title, "v1.0 Release");
        assert_eq!(milestone.description, Some("First stable release".into()));
        assert_eq!(milestone.state, State::Open);
        assert!(milestone.due_on.is_some());
        assert_eq!(milestone.closed_issues, 10);
        assert_eq!(milestone.open_issues, 5);
    }

    #[test]
    fn test_should_serialize_milestone_data_to_camel_case_json() {
        let json = sample_milestone_json();
        let milestone: MilestoneData = serde_json::from_str(json).expect("deserialize");
        let serialized = serde_json::to_string(&milestone).expect("serialize");
        assert!(serialized.contains("\"closedIssues\""));
        assert!(serialized.contains("\"openIssues\""));
        assert!(serialized.contains("\"dueOn\""));
        assert!(!serialized.contains("\"closed_issues\""));
        assert!(!serialized.contains("\"open_issues\""));
    }

    #[test]
    fn test_should_roundtrip_milestone_data_via_serde() {
        let json = sample_milestone_json();
        let milestone: MilestoneData = serde_json::from_str(json).expect("deserialize");
        let re_serialized = serde_json::to_string(&milestone).expect("serialize");
        let round_tripped: MilestoneData =
            serde_json::from_str(&re_serialized).expect("re-deserialize");
        assert_eq!(round_tripped.number, milestone.number);
        assert_eq!(round_tripped.title, milestone.title);
        assert_eq!(round_tripped.description, milestone.description);
        assert_eq!(round_tripped.state, milestone.state);
        assert_eq!(round_tripped.due_on, milestone.due_on);
        assert_eq!(round_tripped.closed_issues, milestone.closed_issues);
        assert_eq!(round_tripped.open_issues, milestone.open_issues);
    }

    #[test]
    fn test_should_deserialize_milestone_with_null_optional_fields() {
        let json = r#"{
            "number": 2,
            "title": "v1.1",
            "description": null,
            "state": "closed",
            "dueOn": null,
            "closedIssues": 20,
            "openIssues": 0
        }"#;
        let milestone: MilestoneData = serde_json::from_str(json).expect("deserialize");
        assert!(milestone.description.is_none());
        assert!(milestone.due_on.is_none());
        assert_eq!(milestone.state, State::Closed);
    }

    #[test]
    fn test_should_serialize_milestone_skips_null_fields() {
        let milestone = MilestoneData {
            number: 3,
            title: "Future".into(),
            description: None,
            state: State::Open,
            due_on: None,
            closed_issues: 0,
            open_issues: 3,
        };
        let json = serde_json::to_string(&milestone).expect("serialize");
        assert!(!json.contains("null"));
        assert!(!json.contains("\"description\":null"));
        assert!(!json.contains("\"dueOn\":null"));
    }

    #[test]
    fn test_should_derive_debug_for_milestone_data() {
        let milestone = MilestoneData {
            number: 1,
            title: "v1.0".into(),
            description: None,
            state: State::Open,
            due_on: None,
            closed_issues: 0,
            open_issues: 1,
        };
        let debug = format!("{milestone:?}");
        assert!(debug.contains("MilestoneData"));
        assert!(debug.contains("v1.0"));
    }

    // --- CreateMilestoneArgs tests ---

    #[test]
    fn test_should_create_milestone_args_with_all_fields() {
        let args = CreateMilestoneArgs {
            title: "v2.0".into(),
            description: Some("Major release".into()),
            due_on: Some("2026-12-01T00:00:00Z".parse().expect("valid date")),
        };
        assert_eq!(args.title, "v2.0");
        assert_eq!(args.description, Some("Major release".into()));
        assert!(args.due_on.is_some());
    }

    #[test]
    fn test_should_create_milestone_args_minimal() {
        let args = CreateMilestoneArgs {
            title: "Backlog".into(),
            description: None,
            due_on: None,
        };
        assert_eq!(args.title, "Backlog");
        assert!(args.description.is_none());
        assert!(args.due_on.is_none());
    }
}
