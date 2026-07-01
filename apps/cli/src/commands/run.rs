//! Run command -- argument definitions.
//!
//! The `run` subcommand is deprecated; the specific subcommands
//! (`issue`, `pr`, etc.) should be used instead. This module retains
//! only the argument struct so clap can continue to parse (and reject)
//! legacy invocations with a clear deprecation message.

use clap::Args;

/// Arguments for the `run` subcommand.
#[derive(Debug, Args)]
pub struct RunArgs {
    /// Override the configuration name (defaults to the name from
    /// the layered config or `"gitflow-cli"`).
    #[arg(short, long)]
    pub name: Option<String>,
}
