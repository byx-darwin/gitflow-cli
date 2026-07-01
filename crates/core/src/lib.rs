//! gitflow-cli core library.
//!
//! Provides domain types, error handling, and core business logic.

#![forbid(unsafe_code)]
// Tests legitimately need to unwrap/expect fixture data and index into
// collections to assert on known shapes; the workspace-wide prohibition
// applies only to production code.
#![cfg_attr(
    test,
    allow(
        clippy::expect_used,
        clippy::unwrap_used,
        clippy::indexing_slicing,
        clippy::useless_format,
        clippy::clone_on_copy,
        reason = "Tests legitimately need to unwrap fixture data and index into known-shape \
                  collections"
    )
)]

pub mod output;

use std::path::{Path, PathBuf};

use thiserror::Error;

/// Application error type.
///
/// Library functions return this error type. Use `thiserror` for
/// derive macros and proper error context.
#[allow(
    clippy::module_name_repetitions,
    reason = "Avoids collision with std::error::Error and thiserror::Error"
)]
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum CoreError {
    /// An I/O operation failed.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// A serialization or deserialization failure.
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// An application-level error for cases that don't warrant a dedicated variant.
    /// This variant carries no stability guarantee and should not be matched on programmatically.
    #[error("application error: {0}")]
    App(String),

    /// A path traversal or path validation failure.
    #[error("path error: {0}")]
    Path(#[from] PathError),

    /// A platform operation error (CLI execution failure, parse error, auth failure, etc.).
    #[error("platform error: {0}")]
    Platform(String),
}

/// Core result type alias.
pub type Result<T> = std::result::Result<T, CoreError>;

// Re-export output types at the crate root for convenience.
pub use output::{CliError, CliOutput};

/// Example domain struct demonstrating CLAUDE.md conventions.
///
/// Implements `Debug`, uses `typed-builder` style for structs
/// with many fields, and validates input at construction.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Config {
    name: String,
    description: Option<String>,
}

impl Config {
    /// Returns the configuration name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the optional description.
    #[must_use]
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Creates a new `Config` with the given name.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::App`] if `name` is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use gitflow_cli_core::Config;
    /// let config = Config::new("my-app")?;
    /// # Ok::<(), gitflow_cli_core::CoreError>(())
    /// ```
    pub fn new(name: impl Into<String>) -> Result<Self> {
        let name = name.into();
        if name.is_empty() {
            return Err(CoreError::App("config name must not be empty".into()));
        }
        Ok(Self {
            name,
            description: None,
        })
    }

    /// Sets the description and returns the updated config.
    #[must_use]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// A path validation error.
#[derive(Debug, Clone)]
pub struct PathError {
    /// The path that caused the error.
    pub path: PathBuf,
    /// The kind of path error.
    pub kind: PathErrorKind,
}

impl std::error::Error for PathError {}

/// The kind of path validation error.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum PathErrorKind {
    /// The path is absolute.
    Absolute,
    /// The path contains `..` components.
    ParentDirTraversal,
    /// The path contains a null byte.
    NullByte,
    /// The path contains a Windows reserved device name (CON, NUL, etc.).
    DeviceName,
    /// The path contains an invalid character.
    InvalidChar {
        /// The invalid character found.
        ch: char,
    },
    /// The path contains a dangerous Unicode character.
    DangerousChar {
        /// The dangerous character found.
        ch: char,
    },
    /// A path component exceeds the maximum allowed byte length.
    ComponentTooLong {
        /// The maximum allowed bytes per component.
        max_bytes: usize,
        /// The actual byte length of the offending component.
        actual_bytes: usize,
    },
}

