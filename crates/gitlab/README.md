# gitflow-cli-gitlab

[![Crates.io](https://img.shields.io/crates/v/gitflow-cli-gitlab)](https://crates.io/crates/gitflow-cli-gitlab)
[![Documentation](https://docs.rs/gitflow-cli-gitlab/badge.svg)](https://docs.rs/gitflow-cli-gitlab)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

GitLab platform implementation for [gitflow-cli](https://github.com/byx-darwin/gitflow-cli) — Issue, MR, Release, and Review providers via `glab` CLI.

## Overview

`gitflow-cli-gitlab` provides GitLab-specific implementations of the core platform traits defined in `gitflow-cli-core`. It uses the official [`glab`](https://gitlab.com/gitlab-org/cli) CLI under the hood for all GitLab API interactions.

## Features

- **Issue Provider**: Create, list, view, and manage GitLab Issues
- **MR Provider**: Create, review, and manage Merge Requests (GitLab's PRs)
- **Release Provider**: Create and manage GitLab Releases
- **Review Provider**: Submit and manage code reviews
- **Pipeline Provider**: Monitor GitLab CI/CD pipelines
- **Authentication**: Leverages `glab auth` for authentication
- **Async Support**: Full async/await support with Tokio

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
gitflow-cli-gitlab = "0.8"
```

### Prerequisites

This crate requires the GitLab CLI (`glab`) to be installed and authenticated:

```bash
# Install glab CLI
brew install glab  # macOS
# See: https://gitlab.com/gitlab-org/cli for other platforms

# Authenticate
glab auth login
```

## Usage

### Creating a GitLab Provider

```rust
use gitflow_cli_gitlab::GitLabProvider;
use gitflow_cli_core::IssueProvider;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = GitLabProvider::new()?;

    // List open issues
    let issues = provider.list_issues("owner/repo", "open").await?;
    for issue in issues {
        println!("#{}: {}", issue.number, issue.title);
    }

    Ok(())
}
```

### Working with Issues

```rust
use gitflow_cli_gitlab::GitLabProvider;
use gitflow_cli_core::{IssueProvider, IssueState};

let provider = GitLabProvider::new()?;

// Create an issue
let issue = provider.create_issue(
    "owner/repo",
    "Bug report",
    "Description of the bug",
    &["bug", "priority::high"],
).await?;

// Update issue state
provider.update_issue_state(
    "owner/repo",
    issue.number,
    IssueState::Closed,
).await?;
```

### Working with Merge Requests

```rust
use gitflow_cli_gitlab::GitLabProvider;
use gitflow_cli_core::PrProvider;

let provider = GitLabProvider::new()?;

// Create a merge request
let mr = provider.create_pr(
    "owner/repo",
    "feature-branch",
    "main",
    "Add new feature",
    "Detailed description",
).await?;

// Merge MR
provider.merge_pr("owner/repo", mr.number).await?;
```

### Working with Pipelines

```rust
use gitflow_cli_gitlab::GitLabProvider;
use gitflow_cli_core::{PipelineProvider, PipelineStatus};

let provider = GitLabProvider::new()?;

// Get pipeline status
let pipeline = provider.get_pipeline("owner/repo", "main").await?;
println!("Pipeline status: {:?}", pipeline.status);
```

## Architecture

```
┌─────────────────────────────────────┐
│     gitflow-cli-core (traits)       │
│  IssueProvider, PrProvider, etc.    │
└──────────────────┬──────────────────┘
                   │ implements
                   ▼
┌─────────────────────────────────────┐
│    gitflow-cli-gitlab (this crate)  │
│      GitLabProvider                 │
└──────────────────┬──────────────────┘
                   │ uses
                   ▼
┌─────────────────────────────────────┐
│        glab CLI (GitLab API)        │
└─────────────────────────────────────┘
```

## Error Handling

All operations return `Result<T, gitflow_cli_core::Error>`:

```rust
use gitflow_cli_gitlab::GitLabProvider;
use gitflow_cli_core::Error;

let provider = GitLabProvider::new()?;
match provider.list_issues("owner/repo", "open").await {
    Ok(issues) => { /* handle issues */ },
    Err(Error::NotFound) => { /* project not found */ },
    Err(Error::Auth) => { /* authentication failed */ },
    Err(e) => { /* other error */ },
}
```

## Environment Variables

- `GITLAB_TOKEN` — GitLab authentication token (optional if `glab auth login` was used)
- `GITLAB_HOST` — GitLab self-hosted instance hostname (optional, defaults to gitlab.com)

## Ecosystem

This crate is part of the gitflow-cli workspace:

- [gitflow-cli-core](https://crates.io/crates/gitflow-cli-core) — Core types and traits
- [gitflow-cli-github](https://crates.io/crates/gitflow-cli-github) — GitHub platform implementation
- **gitflow-cli-gitlab** (this crate) — GitLab platform implementation
- [gitflow-cli-gitcode](https://crates.io/crates/gitflow-cli-gitcode) — GitCode platform implementation
- [gitflow-cli](https://crates.io/crates/gitflow-cli) — CLI application

## Documentation

- [API Documentation](https://docs.rs/gitflow-cli-gitlab)
- [Main Project](https://github.com/byx-darwin/gitflow-cli)
- [GitLab CLI Documentation](https://gitlab.com/gitlab-org/cli/-/tree/main/docs)

## Contributing

Contributions are welcome! Please see the [main repository](https://github.com/byx-darwin/gitflow-cli) for contribution guidelines.

## License

Licensed under the MIT License. See [LICENSE](https://github.com/byx-darwin/gitflow-cli/blob/main/LICENSE) for details.
