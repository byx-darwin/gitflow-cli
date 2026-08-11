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
            Some("RATE_LIMITED") => "API 请求频率超限".into(),
            Some("VALIDATION_FAILED") => "请求参数校验失败".into(),
            Some("CONFLICT") => "存在冲突，请先合并最新变更".into(),
            Some("GONE") => "资源已被删除或迁移".into(),
            _ => format!("GitHub 操作失败：{msg}"),
        };

        let hint = match code.as_deref() {
            Some("RATE_LIMITED") => Some("等待几分钟后重试".into()),
            Some("VALIDATION_FAILED") => Some("检查请求参数格式是否正确".into()),
            Some("CONFLICT") => Some("运行 `git pull --rebase` 解决冲突后重试".into()),
            Some("GONE") => Some("确认资源是否存在，可能已被删除或重命名".into()),
            _ => Some("运行 `gh auth status` 检查认证状态".into()),
        };
        let mut err = PlatformCliError::new(user_message, text.into_owned(), Platform::GitHub);
        err.hint = hint;
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

    #[test]
    fn test_should_parse_gh_forbidden_error() {
        let json = br#"{"message": "Resource not accessible by integration", "code": "FORBIDDEN"}"#;
        let err = parse_gh_error(json);
        assert_eq!(err.code.as_deref(), Some("FORBIDDEN"));
        assert_eq!(err.platform, Platform::GitHub);
        assert!(err.user_message.contains("权限"));
    }

    #[test]
    fn test_should_parse_gh_json_error_without_code() {
        let json = br#"{"message": "Something went wrong"}"#;
        let err = parse_gh_error(json);
        assert!(err.code.is_none());
        assert!(err.user_message.contains("Something went wrong"));
        assert_eq!(err.platform, Platform::GitHub);
    }

    #[test]
    fn test_should_parse_gh_rate_limited_error() {
        let json = br#"{"message": "API rate limit exceeded", "code": "RATE_LIMITED"}"#;
        let err = parse_gh_error(json);
        assert_eq!(err.code.as_deref(), Some("RATE_LIMITED"));
        assert!(err.user_message.contains("频率"));
        assert!(
            err.hint
                .as_ref()
                .is_some_and(|h| h.contains("等待") || h.contains("重试"))
        );
    }

    #[test]
    fn test_should_parse_gh_validation_failed_error() {
        let json = br#"{"message": "Validation failed", "code": "VALIDATION_FAILED"}"#;
        let err = parse_gh_error(json);
        assert_eq!(err.code.as_deref(), Some("VALIDATION_FAILED"));
        assert!(err.hint.is_some());
    }

    #[test]
    fn test_should_parse_gh_conflict_error() {
        let json = br#"{"message": "Merge conflict", "code": "CONFLICT"}"#;
        let err = parse_gh_error(json);
        assert_eq!(err.code.as_deref(), Some("CONFLICT"));
        assert!(err.user_message.contains("冲突"));
    }

    #[test]
    fn test_should_parse_gh_gone_error() {
        let json = br#"{"message": "Resource gone", "code": "GONE"}"#;
        let err = parse_gh_error(json);
        assert_eq!(err.code.as_deref(), Some("GONE"));
        assert!(err.user_message.contains("删除") || err.user_message.contains("迁移"));
    }
}