impl std::fmt::Display for PathError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            PathErrorKind::Absolute => {
                write!(f, "absolute path not allowed: {}", self.path.display())
            }
            PathErrorKind::ParentDirTraversal => {
                write!(f, "'..' not allowed: {}", self.path.display())
            }
            PathErrorKind::NullByte => write!(f, "null byte in path: {}", self.path.display()),
            PathErrorKind::DeviceName => write!(
                f,
                "reserved device name not allowed: {}",
                self.path.display()
            ),
            PathErrorKind::InvalidChar { ch } => write!(
                f,
                "invalid character '{ch}' in path: {}",
                self.path.display()
            ),
            PathErrorKind::DangerousChar { ch } => write!(
                f,
                "dangerous Unicode character U+{:04X} in path: {}",
                ch as u32,
                self.path.display()
            ),
            PathErrorKind::ComponentTooLong {
                max_bytes,
                actual_bytes,
            } => {
                write!(
                    f,
                    "path component too long (max {max_bytes}B, got {actual_bytes}B): {}",
                    self.path.display()
                )
            }
        }
    }
}

/// A validated, safe filesystem path.
///
/// Rejects `..` components, absolute paths, and null bytes.
/// Use this for all externally-supplied file path arguments.
#[derive(Debug, Clone)]
pub struct SafePath(PathBuf);

impl SafePath {
    /// Creates a new `SafePath` from a relative path string.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Path`] if:
    /// - The path contains `..` components
    /// - The path is absolute
    /// - The path contains null bytes
    /// - The path contains a Windows reserved device name
    /// - The path contains invalid characters (e.g., `:` for ADS)
    /// - The path contains dangerous Unicode bidi control characters
    /// - Any path component exceeds 255 bytes
    ///
    /// # Examples
    ///
    /// ```
    /// use gitflow_cli_core::SafePath;
    /// let path = SafePath::new("foo/bar.txt")?;
    /// assert_eq!(path.as_path(), std::path::Path::new("foo/bar.txt"));
    /// # Ok::<(), gitflow_cli_core::CoreError>(())
    /// ```
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        const MAX_COMPONENT_LEN: usize = 255;
        let path = path.as_ref();

        // Absolute path check
        if path.is_absolute() {
            return Err(PathError {
                path: path.to_path_buf(),
                kind: PathErrorKind::Absolute,
            }
            .into());
        }

        // Parent directory traversal check
        if path
            .components()
            .any(|c| c == std::path::Component::ParentDir)
        {
            return Err(PathError {
                path: path.to_path_buf(),
                kind: PathErrorKind::ParentDirTraversal,
            }
            .into());
        }

        // Null byte check
        if path.as_os_str().as_encoded_bytes().contains(&b'\0') {
            return Err(PathError {
                path: path.to_path_buf(),
                kind: PathErrorKind::NullByte,
            }
            .into());
        }

        // Reject colon in path (Windows Alternate Data Streams)
        if path.as_os_str().as_encoded_bytes().contains(&b':') {
            return Err(PathError {
                path: path.to_path_buf(),
                kind: PathErrorKind::InvalidChar { ch: ':' },
            }
            .into());
        }

        // Reject Windows reserved device names (case-insensitive, any extension)
        #[cfg(windows)]
        {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                let upper = stem.to_uppercase();
                if matches!(upper.as_str(), "CON" | "PRN" | "AUX" | "NUL")
                    || (upper.len() == 4
                        && (upper.starts_with("COM") || upper.starts_with("LPT"))
                        && upper[3..].parse::<u8>().is_ok())
                {
                    return Err(PathError {
                        path: path.to_path_buf(),
                        kind: PathErrorKind::DeviceName,
                    }
                    .into());
                }
            }
        }

        // Reject dangerous Unicode bidi control characters
        for component in path.components() {
            let len = component.as_os_str().len();
            if len > MAX_COMPONENT_LEN {
                return Err(PathError {
                    path: path.to_path_buf(),
                    kind: PathErrorKind::ComponentTooLong {
                        max_bytes: MAX_COMPONENT_LEN,
                        actual_bytes: len,
                    },
                }
                .into());
            }

            let s = component.as_os_str().to_string_lossy();
            for ch in s.chars() {
                if matches!(ch, '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}') {
                    return Err(PathError {
                        path: path.to_path_buf(),
                        kind: PathErrorKind::DangerousChar { ch },
                    }
                    .into());
                }
            }
        }

        Ok(Self(path.to_path_buf()))
    }

    /// Returns the inner path.
    #[must_use]
    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

