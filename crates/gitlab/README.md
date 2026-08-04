# gf-gitlab

[![Crates.io](https://img.shields.io/crates/v/gf-gitlab)](https://crates.io/crates/gf-gitlab)
[![Documentation](https://docs.rs/gf-gitlab/badge.svg)](https://docs.rs/gf-gitlab)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

GitLab platform implementation for [gf](https://github.com/byx-darwin/gitflow-cli) — Issue, MR, Release, and Review providers via `glab` CLI.

## Overview

`gf-gitlab` provides GitLab-specific implementations of the core platform traits defined in `gf-core`. It uses the official [`glab`](https://gitlab.com/gitlab-org/cli) CLI under the hood for all GitLab API interactions.

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
gf-gitlab = "0.8"
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
use gf_gitlab::GitLabProvider;
use gf_core::IssueProvider;

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
use gf_gitlab::GitLabProvider;
use gf_core::{IssueProvider, IssueState};

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
use gf_gitlab::GitLabProvider;
use gf_core::PrProvider;

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
use gf_gitlab::GitLabProvider;
use gf_core::{PipelineProvider, PipelineStatus};

let provider = GitLabProvider::new()?;

// Get pipeline status
let pipeline = provider.get_pipeline("owner/repo", "main").await?;
println!("Pipeline status: {:?}", pipeline.status);
```

## Architecture

```
┌─────────────────────────────────────┐
│     gf-core (traits)       │
│  IssueProvider, PrProvider, etc.    │
└──────────────────┬──────────────────┘
                   │ implements
                   ▼
┌─────────────────────────────────────┐
│    gf-gitlab (this crate)  │
│      GitLabProvider                 │
└──────────────────┬──────────────────┘
                   │ uses
                   ▼
┌─────────────────────────────────────┐
│        glab CLI (GitLab API)        │
└─────────────────────────────────────┘
```

## Error Handling

All operations return `Result<T, gf_core::Error>`:

```rust
use gf_gitlab::GitLabProvider;
use gf_core::Error;

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

This crate is part of the gf workspace:

- [gf-core](https://crates.io/crates/gf-core) — Core types and traits
- [gf-github](https://crates.io/crates/gf-github) — GitHub platform implementation
- **gf-gitlab** (this crate) — GitLab platform implementation
- [gf-gitcode](https://crates.io/crates/gf-gitcode) — GitCode platform implementation
- [gf](https://crates.io/crates/gf) — CLI application

## Documentation

- [API Documentation](https://docs.rs/gf-gitlab)
- [Main Project](https://github.com/byx-darwin/gitflow-cli)
- [GitLab CLI Documentation](https://gitlab.com/gitlab-org/cli/-/tree/main/docs)

## Contributing

Contributions are welcome! Please see the [main repository](https://github.com/byx-darwin/gitflow-cli) for contribution guidelines.

## License

Licensed under the MIT License. See [LICENSE](https://github.com/byx-darwin/gitflow-cli/blob/main/LICENSE) for details.
