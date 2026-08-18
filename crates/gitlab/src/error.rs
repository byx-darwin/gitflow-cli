//! GitLab CLI 错误解析。

use gitflow_core::{PlatformCliError, platform::Platform};

/// 解析 `glab` CLI 的 stderr 输出为统一的 [`PlatformCliError`]。
///
/// 优先尝试 JSON 格式解析（`glab` 在 API 错误时输出 JSON），
/// 回退到纯文本模式。
/// 用户可见消息为中文。
#[must_use]
pub fn parse_glab_error(stderr: &[u8]) -> PlatformCliError {
    let text = String::from_utf8_lossy(stderr);

    let is_auth_failure = |t: &str| {
        let lower = t.to_ascii_lowercase();
        lower.contains("not authenticated")
            || lower.contains("unauthorized")
            || lower.contains("401")
            || lower.contains("token")
    };

    // 尝试解析 glab 的 JSON 错误格式
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
            _ if is_auth_failure(&text) => "未登录 GitLab".into(),
            _ => format!("GitLab 操作失败：{msg}"),
        };

        let hint = match code.as_deref() {
            Some("UNAUTHORIZED") => Some("运行 `glab auth login` 完成登录".into()),
            Some("RATE_LIMITED") => Some("等待几分钟后重试".into()),
            Some("VALIDATION_FAILED") => Some("检查请求参数格式是否正确".into()),
            Some("CONFLICT") => Some("运行 `git pull --rebase` 解决冲突后重试".into()),
            Some("NOT_FOUND") => Some("检查资源编号或项目路径是否正确".into()),
            Some("FORBIDDEN") => Some("检查当前账号对该资源的权限".into()),
            _ if is_auth_failure(&text) => Some("运行 `glab auth login` 完成登录".into()),
            _ => None,
        };
        let mut err = PlatformCliError::new(user_message, text.into_owned(), Platform::GitLab);
        err.hint = hint;
        err.doc_link = Some("https://gitlab.com/gitlab-org/cli/-/blob/main/docs/".into());
        err.code = code;
        return err;
    }

    // 回退：纯文本解析
    let is_auth = is_auth_failure(&text);
    let user_message: String = if is_auth {
        "未登录 GitLab".into()
    } else {
        "GitLab CLI 执行失败".into()
    };

    let mut err = PlatformCliError::new(user_message, text.into_owned(), Platform::GitLab);
    if is_auth {
        err.hint = Some("运行 `glab auth login` 完成登录".into());
    }
    err.doc_link = Some("https://gitlab.com/gitlab-org/cli/-/blob/main/docs/".into());
    err
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_parse_glab_json_error_to_platform_cli_error() {
        let json = br#"{"message": "404 Not Found", "code": "NOT_FOUND"}"#;
        let err = parse_glab_error(json);
        assert_eq!(err.code.as_deref(), Some("NOT_FOUND"));
        assert_eq!(err.platform, Platform::GitLab);
        assert!(!err.user_message.is_empty());
        assert!(!err.raw_stderr.is_empty());
    }

    #[test]
    fn test_should_parse_glab_plain_text_error() {
        let stderr = b"ERROR: not authenticated";
        let err = parse_glab_error(stderr);
        assert!(err.hint.is_some());
        assert_eq!(err.platform, Platform::GitLab);
        assert!(err.raw_stderr.contains("not authenticated"));
    }

    #[test]
    fn test_should_not_leak_raw_stderr_in_display() {
        let stderr = b"glab internal trace";
        let err = parse_glab_error(stderr);
        assert!(!err.to_string().contains("glab internal trace"));
    }

    #[test]
    fn test_should_handle_empty_stderr() {
        let err = parse_glab_error(b"");
        assert!(!err.user_message.is_empty());
        assert_eq!(err.user_message, "GitLab CLI 执行失败");
        assert!(err.hint.is_none());
    }

    #[test]
    fn test_should_parse_glab_forbidden_error() {
        let json = br#"{"message": "Forbidden", "code": "FORBIDDEN"}"#;
        let err = parse_glab_error(json);
        assert_eq!(err.code.as_deref(), Some("FORBIDDEN"));
        assert_eq!(err.platform, Platform::GitLab);
        assert!(err.user_message.contains("权限"));
    }

    #[test]
    fn test_should_parse_glab_json_error_without_code() {
        let json = br#"{"message": "Internal error"}"#;
        let err = parse_glab_error(json);
        assert!(err.code.is_none());
        assert!(err.user_message.contains("Internal error"));
        assert_eq!(err.platform, Platform::GitLab);
    }

    #[test]
    fn test_should_parse_glab_plain_text_auth_error() {
        let stderr = b"ERROR: not authenticated";
        let err = parse_glab_error(stderr);
        assert!(err.user_message.contains("登录"));
        assert!(err.hint.as_deref().unwrap_or("").contains("glab auth login"));
        assert_eq!(err.platform, Platform::GitLab);
    }

    #[test]
    fn test_should_parse_glab_rate_limited_error() {
        let json = br#"{"message": "Rate limit exceeded", "code": "RATE_LIMITED"}"#;
        let err = parse_glab_error(json);
        assert_eq!(err.code.as_deref(), Some("RATE_LIMITED"));
        assert!(err.user_message.contains("频率"));
    }

    #[test]
    fn test_should_parse_glab_validation_failed_error() {
        let json = br#"{"message": "Validation failed", "code": "VALIDATION_FAILED"}"#;
        let err = parse_glab_error(json);
        assert_eq!(err.code.as_deref(), Some("VALIDATION_FAILED"));
        assert!(err.hint.is_some());
    }

    #[test]
    fn test_should_parse_glab_conflict_error() {
        let json = br#"{"message": "Conflict detected", "code": "CONFLICT"}"#;
        let err = parse_glab_error(json);
        assert_eq!(err.code.as_deref(), Some("CONFLICT"));
        assert!(err.user_message.contains("冲突"));
    }

    #[test]
    fn test_should_not_hint_auth_login_on_unknown_flag_error() {
        let err = parse_glab_error(b"ERROR: Unknown flag: --output");
        assert!(!err.hint.as_deref().unwrap_or("").contains("glab auth login"));
        assert!(
            err.user_message.contains("执行失败") || !err.user_message.contains("未登录")
        );
    }

    #[test]
    fn test_should_hint_auth_login_on_not_authenticated_error() {
        let err = parse_glab_error(b"ERROR: not authenticated");
        assert!(err.hint.as_deref().unwrap_or("").contains("glab auth login"));
    }

    #[test]
    fn test_should_not_hint_auth_login_on_not_found_json_error() {
        let err = parse_glab_error(br#"{"message": "404 Not Found", "code": "NOT_FOUND"}"#);
        assert!(!err.hint.as_deref().unwrap_or("").contains("glab auth"));
    }
}
