//! 统一的底层平台 CLI 错误类型。

use std::fmt;

use crate::platform::Platform;

/// 统一的底层平台 CLI 错误。
///
/// 各平台 crate 的 `parse_*_error()` 函数返回此类型，
/// 替代原先各自定义的 `GhError`、`GlabError`、`GitcodeError`。
///
/// 用户可见信息（`user_message`、`hint`）为中文主导；
/// `raw_stderr` 仅用于 `tracing::debug!`，不展示给用户。
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct PlatformCliError {
    /// 用户可见的错误消息（中文主导）。
    pub user_message: String,
    /// 底层 CLI 原始 stderr（仅用于调试日志，不展示给用户）。
    pub raw_stderr: String,
    /// 修复建议（中文）。
    pub hint: Option<String>,
    /// 相关文档链接。
    pub doc_link: Option<String>,
    /// 平台错误代码（如 `NOT_FOUND`）。
    pub code: Option<String>,
    /// 来源平台。
    pub platform: Platform,
}

impl PlatformCliError {
    /// 创建一个新的平台 CLI 错误。
    ///
    /// `hint`、`doc_link`、`code` 默认为 `None`，可通过公开字段直接设置。
    #[must_use]
    pub fn new(
        user_message: impl Into<String>,
        raw_stderr: impl Into<String>,
        platform: Platform,
    ) -> Self {
        Self {
            user_message: user_message.into(),
            raw_stderr: raw_stderr.into(),
            hint: None,
            doc_link: None,
            code: None,
            platform,
        }
    }
}

impl fmt::Display for PlatformCliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.user_message)?;
        if let Some(ref hint) = self.hint {
            write!(f, "\n\n🔧 修复建议：{hint}")?;
        }
        if let Some(ref link) = self.doc_link {
            write!(f, "\n📖 文档：{link}")?;
        }
        Ok(())
    }
}

impl std::error::Error for PlatformCliError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_display_user_message_only_when_no_hint_no_link() {
        let err = PlatformCliError {
            user_message: "资源不存在".into(),
            raw_stderr: "gh: NOT_FOUND".into(),
            hint: None,
            doc_link: None,
            code: Some("NOT_FOUND".into()),
            platform: Platform::GitHub,
        };
        assert_eq!(err.to_string(), "资源不存在");
    }

    #[test]
    fn test_should_display_with_hint_and_doc_link() {
        let err = PlatformCliError {
            user_message: "认证失败".into(),
            raw_stderr: "raw error".into(),
            hint: Some("运行 `gh auth login` 重新认证".into()),
            doc_link: Some("https://cli.github.com/manual/".into()),
            code: None,
            platform: Platform::GitHub,
        };
        let display = err.to_string();
        assert!(display.contains("认证失败"));
        assert!(display.contains("🔧 修复建议：运行 `gh auth login` 重新认证"));
        assert!(display.contains("📖 文档：https://cli.github.com/manual/"));
        // raw_stderr must NOT appear in Display
        assert!(!display.contains("raw error"));
    }

    #[test]
    fn test_should_include_raw_stderr_in_debug() {
        let err = PlatformCliError {
            user_message: "错误".into(),
            raw_stderr: "secret debug info".into(),
            hint: None,
            doc_link: None,
            code: None,
            platform: Platform::GitLab,
        };
        let debug = format!("{err:?}");
        assert!(debug.contains("secret debug info"));
    }

    // ── boundary / constructor tests ──

    #[test]
    fn test_should_create_platform_cli_error_with_new() {
        // Arrange & Act
        let err = PlatformCliError::new("错误消息", "raw stderr", Platform::GitHub);

        // Assert
        assert_eq!(err.user_message, "错误消息");
        assert_eq!(err.raw_stderr, "raw stderr");
        assert_eq!(err.platform, Platform::GitHub);
        assert!(err.hint.is_none());
        assert!(err.doc_link.is_none());
        assert!(err.code.is_none());
    }

    #[test]
    fn test_should_handle_empty_strings_in_platform_cli_error() {
        // Arrange & Act
        let err = PlatformCliError::new("", "", Platform::GitLab);

        // Assert
        assert_eq!(err.user_message, "");
        assert_eq!(err.raw_stderr, "");
        assert_eq!(err.to_string(), "");
    }

    #[test]
    fn test_should_set_optional_fields_via_direct_assignment() {
        // Arrange
        let mut err = PlatformCliError::new("错误", "stderr", Platform::GitCode);

        // Act
        err.hint = Some("尝试重新运行".into());
        err.doc_link = Some("https://example.com".into());
        err.code = Some("ERR_001".into());

        // Assert
        assert_eq!(err.hint.as_deref(), Some("尝试重新运行"));
        assert_eq!(err.doc_link.as_deref(), Some("https://example.com"));
        assert_eq!(err.code.as_deref(), Some("ERR_001"));
    }

    use rstest::rstest;

    #[rstest]
    #[case(Platform::GitHub)]
    #[case(Platform::GitLab)]
    #[case(Platform::GitCode)]
    fn test_should_create_error_for_all_platforms(#[case] platform: Platform) {
        // Arrange & Act
        let err = PlatformCliError::new("测试", "stderr", platform);

        // Assert
        assert_eq!(err.platform, platform);
    }
}
