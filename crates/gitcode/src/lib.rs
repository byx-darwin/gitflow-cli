//! `gitflow-gitcode` —— GitCode 平台实现。
//!
//! 本 crate 实现了 `gitflow-core` 中定义的 [`IssueProvider`]、[`PrProvider`]、
//! [`ReleaseProvider`]、[`ReviewProvider`]、[`AuthProvider`]、[`LabelProvider`]、
//! [`MilestoneProvider`]、[`CommitProvider`] 与 [`PipelineProvider`] trait，
//! 通过调用 `gc` CLI 获取数据并解析其 JSON 输出。
//!
//! # 主要类型
//!
//! - [`GitCodeIssueProvider`]：操作 GitCode Issue。
//! - [`GitCodePrProvider`]：操作 GitCode Pull Request。
//! - [`GitCodeReleaseProvider`]：操作 GitCode Release。
//! - [`GitCodeReviewProvider`]：操作 GitCode PR Review。
//! - [`GitCodeAuthProvider`]：处理 GitCode 认证（登录、登出、状态、Token）。
//! - [`GitCodeLabelProvider`]：管理 GitCode 仓库标签。
//! - [`GitCodeMilestoneProvider`]：管理 GitCode 仓库里程碑。
//! - [`GitCodeCommitProvider`]：查看 GitCode Commit 及 Diff/Patch。
//! - [`GitCodePipelineProvider`]：查看 GitCode CI/CD 流水线。
//!
//! # 错误处理
//!
//! 所有平台调用失败时，`gc` 的 stderr 会通过 [`error::parse_gitcode_error`] 解析，
//! 并统一映射为 [`CoreError::Platform`]。
//!
//! [`IssueProvider`]: gitflow_core::issue::IssueProvider
//! [`PrProvider`]: gitflow_core::pr::PrProvider
//! [`ReleaseProvider`]: gitflow_core::release::ReleaseProvider
//! [`ReviewProvider`]: gitflow_core::review::ReviewProvider
//! [`AuthProvider`]: gitflow_core::auth::AuthProvider
//! [`LabelProvider`]: gitflow_core::label::LabelProvider
//! [`MilestoneProvider`]: gitflow_core::label::MilestoneProvider
//! [`CommitProvider`]: gitflow_core::commit::CommitProvider
//! [`PipelineProvider`]: gitflow_core::pipeline::PipelineProvider
//! [`CoreError::Platform`]: gitflow_core::CoreError::Platform

#![forbid(unsafe_code)]
#![allow(
    clippy::doc_markdown,
    reason = "GitCode is a platform brand name, not a Rust code item"
)]
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

pub mod auth;
pub mod commit;
pub mod error;
pub mod issue;
pub mod label;
pub mod pipeline;
pub mod pr;
pub mod release;
pub mod review;
pub mod runner;

pub use auth::GitCodeAuthProvider;
pub use commit::GitCodeCommitProvider;
pub use error::parse_gitcode_error;
pub use issue::GitCodeIssueProvider;
pub use label::{GitCodeLabelProvider, GitCodeMilestoneProvider};
pub use pipeline::GitCodePipelineProvider;
pub use pr::GitCodePrProvider;
pub use release::GitCodeReleaseProvider;
pub use review::GitCodeReviewProvider;

/// Binary name used when no usable GitCode CLI is discovered.
///
/// Deliberately the unambiguous `gitcode` rather than `gc`: on macOS,
/// Graphviz installs an unrelated `/opt/homebrew/bin/gc`, so falling back to
/// `gc` would hand every call to a graph-counting utility instead of failing
/// cleanly with "command not found".
pub(crate) const FALLBACK_BINARY: &str = "gitcode";

/// Return the GitCode CLI binary path.
///
/// Prefers `gitcode` (unambiguous) over `gc` (short native name, but collides
/// with Graphviz's `gc` on macOS). Every discovered candidate must identify
/// itself as the GitCode CLI via [`identifies_as_gitcode_cli`] before it is
/// accepted, so an unrelated same-named binary is rejected and discovery
/// continues.
///
/// Searches PATH first, then pip user install directories
/// (`~/Library/Python/*/bin/` on macOS, `~/.local/bin/` on Linux).
pub(crate) fn gitcode_binary() -> String {
    resolve_gitcode_binary(locate_candidate, identifies_as_gitcode_cli)
}

/// Pick the first candidate that both resolves to a path and passes `probe`.
///
/// Split out from [`gitcode_binary`] so the ordering and rejection logic can
/// be unit-tested without touching PATH or spawning processes.
fn resolve_gitcode_binary<L, P>(lookup: L, probe: P) -> String
where
    L: Fn(&str) -> Option<std::path::PathBuf>,
    P: Fn(&std::path::Path) -> bool,
{
    const CANDIDATES: &[&str] = &["gitcode", "gc"];

    for &candidate in CANDIDATES {
        if let Some(path) = lookup(candidate)
            && probe(&path)
        {
            return path.to_string_lossy().into_owned();
        }
    }
    FALLBACK_BINARY.into()
}

