# Compatibility Matrix gf Version Derived from CARGO_PKG_VERSION

Date: 2026-08-18
Status: In Progress
Issue: #207
Workflow: wf-2026-08-18-004
Mode: fast

## Background

`crates/core/resources/compatibility-matrix.json` top-level `"gitflow_cli_version": "1.0.0"`
is stale hardcoded metadata. It was set in commit `1a86509` (2026-08-04) and never bumped
through the v1.4.0 release, while the actual gf version is **1.4.0**
(`[workspace.package] version = "1.4.0"` → `gf --version` → `gf 1.4.0`).

Root cause: a build-time version is hardcoded in compatibility data JSON with no sync
mechanism. Identified as a non-blocking observation during the Issue #198 review (PR #206).

The field is only used for parse validation in `crates/core/src/compatibility.rs`
(`#[allow(dead_code)]`, not used at runtime); the generated `docs/compatibility-matrix.md`
header `gf v1.0.0` is rendered from this field.

## Goal

Remove the redundant `gitflow_cli_version` field from the JSON so the matrix document
version is derived automatically from `env!("CARGO_PKG_VERSION")` — the version follows
the crate build and never needs manual synchronization.

## Scope

| File | Change |
|------|--------|
| `crates/core/resources/compatibility-matrix.json` | Remove `gitflow_cli_version` field |
| `crates/core/src/compatibility.rs` | Remove `gf_version` field from `MatrixRoot` (with its `#[serde(rename = "gitflow_cli_version")]`) |
| `crates/core/examples/gen_compat_matrix.rs` | Remove `gf_version` from `MatrixRoot`; render `gf v{version}` header from `env!("CARGO_PKG_VERSION")` |
| `docs/compatibility-matrix.md` | Regenerate via `make compatibility-matrix` → header becomes `gf v1.4.0` |
| `apps/cli/tests/metadata_consistency_test.rs` | Add guard test: doc header tracks `gf v{CARGO_PKG_VERSION}`; JSON no longer contains `gitflow_cli_version` |
| `website/src/pages/compatibility.astro` | Derive `gf v{version}` from workspace `Cargo.toml` at build time (was reading removed `matrix.gitflow_cli_version`) |

Out of scope: release workflow, historical planning/review docs.

## Technical Approach

1. **JSON:** delete the `gitflow_cli_version` line. `serde` ignores unknown fields by
   default in both deserialization structs, so once the field is removed from the structs
   too, parsing is clean.
2. **`compatibility.rs`:** remove the `gf_version` field and its `#[serde(rename)]`.
   `MatrixRoot` then holds only `schema_version`, `updated_at`, `platforms`.
3. **`gen_compat_matrix.rs`:** remove `gf_version` from the example `MatrixRoot`; replace
   the `root.gf_version` read in the header with `env!("CARGO_PKG_VERSION")`. The example
   belongs to `gitflow-core`, whose version is inherited from the workspace
   (`version.workspace = true` → `1.4.0`), so `env!("CARGO_PKG_VERSION")` resolves to the
   workspace version at compile time — this is exactly the build-derived value we want.

   Resulting header line:
   ```
   > 更新时间：{updated_at} · gf {env!("CARGO_PKG_VERSION")}
   ```

4. **Regenerate** `docs/compatibility-matrix.md` with `make compatibility-matrix`; header
   becomes `gf v1.4.0`.
5. **Website:** `website/src/pages/compatibility.astro` rendered `matrix.gitflow_cli_version`
   directly — a live consumer of the removed field. Derive `gf v{version}` from the workspace
   `Cargo.toml` `[workspace.package]` version at build time instead, so the website never
   depends on the matrix JSON for its own version (same "version follows the build" principle).

## Acceptance Criteria

- [ ] `crates/core/resources/compatibility-matrix.json` no longer contains `gitflow_cli_version`
- [ ] `crates/core/src/compatibility.rs` `MatrixRoot` no longer contains `gf_version`; `cargo test -p gitflow-core` passes
- [ ] `crates/core/examples/gen_compat_matrix.rs` renders the header with `env!("CARGO_PKG_VERSION")`
- [ ] `make compatibility-matrix` produces `docs/compatibility-matrix.md` header `gf v1.4.0`
- [ ] Future releases (e.g. v1.5.0) need no manual version metadata sync
- [ ] `cargo clippy -p gitflow-core --all-targets --all-features -- -D warnings -W clippy::pedantic` clean
- [ ] `website/src/pages/compatibility.astro` no longer references `matrix.gitflow_cli_version` and builds with a `gf v{workspace version}` lede

## Verification

- `cargo test -p gitflow-core`
- `make compatibility-matrix` then diff `docs/compatibility-matrix.md` header
- `cargo clippy -p gitflow-core --all-targets --all-features -- -D warnings -W clippy::pedantic`
- `cargo test --test metadata_consistency_test` (now with added guard test)
- `cd website && npm ci && npm run build` (confirms the Astro page renders the derived version)

## Exit Criteria

- PR opened with `Closes #207`
- All acceptance criteria met; CI green
