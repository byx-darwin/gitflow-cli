# Rust Language Profile

Shared facts for `gf-quality`, `gf-precommit`, `gf-smell`, `gf-refactor`, and `gf-architecture-diagram`. Read this profile with the language-specific procedure; do not duplicate these facts there.

## Version Source

- Read `edition` and `rust-version` from the project manifest, then read the active compiler with `rustc --version`. Do not infer a project's edition from the compiler version.

## Tools and Installation

| Tool | Source or installation guidance | Used by |
|---|---|---|
| Cargo, rustfmt, Clippy | Rust toolchain; check with `cargo --version` and `rustup component list --installed` | build, format, static checks |
| cargo-llvm-cov | Recommend `cargo install cargo-llvm-cov` when absent | coverage |
| nightly toolchain | Recommend `rustup toolchain install nightly` when required by project formatting | nightly format check |

Never install tools automatically. This repository's `rust-toolchain.toml`, `Cargo.toml`, and `rustfmt.toml` are the source for its pinned version, edition, resolver, and formatting settings.
