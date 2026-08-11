//! GitCode CLI 错误解析。

use gitflow_core::{PlatformCliError, platform::Platform};

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
            Some("PR_DISABLED") => "此仓库未启用拉取请求".into(),
            Some("BRANCH_PROTECTED") => "目标分支受保护，无法直接推送".into(),
            Some("RATE_LIMITED") => "API 请求频率超限".into(),
            Some("VALIDATION_FAILED") => "请求参数校验失败".into(),
            Some("CONFLICT") => "存在冲突，请先合并最新变更".into(),
            _ => format!("GitCode 操作失败：{msg}"),
        };

        let hint = match code.as_deref() {
            Some("PR_DISABLED") => Some("在仓库设置中启用拉取请求功能".into()),
            Some("BRANCH_PROTECTED") => Some("检查分支保护规则，或使用有权限的分支".into()),
            Some("RATE_LIMITED") => Some("等待几分钟后重试".into()),
            Some("VALIDATION_FAILED") => Some("检查请求参数格式是否正确".into()),
            Some("CONFLICT") => Some("运行 `git pull --rebase` 解决冲突后重试".into()),
            _ => Some("运行 `gc auth status` 检查认证状态".into()),
        };
        let mut err = PlatformCliError::new(user_message, text.into_owned(), Platform::GitCode);
        err.hint = hint;
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
    err.hint = Some("运行 `gc auth login` 完成登录".into());
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

    #[test]
    fn test_should_parse_gitcode_not_found_error() {
        let json = br#"{"message": "Resource not found", "code": "NOT_FOUND"}"#;
        let err = parse_gitcode_error(json);
        assert_eq!(err.code.as_deref(), Some("NOT_FOUND"));
        assert!(err.user_message.contains("资源不存在"));
    }

    #[test]
    fn test_should_parse_gitcode_forbidden_error() {
        let json = br#"{"message": "Forbidden", "code": "FORBIDDEN"}"#;
        let err = parse_gitcode_error(json);
        assert_eq!(err.code.as_deref(), Some("FORBIDDEN"));
        assert!(err.user_message.contains("认证") || err.user_message.contains("权限"));
    }

    #[test]
    fn test_should_parse_gitcode_plain_text_login_error() {
        let stderr = b"Error: login required";
        let err = parse_gitcode_error(stderr);
        assert!(err.user_message.contains("登录"));
        assert_eq!(err.platform, Platform::GitCode);
    }

    #[test]
    fn test_should_parse_gitcode_json_error_without_code() {
        let json = br#"{"message": "Internal server error"}"#;
        let err = parse_gitcode_error(json);
        assert!(err.code.is_none());
        assert!(err.user_message.contains("Internal server error"));
        assert_eq!(err.platform, Platform::GitCode);
    }

    #[test]
    fn test_should_parse_gitcode_pr_disabled_error() {
        let json = br#"{"message": "Pull requests are disabled", "code": "PR_DISABLED"}"#;
        let err = parse_gitcode_error(json);
        assert_eq!(err.code.as_deref(), Some("PR_DISABLED"));
        assert!(err.user_message.contains("拉取请求"));
        assert!(err.hint.is_some());
    }

    #[test]
    fn test_should_parse_gitcode_branch_protected_error() {
        let json = br#"{"message": "Branch is protected", "code": "BRANCH_PROTECTED"}"#;
        let err = parse_gitcode_error(json);
        assert_eq!(err.code.as_deref(), Some("BRANCH_PROTECTED"));
        assert!(err.hint.is_some());
    }

    #[test]
    fn test_should_parse_gitcode_rate_limited_error() {
        let json = br#"{"message": "API rate limit exceeded", "code": "RATE_LIMITED"}"#;
        let err = parse_gitcode_error(json);
        assert_eq!(err.code.as_deref(), Some("RATE_LIMITED"));
        assert!(
            err.hint
                .as_ref()
                .is_some_and(|h| h.contains("等待") || h.contains("重试"))
        );
    }

    #[test]
    fn test_should_parse_gitcode_validation_error() {
        let json = br#"{"message": "Validation failed", "code": "VALIDATION_FAILED"}"#;
        let err = parse_gitcode_error(json);
        assert_eq!(err.code.as_deref(), Some("VALIDATION_FAILED"));
        assert!(err.hint.is_some());
    }

    #[test]
    fn test_should_parse_gitcode_conflict_error() {
        let json = br#"{"message": "Merge conflict", "code": "CONFLICT"}"#;
        let err = parse_gitcode_error(json);
        assert_eq!(err.code.as_deref(), Some("CONFLICT"));
        assert!(err.user_message.contains("冲突"));
        assert!(err.hint.is_some());
    }
}
