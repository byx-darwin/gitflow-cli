//! `gitflow-github` —— GitHub 平台实现。
//!
//! 本 crate 实现了 `gitflow-core` 中定义的 [`IssueProvider`]、[`PrProvider`]、
//! [`ReleaseProvider`]、[`ReviewProvider`]、[`AuthProvider`]、[`LabelProvider`]、
//! [`MilestoneProvider`] 与 [`CommitProvider`] trait，
//! 通过调用 `gh` CLI 获取数据并解析其 JSON 输出。
//!
//! # 主要类型
//!
//! - [`GitHubIssueProvider`]：操作 GitHub Issue。
//! - [`GitHubPrProvider`]：操作 GitHub Pull Request。
//! - [`GitHubReleaseProvider`]：操作 GitHub Release。
//! - [`GitHubReviewProvider`]：操作 GitHub PR Review。
//! - [`GitHubAuthProvider`]：处理 GitHub 认证（登录、登出、状态、Token）。
//! - [`GitHubLabelProvider`]：管理 GitHub 仓库标签。
//! - [`GitHubMilestoneProvider`]：管理 GitHub 仓库里程碑。
//! - [`GitHubCommitProvider`]：查看 GitHub Commit 及 Diff/Patch。
//!
//! # 错误处理
//!
//! 所有平台调用失败时，`gh` 的 stderr 会通过 [`error::parse_gh_error`] 解析，
//! 并统一映射为 [`CoreError::Cli`]（携带 [`PlatformCliError`]）。
//!
//! [`IssueProvider`]: gitflow_core::issue::IssueProvider
//! [`PrProvider`]: gitflow_core::pr::PrProvider
//! [`ReleaseProvider`]: gitflow_core::release::ReleaseProvider
//! [`ReviewProvider`]: gitflow_core::review::ReviewProvider
//! [`AuthProvider`]: gitflow_core::auth::AuthProvider
//! [`LabelProvider`]: gitflow_core::label::LabelProvider
//! [`MilestoneProvider`]: gitflow_core::label::MilestoneProvider
//! [`CommitProvider`]: gitflow_core::commit::CommitProvider
//! [`CoreError::Cli`]: gitflow_core::CoreError::Cli
//! [`PlatformCliError`]: gitflow_core::PlatformCliError

#![forbid(unsafe_code)]

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

pub use auth::GitHubAuthProvider;
pub use commit::GitHubCommitProvider;
pub use issue::GitHubIssueProvider;
pub use label::{GitHubLabelProvider, GitHubMilestoneProvider};
pub use pipeline::GitHubPipelineProvider;
pub use pr::GitHubPrProvider;
pub use release::GitHubReleaseProvider;
pub use review::GitHubReviewProvider;
