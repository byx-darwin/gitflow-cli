//! Error auto-report module.
//!
//! Writes structured error reports to `.cache/bug-reports/pending.json`
//! when the CLI is running in non-interactive mode (CI or subprocess).
//! The Claude Code Stop Hook (`hooks/auto-report-bug.sh`) picks up the
//! pending file and triggers the `gitflow-autoreport-bug` skill.

// The error reporter is deliberately sync: it is invoked from error
// paths that may execute before the tokio runtime exists (e.g. remote
// URL resolution, runtime construction) or in signal/panic contexts
// where blocking the executor would be unsafe.
#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "Error reporter runs in sync contexts (pre-runtime, signal handlers)"
)]

use std::{
    io::Write as _,
    path::{Path, PathBuf},
};

use serde::Serialize;

/// Error report written to `pending.json`.
///
/// Contains enough context for the `gitflow-autoreport-bug` skill
/// to analyse, deduplicate, and file a GitHub Issue.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct ErrorReport {
    /// Unique identifier (hex-encoded timestamp + PID hash).
    pub id: String,
    /// Error origin — always `"cli"` for reports from this module.
    pub source: String,
    /// Subcommand the user ran (e.g. `"issue create"`).
    pub command: String,
    /// Target platform (`"github"`, `"gitlab"`, or `"gitcode"`).
    pub platform: String,
    /// Process exit code.
    pub exit_code: i32,
    /// Structured error code (e.g. `"CLI_ERROR"`, `"AUTH_FAILED"`).
    pub error_code: String,
    /// Human-readable error message.
    pub error_message: String,
    /// Optional remediation hint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    /// ISO 8601 UTC timestamp of when the error occurred.
    pub timestamp: String,
}

impl ErrorReport {
    /// Build an error report from raw error context.
    ///
    /// The `id` is derived from the current nanosecond timestamp and
    /// process ID for deduplication. The `source` is always `"cli"`.
    pub(crate) fn from_error(
        command: &str,
        platform: &str,
        error_message: &str,
        error_code: &str,
    ) -> Self {
        Self {
            id: generate_unique_id(),
            source: "cli".into(),
            command: command.into(),
            platform: platform.into(),
            exit_code: 1,
            error_code: error_code.into(),
            error_message: error_message.into(),
            hint: None,
            timestamp: iso8601_utc_now(),
        }
    }

    /// Write this report to `<repo_root>/.cache/bug-reports/pending.json`.
    ///
    /// Creates the directory tree if it does not exist. Overwrites any
    /// existing `pending.json` file.
    ///
    /// # Errors
    ///
    /// Returns an I/O error if the directory cannot be created or the
    /// file cannot be written.
    pub(crate) fn write_to_disk(&self, repo_root: &Path) -> std::io::Result<()> {
        let dir = repo_root.join(".cache").join("bug-reports");
        std::fs::create_dir_all(&dir)?;
        let path = dir.join("pending.json");
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let mut file = std::fs::File::create(&path)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }
}

/// Write an error report for the current process if running non-interactively.
///
/// This is the main entry point called from `main.rs`'s error branch.
/// When stderr is attached to a terminal (interactive mode), the function
/// returns `Ok(())` immediately without writing anything — the user can
/// see the error directly.
///
/// In non-interactive mode (CI, piped output, subprocess), the report is
/// written to `<repo_root>/.cache/bug-reports/pending.json` for pickup
/// by the Stop Hook.
///
/// # Errors
///
/// Returns an I/O error if the repo root cannot be located or the
/// pending file cannot be written. Callers should ignore errors
/// (the report is best-effort).
pub(crate) fn maybe_report_error(
    command: &str,
    platform: &str,
    error_message: &str,
    error_code: &str,
) -> std::io::Result<()> {
    if should_skip_reporting() {
        return Ok(());
    }

    // Only report if user has joined the co-contribution plan
    if !is_co_contribution_enabled() {
        return Ok(());
    }

    let report = ErrorReport::from_error(command, platform, error_message, error_code);
    let repo_root = find_repo_root()?;
    report.write_to_disk(&repo_root)
}

