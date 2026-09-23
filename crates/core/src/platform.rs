//! Platform detection for remote Git repositories.
//!
//! Identifies the hosting platform (GitHub, GitLab, or `GitCode`) from a
//! remote URL so that the CLI can select the correct API and workflow.
//!
//! This module is URL-detection logic only — it does **not** define a
//! unified "Platform trait". The actual per-platform capability
//! abstraction is the set of fine-grained provider traits (`IssueProvider`,
//! `PrProvider`, `LabelProvider`, `ReleaseProvider`, `PipelineProvider`,
//! `CommitProvider`, `ReviewProvider`, `AuthProvider`, `AuthChecker`,
//! `HealthCheck`, `MilestoneProvider`), each defined in its own module and
//! implemented per platform in the adapter crates.

/// Result of detecting a platform from a remote URL.
///
/// Carries both the detected [`Platform`] and whether the match was
/// explicit (the URL contained a known platform domain) or a fallback
/// default. Callers can use [`is_explicit`](Self::is_explicit) to decide
/// whether to warn the user about an uncertain detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlatformDetection {
    /// The detected or default platform.
    pub platform: Platform,
    /// `true` when the URL matched a known platform domain pattern;
    /// `false` when the result is a fallback default for an unrecognized
    /// domain.
    pub explicit: bool,
}

impl PlatformDetection {
    /// Returns `true` if the URL explicitly matched a known platform domain.
    #[inline]
    #[must_use]
    pub fn is_explicit(self) -> bool {
        self.explicit
    }
}

/// The Git platform hosting a remote repository.
///
/// Used to determine which API client and workflow to use for
/// platform-specific operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Platform {
    /// GitHub (github.com or self-hosted GitHub Enterprise).
    GitHub,
    /// GitLab (gitlab.com or self-hosted GitLab).
    GitLab,
    /// `GitCode` (gitcode.com or self-hosted `GitCode`).
    GitCode,
}

impl Platform {
    /// Detects the platform from a remote Git URL.
    ///
    /// Returns a [`PlatformDetection`] containing the platform and whether
    /// the URL explicitly matched a known domain pattern. Unrecognized
    /// domains default to GitLab with `explicit: false`, so callers can
    /// warn the user.
    ///
    /// # Detection Strategy
    ///
    /// - `github.com` or `github.*` → GitHub (explicit)
    /// - `gitcode.com` or `gitcode.*` → `GitCode` (explicit)
    /// - `gitlab.com` or `gitlab.*` → GitLab (explicit)
    /// - All other domains → GitLab (fallback, `explicit: false`)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use gitflow_core::platform::{Platform, PlatformDetection};
    ///
    /// let result = Platform::detect_from_remote_url("https://github.com/owner/repo.git");
    /// assert_eq!(result.platform, Platform::GitHub);
    /// assert!(result.is_explicit());
    ///
    /// let result = Platform::detect_from_remote_url("https://example.com/repo.git");
    /// assert_eq!(result.platform, Platform::GitLab);
    /// assert!(!result.is_explicit());
    /// ```
    #[must_use]
    pub fn detect_from_remote_url(url: &str) -> PlatformDetection {
        let url_lower = url.to_lowercase();
        if url_lower.contains("github.com") || url_lower.contains("github.") {
            PlatformDetection {
                platform: Self::GitHub,
                explicit: true,
            }
        } else if url_lower.contains("gitcode.com") || url_lower.contains("gitcode.") {
            PlatformDetection {
                platform: Self::GitCode,
                explicit: true,
            }
        } else if url_lower.contains("gitlab.com") || url_lower.contains("gitlab.") {
            PlatformDetection {
                platform: Self::GitLab,
                explicit: true,
            }
        } else {
            PlatformDetection {
                platform: Self::GitLab,
                explicit: false,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_detect_github_from_https_url() {
        let result = Platform::detect_from_remote_url("https://github.com/owner/repo.git");
        assert_eq!(result.platform, Platform::GitHub);
        assert!(result.is_explicit());
    }

    #[test]
    fn test_should_detect_github_from_ssh_url() {
        let result = Platform::detect_from_remote_url("git@github.com:owner/repo.git");
        assert_eq!(result.platform, Platform::GitHub);
        assert!(result.is_explicit());
    }

    #[test]
    fn test_should_detect_gitlab_from_https_url() {
        let result = Platform::detect_from_remote_url("https://gitlab.com/group/project.git");
        assert_eq!(result.platform, Platform::GitLab);
        assert!(result.is_explicit());
    }

    #[test]
    fn test_should_detect_gitlab_from_self_hosted_url() {
        let result = Platform::detect_from_remote_url("git@gitlab.mycorp.com:group/project.git");
        assert_eq!(result.platform, Platform::GitLab);
        assert!(result.is_explicit());
    }

    #[test]
    fn test_should_detect_gitcode() {
        let result = Platform::detect_from_remote_url("https://gitcode.com/owner/repo.git");
        assert_eq!(result.platform, Platform::GitCode);
        assert!(result.is_explicit());
    }

    #[test]
    fn test_should_detect_gitlab_from_custom_domain() {
        let result =
            Platform::detect_from_remote_url("git@xyun.git.nyuncloud.com:fusion-cdn/bff/admin.git");
        assert_eq!(result.platform, Platform::GitLab);
        assert!(!result.is_explicit());

        let result =
            Platform::detect_from_remote_url("https://gitlab.mycorp.com/group/project.git");
        assert_eq!(result.platform, Platform::GitLab);
        assert!(result.is_explicit());
    }

    #[test]
    fn test_should_fallback_to_gitlab_for_unrecognized_url() {
        let result = Platform::detect_from_remote_url("https://example.com/repo.git");
        assert_eq!(result.platform, Platform::GitLab);
        assert!(!result.is_explicit());
    }

    #[test]
    fn test_should_be_case_insensitive() {
        let result = Platform::detect_from_remote_url("HTTPS://GITHUB.COM/Owner/Repo.git");
        assert_eq!(result.platform, Platform::GitHub);
        assert!(result.is_explicit());

        let result = Platform::detect_from_remote_url("GIT@GITLAB.COM:Group/Project.git");
        assert_eq!(result.platform, Platform::GitLab);
        assert!(result.is_explicit());

        let result = Platform::detect_from_remote_url("HTTPS://GITCODE.COM/Owner/Repo.git");
        assert_eq!(result.platform, Platform::GitCode);
        assert!(result.is_explicit());
    }
}
