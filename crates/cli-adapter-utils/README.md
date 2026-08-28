# gitflow-cli-adapter-utils

Shared CLI adapter utilities for gitflow-cli platform adapters.

## Overview

This crate provides common types and traits used by the GitHub, GitLab, and GitCode adapter crates in the gitflow-cli workspace:

- `CommandOutput` — structured output from CLI command execution
- `CommandRunner` — trait for testable process spawning
- `RealCommandRunner` — production implementation using `tokio::process::Command`

## Usage

Platform adapter crates re-export these types for use in their business logic:

```rust
pub use gitflow_cli_adapter_utils::{CommandOutput, CommandRunner, RealCommandRunner};
```

## License

MIT