impl AsRef<Path> for SafePath {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

pub mod issue;
pub mod platform;
pub mod pr;
pub mod types;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_new_with_valid_name() -> Result<()> {
        let config = Config::new("test")?;
        assert_eq!(config.name(), "test");
        assert!(config.description().is_none());
        Ok(())
    }

    #[test]
    fn test_config_new_with_empty_name_errors() {
        let err = Config::new("");
        assert!(matches!(err, Err(CoreError::App(_))));
    }

    #[test]
    fn test_config_with_description() -> Result<()> {
        let config = Config::new("test")?.with_description("a test config");
        assert_eq!(config.description(), Some("a test config"));
        Ok(())
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::other("test");
        let core_err = CoreError::from(io_err);
        assert!(matches!(core_err, CoreError::Io(_)));
    }

    #[test]
    fn test_serialization_error_conversion() {
        let io_err = std::io::Error::other("invalid json");
        let serde_err = serde_json::Error::io(io_err);
        let core_err = CoreError::from(serde_err);
        assert!(matches!(core_err, CoreError::Serialization(_)));
    }

    #[test]
    fn test_safe_path_accepts_relative() -> Result<()> {
        let sp = SafePath::new("foo/bar.txt")?;
        assert_eq!(sp.as_path(), Path::new("foo/bar.txt"));
        Ok(())
    }

    #[test]
    fn test_safe_path_rejects_absolute() {
        assert!(matches!(
            SafePath::new("/etc/passwd"),
            Err(CoreError::Path(_))
        ));
    }

    #[test]
    fn test_safe_path_rejects_parent_dir() {
        assert!(matches!(
            SafePath::new("../secret.txt"),
            Err(CoreError::Path(_))
        ));
    }

    #[test]
    fn test_safe_path_rejects_null_byte() {
        assert!(matches!(
            SafePath::new("foo\0bar.txt"),
            Err(CoreError::Path(_))
        ));
    }

    #[test]
    fn test_safe_path_rejects_colon() {
        let err = SafePath::new("foo:bar.txt");
        assert!(matches!(
            err,
            Err(CoreError::Path(PathError {
                kind: PathErrorKind::InvalidChar { ch: ':' },
                ..
            }))
        ));
    }

    #[test]
    fn test_safe_path_rejects_bidi_control_chars() {
        // U+202A (LEFT-TO-RIGHT EMBEDDING)
        let path = format!("foo\u{202A}bar.txt");
        let err = SafePath::new(path);
        assert!(matches!(
            err,
            Err(CoreError::Path(PathError {
                kind: PathErrorKind::DangerousChar { ch: '\u{202A}' },
                ..
            }))
        ));

        // U+2066 (LEFT-TO-RIGHT ISOLATE)
        let path = format!("foo\u{2066}bar.txt");
        let err = SafePath::new(path);
        assert!(matches!(
            err,
            Err(CoreError::Path(PathError {
                kind: PathErrorKind::DangerousChar { ch: '\u{2066}' },
                ..
            }))
        ));
    }

    #[test]
    fn test_safe_path_rejects_component_too_long() {
        let long_component = "a".repeat(256);
        let err = SafePath::new(&long_component);
        assert!(matches!(
            err,
            Err(CoreError::Path(PathError {
                kind: PathErrorKind::ComponentTooLong {
                    max_bytes: 255,
                    actual_bytes: 256
                },
                ..
            }))
        ));
    }

    #[test]
    fn test_safe_path_accepts_max_component() -> Result<()> {
        let max_component = "a".repeat(255);
        let sp = SafePath::new(&max_component)?;
        assert_eq!(sp.as_path().as_os_str().len(), 255);
        Ok(())
    }

    #[test]
    fn test_platform_error_variant() {
        let err = CoreError::Platform("gh not found".into());
        assert!(matches!(err, CoreError::Platform(_)));
        let msg = format!("{err}");
        assert!(msg.contains("platform error"));
        assert!(msg.contains("gh not found"));
    }
}
