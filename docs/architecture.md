# Architecture

This document describes the workspace layout of `gitflow-cli` and the rationale behind it.

## Workspace Layout

```
gitflow-cli/
├── apps/
│   └── cli/          # Binary crate — CLI entry point
├── crates/
│   └── core/         # Library crate — public API, domain types
├── docs/             # Project documentation
├── specs/            # Feature specifications
├── Cargo.toml        # Workspace manifest
├── Makefile          # Automation targets
└── CLAUDE.md         # Agent guide
```

## Crate Roles

### `crates/` — Libraries

Library crates hold the public API, domain types, business logic, and core abstractions. They are:

- **Reusable**: other tools and libraries can depend on them.
- **Testable**: unit tests and doc tests cover the logic without needing a binary.
- **Versioned**: each library crate has its own semver, changelog entry, and API stability guarantees.

The main library crate is `crates/core`. It exposes:

- Domain types (with `Debug`, `Serialize`/`Deserialize`, conversion traits).
- `thiserror`-based error enums.
- Pure functions and async workflows for core operations.

### `apps/` — Binaries

Binary crates are thin entry points. They:

- Parse CLI arguments via `clap`.
- Load and merge configuration.
- Set up logging and signal handlers.
- Wire libraries together and call into `crates/core`.
- Handle process exit codes.

Binaries should contain minimal logic. If a function is complex enough to unit-test, it belongs in a library crate.

## Dependency Flow

```
apps/cli  ──depends on──>  crates/core
```

The dependency arrow is one-way: binaries depend on libraries, never the reverse. `crates/core` must not depend on any `apps/` crate. This enforces:

- **Compile-time isolation**: changing a binary does not recompile libraries.
- **API boundaries**: the library has no knowledge of CLI flags, config file paths, or logging backends.
- **Testability**: library tests are fast and deterministic; they do not need a CLI harness.

## When to Add a New Crate vs a New Module

| Situation                                      | Action                                    |
|------------------------------------------------|-------------------------------------------|
| New domain type or pure logic                  | Add a `pub mod` to `crates/core`.         |
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
