//! Error auto-report module.
//!
//! Writes structured error reports to `.cache/bug-reports/pending.json`
//! when the CLI is running in non-interactive mode (CI or subprocess).
//! The Claude Code Stop Hook (`hooks/auto-report-bug.sh`) picks up the
//! pending file and triggers the `gf-autoreport-bug` skill.

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
    sync::LazyLock,
};

use regex::Regex;
use serde::Serialize;

/// Maximum number of archived `pending.*.json` reports kept on disk.
///
/// Older archives beyond this cap are deleted on the next write to bound
/// unbounded growth of `.cache/bug-reports/` (a burst of CLI failures
/// otherwise accumulates one archive per failure forever).
const MAX_ARCHIVED_REPORTS: usize = 10;

/// Error report written to `pending.json`.
///
/// Contains enough context for the `gf-autoreport-bug` skill
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
            error_message: sanitize_error_message(error_message),
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

        // Preserve any existing pending report so a burst of failures does
        // not silently drop earlier reports (P1-5).
        if path.exists() {
            let archived = dir.join(format!(
                "pending.{}.json",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_or(0, |d| d.as_millis())
            ));
            std::fs::rename(&path, &archived)?;
            prune_archived_reports(&dir);
        }

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let mut file = std::fs::File::create(&path)?;
        #[cfg(unix)]
        set_pending_file_permissions(&file)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }
}

/// Redacts GitHub personal access tokens from error messages.
///
/// Matches both classic tokens (`ghp_…`) and fine-grained tokens
/// (`github_pat_…`). Compiled once and reused across calls.
static GITHUB_TOKEN_RE: LazyLock<Regex> = LazyLock::new(|| {
    #[allow(
        clippy::expect_used,
        reason = "regex pattern is a compile-time literal; a compile failure is a programming \
                  error"
    )]
    Regex::new(r"(?:ghp_[A-Za-z0-9]+|github_pat_[A-Za-z0-9_]+)")
        .expect("GitHub token regex must be statically valid")
});

/// Redacts generic `sk-`-prefixed API keys (Anthropic, `OpenAI`, and
/// similarly-shaped vendor tokens) from error messages.
static GENERIC_SK_TOKEN_RE: LazyLock<Regex> = LazyLock::new(|| {
    #[allow(
        clippy::expect_used,
        reason = "regex pattern is a compile-time literal; a compile failure is a programming \
                  error"
    )]
    Regex::new(r"\bsk-[A-Za-z0-9_-]{10,}")
        .expect("generic sk- token regex must be statically valid")
});

/// Redacts GitLab personal access tokens from error messages.
static GITLAB_TOKEN_RE: LazyLock<Regex> = LazyLock::new(|| {
    #[allow(
        clippy::expect_used,
        reason = "regex pattern is a compile-time literal; a compile failure is a programming \
                  error"
    )]
    Regex::new(r"glpat-[A-Za-z0-9_-]{20,}").expect("GitLab token regex must be statically valid")
});

/// Minimum byte length an environment variable's value must have before
/// [`redact_env_values`] will treat a match against it as significant —
/// avoids false-positive redaction against trivial short values (e.g. a
/// credential-named var accidentally set to `"1"` or `""`).
const MIN_REDACTABLE_ENV_VALUE_LEN: usize = 8;

/// Environment variable name fragments (case-insensitive) that mark a
/// variable as credential-shaped for [`redact_env_values`].
const CREDENTIAL_NAME_FRAGMENTS: &[&str] = &["TOKEN", "KEY", "SECRET", "PASSWORD", "CREDENTIAL"];

