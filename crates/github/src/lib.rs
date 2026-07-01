//! `gitflow-cli-github` —— GitHub 平台实现。
//!
//! 本 crate 实现了 `gitflow-cli-core` 中定义的 [`IssueProvider`] 与 [`PrProvider`] trait，
//! 通过调用 `gh` CLI 获取数据并解析其 JSON 输出。
//!
//! # 主要类型
//!
//! - [`GitHubIssueProvider`]：操作 GitHub Issue。
//! - [`GitHubPrProvider`]：操作 GitHub Pull Request。
//!
//! # 错误处理
//!
//! 所有平台调用失败时，`gh` 的 stderr 会通过 [`error::parse_gh_error`] 解析，
//! 并统一映射为 [`CoreError::Platform`]。
//!
//! [`IssueProvider`]: gitflow_cli_core::issue::IssueProvider
//! [`PrProvider`]: gitflow_cli_core::pr::PrProvider
//! [`CoreError::Platform`]: gitflow_cli_core::CoreError::Platform

#![forbid(unsafe_code)]

pub mod error;
pub mod issue;
pub mod pr;

pub use error::GhError;
pub use issue::GitHubIssueProvider;
pub use pr::GitHubPrProvider;
