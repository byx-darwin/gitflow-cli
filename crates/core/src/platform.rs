//! Platform detection for remote Git repositories.
//!
//! Identifies the hosting platform (GitHub, GitLab, or `GitCode`) from a
//! remote URL so that the CLI can select the correct API and workflow.

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
    /// Performs a case-insensitive substring match against known platform
    /// domain patterns. Supports both HTTPS and SSH remote formats, as
    /// well as self-hosted instances (e.g., `gitlab.mycorp.com`).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Doctest skipped: crate name hyphen-to-underscore mapping
    /// // prevents `use gitflow_cli_core::Platform` from resolving.
    /// use gitflow_cli_core::Platform;
    ///
    /// assert_eq!(
    ///     Platform::detect_from_remote_url("https://github.com/owner/repo.git"),
    ///     Some(Platform::GitHub),
    /// );
    /// assert_eq!(
    ///     Platform::detect_from_remote_url("git@gitlab.mycorp.com:group/project.git"),
    ///     Some(Platform::GitLab),
    /// );
    /// assert!(Platform::detect_from_remote_url("https://example.com/repo.git").is_none());
    /// ```
    #[must_use]
    pub fn detect_from_remote_url(url: &str) -> Option<Self> {
        let url_lower = url.to_lowercase();
        if url_lower.contains("github.com") || url_lower.contains("github.") {
            Some(Self::GitHub)
        } else if url_lower.contains("gitlab.com") || url_lower.contains("gitlab.") {
            Some(Self::GitLab)
        } else if url_lower.contains("gitcode.com") || url_lower.contains("gitcode.") {
            Some(Self::GitCode)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_detect_github_from_https_url() {
        assert_eq!(
            Platform::detect_from_remote_url("https://github.com/owner/repo.git"),
            Some(Platform::GitHub),
        );
    }

    #[test]
    fn test_should_detect_github_from_ssh_url() {
        assert_eq!(
            Platform::detect_from_remote_url("git@github.com:owner/repo.git"),
            Some(Platform::GitHub),
        );
    }

    #[test]
    fn test_should_detect_gitlab_from_https_url() {
        assert_eq!(
            Platform::detect_from_remote_url("https://gitlab.com/group/project.git"),
            Some(Platform::GitLab),
        );
    }

    #[test]
    fn test_should_detect_gitlab_from_self_hosted_url() {
        assert_eq!(
            Platform::detect_from_remote_url("git@gitlab.mycorp.com:group/project.git"),
            Some(Platform::GitLab),
        );
    }

    #[test]
    fn test_should_detect_gitcode() {
        assert_eq!(
            Platform::detect_from_remote_url("https://gitcode.com/owner/repo.git"),
            Some(Platform::GitCode),
        );
    }

    #[test]
    fn test_should_return_none_for_unrecognized_url() {
        assert!(Platform::detect_from_remote_url("https://example.com/repo.git").is_none());
    }

    #[test]
    fn test_should_be_case_insensitive() {
        assert_eq!(
            Platform::detect_from_remote_url("HTTPS://GITHUB.COM/Owner/Repo.git"),
            Some(Platform::GitHub),
        );
        assert_eq!(
            Platform::detect_from_remote_url("GIT@GITLAB.COM:Group/Project.git"),
            Some(Platform::GitLab),
        );
        assert_eq!(
            Platform::detect_from_remote_url("HTTPS://GITCODE.COM/Owner/Repo.git"),
            Some(Platform::GitCode),
        );
    }
}