/// Redact any occurrence of a credential-looking environment variable's
/// value from `message`.
///
/// A variable counts as credential-looking when its name
/// case-insensitively contains any of [`CREDENTIAL_NAME_FRAGMENTS`] and
/// its value is at least [`MIN_REDACTABLE_ENV_VALUE_LEN`] bytes. This
/// catches secrets regardless of vendor-specific shape — e.g. a
/// `ANTHROPIC_AUTH_TOKEN` sitting in the environment that happens to leak
/// into an error message, which no fixed-format regex would recognize.
fn redact_env_values(message: &str, vars: impl IntoIterator<Item = (String, String)>) -> String {
    let mut result = message.to_string();
    for (name, value) in vars {
        if value.len() < MIN_REDACTABLE_ENV_VALUE_LEN {
            continue;
        }
        let upper_name = name.to_uppercase();
        let looks_like_credential = CREDENTIAL_NAME_FRAGMENTS
            .iter()
            .any(|frag| upper_name.contains(frag));
        if looks_like_credential {
            result = result.replace(&value, "[REDACTED]");
        }
    }
    result
}

/// Sanitize a raw error message before it is persisted to `pending.json`.
///
/// Several categories of sensitive data are redacted, in this order:
///
/// 1. **Credential-named environment variable values** — any value of an environment variable whose
///    name looks credential-shaped (see [`redact_env_values`]), regardless of the value's format.
///    This catches vendor-specific secrets that no fixed-format regex would recognize. This step
///    runs first so that a credential value which happens to contain the home directory path (e.g.
///    a `*_TOKEN_FILE` var pointing under `~`) is still matched verbatim, before the home-path
///    substitution below would otherwise rewrite it out from under the match.
/// 2. **Home directory paths** — the current user's home directory, as reported by
///    [`dirs::home_dir`], is replaced with `~`. Absolute user paths therefore never leak into bug
///    reports.
/// 3. **GitHub tokens** — classic personal access tokens (`ghp_…`) and fine-grained personal access
///    tokens (`github_pat_…`) are replaced with `[REDACTED]`.
/// 4. **Generic `sk-`-prefixed tokens** — Anthropic, `OpenAI`, and similarly-shaped vendor API
///    keys.
/// 5. **GitLab tokens** — personal access tokens (`glpat-…`).
///
/// Safe messages that contain none of the above are returned unchanged.
///
/// # Examples
///
/// ```text
/// "failed to read /Users/alice/.config/git/config"
///     → "failed to read ~/.config/git/config"
/// "clone failed: token ghp_1234567890abcdefghijklmnopqrstuvwxyz"
///     → "clone failed: token [REDACTED]"
/// ```
fn sanitize_error_message(message: &str) -> String {
    // Env-value redaction must run BEFORE home-path substitution: if a
    // credential-named env var's value contains the home directory path
    // (e.g. `SOME_TOKEN_FILE=/Users/x/.creds/tok`), rewriting the home
    // path to `~` first would make the literal env value no longer match,
    // so it would silently escape redaction.
    //
    // `vars_os()` is used instead of `std::env::vars()` because `vars()`
    // PANICS if any environment variable's name or value is not valid
    // Unicode (non-UTF-8 locales, some Windows setups, a mangled `PATH`
    // entry). This function sits on the best-effort error-reporting path,
    // so a non-UTF-8 var must be skipped, never allowed to abort the
    // process while it is already handling an error.
    let vars = std::env::vars_os()
        .filter_map(|(k, v)| Some((k.into_string().ok()?, v.into_string().ok()?)));
    let sanitized = redact_env_values(message, vars);

    let sanitized = if let Some(home) = dirs::home_dir() {
        let home_str = home.to_string_lossy();
        sanitized.replace(home_str.as_ref(), "~")
    } else {
        sanitized
    };
    let sanitized = GITHUB_TOKEN_RE
        .replace_all(&sanitized, "[REDACTED]")
        .into_owned();
    let sanitized = GENERIC_SK_TOKEN_RE
        .replace_all(&sanitized, "[REDACTED]")
        .into_owned();
    GITLAB_TOKEN_RE
        .replace_all(&sanitized, "[REDACTED]")
        .into_owned()
}

