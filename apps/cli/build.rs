//! Build script for `gitflow-cli-cli`.
//!
//! Generates build-time metadata (version, commit, target triple, etc.)
//! via the `built` crate and writes it to `OUT_DIR/built.rs` for inclusion
//! in the binary.

#![allow(
    clippy::expect_used,
    reason = "Build scripts run once at compile time; panicking on failure is appropriate"
)]
#![allow(
    missing_docs,
    reason = "Build script `main` is an implementation detail, not a public API"
)]

fn main() {
    built::write_built_file().expect("Failed to acquire build-time information");
}