/// Locate `candidate` on PATH, then in pip user install directories.
#[allow(
    clippy::disallowed_methods,
    reason = "binary discovery runs at startup before async runtime is ready"
)]
fn locate_candidate(candidate: &str) -> Option<std::path::PathBuf> {
    if let Ok(path) = which::which(candidate) {
        return Some(path);
    }

    let home = std::env::var("HOME").ok()?;

    // pip user install paths (e.g. ~/Library/Python/3.9/bin/)
    let lib = std::path::PathBuf::from(&home).join("Library/Python");
    if let Ok(entries) = std::fs::read_dir(&lib) {
        for entry in entries.flatten() {
            let path = entry.path().join("bin").join(candidate);
            if path.exists() {
                return Some(path);
            }
        }
    }

    let path = std::path::PathBuf::from(&home)
        .join(".local/bin")
        .join(candidate);
    path.exists().then_some(path)
}

/// Report whether `path` identifies itself as the GitCode CLI.
///
/// Probes with `--help` rather than `--version`: the real CLI rejects
/// `--version` with `unknown flag` (exit 2), so a version probe would discard
/// the very binary it is meant to find.
#[allow(
    clippy::disallowed_types,
    reason = "binary discovery runs at startup before async runtime is ready"
)]
fn identifies_as_gitcode_cli(path: &std::path::Path) -> bool {
    std::process::Command::new(path)
        .arg("--help")
        .output()
        .is_ok_and(|output| {
            is_gitcode_help_output(
                output.status.success(),
                &String::from_utf8_lossy(&output.stdout),
                &String::from_utf8_lossy(&output.stderr),
            )
        })
}

/// Decide whether a `--help` probe result came from the GitCode CLI.
///
/// Requires both a successful exit and a `gitcode` marker in the output.
/// Graphviz's `gc` fails on both counts: it exits 1 and prints
/// `gc: option -- unrecognized`.
fn is_gitcode_help_output(success: bool, stdout: &str, stderr: &str) -> bool {
    success
        && format!("{stdout}{stderr}")
            .to_lowercase()
            .contains("gitcode")
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::*;

    #[test]
    fn test_should_return_non_empty_binary_name() {
        let binary = gitcode_binary();
        assert!(!binary.is_empty());
    }

    #[test]
    fn test_should_prefer_gitcode_over_gc_when_both_resolve() {
        let lookup = |candidate: &str| Some(PathBuf::from(format!("/usr/bin/{candidate}")));
        let probe = |_: &Path| true;

        assert_eq!(
            resolve_gitcode_binary(lookup, probe),
            "/usr/bin/gitcode",
            "the unambiguous name must win so an unrelated `gc` cannot shadow the CLI"
        );
    }

    #[test]
    fn test_should_reject_candidate_that_fails_probe() {
        // Graphviz ships /opt/homebrew/bin/gc, which is not the GitCode CLI.
        let lookup =
            |candidate: &str| (candidate == "gc").then(|| PathBuf::from("/opt/homebrew/bin/gc"));
        let probe = |_: &Path| false;

        assert_eq!(
            resolve_gitcode_binary(lookup, probe),
            FALLBACK_BINARY,
            "a binary that does not identify as the GitCode CLI must be rejected"
        );
    }

    #[test]
    fn test_should_accept_gc_when_it_passes_probe() {
        let lookup =
            |candidate: &str| (candidate == "gc").then(|| PathBuf::from("/usr/local/bin/gc"));
        let probe = |_: &Path| true;

        assert_eq!(resolve_gitcode_binary(lookup, probe), "/usr/local/bin/gc");
    }

    #[test]
    fn test_should_fall_back_to_unambiguous_name_when_nothing_resolves() {
        let lookup = |_: &str| None;
        let probe = |_: &Path| true;

        assert_eq!(
            resolve_gitcode_binary(lookup, probe),
            "gitcode",
            "falling back to `gc` would re-introduce the shadowing bug"
        );
    }

    #[test]
    fn test_should_identify_real_gitcode_cli_from_help_output() {
        // Captured verbatim from `gitcode --help` (exit 0).
        let stdout = "gitcode is a command line tool for GitCode.\n\nIt provides convenient \
                      access to GitCode features including:\n  - Authentication management (auth \
                      login, auth status)\n";

        assert!(is_gitcode_help_output(true, stdout, ""));
    }

    #[test]
    fn test_should_reject_graphviz_gc_help_output() {
        // Captured verbatim from Graphviz `gc --help` (exit 1).
        let stdout = "Usage: gc [-necCaDUrsv?] <files>\n  -n - print number of nodes\n";
        let stderr = "gc: option -- unrecognized\n";

        assert!(
            !is_gitcode_help_output(false, stdout, stderr),
            "Graphviz gc must never be mistaken for the GitCode CLI"
        );
    }

    #[test]
    fn test_should_reject_successful_help_without_gitcode_marker() {
        assert!(
            !is_gitcode_help_output(true, "Usage: gc [-necCaDUrsv?] <files>", ""),
            "exit 0 alone is not enough — the output must name GitCode"
        );
    }

    #[test]
    fn test_should_match_gitcode_marker_case_insensitively() {
        assert!(is_gitcode_help_output(true, "A CLI for GitCode.", ""));
    }
}