/// Delete archived `pending.<millis>.json` reports beyond
/// [`MAX_ARCHIVED_REPORTS`], oldest first.
///
/// Best-effort: any I/O error while listing or removing a file is
/// swallowed. The current `pending.json` is never touched by this
/// function — only files matching `pending.<digits>.json`.
fn prune_archived_reports(dir: &Path) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    let mut archives: Vec<(u128, PathBuf)> = entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let name = entry.file_name();
            let name = name.to_str()?;
            let millis = name
                .strip_prefix("pending.")
                .and_then(|rest| rest.strip_suffix(".json"))
                .and_then(|ts| ts.parse::<u128>().ok())?;
            Some((millis, entry.path()))
        })
        .collect();

    if archives.len() <= MAX_ARCHIVED_REPORTS {
        return;
    }

    archives.sort_by_key(|(millis, _)| *millis);
    let excess = archives.len() - MAX_ARCHIVED_REPORTS;
    for (_, path) in archives.into_iter().take(excess) {
        let _ = std::fs::remove_file(path);
    }
}

/// Restrict `pending.json` to owner-only read/write (mode `0o600`).
///
/// The report contains error context that may include sensitive paths or
/// environment details; on multi-user systems it must not be readable by
/// other users. This is a no-op on non-Unix platforms.
#[cfg(unix)]
fn set_pending_file_permissions(file: &std::fs::File) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt as _;
    file.set_permissions(std::fs::Permissions::from_mode(0o600))
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

    if is_ci_environment() {
        return Ok(());
    }

    // Only report if user has joined the co-contribution plan
    if !is_co_contribution_enabled() {
        return Ok(());
    }

    // User input/argument errors are not CLI bugs — skip auto-reporting so
    // they do not pollute the Issue stream as false positives.
    if is_user_input_error(error_code) {
        return Ok(());
    }

    let report = ErrorReport::from_error(command, platform, error_message, error_code);
    let repo_root = find_repo_root()?;
    report.write_to_disk(&repo_root)
}

/// Check whether the co-contribution plan is enabled for the current
/// project.
///
/// Checks **only** `<repo_root>/.claude/settings.json` — a global-only
/// opt-in (`~/.claude/settings.json`) no longer silently enables
/// reporting in every project; see [`global_co_contribution_pending_ack_with`]
/// for the mechanism that surfaces that gap via `gf doctor` instead.
/// Returns `false` if the repo root cannot be found or the field is
/// missing/false.
fn is_co_contribution_enabled() -> bool {
    let Ok(repo_root) = find_repo_root() else {
        return false;
    };
    read_co_contribution_flag(&repo_root.join(".claude/settings.json"))
}

/// Resolve `<repo_root>/.claude/settings.json` for the current project,
/// using the same repo-root resolution as [`is_co_contribution_enabled`].
///
/// Returns `None` if the repo root cannot be found (not inside a git
/// repo). Exposed for `gf doctor`'s `CoContributionCheck`, which needs to
/// read the same project-level file that actually determines whether
/// reporting is active — see Finding 3 in the hardening-pass review: the
/// check must not fall back to reading the global-only settings file.
pub(crate) fn project_settings_path() -> Option<PathBuf> {
    find_repo_root()
        .ok()
        .map(|root| root.join(".claude/settings.json"))
}

/// Read the `gitflow.co_contribution` flag from a specific settings file.
///
/// Returns `false` if the file doesn't exist, can't be read, or the field
/// is missing/not a boolean. Convenience wrapper over
/// [`read_co_contribution_field`] for call sites that don't need to
/// distinguish "explicitly false" from "absent".
pub(crate) fn read_co_contribution_flag(path: &Path) -> bool {
    read_co_contribution_field(path).unwrap_or(false)
}