/// Check whether the co-contribution plan is enabled.
///
/// Checks two locations in order:
/// 1. `<repo_root>/.claude/settings.json` (project-level)
/// 2. `~/.claude/settings.json` (global, from `-g` install)
///
/// Returns `true` if either location has `gitflow.co_contribution = true`.
/// Returns `false` if neither file exists, or the field is missing/false.
/// Any I/O or parse error silently degrades to `false`.
fn is_co_contribution_enabled() -> bool {
    if let Ok(repo_root) = find_repo_root() {
        let project_settings = repo_root.join(".claude/settings.json");
        if read_co_contribution_flag(&project_settings) {
            return true;
        }
    }

    if let Some(home) = dirs::home_dir() {
        let global_settings = home.join(".claude/settings.json");
        if read_co_contribution_flag(&global_settings) {
            return true;
        }
    }

    false
}

/// Read the `gitflow.co_contribution` flag from a specific settings file.
///
/// Returns `false` if the file doesn't exist, can't be read, or the field
/// is missing/not a boolean.
fn read_co_contribution_flag(path: &Path) -> bool {
    let Ok(content) = std::fs::read_to_string(path) else {
        return false;
    };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) else {
        return false;
    };
    json.pointer("/gitflow/co_contribution")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
}

/// Returns `true` when error reporting should be skipped because
/// stderr is attached to a terminal (interactive mode).
///
/// Extracted for testability — unit tests can assert on the mapping
/// between `is_terminal()` and the skip decision.
fn should_skip_reporting() -> bool {
    use is_terminal::IsTerminal;
    std::io::stderr().is_terminal()
}

/// Generate a unique report identifier from the current nanosecond
/// timestamp and process ID.
///
/// The two values are XOR-mixed with a Fibonacci-hashing constant so
/// that the resulting 128-bit hex string is compact and collision-resistant
/// across rapid successive invocations.
fn generate_unique_id() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let pid = std::process::id();
    // Fibonacci hashing constant for good bit dispersion.
    let mixed = nanos ^ (u128::from(pid) * 0x9E37_79B9_7F4A_7C15);
    format!("{mixed:032x}")
}

/// Find the repository root via `git rev-parse --show-toplevel`.
///
/// # Errors
///
/// Returns an error if the git command fails (not inside a repo)
/// or the output cannot be decoded as a UTF-8 path.
fn find_repo_root() -> std::io::Result<PathBuf> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("git rev-parse --show-toplevel failed: {stderr}"),
        ));
    }
    let path_str = String::from_utf8_lossy(&output.stdout);
    Ok(PathBuf::from(path_str.trim()))
}

/// Format the current UTC time as an ISO 8601 string without
/// requiring the `chrono` crate.
///
/// Delegates to [`unix_secs_to_iso8601`] which is pure and easy to test.
fn iso8601_utc_now() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    unix_secs_to_iso8601(secs)
}

