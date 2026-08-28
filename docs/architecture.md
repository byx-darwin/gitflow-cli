# Architecture

This document describes the workspace layout of `gf` and the rationale behind it.

## Workspace Layout

```
gf/
├── apps/
│   └── cli/                  # Binary crate — gf CLI entry point
├── crates/
│   ├── core/                 # Library crate — traits, domain types, core logic
│   ├── github/               # Library crate — GitHub platform adapter (gh CLI)
│   ├── gitlab/               # Library crate — GitLab platform adapter (glab CLI)
│   ├── gitcode/              # Library crate — GitCode platform adapter (gc CLI)
│   ├── e2e-core/             # Test library — shared E2E test utilities
│   ├── e2e-github/           # Test crate — GitHub platform E2E tests
│   └── release-signer/       # Binary crate — Ed25519 release asset signing
├── docs/                     # Project documentation
├── specs/                    # Feature specifications
├── Cargo.toml                # Workspace manifest
├── Makefile                  # Automation targets
└── CLAUDE.md                 # Agent guide
```

## Crate Roles

### `apps/cli` — CLI Binary

The `gf` command-line entry point. Responsibilities:

- Parse CLI arguments via `clap` (14 subcommands: issue, pr, release, review, auth, label, milestone, commit, pipeline, workflow, doctor, skills, update, completions).
- Detect platform from git remote URL (`resolve_platform`).
- Dispatch commands to platform-specific provider implementations.
- Format output as JSON, text, TOON (LLM format), or auto-detect.
- Handle process exit codes and error reporting.

The binary is thin (~630 lines). All business logic lives in library crates.

### `crates/core` (gitflow-core) — Core Library

The central library crate. Exposes:

- **Provider traits**: `IssueProvider`, `PrProvider`, `ReleaseProvider`, `ReviewProvider`, `AuthProvider`, `LabelProvider`, `MilestoneProvider`, `CommitProvider`, `PipelineProvider`.
- **Domain types**: `IssueData`, `PrData`, `ReleaseData`, `ReviewData`, `CommentData`, `MergeResult`, `UserSummary`, `State`, `Label`.
- **Error types**: `CoreError`, `PlatformCliError` (thiserror-based).
- **SafePath**: path traversal validation (null bytes, bidi characters, component length).
- **TOON output**: Token-Oriented Object Notation for LLM consumption.
- **Platform detection**: `Platform::detect_from_remote_url()` (GitHub / GitLab / GitCode).

### `crates/github` (gitflow-github) — GitHub Adapter

Implements all provider traits by shelling out to the `gh` CLI and parsing its JSON output.

Key types: `GitHubIssueProvider`, `GitHubPrProvider`, `GitHubReleaseProvider`, `GitHubReviewProvider`, `GitHubAuthProvider`, `GitHubLabelProvider`, `GitHubMilestoneProvider`, `GitHubCommitProvider`.

### `crates/gitlab` (gitflow-gitlab) — GitLab Adapter

Implements all provider traits by shelling out to the `glab` CLI. GitLab uses "Merge Request" instead of "Pull Request", so `GitLabMrProvider` implements `PrProvider`.

Also includes `GitLabPipelineProvider` for CI/CD pipeline operations.

### `crates/gitcode` (gitflow-gitcode) — GitCode Adapter

Implements all provider traits by shelling out to the `gc` CLI. Structurally identical to the GitHub adapter but handles GitCode-specific JSON field formats (e.g., string IDs where GitHub returns integers).

### `crates/e2e-core` — E2E Test Utilities

Shared test library (not published). Provides:

- `TestConfig` / `TestMode`: test environment configuration.
- `TestFixture` / `TestResource`: fixture management and cleanup.
- `TtyRunner` / `TtyMode`: TTY-controlled subprocess testing.

### `crates/e2e-github` — GitHub E2E Tests

Integration tests against real GitHub API via `gh` CLI. Covers auth, issue, PR, and no-auth scenarios. Not published.

### `crates/release-signer` — Release Signing Tool

Standalone binary for Ed25519 signing of release archives (zipsign format). Used in CI to sign release assets. Provides `generate-key` and `sign` subcommands. Not published.

## Architecture Pattern: Provider Trait + CLI Adapter

