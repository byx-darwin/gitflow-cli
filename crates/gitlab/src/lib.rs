//! `gitflow-cli-gitlab` -- GitLab platform implementation.
//!
//! This crate implements the `IssueProvider`, `PrProvider`,
//! `ReleaseProvider`, `ReviewProvider`, `AuthProvider`, `LabelProvider`,
//! `MilestoneProvider`, `CommitProvider` and `PipelineProvider` traits defined
//! in `gitflow-cli-core`, by calling the `glab` CLI to fetch data and parse
//! its JSON output.
//!
//! # Main Types
//!
//! - `GitLabIssueProvider`: Operate GitLab Issues.
//! - `GitLabMrProvider`: Operate GitLab Merge Requests (implements `PrProvider`).
//! - `GitLabReleaseProvider`: Operate GitLab Releases.
//! - `GitLabReviewProvider`: Operate GitLab MR Reviews.
//! - `GitLabAuthProvider`: Handle GitLab authentication (login, logout, status, token).
//! - `GitLabLabelProvider`: Manage GitLab repository labels.
//! - `GitLabMilestoneProvider`: Manage GitLab repository milestones.
//! - `GitLabCommitProvider`: View GitLab commits and diff/patch.
//! - `GitLabPipelineProvider`: View GitLab CI/CD pipelines.

#![forbid(unsafe_code)]
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
pub mod mr;
pub mod pipeline;
pub mod release;
pub mod review;

pub use auth::GitLabAuthProvider;
pub use commit::GitLabCommitProvider;
pub use error::GlabError;
pub use issue::GitLabIssueProvider;
pub use label::{GitLabLabelProvider, GitLabMilestoneProvider};
pub use mr::GitLabMrProvider;
pub use pipeline::GitLabPipelineProvider;
pub use release::GitLabReleaseProvider;
pub use review::GitLabReviewProvider;
