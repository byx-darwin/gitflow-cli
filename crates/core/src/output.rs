//! CLI output types for uniform JSON serialization.
//!
//! All commands return a [`CliOutput`] that serializes to a consistent
//! JSON envelope with `success`, optional `data`, optional `error`,
//! and metadata fields.

use serde::{Deserialize, Serialize};

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
    /// 分页元数据，仅列表类命令出现。
    ///
    /// 非列表命令不序列化此字段，因此其输出与本特性引入前逐字节相同。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<PaginationMeta>,
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
            pagination: None,
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
            pagination: None,
            platform: platform.into(),
            command: String::new(),
        }
    }

    /// 创建带分页元数据的成功输出。
    #[must_use]
    pub fn success_paged(
        data: T,
        pagination: PaginationMeta,
        platform: &str,
        command: &str,
    ) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            pagination: Some(pagination),
            platform: platform.into(),
            command: command.into(),
        }
    }
}

/// 列表类命令的分页元数据。
///
/// 出现在输出信封上而非 `data` 内部，因此 `data` 仍是数组，既有的
/// `.data[]` 消费方（jq 脚本、skill）不受影响。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginationMeta {
    /// 是否因触顶而丢弃了更多条目。
    pub truncated: bool,
    /// 本次实际返回的条目数。
    pub returned: usize,
    /// 本次生效的上限。
    pub limit: u32,
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
        let error =
            CliError::new("AUTH_FAILED", "Authentication failed").with_hint("Run 'gf auth login'");
        let output: CliOutput<serde_json::Value> = CliOutput::failure(error, "github");
        let json = serde_json::to_string(&output).expect("failed to serialize");

        assert!(json.contains(r#""hint":"Run 'gf auth login'"#));
    }

    #[test]
    fn test_cli_error_new_and_with_hint() {
        let error = CliError::new("TEST_ERROR", "test message").with_hint("test hint");

        assert_eq!(error.code, "TEST_ERROR");
        assert_eq!(error.message, "test message");
        assert_eq!(error.hint, Some("test hint".into()));
    }

    #[test]
    fn test_should_omit_pagination_key_for_non_list_output() {
        let output = CliOutput::success("payload", "github", "issue view");
        let json = serde_json::to_value(&output).expect("serialize");
        assert!(
            json.get("pagination").is_none(),
            "非列表命令的输出必须与改动前逐字节相同，不得出现 pagination 键"
        );
    }

    #[test]
    fn test_should_keep_data_as_array_for_paged_output() {
        let paged = crate::paging::Paged {
            items: vec![1_u32, 2, 3],
            truncated: false,
            limit: 1000,
        };
        let (items, meta) = paged.into_parts();
        let output = CliOutput::success_paged(items, meta, "github", "issue list");
        let json = serde_json::to_value(&output).expect("serialize");
        assert!(
            json["data"].is_array(),
            "data 必须仍是数组，不得被包装成 {{items: [..]}}"
        );
        assert_eq!(json["data"].as_array().map(Vec::len), Some(3));
    }

    #[test]
    fn test_should_emit_pagination_meta_in_camel_case() {
        let paged = crate::paging::Paged {
            items: vec![1_u32],
            truncated: true,
            limit: 1,
        };
        let (items, meta) = paged.into_parts();
        let output = CliOutput::success_paged(items, meta, "github", "issue list");
        let json = serde_json::to_value(&output).expect("serialize");
        assert_eq!(json["pagination"]["truncated"], serde_json::json!(true));
        assert_eq!(json["pagination"]["returned"], serde_json::json!(1));
        assert_eq!(json["pagination"]["limit"], serde_json::json!(1));
    }

    #[test]
    fn test_should_derive_returned_from_item_count() {
        let paged = crate::paging::Paged {
            items: vec!["a", "b"],
            truncated: false,
            limit: 1000,
        };
        let (_, meta) = paged.into_parts();
        assert_eq!(meta.returned, 2);
        assert!(!meta.truncated);
        assert_eq!(meta.limit, 1000);
    }
}