/// Convert a Unix timestamp (seconds since 1970-01-01T00:00:00Z) to
/// ISO 8601 format `YYYY-MM-DDTHH:MM:SSZ`.
///
/// Uses Howard Hinnant's `civil_from_days` algorithm to derive the
/// `(year, month, day)` triple from the day count since the Unix epoch.
/// Reference: <http://howardhinnant.github.io/date_algorithms.html>
#[allow(
    clippy::cast_possible_wrap,
    clippy::cast_lossless,
    clippy::cast_sign_loss,
    reason = "Howard Hinnant's algorithm operates on mixed-sign integer ranges within known bounds"
)]
fn unix_secs_to_iso8601(unix_secs: u64) -> String {
    let day_secs = unix_secs % 86_400;
    let hours = day_secs / 3_600;
    let minutes = (day_secs % 3_600) / 60;
    let seconds = day_secs % 60;

    let days = (unix_secs / 86_400) as i64;
    // Howard Hinnant's civil_from_days algorithm.
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64; // day of era [0, 146096]
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365; // year of era [0, 399]
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // day of year [0, 365]
    let mp = (5 * doy + 2) / 153; // month index [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // day [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // month [1, 12]
    let y = yoe as i64 + era * 400;
    let y = if m <= 2 { y + 1 } else { y };

    format!("{y:04}-{m:02}-{d:02}T{hours:02}:{minutes:02}:{seconds:02}Z")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_create_error_report_from_error() {
        let report =
            ErrorReport::from_error("issue create", "github", "auth failed", "AUTH_FAILED");
        assert_eq!(report.source, "cli");
        assert_eq!(report.command, "issue create");
        assert_eq!(report.platform, "github");
        assert_eq!(report.exit_code, 1);
        assert_eq!(report.error_code, "AUTH_FAILED");
        assert_eq!(report.error_message, "auth failed");
        assert!(report.hint.is_none());
        assert!(!report.id.is_empty());
        assert!(report.timestamp.ends_with('Z'));
        assert!(report.timestamp.contains('T'));
    }

    #[test]
    fn test_should_write_pending_json_to_disk() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let report = ErrorReport::from_error("pr list", "gitlab", "not found", "NOT_FOUND");
        report.write_to_disk(tmp.path()).expect("write_to_disk");

        let pending = tmp.path().join(".cache/bug-reports/pending.json");
        assert!(pending.exists(), "pending.json must be created");

        let contents = std::fs::read_to_string(&pending).expect("read pending.json");
        let parsed: serde_json::Value = serde_json::from_str(&contents).expect("valid JSON");
        assert_eq!(parsed["source"], "cli");
        assert_eq!(parsed["command"], "pr list");
        assert_eq!(parsed["platform"], "gitlab");
        assert_eq!(parsed["error_code"], "NOT_FOUND");
        assert_eq!(parsed["error_message"], "not found");
        assert_eq!(parsed["exit_code"], 1);
        assert!(parsed.get("id").is_some());
        assert!(parsed.get("timestamp").is_some());
    }

    #[test]
    fn test_should_generate_unique_id() {
        let id1 = generate_unique_id();
        // Sleep briefly to ensure the clock advances at least one nanosecond.
        std::thread::sleep(std::time::Duration::from_millis(2));
        let id2 = generate_unique_id();
        assert_ne!(id1, id2, "successive calls must produce distinct IDs");
        // Both IDs should be 32-character hex strings (128-bit).
        assert_eq!(id1.len(), 32);
        assert_eq!(id2.len(), 32);
    }

    #[test]
    fn test_should_skip_when_interactive_terminal() {
        // We cannot force stderr to be a terminal inside `cargo test`,
        // so we verify the `should_skip_reporting` mapping directly.
        //
        // The contract: skip iff stderr is a terminal.
        use is_terminal::IsTerminal;
        let is_tty = std::io::stderr().is_terminal();
        assert_eq!(
            should_skip_reporting(),
            is_tty,
            "should_skip_reporting() must equal stderr.is_terminal()"
        );
        // In `cargo test`, stderr is piped, so `is_tty` is `false`
        // and reporting would proceed. In an interactive shell it
        // would be `true` and reporting would be skipped.
    }

    #[test]
    fn test_should_format_iso8601_unix_epoch() {
        assert_eq!(unix_secs_to_iso8601(0), "1970-01-01T00:00:00Z");
    }

    #[test]
    fn test_should_format_iso8601_known_date() {
        // 2024-01-01T00:00:00Z = 1704067200 seconds since epoch
        assert_eq!(unix_secs_to_iso8601(1_704_067_200), "2024-01-01T00:00:00Z");
    }

    #[test]
    fn test_should_format_iso8601_with_time() {
        // 1970-01-01T12:34:56Z = 45296 seconds
        assert_eq!(unix_secs_to_iso8601(45_296), "1970-01-01T12:34:56Z");
    }

    #[test]
    fn test_should_format_iso8601_day_after_epoch() {
        assert_eq!(unix_secs_to_iso8601(86_400), "1970-01-02T00:00:00Z");
    }

    #[test]
    fn test_should_return_false_for_missing_settings_file() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let missing = tmp.path().join("nonexistent.json");
        assert!(!read_co_contribution_flag(&missing));
    }

    #[test]
    fn test_should_return_false_for_settings_without_gitflow() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("settings.json");
        std::fs::write(&path, r#"{"hooks": {}}"#).expect("write");
        assert!(!read_co_contribution_flag(&path));
    }

    #[test]
    fn test_should_return_false_for_gitflow_without_co_contribution() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("settings.json");
        std::fs::write(&path, r#"{"gitflow": {}}"#).expect("write");
        assert!(!read_co_contribution_flag(&path));
    }

    #[test]
    fn test_should_return_true_for_co_contribution_enabled() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("settings.json");
        std::fs::write(&path, r#"{"gitflow": {"co_contribution": true}}"#).expect("write");
        assert!(read_co_contribution_flag(&path));
    }

    #[test]
    fn test_should_return_false_for_co_contribution_disabled() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("settings.json");
        std::fs::write(&path, r#"{"gitflow": {"co_contribution": false}}"#).expect("write");
        assert!(!read_co_contribution_flag(&path));
    }

    #[test]
    fn test_should_return_false_for_invalid_json() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("settings.json");
        std::fs::write(&path, "not json").expect("write");
        assert!(!read_co_contribution_flag(&path));
    }
}
