//! GitCode CLI 错误解析。

use gitflow_cli_core::{PlatformCliError, platform::Platform};

/// 解析 `gitcode` CLI 的 stderr 输出为统一的 [`PlatformCliError`]。
///
/// 优先尝试 JSON 格式解析（`gitcode` 在 API 错误时输出 JSON），
/// 回退到纯文本模式。
/// 用户可见消息为中文。
#[must_use]
pub fn parse_gitcode_error(stderr: &[u8]) -> PlatformCliError {
    let text = String::from_utf8_lossy(stderr);

    // 尝试解析 gitcode 的 JSON 错误格式
    if let Ok(json) = serde_json::from_slice::<serde_json::Value>(stderr)
        && let Some(msg) = json.get("message").and_then(serde_json::Value::as_str)
    {
        let code = json
            .get("code")
            .and_then(serde_json::Value::as_str)
            .map(String::from);

        let user_message: String = match code.as_deref() {
            Some("UNAUTHORIZED" | "FORBIDDEN") => "认证失败或权限不足".into(),
            Some("NOT_FOUND") => "资源不存在".into(),
            _ => format!("GitCode 操作失败：{msg}"),
        };

        let mut err = PlatformCliError::new(user_message, text.into_owned(), Platform::GitCode);
        err.hint = Some("运行 `gitcode auth status` 检查认证状态".into());
        err.doc_link = Some("https://gitcode.com/gitcode-cli/cli/blob/main/README.md".into());
        err.code = code;
        return err;
    }

    // 回退：纯文本解析
    let user_message: String = if text.contains("auth") || text.contains("login") {
        "未登录 GitCode".into()
    } else {
        "GitCode CLI 执行失败".into()
    };

    let mut err = PlatformCliError::new(user_message, text.into_owned(), Platform::GitCode);
    err.hint = Some("运行 `gitcode auth login` 完成登录".into());
    err.doc_link = Some("https://gitcode.com/gitcode-cli/cli/blob/main/README.md".into());
    err
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_parse_gitcode_json_error() {
        let json = br#"{"message": "Unauthorized", "code": "UNAUTHORIZED"}"#;
        let err = parse_gitcode_error(json);
        assert_eq!(err.code.as_deref(), Some("UNAUTHORIZED"));
        assert_eq!(err.platform, Platform::GitCode);
        assert!(err.user_message.contains("认证") || err.user_message.contains("权限"));
    }

    #[test]
    fn test_should_parse_gitcode_plain_text_error() {
        let stderr = b"Error: authentication required";
        let err = parse_gitcode_error(stderr);
        assert!(err.hint.is_some());
        assert_eq!(err.platform, Platform::GitCode);
    }

    #[test]
    fn test_should_not_leak_raw_stderr() {
        let stderr = b"gitcode internal panic trace";
        let err = parse_gitcode_error(stderr);
        assert!(!err.to_string().contains("internal panic"));
    }

    #[test]
    fn test_should_handle_empty_stderr() {
        let err = parse_gitcode_error(b"");
        assert!(!err.user_message.is_empty());
        assert!(err.hint.is_some());
    }
}
