//! Shared session context for adapter operations.
//!
//! The [`Session`] struct holds state that can be reused across multiple
//! provider operations, such as repository information and cached auth tokens.
//! This is particularly useful for workflow-chain commands that perform
//! multiple operations in sequence.
//!
//! # Example
//!
//! ```
//! use gitflow_core::Session;
//!
//! let session = Session::new("owner/repo", "github");
//! assert_eq!(session.repo, "owner/repo");
//! ```

/// Shared session context for adapter operations.
///
/// Holds repository information and optional cached auth tokens to avoid
/// redundant subprocess calls when performing multiple operations in sequence.
///
/// # Fields
///
/// - `repo`: Target repository in `"owner/repo"` format.
/// - `platform`: Platform identifier (`"github"`, `"gitlab"`, or `"gitcode"`).
/// - `cached_token`: Optional cached auth token. Currently unused but reserved for future
///   token-caching optimization when workflow-chain commands exist.
#[derive(Debug, Clone)]
pub struct Session {
    /// Target repository in `"owner/repo"` format.
    pub repo: String,
    /// Platform identifier (`"github"`, `"gitlab"`, or `"gitcode"`).
    pub platform: String,
    /// Cached auth token (reserved for future use).
    ///
    /// True caching requires interior mutability (`Mutex<String>` or
    /// `RefCell<Option<String>>`) because provider methods take `&self`.
    /// For now, this field documents the intent and can be populated
    /// when a workflow-chain command demonstrates the performance benefit.
    #[allow(dead_code, reason = "Reserved for future token-caching optimization")]
    pub(crate) cached_token: Option<String>,
}

impl Session {
    /// Create a new session with the given repository and platform.
    ///
    /// # Arguments
    ///
    /// * `repo` — Repository in `"owner/repo"` format.
    /// * `platform` — Platform identifier: `"github"`, `"gitlab"`, or `"gitcode"`.
    ///
    /// # Examples
    ///
    /// ```
    /// use gitflow_core::Session;
    /// let session = Session::new("byx-darwin/gitflow-cli", "github");
    /// assert_eq!(session.repo, "byx-darwin/gitflow-cli");
    /// assert_eq!(session.platform, "github");
    /// ```
    #[must_use]
    pub fn new(repo: impl Into<String>, platform: impl Into<String>) -> Self {
        Self {
            repo: repo.into(),
            platform: platform.into(),
            cached_token: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_create_session_with_repo_and_platform() {
        let session = Session::new("owner/repo", "github");
        assert_eq!(session.repo, "owner/repo");
        assert_eq!(session.platform, "github");
        assert!(session.cached_token.is_none());
    }

    #[test]
    fn test_should_clone_session() {
        let session = Session::new("owner/repo", "gitlab");
        let cloned = session.clone();
        assert_eq!(cloned.repo, session.repo);
        assert_eq!(cloned.platform, session.platform);
    }

    #[test]
    fn test_should_debug_format_session() {
        let session = Session::new("test/repo", "gitcode");
        let debug_str = format!("{session:?}");
        assert!(debug_str.contains("test/repo"));
        assert!(debug_str.contains("gitcode"));
    }
}
