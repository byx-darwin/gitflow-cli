# gitflow-core

[![Crates.io](https://img.shields.io/crates/v/gitflow-core)](https://crates.io/crates/gitflow-core)
[![Documentation](https://docs.rs/gitflow-core/badge.svg)](https://docs.rs/gitflow-core)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

Core domain types, traits, and business logic for the [gf](https://github.com/byx-darwin/gitflow-cli) workspace.

## Overview

`gitflow-core` provides the foundational types and abstractions used across the gf ecosystem. This crate is designed to be platform-agnostic, defining the core interfaces that platform-specific implementations (GitHub, GitLab, GitCode) must implement.

## Features

- **Domain Types**: Issue, PR, Release, Review, Pipeline, Label, and more
- **Platform Abstractions**: Traits for cross-platform Git hosting services
- **Error Handling**: Consistent error types with `thiserror`
- **Output Formatting**: CLI output utilities with `miette` integration
- **Auth Checking**: Authentication status verification utilities
- **Type Safety**: Strongly typed enums and newtypes for domain concepts

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
gitflow-core = "0.8"
```

## Usage

### Domain Types

```rust
use gf_core::{Issue, IssueState, IssueProvider};

// Work with issues
let issue = Issue {
    number: 42,
    title: "Fix bug".to_string(),
    state: IssueState::Open,
    // ...
};
```

### Platform Traits

```rust
use gf_core::{IssueProvider, PrProvider, ReleaseProvider};

// Implement platform-specific providers
struct MyPlatform;

impl IssueProvider for MyPlatform {
    // Implement issue operations
}

impl PrProvider for MyPlatform {
    // Implement PR operations
}
```

### Error Handling

```rust
use gf_core::Error;

fn do_something() -> Result<(), Error> {
    // Use the core error type
    Ok(())
}
```

### Output Formatting

```rust
use gf_core::{CliOutput, CliError};

// Format CLI output
let output = CliOutput::success("Operation completed");
println!("{}", output);

// Handle errors
let error = CliError::new("Something went wrong");
eprintln!("{}", error);
```

## Core Modules

### `issue`
Issue tracking types and operations:
- `Issue` - Issue data structure
- `IssueState` - Open/Closed state
- `IssueProvider` - Provider trait for issue operations

### `pr`
Pull request types and operations:
- `PullRequest` - PR data structure
- `PrState` - Open/Closed/Merged state
- `PrProvider` - Provider trait for PR operations

### `release`
Release management types:
- `Release` - Release data structure
- `ReleaseProvider` - Provider trait for release operations

### `review`
Code review types:
- `Review` - Review data structure
- `ReviewProvider` - Provider trait for review operations

### `pipeline`
CI/CD pipeline types:
- `Pipeline` - Pipeline data structure
- `PipelineStatus` - Success/Failure/Running status
- `PipelineProvider` - Provider trait for pipeline operations

### `platform`
URL-based platform detection (not a capability trait — see the provider
traits above for the actual per-platform abstraction):
- `Platform` - Platform enum (GitHub/GitLab/GitCode)
- `PlatformDetection` - detection result, with `Platform::detect_from_remote_url()`

### `auth`
Authentication types and checking:
- `AuthChecker` - Authentication status verification
- `AuthCheckResult` - Auth check results

## Design Principles

1. **Platform Agnostic**: Core types work across all supported platforms
2. **Type Safety**: Use enums and newtypes to prevent invalid states
3. **Error Handling**: Consistent error types with context
4. **No I/O**: Pure domain logic without network or file I/O
5. **Serializable**: All types support `serde` for JSON/YAML/TOML

## Ecosystem

This crate is part of the gf workspace:

- **gitflow-core** (this crate) - Core types and traits
- [gitflow-github](https://crates.io/crates/gitflow-github) - GitHub platform implementation
- [gitflow-gitlab](https://crates.io/crates/gitflow-gitlab) - GitLab platform implementation
- [gitflow-gitcode](https://crates.io/crates/gitflow-gitcode) - GitCode platform implementation
- [gf](https://crates.io/crates/gf) - CLI application

## Documentation

- [API Documentation](https://docs.rs/gitflow-core)
- [Main Project](https://github.com/byx-darwin/gitflow-cli)
- [User Guide](https://github.com/byx-darwin/gitflow-cli#readme)

## Contributing

Contributions are welcome! Please see the [main repository](https://github.com/byx-darwin/gitflow-cli) for contribution guidelines.

## License

Licensed under the MIT License. See [LICENSE](https://github.com/byx-darwin/gitflow-cli/blob/main/LICENSE) for details.
