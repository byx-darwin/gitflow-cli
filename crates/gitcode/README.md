# gitflow-gitcode

[![Crates.io](https://img.shields.io/crates/v/gitflow-gitcode)](https://crates.io/crates/gitflow-gitcode)
[![Documentation](https://docs.rs/gitflow-gitcode/badge.svg)](https://docs.rs/gitflow-gitcode)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

GitCode platform implementation for [gf](https://github.com/byx-darwin/gitflow-cli) — Issue, PR, Release, and Review providers via `gc` CLI.

## Overview

`gitflow-gitcode` provides GitCode-specific implementations of the core platform traits defined in `gitflow-core`. It uses the [`gc`](https://gitcode.com) CLI under the hood for all GitCode API interactions.

## Features

- **Issue Provider**: Create, list, view, and manage GitCode Issues
- **PR Provider**: Create, review, and manage Pull Requests
- **Release Provider**: Create and manage GitCode Releases
- **Review Provider**: Submit and manage code reviews
- **Pipeline Provider**: Monitor GitCode CI/CD pipelines
- **Authentication**: Leverages `gc auth` for authentication
- **Async Support**: Full async/await support with Tokio

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
gitflow-gitcode = "0.8"
```

### Prerequisites

This crate requires the GitCode CLI (`gc`) to be installed and authenticated:

```bash
# Install gc CLI (if available)
# See: https://gitcode.com for installation instructions

# Authenticate
gc auth login
```

## Usage

### Creating a GitCode Provider

```rust
use gf_gitcode::GitCodeProvider;
use gf_core::IssueProvider;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = GitCodeProvider::new()?;

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
use gf_gitcode::GitCodeProvider;
use gf_core::{IssueProvider, IssueState};

let provider = GitCodeProvider::new()?;

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
use gf_gitcode::GitCodeProvider;
use gf_core::PrProvider;

let provider = GitCodeProvider::new()?;

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

## Architecture

```
┌─────────────────────────────────────┐
│     gitflow-core (traits)       │
│  IssueProvider, PrProvider, etc.    │
└──────────────────┬──────────────────┘
                   │ implements
                   ▼
┌─────────────────────────────────────┐
│   gitflow-gitcode (this crate)  │
│      GitCodeProvider                │
└──────────────────┬──────────────────┘
                   │ uses
                   ▼
┌─────────────────────────────────────┐
│         gc CLI (GitCode API)        │
└─────────────────────────────────────┘
```

## Error Handling

All operations return `Result<T, gf_core::Error>`:

```rust
use gf_gitcode::GitCodeProvider;
use gf_core::Error;

let provider = GitCodeProvider::new()?;
match provider.list_issues("owner/repo", "open").await {
    Ok(issues) => { /* handle issues */ },
    Err(Error::NotFound) => { /* repo not found */ },
    Err(Error::Auth) => { /* authentication failed */ },
    Err(e) => { /* other error */ },
}
```

## Environment Variables

- `GITCODE_TOKEN` — GitCode authentication token (optional if `gc auth login` was used)
- `GITCODE_HOST` — GitCode instance hostname (optional)

## Ecosystem

This crate is part of the gf workspace:

- [gitflow-core](https://crates.io/crates/gitflow-core) — Core types and traits
- [gitflow-github](https://crates.io/crates/gitflow-github) — GitHub platform implementation
- [gitflow-gitlab](https://crates.io/crates/gitflow-gitlab) — GitLab platform implementation
- **gitflow-gitcode** (this crate) — GitCode platform implementation
- [gf](https://crates.io/crates/gf) — CLI application

## Documentation

- [API Documentation](https://docs.rs/gitflow-gitcode)
- [Main Project](https://github.com/byx-darwin/gitflow-cli)

## Contributing

Contributions are welcome! Please see the [main repository](https://github.com/byx-darwin/gitflow-cli) for contribution guidelines.

## License

Licensed under the MIT License. See [LICENSE](https://github.com/byx-darwin/gitflow-cli/blob/main/LICENSE) for details.
