//! `gh` CLI 错误解析模块。
//!
//! 提供 [`parse_gh_error`] 函数，用于将 `gh` CLI 的 stderr 输出
//! 解析为结构化的 [`GhError`]。优先尝试 JSON 解析 gh 的标准错误格式，
//! 失败时回退到文本前三行。

use std::fmt;

/// `gh` CLI 错误。
///
/// 由 [`parse_gh_error`] 生成，包含从 stderr 解析出的错误信息。
/// 当 gh 返回非零退出码时，其 stderr 可能为 JSON 格式或纯文本，
/// 本结构体统一封装这两种情况。
#[derive(Debug, Clone)]
pub struct GhError {
    /// 错误主信息。
    pub message: String,
    /// 可选的错误代码（仅当 gh 输出为 JSON 且包含 `code` 字段时存在）。
    pub code: Option<String>,
    /// 可选的修复提示。
    pub hint: Option<String>,
}

impl fmt::Display for GhError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "gh: {}", self.message)?;
        if let Some(ref code) = self.code {
            write!(f, " [{code}]")?;
        }
        if let Some(ref hint) = self.hint {
            write!(f, "\nHint: {hint}")?;
        }
        Ok(())
    }
}

/// 解析 `gh` CLI 的 stderr 输出为结构化错误。
///
/// 解析策略：
/// 1. 优先尝试将 stderr 解析为 JSON，提取 `message` 与 `code` 字段。
/// 2. 若 JSON 解析失败或不包含 `message`，则回退为取 stderr 文本的前三行。
///
/// # Examples
///
/// ```
/// use gitflow_cli_github::error::parse_gh_error;
///
/// let stderr = b"gh: Not logged in";
/// let err = parse_gh_error(stderr);
/// assert_eq!(err.message, "gh: Not logged in");
/// assert!(err.hint.is_some());
/// ```
#[must_use]
pub fn parse_gh_error(stderr: &[u8]) -> GhError {
    let text = String::from_utf8_lossy(stderr);

    // 尝试解析 gh 的 JSON 错误格式
    if let Ok(json) = serde_json::from_slice::<serde_json::Value>(stderr) {
        if let Some(msg) = json.get("message").and_then(serde_json::Value::as_str) {
            return GhError {
                message: msg.into(),
                code: json
                    .get("code")
                    .and_then(serde_json::Value::as_str)
                    .map(String::from),
                hint: None,
            };
        }
    }

    // 回退：取 stderr 文本的前三行
    let message = text.lines().take(3).collect::<Vec<_>>().join("\n");
    GhError {
        message,
        code: None,
        hint: Some("Run 'gh auth status' to verify authentication.".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_parse_gh_error_from_json_stderr() {
        let json = br#"{"message": "GraphQL: Could not resolve to a user with the login 'nobody'.", "code": "NOT_FOUND"}"#;
        let err = parse_gh_error(json);

        assert_eq!(
            err.message,
            "GraphQL: Could not resolve to a user with the login 'nobody'."
        );
        assert_eq!(err.code.as_deref(), Some("NOT_FOUND"));
        assert!(err.hint.is_none());
    }

    #[test]
    fn test_should_parse_gh_error_from_plain_text_stderr() {
        let stderr = b"gh: Not logged in. Please run `gh auth login` to authenticate.\nSecond line.\nThird line.\nFourth line should be dropped.";
        let err = parse_gh_error(stderr);

        assert_eq!(
            err.message,
            "gh: Not logged in. Please run `gh auth login` to authenticate.\nSecond line.\nThird \
             line."
        );
        assert!(err.code.is_none());
        assert_eq!(
            err.hint.as_deref(),
            Some("Run 'gh auth status' to verify authentication.")
        );
    }

    #[test]
    fn test_should_display_gh_error_with_code() {
        let err = GhError {
            message: "not found".into(),
            code: Some("NOT_FOUND".into()),
            hint: None,
        };
        assert_eq!(format!("{err}"), "gh: not found [NOT_FOUND]");
    }

    #[test]
    fn test_should_display_gh_error_with_hint() {
        let err = GhError {
            message: "auth failed".into(),
            code: None,
            hint: Some("run gh auth login".into()),
        };
        let display = format!("{err}");
        assert!(display.contains("gh: auth failed"));
        assert!(display.contains("Hint: run gh auth login"));
    }

    #[test]
    fn test_should_fallback_when_json_has_no_message() {
        let json = br#"{"error": "something else"}"#;
        let err = parse_gh_error(json);

        // 无 `message` 字段 → 回退到文本解析
        assert_eq!(err.message, r#"{"error": "something else"}"#);
        assert!(err.code.is_none());
        assert!(err.hint.is_some());
    }

    #[test]
    fn test_should_handle_empty_stderr() {
        let err = parse_gh_error(b"");
        assert!(err.message.is_empty());
        assert!(err.code.is_none());
        assert!(err.hint.is_some());
    }
}
