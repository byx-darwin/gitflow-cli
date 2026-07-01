//! CLI output types for uniform JSON serialization.
//!
//! All commands return a [`CliOutput`] that serializes to a consistent
//! JSON envelope with `success`, optional `data`, optional `error`,
//! and metadata fields.

use serde::Serialize;

/// CLI error information.
///
/// Returned as part of [`CliOutput`] when a command fails.
#[derive(Debug, Clone, Serialize)]
pub struct CliError {
    /// Machine-readable error code (e.g. `"AUTH_FAILED"`).
    pub code: String,
    /// Human-readable error message.
    pub message: String,
    /// Optional hint for the user on how to resolve the error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

impl CliError {
    /// Create a new error with a code and message.
    #[must_use]
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            hint: None,
        }
    }

    /// Attach a hint to the error.
    #[must_use]
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }
}

/// Unified CLI JSON output envelope.
///
/// On success, `data` is populated and `error` is `None`.
/// On failure, `error` is populated and `data` is `None`.
/// Fields with `None` values are omitted from serialized output.
#[derive(Debug, Clone, Serialize)]
pub struct CliOutput<T: Serialize> {
    /// Whether the command succeeded.
    pub success: bool,
    /// The payload data, present only on success.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    /// Error details, present only on failure.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<CliError>,
    /// The detected platform (e.g. `"github"`).
    pub platform: String,
    /// The command that was executed (e.g. `"issue create"`).
    pub command: String,
}

impl<T: Serialize> CliOutput<T> {
    /// Create a successful output with the given data.
    #[must_use]
    pub fn success(data: T, platform: &str, command: &str) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            platform: platform.into(),
            command: command.into(),
        }
    }

    /// Create a failed output with the given error.
    #[must_use]
    pub fn failure(error: CliError, platform: &str) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
            platform: platform.into(),
            command: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Serialize)]
    struct SampleData {
        url: String,
        number: u64,
    }

    #[test]
    fn test_success_output_serializes_with_data_no_error() {
        let data = SampleData {
            url: "https://github.com/user/repo/issues/1".into(),
            number: 1,
        };
        let output = CliOutput::success(data, "github", "issue create");
        let json = serde_json::to_string(&output).expect("failed to serialize");

        assert!(json.contains(r#""success":true"#));
        assert!(json.contains(r#""url""#));
        assert!(json.contains(r#""number":1"#));
        assert!(json.contains(r#""platform":"github""#));
        assert!(json.contains(r#""command":"issue create""#));
        assert!(!json.contains(r#""error""#));
        assert!(!json.contains(r#""data":null"#));
    }

    #[test]
    fn test_failure_output_serializes_with_error_no_data() {
        let error = CliError::new("AUTH_FAILED", "Authentication failed");
        let output: CliOutput<serde_json::Value> = CliOutput::failure(error, "github");
        let json = serde_json::to_string(&output).expect("failed to serialize");

        assert!(json.contains(r#""success":false"#));
        assert!(json.contains(r#""code":"AUTH_FAILED""#));
        assert!(json.contains(r#""message":"Authentication failed""#));
        assert!(json.contains(r#""platform":"github""#));
        assert!(!json.contains(r#""data""#));
    }

    #[test]
    fn test_hint_omitted_when_none() {
        let error = CliError::new("NOT_FOUND", "Resource not found");
        let output: CliOutput<serde_json::Value> = CliOutput::failure(error, "gitlab");
        let json = serde_json::to_string(&output).expect("failed to serialize");

        assert!(!json.contains(r#""hint""#));
    }

    #[test]
    fn test_with_hint_adds_hint() {
        let error = CliError::new("AUTH_FAILED", "Authentication failed")
            .with_hint("Run 'gitflow auth login'");
        let output: CliOutput<serde_json::Value> = CliOutput::failure(error, "github");
        let json = serde_json::to_string(&output).expect("failed to serialize");

        assert!(json.contains(r#""hint":"Run 'gitflow auth login'"#));
    }

    #[test]
    fn test_cli_error_new_and_with_hint() {
        let error = CliError::new("TEST_ERROR", "test message").with_hint("test hint");

        assert_eq!(error.code, "TEST_ERROR");
        assert_eq!(error.message, "test message");
        assert_eq!(error.hint, Some("test hint".into()));
    }
}
