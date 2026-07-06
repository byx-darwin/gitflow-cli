# gitflow-cli Agent Guide

This repository is a reusable Rust 2024 workspace template. These rules are mandatory when working in this template.

## Non-negotiables

- Never enter plan mode automatically.
- Preserve template placeholders such as `gitflow-cli` unless the user explicitly asks to instantiate the template.
- Do not replace template variables with concrete project names during maintenance.
- Use `CLAUDE.md` as the single project-level agent instruction file.
- Use Ruflo for agent workflow/orchestration. Do not maintain project-local `.claude/skills` unless explicitly requested.
- Keep Ruflo or Claude-flow runtime state out of git; persist reusable guidance in `CLAUDE.md`, `docs/`, or `specs/`.
- Do not commit, push, merge, release, deploy, install dependencies, or change ticket state without explicit user permission.
- Never run `cargo clean`; ask first if it is truly required.
- Never write `TODO`, `todo!()`, temporary stubs, or incomplete code. If blocked, stop, reassess, and implement the complete solution.
- Remove dead code instead of suppressing it. Do not add deprecation layers unless explicitly requested.
- Never expose secrets in commands, logs, URLs, comments, errors, or tool arguments.
- **禁止修改以下配置文件（除非用户明确要求）：`deny.toml` 中的 `allow`/`deny`/`wildcards` 等策略配置、`.pre-commit-config.yaml` 中的 hook 类型和阶段配置、`rust-toolchain.toml` 中的工具链版本。这些配置影响 CI/CD 流程，变更需用户确认。**
- **严格执行 gitflow-workflow 流程**：当使用 `/gitflow-workflow` 时，必须按照四阶段完整流程执行，不得跳过任何 Superpowers skills。支持两种模式：
  - **完整模式**（默认）：适用于新功能开发，必须走完所有步骤
  - **快速模式**（`--fast`）：适用于 bug 修复，可跳过 brainstorming/issue-review/writing-plans/subagent-dev，但 TDD 和 code-review 不可跳过

### Completion Discipline

- **Do Not Stop Early**: If the user's requested outcome is not fully complete, do not stop at a draft, partial pass, or "good enough" result. Continue reviewing and improving until the request is genuinely handled or a concrete blocker requires user input.
- **Polish Bar**: Before declaring work complete, ask whether the result is fully polished, concrete, correct, complete, and elegant. If there is doubt, review the work again and update it.
- **Honest Status**: Do not claim a task is finished when it is only a first pass, scaffold, or partial draft. State the remaining gaps and keep working unless the user explicitly asks to pause.

### Code Quality

## Working Process

- Start by understanding the relevant files, symbols, tests, and specs before editing.
- Keep changes minimal, cohesive, and aligned with SOLID, DRY, and KISS.
- Check for existing user changes before editing; never overwrite unrelated work.
- Prefer existing Makefile targets. For new automation, add a `Makefile` target instead of ad-hoc shell scripts.
- For dependency, Helm chart, or external-resource changes, check current upstream usage and security posture first. Put deep research under `docs/research/` after checking existing research.
- For specs, inspect `specs/`, place new files there, name them `{feature-name}-{type}.md`, and update `specs/index.md`.
- For docs, inspect `docs/`, place new files there, and update `docs/index.md`. If documentation was not explicitly requested but is useful, still place it under `docs/`.

## Required Validation

- Prefer the smallest validation that proves the change.
- Prefer existing Makefile targets over raw cargo commands: `make build`, `make test`, `make fmt`, `make clippy`, and `make lint`.
- Docs-only changes do not require the full Cargo suite; code, API, and dependency changes require relevant tests plus formatting and linting.
- When touching production Rust code, run `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic`.
- Run `cargo audit` and `cargo-deny` when dependency, license, or supply-chain risk changes are involved.
- Do not hide failing checks. Diagnose, fix, and rerun; ask the user only when blocked.

## Rust Baseline

- Use Rust 2024 and the pinned toolchain in `rust-toolchain.toml`.
- Keep workspace-wide lint policy in `Cargo.toml` via `[workspace.lints]` when possible.
- Forbid unsafe code at crate roots with `#![forbid(unsafe_code)]`.
- Enable core lint coverage such as `missing_docs` and `missing_debug_implementations`.
- All public items require documentation, including examples where useful and `# Errors`, `# Panics`, or `# Safety` sections when applicable.
- Derive or implement `Debug` for all types; redact sensitive fields manually.

## Toolchain & Build

