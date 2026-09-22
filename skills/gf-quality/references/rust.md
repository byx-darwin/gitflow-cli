# Rust Quality Toolchain

**Shared language profile:** `gf-quality/references/profiles/rust.md`. Read it before running these gates.

## Gate Commands

| # | Gate | Command | Pass Criteria |
|---|------|---------|---------------|
| 1 | build | `cargo build --workspace --quiet` | exit 0 |
| 2 | test | `cargo test --workspace --quiet` | all pass |
| 3 | coverage | `cargo llvm-cov --workspace --fail-under-lines ${COV_THRESHOLD:-80}` | exit 0 (total line coverage ≥ threshold); N/A if no `.rs` in change set |
| 4 | format | `cargo +nightly fmt -- --check` | exit 0, no diff |
| 5 | static | `cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::pedantic` | exit 0, no warnings |
| 6 | pre-commit | `pre-commit run --all-files` | all hooks pass (or N/A if no `.pre-commit-config.yaml`) |

## Gate Notes

- Run workspace gates at the workspace root so every member is covered.
- Gate 3 is SKIPPED if `cargo-llvm-cov` is absent and N/A only when the change set has no `.rs` file.
- Gate 4 is SKIPPED if the required nightly toolchain is absent.
- Gate 5 uses `-D warnings`, so any warning fails the gate.

### Makefile-First Rule

If project root contains a `Makefile` with matching targets, prefer `make` commands over direct tool invocations:

| Gate | Preferred Command | Fallback |
|------|-------------------|----------|
| build | `make build` | `cargo build --workspace --quiet` |
| test | `make test` | `cargo test --workspace --quiet` |
| format | `make fmt` | `cargo +nightly fmt -- --check` |
| static | `make clippy` | `cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::pedantic` |

Detection: `make -n <target> >/dev/null 2>&1` returns 0 → target exists.


## Forbidden Actions

- ❌ Never run `cargo clean`
- ❌ Never auto-fix with `cargo clippy --fix` — report only
- ❌ Never auto-fix with `cargo fmt` (without `--check`) — report only

## Quality Gate Configuration

### Configuration Examples

#### rustfmt.toml

```toml
edition = "2024"
max_width = 100
wrap_comments = true
comment_width = 100
format_strings = true
imports_granularity = "Crate"
group_imports = "StdExternalCrate"
style_edition = "2024"
```

#### clippy.toml

Read the target project's `clippy.toml`. In this repository it defines
`disallowed-types` and `disallowed-methods`; do not assume a generic threshold
file represents its lint policy.

#### Cargo.toml (workspace)

```toml
[workspace]
members = ["crates/*", "apps/*"]
resolver = "3"

[workspace.lints.clippy]
pedantic = { level = "warn", priority = -1 }
```

## Troubleshooting

### Common Errors

| Error | Cause | Fix |
|-------|-------|-----|
| `cargo-llvm-cov: command not found` | Tool not installed | See the shared Rust profile |
| `error: toolchain 'nightly' is not installed` | Nightly missing | See the shared Rust profile |
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
A: Ensure `cargo-llvm-cov` is installed and project builds successfully. Check for `#[cfg(test)]` modules.

**Q: How to skip doc tests?**
A: Run `cargo test --lib --bins` instead of `cargo test --workspace`.

**Q: Workspace build slow?**
A: Use `cargo build --workspace --quiet` to reduce output. Enable incremental compilation in `Cargo.toml`.

### Performance Tips

- Use `cargo build --workspace --quiet` to reduce output noise
- Enable parallel test execution: `cargo test --workspace -- --test-threads=4`
- Use incremental compilation: add `profile.dev.incremental = true` to `Cargo.toml`
- Skip doc tests if not needed: `cargo test --lib --bins`
