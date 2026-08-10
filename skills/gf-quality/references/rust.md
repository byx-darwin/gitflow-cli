# Rust Quality Toolchain

**Detection:** `Cargo.toml` in project root.

## Gate Commands

| # | Gate | Command | Pass Criteria |
|---|------|---------|---------------|
| 1 | build | `cargo build --workspace --quiet` | exit 0 |
| 2 | test | `cargo test --workspace --quiet` | all pass |
| 3 | coverage | `cargo tarpaulin --workspace 2>&1 \| tail -3` | > `COV_THRESHOLD` (default 80%) |
| 4 | format | `cargo +nightly fmt -- --check` | exit 0, no diff |
| 5 | static | `cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::pedantic` | exit 0, no warnings |
| 6 | pre-commit | `pre-commit run --all-files` | all hooks pass (or N/A if no `.pre-commit-config.yaml`) |

## Tool Installation

| Tool | Install Command | Required By |
|------|----------------|-------------|
| cargo-tarpaulin | `cargo install cargo-tarpaulin` | Gate 3 (coverage) |
| nightly toolchain | `rustup toolchain install nightly` | Gate 4 (format) |

If a tool is missing, **warn the user and recommend install** — do NOT auto-install.

## Environment Variables

| Variable | Effect |
|----------|--------|
| `COV_THRESHOLD` / `COVERAGE_THRESHOLD` | Override coverage threshold (default: 80%) |

## Forbidden Actions

- ❌ Never run `cargo clean`
- ❌ Never auto-fix with `cargo clippy --fix` — report only
- ❌ Never auto-fix with `cargo fmt` (without `--check`) — report only

## Makefile-First Rule

If project root contains a `Makefile` with matching targets, prefer `make` commands over direct tool invocations:

| Gate | Preferred Command | Fallback |
|------|-------------------|----------|
| build | `make build` | `cargo build --workspace --quiet` |
| test | `make test` | `cargo test --workspace --quiet` |
| format | `make fmt` | `cargo +nightly fmt -- --check` |
| static | `make clippy` | `cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::pedantic` |

Detection: `make -n <target> >/dev/null 2>&1` returns 0 → target exists.

## Configuration

### Tool Setup

| Tool | Install | Config File | Required |
|------|---------|-------------|----------|
| cargo-tarpaulin | `cargo install cargo-tarpaulin` | — | Gate 3 (coverage) |
| nightly toolchain | `rustup toolchain install nightly` | — | Gate 4 (format) |
| rustfmt | Included with rustup | `rustfmt.toml` | Gate 4 |
| clippy | Included with rustup | `clippy.toml` | Gate 5 |

### Config File Examples

#### rustfmt.toml

```toml
edition = "2021"
max_width = 100
imports_layout = "Mixed"
```

#### clippy.toml

```toml
cognitive-complexity-threshold = 30
too-many-arguments-threshold = 7
```

#### Cargo.toml (workspace)

```toml
[workspace]
members = ["crates/*", "apps/*"]
resolver = "2"

[workspace.lints.clippy]
pedantic = { level = "warn", priority = -1 }
```

### Environment Variables

| Variable | Effect | Default |
|----------|--------|---------|
| `COV_THRESHOLD` / `COVERAGE_THRESHOLD` | Override coverage threshold | 80% |
| `RUSTFLAGS` | Pass flags to rustc | — |
| `CARGO_HOME` | Cargo cache location | `~/.cargo` |

### Language-Specific Notes

- For Rust workspaces, run gates at workspace root (covers all members)
- Gate 3 requires `cargo-tarpaulin` — if missing, mark SKIPPED
- Gate 4 requires nightly toolchain — if missing, mark SKIPPED
- Gate 5 uses `-D warnings` — any warning fails the gate

## Troubleshooting

### Common Errors

| Error | Cause | Fix |
|-------|-------|-----|
| `cargo-tarpaulin: command not found` | Tool not installed | `cargo install cargo-tarpaulin` |
| `error: toolchain 'nightly' is not installed` | Nightly missing | `rustup toolchain install nightly` |
| `error: could not compile` | Compilation error | Read error message, fix code |
| `test failed, doctests failed` | Test failure | Run `cargo test --workspace -- --nocapture` |

### Exit Code Reference

| Code | Meaning | Action |
|------|---------|--------|
| 0 | Success | Continue to next gate |
| 101 | Compilation error | Fix compilation errors |
| 102 | Test failure | Fix failing tests |
| 1 | Clippy warnings (with -D) | Fix lint warnings |

### FAQ

**Q: Why does coverage show 0%?**
A: Ensure `cargo-tarpaulin` is installed and project builds successfully. Check for `#[cfg(test)]` modules.

**Q: How to skip doc tests?**
A: Run `cargo test --lib --bins` instead of `cargo test --workspace`.

**Q: Workspace build slow?**
A: Use `cargo build --workspace --quiet` to reduce output. Enable incremental compilation in `Cargo.toml`.

### Performance Tips

- Use `cargo build --workspace --quiet` to reduce output noise
- Enable parallel test execution: `cargo test --workspace -- --test-threads=4`
- Use incremental compilation: add `profile.dev.incremental = true` to `Cargo.toml`
- Skip doc tests if not needed: `cargo test --lib --bins`
