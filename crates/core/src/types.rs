//! Shared domain types for gitflow-cli.
//!
//! These are pure data structures used across platform implementations
//! for representing users, states, and labels.

use serde::{Deserialize, Serialize};

/// A summary of a platform user.
///
/// Contains the minimal identifying information needed to reference
/// a user across API responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSummary {
    /// The user's login name.
    pub login: String,
    /// The user's numeric ID.
    pub id: u64,
}

/// The state of an Issue or Pull Request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum State {
    /// Open and active.
    Open,
    /// Closed or merged.
    Closed,
}

/// A label attached to an Issue or Pull Request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    /// The label name.
    pub name: String,
    /// The label color as a hex string (e.g. `"ff0000"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// A human-readable description of the label.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_serialize_state_to_snake_case() {
        let open = State::Open;
        let json = serde_json::to_string(&open).expect("serialize Open");
        assert_eq!(json, "\"open\"");

        let closed = State::Closed;
        let json = serde_json::to_string(&closed).expect("serialize Closed");
        assert_eq!(json, "\"closed\"");
    }

    #[test]
    fn test_should_deserialize_state_from_snake_case() {
        let state: State = serde_json::from_str("\"open\"").expect("deserialize open");
        assert_eq!(state, State::Open);

        let state: State = serde_json::from_str("\"closed\"").expect("deserialize closed");
        assert_eq!(state, State::Closed);
    }

    #[test]
    fn test_should_deserialize_user_summary_from_json() {
        let json = r#"{"login":"octocat","id":12345}"#;
        let user: UserSummary = serde_json::from_str(json).expect("valid UserSummary");
        assert_eq!(user.login, "octocat");
        assert_eq!(user.id, 12345);
    }

    #[test]
    fn test_should_serialize_user_summary_to_json() {
        let user = UserSummary {
            login: "octocat".into(),
            id: 12345,
        };
        let json = serde_json::to_string(&user).expect("serialize UserSummary");
        assert!(json.contains("\"login\":\"octocat\""));
        assert!(json.contains("\"id\":12345"));
    }

    #[test]
    fn test_should_deserialize_label_with_optional_fields() {
        // With all fields
        let json = r#"{"name":"bug","color":"d73a4a","description":"Something isn't working"}"#;
        let label: Label = serde_json::from_str(json).expect("valid Label");
        assert_eq!(label.name, "bug");
        assert_eq!(label.color, Some("d73a4a".into()));
        assert_eq!(label.description, Some("Something isn't working".into()));
    }

    #[test]
    fn test_should_deserialize_label_with_missing_optional_fields() {
        let json = r#"{"name":"wip"}"#;
        let label: Label = serde_json::from_str(json).expect("valid Label");
        assert_eq!(label.name, "wip");
        assert!(label.color.is_none());
        assert!(label.description.is_none());
    }

    #[test]
    fn test_should_serialize_label_skips_null_fields() {
        let label = Label {
            name: "wip".into(),
            color: None,
            description: None,
        };
        let json = serde_json::to_string(&label).expect("serialize Label");
        // Optional fields with None are omitted per CLAUDE.md serialization policy
        assert_eq!(json, r#"{"name":"wip"}"#);
        assert!(!json.contains("null"));
    }

    #[test]
    fn test_should_derive_debug_for_user_summary() {
        let user = UserSummary {
            login: "test".into(),
            id: 1,
        };
        let debug = format!("{user:?}");
        assert!(debug.contains("UserSummary"));
        assert!(debug.contains("login"));
        assert!(debug.contains("test"));
    }

    #[test]
    fn test_should_derive_debug_for_label() {
        let label = Label {
            name: "bug".into(),
            color: Some("ff0000".into()),
            description: None,
        };
        let debug = format!("{label:?}");
        assert!(debug.contains("Label"));
        assert!(debug.contains("bug"));
    }

    #[test]
    fn test_should_derive_clone_for_state() {
        let state = State::Open;
        let cloned = state.clone();
        assert_eq!(state, cloned);
    }

    #[test]
    fn test_should_derive_clone_for_user_summary() {
        let user = UserSummary {
            login: "test".into(),
            id: 42,
        };
        let cloned = user.clone();
        assert_eq!(user.login, cloned.login);
        assert_eq!(user.id, cloned.id);
    }

    #[test]
    fn test_should_derive_clone_for_label() {
        let label = Label {
            name: "bug".into(),
            color: None,
            description: None,
        };
        let cloned = label.clone();
        assert_eq!(label.name, cloned.name);
    }
}
