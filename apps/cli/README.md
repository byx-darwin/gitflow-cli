# gitflow-cli

[![Crates.io](https://img.shields.io/crates/v/gitflow-cli)](https://crates.io/crates/gitflow-cli)
[![Documentation](https://docs.rs/gitflow-cli/badge.svg)](https://docs.rs/gitflow-cli)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![CI](https://img.shields.io/github/actions/workflow/status/byx-darwin/gitflow-cli/ci.yml?branch=main)](https://github.com/byx-darwin/gitflow-cli/actions)

Cross-platform Git engineering workflow orchestration tool — manage Issues, PRs, Releases, and CI/CD pipelines across GitHub, GitLab, and GitCode from a unified CLI.

## Overview

`gitflow-cli` is a powerful command-line tool that provides a unified interface for managing Git hosting platforms. Whether you're working with GitHub, GitLab, or GitCode, `gitflow-cli` offers consistent commands for issue tracking, pull requests, releases, and more.

## Features

- **Multi-Platform Support**: GitHub, GitLab, and GitCode with a single CLI
- **Issue Management**: Create, list, view, and manage issues
- **Pull Requests**: Create, review, merge, and manage PRs/MRs
- **Release Management**: Create releases with auto-generated changelogs
- **Code Reviews**: Submit and manage code reviews
- **Pipeline Monitoring**: Check CI/CD pipeline status
- **Authentication Checking**: Verify platform authentication status
- **Shell Completions**: Auto-completion for bash, zsh, and fish
- **JSON Output**: Machine-readable output for scripting

## Installation

### From crates.io

```bash
cargo install gitflow-cli
```

### From GitHub Releases

Download pre-built binaries from the [releases page](https://github.com/byx-darwin/gitflow-cli/releases):

```bash
# macOS (Apple Silicon)
curl -LO https://github.com/byx-darwin/gitflow-cli/releases/latest/download/gitflow-cli-aarch64-apple-darwin.tar.gz
tar -xzf gitflow-cli-aarch64-apple-darwin.tar.gz
sudo mv gitflow-cli /usr/local/bin/

# macOS (Intel)
curl -LO https://github.com/byx-darwin/gitflow-cli/releases/latest/download/gitflow-cli-x86_64-apple-darwin.tar.gz
tar -xzf gitflow-cli-x86_64-apple-darwin.tar.gz
sudo mv gitflow-cli /usr/local/bin/

# Linux (x86_64)
curl -LO https://github.com/byx-darwin/gitflow-cli/releases/latest/download/gitflow-cli-x86_64-unknown-linux-gnu.tar.gz
tar -xzf gitflow-cli-x86_64-unknown-linux-gnu.tar.gz
sudo mv gitflow-cli /usr/local/bin/
```

### From Source

```bash
git clone https://github.com/byx-darwin/gitflow-cli.git
cd gitflow-cli
cargo build --release
cargo install --path apps/cli
```

## Quick Start

### Check Authentication

```bash
# Check if authenticated to platforms
gitflow-cli auth status
```

### Issue Management

```bash
# List open issues
gitflow-cli issue list --state open

# Create a new issue
gitflow-cli issue create --title "Bug report" --body "Description"

# View issue details
gitflow-cli issue view 42
```

### Pull Request Management

```bash
# List open PRs
gitflow-cli pr list --state open

# Create a new PR
gitflow-cli pr create --title "Feature" --body "Description" --source feature-branch --target main

# Review a PR
gitflow-cli pr review 42 --approve --comment "LGTM!"
```

### Release Management

```bash
# List releases
gitflow-cli release list

# Create a new release
gitflow-cli release create --tag v1.0.0 --name "Release v1.0.0" --notes "Release notes"
```

### Pipeline Status

```bash
# Check pipeline status
gitflow-cli pipeline status
```

## Commands

### `auth` — Authentication

```bash
gitflow-cli auth status              # Check authentication status
gitflow-cli auth login               # Login to platform
```

### `issue` — Issue Management

```bash
gitflow-cli issue list               # List issues
gitflow-cli issue create             # Create new issue
gitflow-cli issue view <number>      # View issue details
gitflow-cli issue update <number>    # Update issue
gitflow-cli issue close <number>     # Close issue
```

### `pr` — Pull Request Management

```bash
gitflow-cli pr list                  # List pull requests
gitflow-cli pr create                # Create new PR
gitflow-cli pr view <number>         # View PR details
gitflow-cli pr review <number>       # Review PR
gitflow-cli pr merge <number>        # Merge PR
```

### `release` — Release Management

```bash
gitflow-cli release list             # List releases
gitflow-cli release create           # Create new release
gitflow-cli release view <tag>       # View release details
gitflow-cli release upload <tag>     # Upload assets
```

### `review` — Code Review

```bash
gitflow-cli review list              # List reviews
gitflow-cli review submit            # Submit review
```

### `pipeline` — CI/CD Pipelines

```bash
gitflow-cli pipeline status          # Check pipeline status
```

### `completions` — Shell Completions

```bash
gitflow-cli completions --install    # Install shell completions
```

## Platform Support

| Feature | GitHub | GitLab | GitCode |
|---------|--------|--------|---------|
| Issues | ✅ | ✅ | ✅ |
| Pull Requests | ✅ | ✅ (MRs) | ✅ |
| Releases | ✅ | ✅ | ✅ |
| Reviews | ✅ | ✅ | ✅ |
| Pipelines | ✅ | ✅ | ✅ |
| Authentication | ✅ (gh) | ✅ (glab) | ✅ (gc) |

## Configuration

`gitflow-cli` uses platform-specific CLIs for authentication:

- **GitHub**: [`gh`](https://cli.github.com/) CLI
- **GitLab**: [`glab`](https://gitlab.com/gitlab-org/cli) CLI
- **GitCode**: [`gc`](https://gitcode.com) CLI

Install and authenticate with the appropriate CLI for your platform.

## Environment Variables

- `GITHUB_TOKEN` / `GH_TOKEN` — GitHub authentication token
- `GITLAB_TOKEN` — GitLab authentication token
- `GITCODE_TOKEN` — GitCode authentication token
- `GITFLOW_PLATFORM` — Force specific platform (github/gitlab/gitcode)

## Shell Completions

Auto-completion is available for bash, zsh, and fish:

```bash
# Install completions
gitflow-cli completions --install

# Or manually generate
gitflow-cli completions --shell bash > ~/.bash_completion.d/gitflow-cli
gitflow-cli completions --shell zsh > ~/.zsh/completions/_gitflow-cli
gitflow-cli completions --shell fish > ~/.config/fish/completions/gitflow-cli.fish
```

## JSON Output

Use `--json` flag for machine-readable output:

```bash
gitflow-cli issue list --json
gitflow-cli pr view 42 --json
```

## Architecture

```
┌─────────────────────────────────────────┐
│           gitflow-cli (CLI)             │
└──────────────────┬──────────────────────┘
                   │ uses
                   ▼
┌─────────────────────────────────────────┐
│         gitflow-cli-core (traits)       │
└──────────────────┬──────────────────────┘
                   │ implementations
        ┌──────────┼──────────┐
        ▼          ▼          ▼
   ┌────────┐ ┌────────┐ ┌────────┐
   │ GitHub │ │ GitLab │ │GitCode │
   └────┬───┘ └────┬───┘ └────┬───┘
        │          │          │
        ▼          ▼          ▼
      gh CLI    glab CLI    gc CLI
```

## Ecosystem

This crate is part of the gitflow-cli workspace:

- [gitflow-cli-core](https://crates.io/crates/gitflow-cli-core) — Core types and traits
- [gitflow-cli-github](https://crates.io/crates/gitflow-cli-github) — GitHub platform implementation
- [gitflow-cli-gitlab](https://crates.io/crates/gitflow-cli-gitlab) — GitLab platform implementation
- [gitflow-cli-gitcode](https://crates.io/crates/gitflow-cli-gitcode) — GitCode platform implementation
- **gitflow-cli** (this crate) — CLI application

## Documentation

- [API Documentation](https://docs.rs/gitflow-cli)
- [Main Project](https://github.com/byx-darwin/gitflow-cli)
- [User Guide](https://github.com/byx-darwin/gitflow-cli#readme)

## Contributing

Contributions are welcome! Please see the [main repository](https://github.com/byx-darwin/gitflow-cli) for contribution guidelines.

## License

Licensed under the MIT License. See [LICENSE](https://github.com/byx-darwin/gitflow-cli/blob/main/LICENSE) for details.
