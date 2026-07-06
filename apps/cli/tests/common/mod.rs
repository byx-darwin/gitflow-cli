//! Shared test helpers for workflow SKILL.md structure verification tests.

use std::fs;

/// Load SKILL.md content. `CARGO_MANIFEST_DIR` points to `apps/cli` when running
/// under `cargo test -p gitflow-cli`, so we navigate two levels up to the workspace root.
///
/// Synchronous I/O is acceptable here because this is test-only code that runs once
/// per test suite and does not benefit from async overhead.
#[allow(
    clippy::disallowed_methods,
    clippy::panic,
    reason = "Synchronous fs::read_to_string and panic are acceptable in test-only helpers"
)]
pub fn load_skill_md() -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let path = format!("{manifest_dir}/../../skills/gitflow-workflow/SKILL.md");
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to read SKILL.md at {path}: {e}"))
}
