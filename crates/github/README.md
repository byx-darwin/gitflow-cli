# gf-github

[![Crates.io](https://img.shields.io/crates/v/gf-github)](https://crates.io/crates/gf-github)
[![Documentation](https://docs.rs/gf-github/badge.svg)](https://docs.rs/gf-github)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

GitHub platform implementation for [gf](https://github.com/byx-darwin/gitflow-cli) — Issue, PR, Release, and Review providers via `gh` CLI.

## Overview

`gf-github` provides GitHub-specific implementations of the core platform traits defined in `gf-core`. It uses the official [`gh`](https://cli.github.com/) CLI under the hood for all GitHub API interactions.

## Features

- **Issue Provider**: Create, list, view, and manage GitHub Issues
- **PR Provider**: Create, review, and manage Pull Requests
- **Release Provider**: Create and manage GitHub Releases
- **Review Provider**: Submit and manage code reviews
- **Pipeline Provider**: Monitor GitHub Actions workflows
- **Authentication**: Leverages `gh auth` for authentication
- **Async Support**: Full async/await support with Tokio

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
gf-github = "0.8"
```

### Prerequisites

This crate requires the GitHub CLI (`gh`) to be installed and authenticated:

```bash
# Install gh CLI
brew install gh  # macOS
# See: https://cli.github.com/ for other platforms

# Authenticate
gh auth login
```

## Usage

### Creating a GitHub Provider

```rust
use gitflow_cli_github::GitHubProvider;
use gitflow_cli_core::IssueProvider;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = GitHubProvider::new()?;

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
use gitflow_cli_github::GitHubProvider;
use gitflow_cli_core::{IssueProvider, IssueState};

let provider = GitHubProvider::new()?;

// Create an issue
let issue = provider.create_issue(
    "owner/repo",
    "Bug report",
    "Description of the bug",
    &["bug", "priority:high"],
).await?;

// Update issue state
provider.update_issue_state(
    "owner/repo",
    issue.number,
    IssueState::Closed,
).await?;
```

### Working with Pull Requests

```rust
use gitflow_cli_github::GitHubProvider;
use gitflow_cli_core::PrProvider;

let provider = GitHubProvider::new()?;

// Create a PR
let pr = provider.create_pr(
    "owner/repo",
    "feature-branch",
    "main",
    "Add new feature",
    "Detailed description",
).await?;

// Merge PR
provider.merge_pr("owner/repo", pr.number).await?;
```

### Working with Releases

```rust
use gitflow_cli_github::GitHubProvider;
use gitflow_cli_core::ReleaseProvider;

let provider = GitHubProvider::new()?;

// Create a release
let release = provider.create_release(
    "owner/repo",
    "v1.0.0",
    "Release v1.0.0",
    "Release notes here",
).await?;

// List releases
let releases = provider.list_releases("owner/repo").await?;
```

### Code Reviews

```rust
use gitflow_cli_github::GitHubProvider;
use gitflow_cli_core::{ReviewProvider, ReviewState};

let provider = GitHubProvider::new()?;

// Submit a review
provider.submit_review(
    "owner/repo",
    42,  // PR number
    ReviewState::Approve,
    "Looks good!",
).await?;
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
│    gf-github (this crate)  │
│      GitHubProvider                 │
└──────────────────┬──────────────────┘
                   │ uses
                   ▼
┌─────────────────────────────────────┐
│         gh CLI (GitHub API)         │
└─────────────────────────────────────┘
```

## Error Handling

All operations return `Result<T, gitflow_cli_core::Error>`:

```rust
use gitflow_cli_github::GitHubProvider;
use gitflow_cli_core::Error;

let provider = GitHubProvider::new()?;
match provider.list_issues("owner/repo", "open").await {
    Ok(issues) => { /* handle issues */ },
    Err(Error::NotFound) => { /* repo not found */ },
    Err(Error::Auth) => { /* authentication failed */ },
    Err(e) => { /* other error */ },
}
```

## Environment Variables

- `GH_TOKEN` / `GITHUB_TOKEN` — GitHub authentication token (optional if `gh auth login` was used)
- `GH_HOST` — GitHub Enterprise hostname (optional)

## Ecosystem

This crate is part of the gf workspace:

- [gf-core](https://crates.io/crates/gf-core) — Core types and traits
- **gf-github** (this crate) — GitHub platform implementation
- [gf-gitlab](https://crates.io/crates/gf-gitlab) — GitLab platform implementation
- [gf-gitcode](https://crates.io/crates/gf-gitcode) — GitCode platform implementation
- [gf](https://crates.io/crates/gf) — CLI application

## Documentation

- [API Documentation](https://docs.rs/gf-github)
- [Main Project](https://github.com/byx-darwin/gitflow-cli)
- [GitHub CLI Documentation](https://cli.github.com/manual/)

## Contributing

Contributions are welcome! Please see the [main repository](https://github.com/byx-darwin/gitflow-cli) for contribution guidelines.

## License

Licensed under the MIT License. See [LICENSE](https://github.com/byx-darwin/gitflow-cli/blob/main/LICENSE) for details.
