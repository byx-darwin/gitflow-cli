//! GitHub CLI 错误解析。

use gitflow_core::{PlatformCliError, platform::Platform};

/// 解析 `gh` CLI 的 stderr 输出为统一的 [`PlatformCliError`]。
///
/// 优先尝试 JSON 格式解析（`gh` 在 API 错误时输出 JSON），
/// 回退到纯文本模式（取前三行作为内部详情）。
/// 用户可见消息为中文。
#[must_use]
pub fn parse_gh_error(stderr: &[u8]) -> PlatformCliError {
    let text = String::from_utf8_lossy(stderr);

    // 尝试解析 gh 的 JSON 错误格式
    if let Ok(json) = serde_json::from_slice::<serde_json::Value>(stderr)
        && let Some(msg) = json.get("message").and_then(serde_json::Value::as_str)
    {
        let code = json
            .get("code")
            .and_then(serde_json::Value::as_str)
            .map(String::from);

        let user_message: String = match code.as_deref() {
            Some("NOT_FOUND") => "资源不存在".into(),
            Some("FORBIDDEN") => "权限不足".into(),
            _ => format!("GitHub 操作失败：{msg}"),
        };

        let mut err = PlatformCliError::new(user_message, text.into_owned(), Platform::GitHub);
        err.hint = Some("运行 `gh auth status` 检查认证状态".into());
        err.doc_link = Some("https://cli.github.com/manual/".into());
        err.code = code;
        return err;
    }

    // 回退：纯文本解析
    let user_message: String = if text.contains("Not logged in") || text.contains("auth") {
        "未登录 GitHub".into()
    } else {
        "GitHub CLI 执行失败".into()
    };

    let mut err = PlatformCliError::new(user_message, text.into_owned(), Platform::GitHub);
    err.hint = Some("运行 `gh auth login` 完成登录".into());
    err.doc_link = Some("https://cli.github.com/manual/".into());
    err
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_parse_gh_json_error_to_platform_cli_error() {
        let json = br#"{"message": "GraphQL: Could not resolve to a user with the login 'nobody'.", "code": "NOT_FOUND"}"#;
        let err = parse_gh_error(json);
        assert_eq!(err.code.as_deref(), Some("NOT_FOUND"));
        assert_eq!(err.platform, Platform::GitHub);
        assert!(!err.user_message.is_empty());
        assert!(!err.raw_stderr.is_empty());
    }

    #[test]
    fn test_should_parse_gh_plain_text_error() {
        let stderr = b"gh: Not logged in. Please run `gh auth login` to authenticate.";
        let err = parse_gh_error(stderr);
        assert!(err.user_message.contains("认证") || err.user_message.contains("登录"));
        assert!(err.hint.is_some());
        assert_eq!(err.platform, Platform::GitHub);
        assert!(err.raw_stderr.contains("Not logged in"));
    }

    #[test]
    fn test_should_not_leak_raw_stderr_in_display() {
        let stderr = b"internal gh debug trace line";
        let err = parse_gh_error(stderr);
        let display = err.to_string();
        assert!(!display.contains("internal gh debug trace"));
    }

    #[test]
    fn test_should_handle_empty_stderr() {
        let err = parse_gh_error(b"");
        assert!(!err.user_message.is_empty());
        assert!(err.hint.is_some());
    }
}