- Always use Rust 2024 edition with latest stable version. Pin version in `rust-toolchain.toml`.
- Verification must be scoped to the change, not run mechanically. Before finishing, inspect the diff and run the smallest meaningful checks that can catch regressions in the touched surface. Explain any skipped heavyweight gate.
- Run the full Rust gate set (`cargo build`, `cargo test`, `cargo +nightly fmt`, and `cargo clippy -- -D warnings`) when Rust source, public Rust APIs, tests, examples, build scripts, feature flags, workspace manifests, or generated Rust artifacts change.
- Use `cargo clippy -- -D warnings -W clippy::pedantic` for stricter linting on Rust code changes where it adds signal. Allow specific lints with justification.
- Run `cargo audit` and `cargo deny check` when dependencies, lockfiles, license policy, supply-chain configuration, or release packaging change. Otherwise run them periodically, not for unrelated documentation edits.
- For documentation/spec/skill-only changes that do not alter Rust code, APIs, Cargo manifests, examples/doctests, generated artifacts, or release packaging, do not run Rust build/test/clippy. Validate the changed artifacts instead: proofread rendered Markdown as needed, check touched links/indexes, run `make check-agent-sync` for AGENTS/CLAUDE/skill edits, and run skill validation when skill folders change.
- If unsure whether code behavior is affected, choose the narrowest Rust command that answers the question first (for example `cargo test -p <crate> <test>` or `cargo check -p <crate>`) and broaden only when the result warrants it.
- Enable all rustc lints in Cargo.toml: `#![warn(rust_2024_compatibility, missing_docs, missing_debug_implementations)]`.
- DO NOT use `cargo clean` at any time. If you indeed need it, ask user for permission

## Error Handling

- Never use `unwrap()` or `expect()` in production code.
- Return `Result<T>` for fallible operations; do not use `Option<T>` to hide errors.
- Use `thiserror` for library or domain error enums and `miette` for application-level CLI error reporting. `anyhow` is available as a workspace dependency for non-CLI application code; prefer `miette` for binaries that render diagnostics to a terminal.
- Add context with `.context()` or `.with_context()` when propagating errors.
- Panics are acceptable only for truly unrecoverable application bugs, never for library errors or external input.

## Type and API Design

- Make illegal states unrepresentable with newtypes, enums, `NonZero*`, and private fields.
- Prefer `From`, `TryFrom`, and `FromStr` for conversions; prefer `winnow` for custom grammars.
- Use `typed-builder` for structs with more than five fields; simple constructors are fine for small types.
- Mark library-facing structs `#[non_exhaustive]` when future fields are likely.
- Do not use `Option<T>` when `T` has a natural empty or default value such as `Vec`, `HashMap`, or `HashSet`.
- Prefer explicit public API types over `impl Trait`; use `impl Trait` for internal helpers.

## Async and Concurrency

- Use Tokio with explicit features when an async runtime is needed.
- Prefer actors and message passing over shared mutable state.
- Use `tokio::sync::mpsc` for MPSC and `flume` when a faster channel is justified.
- For non-`Send` or non-`Sync` resources, isolate ownership in a dedicated actor or thread instead of wrapping them in locks.
- Prefer `DashMap` over `Mutex<HashMap>` or `RwLock<HashMap>` for concurrent maps.
- Use `ArcSwap` for infrequently updated shared configuration.
- Handle all spawned task results and panics; prefer `JoinSet` for groups of tasks.
- Avoid blocking inside async code; use `tokio::task::spawn_blocking` when required.
- Use native async traits unless object safety requires `async-trait`; document that reason at module level.

## Input, Security, and Resource Boundaries

- Treat every value crossing HTTP, IPC, file, env, CLI, deserialization, or queue boundaries as hostile until validated.
- Validate immediately at deserialization and parse boundaries; reject invalid data instead of sanitizing it.
- Bound all externally supplied strings by byte length, all collections by element count, and all numbers by explicit ranges.
- Use charset allowlists for identifiers and slugs; avoid blocklists.
- Prevent path traversal by rejecting `..`, absolute paths, NUL bytes, and separators before canonicalization.
- Use the `SafePath` type from `gitflow-cli-core` for all externally-supplied file path arguments. It validates at construction time.
- Prevent SSRF by parsing URLs, allowlisting schemes, rejecting private, loopback, and link-local targets, and pinning resolved IPs.
- Use parameterized database APIs; never format user input into SQL.
- Use argv-form process execution; never concatenate user input into shell commands.
- Use the `regex` crate for untrusted text matching; cap untrusted regex pattern size before compilation.
- Add body limits, timeouts, concurrency caps, recursion limits, decompression limits, and rate limits at trust boundaries.
- Use checked, saturating, or explicitly wrapping arithmetic for external numeric input.

## Cryptography and Secrets

- Use `rustls` with the `aws-lc-rs` backend for TLS in new code.
- Use constant-time comparison for tokens, MACs, signatures, password hashes, and similar secrets.
- Use Argon2id for password hashing with parameters tuned for at least 250 ms on target hardware.
- Use OS randomness such as `OsRng` or `getrandom` for security-sensitive keys, tokens, nonces, and IDs.
- Wrap secrets with `secrecy` types and assert redacted `Debug` output in tests for custom secret-bearing types.
- Load secrets from environment variables or secret managers only; never hard-code or commit `.env*` files.
- Design key and token systems for rotation with multiple active keys.

