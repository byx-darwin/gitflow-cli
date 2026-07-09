//! `gitflow-cli-gitcode` —— GitCode 平台实现。
//!
//! 本 crate 实现了 `gitflow-cli-core` 中定义的 [`IssueProvider`]、[`PrProvider`]、
//! [`ReleaseProvider`]、[`ReviewProvider`]、[`AuthProvider`]、[`LabelProvider`]、
//! [`MilestoneProvider`]、[`CommitProvider`] 与 [`PipelineProvider`] trait，
//! 通过调用 `gitcode` CLI 获取数据并解析其 JSON 输出。
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
//! 所有平台调用失败时，`gitcode` 的 stderr 会通过 [`error::parse_gitcode_error`] 解析，
//! 并统一映射为 [`CoreError::Platform`]。
//!
//! [`IssueProvider`]: gitflow_cli_core::issue::IssueProvider
//! [`PrProvider`]: gitflow_cli_core::pr::PrProvider
//! [`ReleaseProvider`]: gitflow_cli_core::release::ReleaseProvider
//! [`ReviewProvider`]: gitflow_cli_core::review::ReviewProvider
//! [`AuthProvider`]: gitflow_cli_core::auth::AuthProvider
//! [`LabelProvider`]: gitflow_cli_core::label::LabelProvider
//! [`MilestoneProvider`]: gitflow_cli_core::label::MilestoneProvider
//! [`CommitProvider`]: gitflow_cli_core::commit::CommitProvider
//! [`PipelineProvider`]: gitflow_cli_core::pipeline::PipelineProvider
//! [`CoreError::Platform`]: gitflow_cli_core::CoreError::Platform

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
pub use error::GitcodeError;
pub use issue::GitCodeIssueProvider;
pub use label::{GitCodeLabelProvider, GitCodeMilestoneProvider};
pub use pipeline::GitCodePipelineProvider;
pub use pr::GitCodePrProvider;
pub use release::GitCodeReleaseProvider;
pub use review::GitCodeReviewProvider;

/// Return the GitCode CLI binary path.
///
/// Searches PATH first, then pip user install directories
/// (`~/Library/Python/*/bin/` on macOS, `~/.local/bin/` on Linux).
#[allow(
    clippy::disallowed_methods,
    reason = "binary discovery runs at startup before async runtime is ready"
)]
pub(crate) fn gitcode_binary() -> String {
    // 1. which gitcode (pip/wheel/DEB/RPM install)
    if let Ok(p) = which::which("gitcode") {
        return p.to_string_lossy().into_owned();
    }
    // 2. pip user install paths (e.g. ~/Library/Python/3.9/bin/)
    if let Ok(home) = std::env::var("HOME") {
        let lib = std::path::PathBuf::from(&home).join("Library/Python");
        if let Ok(entries) = std::fs::read_dir(&lib) {
            for entry in entries.flatten() {
                let p = entry.path().join("bin/gitcode");
                if p.exists() {
                    return p.to_string_lossy().into_owned();
                }
            }
        }
        // Also check ~/.local/bin
        let p = std::path::PathBuf::from(&home).join(".local/bin/gitcode");
        if p.exists() {
            return p.to_string_lossy().into_owned();
        }
    }
    // 3. Desperate fallback — hope it's in PATH
    "gitcode".into()
}
