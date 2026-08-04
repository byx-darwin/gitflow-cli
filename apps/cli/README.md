# gf

[![Crates.io](https://img.shields.io/crates/v/gf)](https://crates.io/crates/gf)
[![Documentation](https://docs.rs/gf/badge.svg)](https://docs.rs/gf)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![CI](https://img.shields.io/github/actions/workflow/status/byx-darwin/gitflow-cli/ci.yml?branch=main)](https://github.com/byx-darwin/gitflow-cli/actions)

Cross-platform Git engineering workflow orchestration tool — manage Issues, PRs, Releases, and CI/CD pipelines across GitHub, GitLab, and GitCode from a unified CLI.

## Overview

`gf` is a powerful command-line tool that provides a unified interface for managing Git hosting platforms. Whether you're working with GitHub, GitLab, or GitCode, `gf` offers consistent commands for issue tracking, pull requests, releases, and more.

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
cargo install gf
```

### From GitHub Releases

Download pre-built binaries from the [releases page](https://github.com/byx-darwin/gitflow-cli/releases):

```bash
# macOS (Apple Silicon)
curl -LO https://github.com/byx-darwin/gitflow-cli/releases/latest/download/gf-aarch64-apple-darwin.tar.gz
tar -xzf gf-aarch64-apple-darwin.tar.gz
sudo mv gf /usr/local/bin/

# macOS (Intel)
curl -LO https://github.com/byx-darwin/gitflow-cli/releases/latest/download/gf-x86_64-apple-darwin.tar.gz
tar -xzf gf-x86_64-apple-darwin.tar.gz
sudo mv gf /usr/local/bin/

# Linux (x86_64)
curl -LO https://github.com/byx-darwin/gitflow-cli/releases/latest/download/gf-x86_64-unknown-linux-gnu.tar.gz
tar -xzf gf-x86_64-unknown-linux-gnu.tar.gz
sudo mv gf /usr/local/bin/
```

### From Source

```bash
git clone https://github.com/byx-darwin/gitflow-cli.git
cd gf
cargo build --release
cargo install --path apps/cli
```

## Quick Start

### Check Authentication

```bash
# Check if authenticated to platforms
gf auth status
```

### Issue Management

```bash
# List open issues
gf issue list --state open

# Create a new issue
gf issue create --title "Bug report" --body "Description"

# View issue details
gf issue view 42
```

### Pull Request Management

```bash
# List open PRs
gf pr list --state open

# Create a new PR
gf pr create --title "Feature" --body "Description" --source feature-branch --target main

# Review a PR
gf pr review 42 --approve --comment "LGTM!"
```

### Release Management

```bash
# List releases
gf release list

# Create a new release
gf release create --tag v1.0.0 --name "Release v1.0.0" --notes "Release notes"
```

### Pipeline Status

```bash
# Check pipeline status
gf pipeline status
```

## Commands

### `auth` — Authentication

```bash
gf auth status              # Check authentication status
gf auth login               # Login to platform
```

### `issue` — Issue Management

```bash
gf issue list               # List issues
gf issue create             # Create new issue
gf issue view <number>      # View issue details
gf issue update <number>    # Update issue
gf issue close <number>     # Close issue
```

### `pr` — Pull Request Management

```bash
gf pr list                  # List pull requests
gf pr create                # Create new PR
gf pr view <number>         # View PR details
gf pr review <number>       # Review PR
gf pr merge <number>        # Merge PR
```

### `release` — Release Management

```bash
gf release list             # List releases
gf release create           # Create new release
gf release view <tag>       # View release details
gf release upload <tag>     # Upload assets
```

### `review` — Code Review

```bash
gf review list              # List reviews
gf review submit            # Submit review
```

### `pipeline` — CI/CD Pipelines

```bash
gf pipeline status          # Check pipeline status
```

### `completions` — Shell Completions

```bash
gf completions --install    # Install shell completions
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

`gf` uses platform-specific CLIs for authentication:

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
gf completions --install

# Or manually generate
gf completions --shell bash > ~/.bash_completion.d/gf
gf completions --shell zsh > ~/.zsh/completions/_gf
gf completions --shell fish > ~/.config/fish/completions/gf.fish
```

## JSON Output

Use `--json` flag for machine-readable output:

```bash
gf issue list --json
gf pr view 42 --json
```

## Architecture

```
┌─────────────────────────────────────────┐
│           gf (CLI)             │
└──────────────────┬──────────────────────┘
                   │ uses
                   ▼
┌─────────────────────────────────────────┐
│         gf-core (traits)       │
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

This crate is part of the gf workspace:

- [gf-core](https://crates.io/crates/gf-core) — Core types and traits
- [gf-github](https://crates.io/crates/gf-github) — GitHub platform implementation
- [gf-gitlab](https://crates.io/crates/gf-gitlab) — GitLab platform implementation
- [gf-gitcode](https://crates.io/crates/gf-gitcode) — GitCode platform implementation
- **gf** (this crate) — CLI application

## Documentation

- [API Documentation](https://docs.rs/gf)
- [Main Project](https://github.com/byx-darwin/gitflow-cli)
- [User Guide](https://github.com/byx-darwin/gitflow-cli#readme)

## Contributing

Contributions are welcome! Please see the [main repository](https://github.com/byx-darwin/gitflow-cli) for contribution guidelines.

## License

Licensed under the MIT License. See [LICENSE](https://github.com/byx-darwin/gitflow-cli/blob/main/LICENSE) for details.
