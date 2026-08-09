# Plan: Refresh stale `package.metadata.binstall` metadata (Issue #153)

**Date:** 2026-08-09
**Mode:** fast
**Issue:** #153
**Workflow:** wf-2026-08-09-153

## Problem

`apps/cli/Cargo.toml` `[package.metadata.binstall]` still describes the pre-#124
rename release layout:

```toml
pkg-url = "{ repo }/releases/download/v{ version }/{ name }-{ target }.{ archive-format }"
bin-dir = "{ name }-{ target }/{ bin }{ binary-ext }"
pkg-fmt = "tgz"
```

Actual v1.0.0+ release assets (`.github/workflows/release.yml`):

- Asset filename: `gf-{target}.tgz` (`env.BINARY_NAME: gf`, line 96: `tar -czvf ... $BINARY_NAME`)
- Binary inside archive: at root (`gf` / `gf.exe`) — no subdirectory
- Crate name: `gitflow-cli` → template `{ name }` expands to `gitflow-cli`, mismatching `gf`

## Fix

Update `apps/cli/Cargo.toml`:

```toml
pkg-url = "{ repo }/releases/download/v{ version }/{ bin }-{ target }.{ archive-format }"
bin-dir = "{ bin }{ binary-ext }"
pkg-fmt = "tgz"
```

- `pkg-url`: `{ bin }` → `gf`, matching real asset `gf-{target}.tgz`
- `bin-dir`: root-level binary only (`gf` / `gf.exe`)
- `pkg-fmt`: unchanged (`tgz`)

## Scope / Non-goals

- `docs/release.md` binstall example uses generic `{{ project-name }}` template
  placeholders and is not a concrete stale reference to the `gf` layout — out of scope.
- `docs/superpowers/plans|reviews/*` are historical records — not edited.
- No Rust code changes.

## Validation

- `cargo binstall` not installed → verify asset-name consistency against
  `.github/workflows/release.yml` (asset `gf-{target}.tgz`).
- `cargo metadata` sanity: metadata parses.
- TDD: TOML metadata has no Rust test surface; validation is config-level.
