# Rust Pre-commit Checks

**Shared language profile:** `gf-quality/references/profiles/rust.md`. Read it for tools, version sources, and scan exclusions.


## Check Commands

| Check | Command | Fix Command |
|-------|---------|-------------|
| format | `cargo fmt -- --check` | `cargo fmt` |
| lint | `cargo clippy --all-targets --all-features -- -D warnings` | `cargo clippy --fix --allow-dirty` |
| test | `cargo test --workspace` | — |

## Strict Mode

Append `-W clippy::pedantic` to lint command for stricter checks.

## Notes

- Never run `cargo clean`
- Fix commands require user confirmation before execution
