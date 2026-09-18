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

use std::sync::OnceLock;

use tracing::debug;

/// `gitcode api` 端点单页最大条目数，与 GitHub/GitLab REST API 的惯例对齐。
///
/// GitCode CLI 的 `api` 子命令是否真的支持 `per_page`/`page` 查询参数**未经实测**
/// （本环境无法获取 GitCode CLI）。若该平台忽略这两个参数，首页会短于
/// `per_page`，翻页循环在第一次调用后就因短页而终止——退化为今天「只取首页」
/// 的行为，既不会死循环，也不会丢数据。
pub(crate) const GITCODE_API_MAX_PER_PAGE: u32 = 100;

/// `issue list` / `pr list` / `release list` 在未显式传 `--limit` 时使用的默认值。
///
/// GitCode CLI 在本环境无法获取，其 `--limit` 的真实取值范围未经实测。本 crate 的
/// `api` 分页路径已经假定 100 是 GitCode 的单页上限
/// （见 [`GITCODE_API_MAX_PER_PAGE`]）；在缺乏进一步证据的情况下，为
/// `list` 子命令选用同一个保守值，既能避免用「`DEFAULT_LIST_LIMIT + 1` = 1001」这样
/// 的探测值撞上未知的服务端上限而报错，又仍然驱动 `fetch_capped` 的 N+1 探测、
/// 继续诚实报告截断。**用户显式传入的 `--limit` 不受此值影响**——那是调用方自己
/// 的选择，且在本分支之前就必须落在 GitCode 允许的范围内，否则早已报错。
pub(crate) const GITCODE_DEFAULT_LIST_LIMIT: u32 = GITCODE_API_MAX_PER_PAGE;

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

/// Cached result of binary discovery, so the `--help` probe runs at most once
/// per process instead of forking a subprocess on every CLI call.
static GITCODE_BINARY: OnceLock<String> = OnceLock::new();

/// Return the GitCode CLI binary path.
///
/// Prefers `gitcode` (unambiguous) over `gc` (short native name, but collides
/// with Graphviz's `gc` on macOS). Every discovered candidate must identify
/// itself as the GitCode CLI via [`identifies_as_gitcode_cli`] before it is
/// accepted; a candidate that fails the probe is rejected and discovery moves
/// on to the **next candidate name**. It does not keep searching other
/// locations for the same name — [`locate_candidate`] yields only the first
/// hit per name, so a failing `gitcode` earlier on PATH shadows a working one
/// in `~/.local/bin`. Rejections are logged at `debug` level with the
/// candidate path so this case is diagnosable.
///
/// Searches PATH first, then pip user install directories
/// (`~/Library/Python/*/bin/` on macOS, `~/.local/bin/` on Linux).
///
/// The result is memoized in a [`OnceLock`]: the probe runs once per process,
/// and subsequent calls only clone the cached string. This matters because
/// callers are overwhelmingly `async fn` bodies, where a per-call
/// `std::process::Command` would block a Tokio worker thread.
pub(crate) fn gitcode_binary() -> String {
    GITCODE_BINARY
        .get_or_init(|| resolve_gitcode_binary(locate_candidate, identifies_as_gitcode_cli))
        .clone()
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
    reason = "one-time synchronous PATH lookup, memoized behind `GITCODE_BINARY`"
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
/// A candidate that runs but does not identify as the GitCode CLI is rejected
/// silently from the user's perspective, so the rejection is recorded via
/// `tracing` with the candidate path — otherwise a version change in GitCode's
/// help text would surface only as `command not found: gitcode`.
#[allow(
    clippy::disallowed_types,
    reason = "one-time synchronous probe, memoized behind `GITCODE_BINARY`, called from sync \
              context"
)]
fn identifies_as_gitcode_cli(path: &std::path::Path) -> bool {
    let Ok(output) = std::process::Command::new(path).arg("--help").output() else {
        debug!(
            candidate = %path.display(),
            "candidate could not be executed for the --help probe"
        );
        return false;
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let accepted = is_gitcode_help_output(output.status.success(), &stdout, &stderr);

    if !accepted {
        debug!(
            candidate = %path.display(),
            exit_success = output.status.success(),
            "candidate rejected: `--help` did not identify it as the GitCode CLI"
        );
    }

    accepted
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
    fn test_should_cache_binary_discovery_across_calls() {
        assert_eq!(
            gitcode_binary(),
            gitcode_binary(),
            "discovery is memoized, so repeated calls must not re-probe"
        );
        assert!(
            GITCODE_BINARY.get().is_some(),
            "the first call must populate the cache so no later call forks a subprocess"
        );
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