## Serialization and Configuration

- Use strongly typed `serde` models. Use `serde_json::Value` only for truly dynamic schemas.
- Use `#[serde(rename_all = "camelCase")]` for JSON-facing types.
- Use `#[serde(alias = "...")]` for backward compatibility and `#[serde(default)]` for defaultable fields.
- Use `#[serde(skip_serializing_if = "Option::is_none")]` to omit null JSON fields.
- Validate deserialized data immediately, with custom deserializers or validated newtypes when needed.
- Prefer the `config` crate and YAML files for runtime-tunable configuration; keep compile-time constants in code.

## Testing (TDD)

Follow TDD for every feature and bug fix. The cycle is: **RED → GREEN → REFACTOR**.

### Mandatory workflow
- **RED first**: Write a failing test that describes the expected behavior BEFORE writing implementation.
- **GREEN second**: Write the minimal code to make the test pass. Do not write extra logic.
- **REFACTOR third**: Clean up code while keeping tests green. Run `make lint` after.
- Run `make test` to confirm each stage. Use `make test-watch` during active development.

### Test conventions
- Name tests `test_should_<expected_behavior>`.
- Put unit tests in the same file under `#[cfg(test)] mod tests`.
- Cover error paths with `matches!()`. Every fallible function must have both success and failure test cases.
- Use `rstest` for parameterized cases (`#[case]`). Use `proptest` for invariants (`proptest!`).
- Use `mockall` or `wiremock` only when isolation is valuable; prefer fast real implementations.
- Doc tests for public examples. Mark slow tests `#[ignore]`.

## Logging and Observability

- Use `tracing`; never use `println!` or `dbg!` in production code.
- Prefer structured fields over string concatenation, especially for user-controlled values.
- Use `error!`, `warn!`, `info!`, `debug!`, and `trace!` intentionally.
- Add `#[instrument]` to meaningful async boundaries and skip large or sensitive parameters.
- Use JSON logging for production and human-readable output for local development.

## Performance

- Profile before optimizing.
- Avoid unnecessary allocation and cloning; prefer borrowing, `Arc`, `Cow<str>`, and `Bytes` where appropriate.
- Preallocate with `Vec::with_capacity()` when final size is known.
- Prefer iterators and small focused functions.
- Consider `SmallVec` or `smallbox` only when profiling or data shape justifies it.
- Add Criterion benchmarks only after behavior stabilizes.

## Dependencies

- Minimize dependency count and prefer pure Rust crates over FFI bindings.
- Use workspace dependencies for shared crates.
- Pin intentionally: `~` for patch-only updates when needed, default caret requirements for normal minor updates.
- Audit maintenance status, security history, and code quality before adding a dependency.
- Use package managers for dependency changes; do not manually edit lockfiles or manifests for installs or upgrades.

## Documentation

- Write doc comments for all public items.
- Use `//!` for module-level documentation.
- Include examples where helpful, and document `# Errors`, `# Panics`, and `# Safety` sections when applicable.
- Generate docs with `cargo doc --no-deps` when documentation rendering needs verification.

## Code Style

- Import order: standard library, external dependencies, local modules.
- Use specific imports and refer to imported names directly; avoid fully qualified paths in implementations except for macros.
- Follow Rust naming conventions: `snake_case`, `PascalCase`, and `SCREAMING_SNAKE_CASE`.
- Keep functions small and focused; split complex logic into named helpers.
- Order items consistently: imports, constants, types, functions, tests.
- Use trailing commas in multi-line calls and literals.
- Run `rustfmt` rather than hand-formatting.

## Clippy Pedantic Alignment

All code should pass `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic`. Prefer these idioms:

- `x.map_or(a, f)` instead of `x.map(f).unwrap_or(a)`.
- `v.and_then(Value::as_u64)` instead of redundant closures.
- `"value: {x}"` instead of positional format arguments.
- Backtick identifiers in docs, including environment variables and fields.
- Combine same-body match arms with `|`; keep wildcard arms last.
- Restructure instead of using needless `continue`.
- Collapse nested `if` expressions when conditions can be combined.
- Use `&str` instead of `String` for non-consuming parameters.
- Add `#[must_use]` to pure value-returning functions.
- Avoid wildcard imports and similar-looking variable names.
- Use `.try_into()` for lossy conversions and `.into()` or `as` only for provably lossless ones.
- Return the inner type when a function always returns `Some` or `Ok`.
- Prefer one-pass `.filter_map(f)` over `.filter().map()` or `.map().flatten()`.
