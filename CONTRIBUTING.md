# Contributing to gitflow-cli

## Prerequisites

- Rust toolchain: see `rust-toolchain.toml`
- Pre-commit: `pre-commit install`
- Development tools: run `make install-tools`

## Workflow

1. Fork the repository and create a branch.
2. Make your changes, following the code style in `CLAUDE.md`.
3. Run `make lint` to validate formatting and clippy lints.
4. Run `make test` to verify all tests pass.
5. Commit using conventional commits (`feat:`, `fix:`, `docs:`, `chore:`, etc.).
6. Open a pull request against `master`.

## Release Process

Releases are automated via `make release`, which runs `cargo-release` and `git-cliff` to tag, generate a changelog, and push. Only maintainers can trigger releases.
