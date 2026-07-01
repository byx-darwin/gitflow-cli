//! `gitflow-cli-github` —— GitHub 平台实现。
//!
//! 本 crate 实现了 `gitflow-cli-core` 中定义的 [`IssueProvider`]、[`PrProvider`]、
//! [`ReleaseProvider`] 与 [`ReviewProvider`] trait，
//! 通过调用 `gh` CLI 获取数据并解析其 JSON 输出。
//!
//! # 主要类型
//!
//! - [`GitHubIssueProvider`]：操作 GitHub Issue。
//! - [`GitHubPrProvider`]：操作 GitHub Pull Request。
//! - [`GitHubReleaseProvider`]：操作 GitHub Release。
//! - [`GitHubReviewProvider`]：操作 GitHub PR Review。
//!
//! # 错误处理
//!
//! 所有平台调用失败时，`gh` 的 stderr 会通过 [`error::parse_gh_error`] 解析，
//! 并统一映射为 [`CoreError::Platform`]。
//!
//! [`IssueProvider`]: gitflow_cli_core::issue::IssueProvider
//! [`PrProvider`]: gitflow_cli_core::pr::PrProvider
//! [`ReleaseProvider`]: gitflow_cli_core::release::ReleaseProvider
//! [`ReviewProvider`]: gitflow_cli_core::review::ReviewProvider
//! [`CoreError::Platform`]: gitflow_cli_core::CoreError::Platform

#![forbid(unsafe_code)]

pub mod error;
pub mod issue;
pub mod pr;
pub mod release;
pub mod review;

pub use error::GhError;
pub use issue::GitHubIssueProvider;
pub use pr::GitHubPrProvider;
pub use release::GitHubReleaseProvider;
pub use review::GitHubReviewProvider;