```
apps/cli (thin binary)
  ├── crates/core (traits + domain types)
  ├── crates/github (gh CLI adapter)
  ├── crates/gitlab (glab CLI adapter)
  └── crates/gitcode (gc CLI adapter)
```

Core defines abstract traits (`IssueProvider`, `PrProvider`, etc.). Each platform crate implements these traits by shelling out to the respective CLI tool (`gh`, `glab`, `gc`) and parsing JSON output.

**Command dispatch** in `apps/cli` creates `Box<dyn PrProvider>` (or other provider) based on the detected platform string, then calls the trait method. This means:

- Adding a new platform = implement the traits in a new crate. No changes to command handlers.
- The `CommandRunner` trait abstracts process spawning, enabling `MockCommandRunner` and `SequencedMockCommandRunner` for deterministic testing.

## Dependency Flow

```
apps/cli  ──depends on──>  crates/github | crates/gitlab | crates/gitcode
                                │                │                │
                                └────────────────┼────────────────┘
                                                 │
                                                 v
                                          crates/core
```

The dependency arrow is one-way: binaries and adapters depend on core, never the reverse. `crates/core` must not depend on any adapter crate. This enforces:

- **Compile-time isolation**: changing an adapter does not recompile core or other adapters.
- **API boundaries**: core has no knowledge of CLI tool names, config file paths, or platform-specific JSON formats.
- **Testability**: core tests are fast and deterministic; adapter tests use `MockCommandRunner`.

### Ecosystem Dependencies

```
apps/cli  ──invokes──>  gh CLI (GitHub)
apps/cli  ──invokes──>  glab CLI (GitLab)
apps/cli  ──invokes──>  gc CLI (GitCode)
apps/cli  ──invokes──>  git binary (remote URL detection)
```

All platform operations shell out to external CLI tools. `prerequisites.rs` checks for their presence before commands run. The `doctor` command diagnoses environment issues.

### AI Skills & Hooks

26+ AI agent skills (`gf-*` commands) extend the CLI by invoking `gf` subcommands via shell. Git hooks (`auto-report-bug`, `pre-commit`) integrate with the command pipeline.

## When to Add a New Crate vs a New Module

| Situation                                      | Action                                    |
|------------------------------------------------|-------------------------------------------|
| New domain type or pure logic                  | Add a `pub mod` to `crates/core`.         |
| New platform adapter                           | New crate under `crates/` (e.g., `bitbucket`). |
| Functionality reused by multiple binaries      | New library crate under `crates/`.        |
| New binary (CLI, daemon, migration tool)       | New crate under `apps/`.                  |
| Private implementation detail                  | `mod` (non-`pub`) in the relevant crate.  |
| Third-party integration (e.g., database)       | New library crate if it pulls in many deps; otherwise a module in `core` behind a feature flag. |

### Crate splitting guidelines

Split to a new library crate when:

1. The module has significantly different dependencies (e.g., `crates/db` with `sqlx`).
2. The module has an independent release cadence.
3. Multiple binaries need it but `core` does not.
4. The compilation unit is large enough to benefit from parallel builds.

Keep modules together when:

1. They share the same dependency graph.
2. They evolve together and share types.
3. The API surface is small and the crate-count overhead is not justified.

## Workspace Cargo.toml

The root `Cargo.toml` is a workspace manifest:

```toml
[workspace]
members = ["crates/*", "apps/*"]
resolver = "3"

[workspace.package]
version = "0.1.0"
edition = "2024"
license = "MIT"
repository = "https://github.com/byx-darwin/rust-lib-template"

[workspace.lints.rust]
unsafe_code = "forbid"
missing_docs = "warn"
missing_debug_implementations = "warn"
```

All crates inherit `[workspace.package]` and `[workspace.lints]` via:

```toml
[package]
name = "core"
version.workspace = true
edition.workspace = true
license.workspace = true

[lints]
workspace = true
```

## Compile Time Strategy

- Library crates compile in parallel.
- A binary change only re-links the final binary; library crates remain cached.
- CI pipelines can build and test libraries before building binaries.
- Feature flags in library crates allow consumers to pull in only what they need.

## Security

- `#![forbid(unsafe_code)]` at workspace level.
- `SafePath` validation rejects path traversal, null bytes, bidi characters, and oversized components.
- `secrecy` crate for token handling.
- Ed25519 release signing via `release-signer` binary and `self_update` with signature verification.
