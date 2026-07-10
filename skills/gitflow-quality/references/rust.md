# Rust Quality Toolchain

**Detection:** `Cargo.toml` in project root.

## Gate Commands

| # | Gate | Command | Pass Criteria |
|---|------|---------|---------------|
| 1 | build | `cargo build --workspace --quiet` | exit 0 |
| 2 | test | `cargo test --workspace --quiet` | all pass |
| 3 | coverage | `cargo tarpaulin --workspace 2>&1 \| tail -3` | > `COV_THRESHOLD` (default 80%) |
| 4 | format | `cargo +nightly fmt -- --check` | exit 0, no diff |
| 5 | static | `cargo clippy --workspace --all-targets -- -D warnings` | exit 0, no warnings |
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

## Strict Mode (Optional)

Append `-W clippy::pedantic` to Gate 5 for stricter linting:

```bash
cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic
```