/// Read the `gitflow.co_contribution` flag from a specific settings file,
/// distinguishing an explicit decision from absence.
///
/// Returns `None` if the file doesn't exist, can't be read, can't be
/// parsed as JSON, or the field is missing/not a boolean — i.e. "no
/// decision has been made here." Returns `Some(bool)` for an explicit
/// `true` or `false`.
fn read_co_contribution_field(path: &Path) -> Option<bool> {
    let content = std::fs::read_to_string(path).ok()?;
    let json = serde_json::from_str::<serde_json::Value>(&content).ok()?;
    json.pointer("/gitflow/co_contribution")
        .and_then(serde_json::Value::as_bool)
}

/// Testable core of the global co-contribution pending-ack check —
/// returns `true` when the global co-contribution opt-in is active but
/// this project has never made its own explicit decision. Takes both
/// settings paths explicitly instead of resolving them from `HOME`/the
/// git repo root; `gf doctor`'s `CoContributionCheck` calls this directly
/// after resolving the real paths, so both call sites share one
/// definition of "pending ack" — the gap `CoContributionCheck` surfaces
/// so a global-only opt-in doesn't silently cover every future project
/// with no visibility.
pub(crate) fn global_co_contribution_pending_ack_with(
    global_path: &Path,
    project_path: &Path,
) -> bool {
    read_co_contribution_field(global_path) == Some(true)
        && read_co_contribution_field(project_path).is_none()
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

/// Environment variables set by common CI providers, checked to keep
/// `gf-autoreport-bug` from ever firing inside a pipeline run.
///
/// `gf-regression`'s skill doc already documents "never autoreport from
/// CI" as a hard rule; this makes that rule code-enforced rather than
/// relying on the LLM to honor the documentation. Note this is
/// independent of [`should_skip_reporting`] (the TTY check): CI runs are
/// almost always non-interactive, so without this check the TTY gate
/// alone would let CI failures straight through.
const CI_ENV_VARS: &[&str] = &[
    "CI",
    "GITHUB_ACTIONS",
    "GITLAB_CI",
    "CI_PIPELINE_ID",
    "CIRCLECI",
    "BUILDKITE",
    "JENKINS_URL",
];

/// Returns `true` when any known CI environment variable is present
/// *and* non-empty.
///
/// A variable that is set but empty (`CI=""`, which some shells and
/// tools export) must not count as "CI present" — the spec requires
/// "set and non-empty" so a stray `export CI=` in a user's shell
/// profile does not silently disable all bug reporting.
///
/// Extracted for testability — see [`is_ci_environment_with`].
fn is_ci_environment() -> bool {
    is_ci_environment_with(|name| std::env::var_os(name).is_some_and(|v| !v.is_empty()))
}

/// Testable core of [`is_ci_environment`]: takes a presence-check
/// closure instead of reading the real environment directly, so tests
/// don't need to mutate global process state. The closure's contract is
/// "should this var count as present" — i.e. it already encodes the
/// set-and-non-empty rule, not merely set-ness.
fn is_ci_environment_with(has_var: impl Fn(&str) -> bool) -> bool {
    CI_ENV_VARS.iter().any(|var| has_var(var))
}

/// Returns `true` when the given error code represents a user input or
/// argument error (not a real CLI defect).
///
/// Such errors are silently skipped by [`maybe_report_error`] so that
/// invalid user input is never auto-reported as a bug.
fn is_user_input_error(error_code: &str) -> bool {
    error_code == "USER_INPUT_ERROR"
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
    fn test_should_sanitize_home_directory_in_error_message() {
        let home = dirs::home_dir().expect("home dir must resolve in tests");
        let home_str = home.to_string_lossy();
        let message = format!("failed to read {home_str}/.claude/settings.json");
        let sanitized = sanitize_error_message(&message);
        assert_eq!(
            sanitized, "failed to read ~/.claude/settings.json",
            "home directory path must be replaced with ~"
        );
    }

    #[test]
    fn test_should_sanitize_token_in_error_message() {
        let classic = "auth failed: token ghp_1234567890abcdefghijklmnopqrstuvwxyz rejected";
        let sanitized = sanitize_error_message(classic);
        assert!(
            !sanitized.contains("ghp_"),
            "classic GitHub token must be redacted: {sanitized}"
        );
        assert!(sanitized.contains("[REDACTED]"));

        let fine_grained =
            "clone failed: \
             github_pat_1234567890abcdef_GHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let sanitized = sanitize_error_message(fine_grained);
        assert!(
            !sanitized.contains("github_pat_"),
            "fine-grained GitHub token must be redacted: {sanitized}"
        );
        assert!(sanitized.contains("[REDACTED]"));
    }

    #[test]
    fn test_should_not_modify_safe_error_message() {
        let message = "issue not found: pull request #42 does not exist";
        assert_eq!(sanitize_error_message(message), message);
    }

    #[test]
    fn test_should_not_modify_message_containing_task_dash_word() {
        // Regression for the missing left word boundary on GENERIC_SK_TOKEN_RE:
        // "sk-" must only match as a token prefix, not inside ordinary words
        // like "task-", "risk-", "disk-" that show up in branch/file names.
        let message = "failed to update task-manager-config file";
        assert_eq!(sanitize_error_message(message), message);
    }

    #[test]
    fn test_should_not_modify_message_containing_risk_or_disk_dash_word() {
        assert_eq!(
            sanitize_error_message("risk-assessment-report failed"),
            "risk-assessment-report failed"
        );
        assert_eq!(
            sanitize_error_message("disk-utilization-high"),
            "disk-utilization-high"
        );
    }

    #[test]
    fn test_should_redact_env_var_value_when_name_looks_like_credential() {
        let vars = vec![("MY_API_TOKEN".to_string(), "abcdef123456".to_string())];
        let message = "request failed: token abcdef123456 rejected";
        let redacted = redact_env_values(message, vars);
        assert_eq!(redacted, "request failed: token [REDACTED] rejected");
    }

    #[test]
    fn test_should_not_redact_when_env_var_name_does_not_look_like_credential() {
        let vars = vec![(
            "SOME_LONG_PATH_VALUE".to_string(),
            "abcdef123456".to_string(),
        )];
        let message = "request failed: token abcdef123456 rejected";
        let redacted = redact_env_values(message, vars);
        assert_eq!(
            redacted, message,
            "a non-credential-named var must not trigger redaction"
        );
    }

    #[test]
    fn test_should_not_redact_short_env_var_values() {
        let vars = vec![("API_TOKEN".to_string(), "1".to_string())];
        let message = "exit code: 1";
        let redacted = redact_env_values(message, vars);
        assert_eq!(
            redacted, message,
            "values shorter than the minimum length must not be redacted"
        );
    }

    #[test]
    fn test_should_redact_multiple_occurrences_of_env_var_value() {
        let vars = vec![("SECRET_KEY".to_string(), "topsecretvalue".to_string())];
        let message = "topsecretvalue appears twice: topsecretvalue";
        let redacted = redact_env_values(message, vars);
        assert_eq!(redacted, "[REDACTED] appears twice: [REDACTED]");
    }

    #[test]
    fn test_should_sanitize_generic_sk_prefixed_token() {
        let message = "auth failed: sk-ant-api03-abcdefghijklmnopqrstuvwxyz rejected";
        let sanitized = sanitize_error_message(message);
        assert!(
            !sanitized.contains("sk-ant-"),
            "sk-prefixed token must be redacted: {sanitized}"
        );
        assert!(sanitized.contains("[REDACTED]"));
    }

    #[test]
    fn test_should_sanitize_gitlab_token() {
        let message = "clone failed: token glpat-1234567890abcdefghij rejected"; // gitleaks:allow
        let sanitized = sanitize_error_message(message);
        assert!(
            !sanitized.contains("glpat-"),
            "GitLab token must be redacted: {sanitized}"
        );
        assert!(sanitized.contains("[REDACTED]"));
    }

    #[test]
    fn test_should_set_pending_json_permissions_to_600() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let report = ErrorReport::from_error("pr list", "gitlab", "not found", "NOT_FOUND");
        report.write_to_disk(tmp.path()).expect("write_to_disk");

        let pending = tmp.path().join(".cache/bug-reports/pending.json");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            let metadata = std::fs::metadata(&pending).expect("metadata");
            assert_eq!(
                metadata.permissions().mode() & 0o777,
                0o600,
                "pending.json must be readable/writable only by the owner"
            );
        }
        #[cfg(not(unix))]
        {
            // Permission control is a no-op on non-Unix platforms.
            assert!(pending.exists(), "pending.json must be created");
        }
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

    #[test]
    fn test_should_return_none_for_missing_co_contribution_field() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("settings.json");
        std::fs::write(&path, r#"{"gitflow": {}}"#).expect("write");
        assert_eq!(read_co_contribution_field(&path), None);
    }

    #[test]
    fn test_should_return_none_for_missing_settings_file_tri_state() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let missing = tmp.path().join("nonexistent.json");
        assert_eq!(read_co_contribution_field(&missing), None);
    }

    #[test]
    fn test_should_return_some_true_for_co_contribution_field_true() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("settings.json");
        std::fs::write(&path, r#"{"gitflow": {"co_contribution": true}}"#).expect("write");
        assert_eq!(read_co_contribution_field(&path), Some(true));
    }

    #[test]
    fn test_should_return_some_false_for_co_contribution_field_false() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("settings.json");
        std::fs::write(&path, r#"{"gitflow": {"co_contribution": false}}"#).expect("write");
        assert_eq!(read_co_contribution_field(&path), Some(false));
    }

    #[test]
    fn test_pending_ack_true_when_global_true_and_project_absent() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let global = tmp.path().join("global.json");
        std::fs::write(&global, r#"{"gitflow": {"co_contribution": true}}"#).expect("write");
        let project = tmp.path().join("project.json");
        std::fs::write(&project, r"{}").expect("write");
        assert!(global_co_contribution_pending_ack_with(&global, &project));
    }

    #[test]
    fn test_pending_ack_false_when_project_already_decided_true() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let global = tmp.path().join("global.json");
        std::fs::write(&global, r#"{"gitflow": {"co_contribution": true}}"#).expect("write");
        let project = tmp.path().join("project.json");
        std::fs::write(&project, r#"{"gitflow": {"co_contribution": true}}"#).expect("write");
        assert!(!global_co_contribution_pending_ack_with(&global, &project));
    }

    #[test]
    fn test_pending_ack_false_when_project_already_decided_false() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let global = tmp.path().join("global.json");
        std::fs::write(&global, r#"{"gitflow": {"co_contribution": true}}"#).expect("write");
        let project = tmp.path().join("project.json");
        std::fs::write(&project, r#"{"gitflow": {"co_contribution": false}}"#).expect("write");
        assert!(!global_co_contribution_pending_ack_with(&global, &project));
    }

    #[test]
    fn test_pending_ack_false_when_global_false() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let global = tmp.path().join("global.json");
        std::fs::write(&global, r#"{"gitflow": {"co_contribution": false}}"#).expect("write");
        let project = tmp.path().join("project.json");
        std::fs::write(&project, r"{}").expect("write");
        assert!(!global_co_contribution_pending_ack_with(&global, &project));
    }

    #[test]
    fn test_should_classify_user_input_error() {
        assert!(is_user_input_error("USER_INPUT_ERROR"));
        assert!(!is_user_input_error("CLI_ERROR"));
        assert!(!is_user_input_error("AUTH_FAILED"));
        assert!(!is_user_input_error(""));
    }

    #[test]
    fn test_should_detect_ci_from_known_env_vars() {
        for var in [
            "CI",
            "GITHUB_ACTIONS",
            "GITLAB_CI",
            "CI_PIPELINE_ID",
            "CIRCLECI",
            "BUILDKITE",
            "JENKINS_URL",
        ] {
            let present = |name: &str| name == var;
            assert!(
                is_ci_environment_with(present),
                "{var} must be recognized as a CI indicator"
            );
        }
    }

    #[test]
    fn test_should_not_detect_ci_when_no_known_vars_set() {
        let present = |_: &str| false;
        assert!(!is_ci_environment_with(present));
    }

    #[test]
    fn test_should_not_detect_ci_when_var_is_set_but_empty() {
        // Simulates `CI=""` (some shells/tools export CI as an empty
        // string). The closure here plays the same role
        // `is_ci_environment`'s real closure plays: it looks up a raw
        // value per name (as `var_os` would) and only reports the var
        // as present when that value is non-empty, per the "set and
        // non-empty" spec contract.
        let raw_value = |name: &str| if name == "CI" { Some("") } else { None };
        let has_var = |name: &str| raw_value(name).is_some_and(|v| !v.is_empty());
        assert!(
            !is_ci_environment_with(has_var),
            "an env var that is set but empty must not count as CI-present"
        );
    }

    #[test]
    fn test_should_archive_previous_pending_on_second_write() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let first = ErrorReport::from_error("issue list", "github", "first", "CLI_ERROR");
        first.write_to_disk(tmp.path()).expect("first write");

        let second = ErrorReport::from_error("pr list", "github", "second", "CLI_ERROR");
        second.write_to_disk(tmp.path()).expect("second write");

        let dir = tmp.path().join(".cache/bug-reports");
        let pending = dir.join("pending.json");
        assert!(pending.exists(), "pending.json must exist");

        // Old report must be preserved under a timestamped name.
        let archived: Vec<_> = std::fs::read_dir(&dir)
            .expect("read bug-reports dir")
            .filter_map(Result::ok)
            .map(|e| e.file_name().to_string_lossy().to_string())
            .filter(|n| {
                n.starts_with("pending.")
                    && n != "pending.json"
                    && std::path::Path::new(n).extension() == Some(std::ffi::OsStr::new("json"))
            })
            .collect();
        assert_eq!(
            archived.len(),
            1,
            "exactly one archived report: {archived:?}"
        );

        let archived_content =
            std::fs::read_to_string(dir.join(&archived[0])).expect("read archived");
        assert!(
            archived_content.contains("first"),
            "archived report keeps first content"
        );
    }

    #[test]
    fn test_should_prune_archived_reports_beyond_retention_cap() {
        let tmp = tempfile::tempdir().expect("tempdir");

        // Write MAX_ARCHIVED_REPORTS + 3 reports in sequence; each write
        // archives the previous pending.json, so this produces
        // MAX_ARCHIVED_REPORTS + 2 archived files before pruning kicks in.
        for i in 0..(MAX_ARCHIVED_REPORTS + 3) {
            let report = ErrorReport::from_error(
                "issue list",
                "github",
                &format!("failure {i}"),
                "CLI_ERROR",
            );
            report.write_to_disk(tmp.path()).expect("write_to_disk");
            // Ensure filesystem-visible millisecond timestamps differ so
            // archived filenames sort deterministically.
            std::thread::sleep(std::time::Duration::from_millis(2));
        }

        let dir = tmp.path().join(".cache/bug-reports");
        let archived: Vec<_> = std::fs::read_dir(&dir)
            .expect("read bug-reports dir")
            .filter_map(Result::ok)
            .map(|e| e.file_name().to_string_lossy().to_string())
            .filter(|n| n.starts_with("pending.") && n != "pending.json")
            .collect();

        assert_eq!(
            archived.len(),
            MAX_ARCHIVED_REPORTS,
            "archived reports must be capped at MAX_ARCHIVED_REPORTS: {archived:?}"
        );
    }
}
