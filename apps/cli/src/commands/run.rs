//! Run command -- the main application logic.
//!
//! Demonstrates layered configuration loading, core library type
//! usage, async operations, and structured logging with `tracing`.

use clap::Args;
use miette::{miette, Result};

/// Arguments for the `run` subcommand.
#[derive(Debug, Args)]
pub struct RunArgs {
    /// Override the configuration name (defaults to the name from
    /// the layered config or `"gitflow-cli"`).
    #[arg(short, long)]
    pub name: Option<String>,
}

/// Execute the run command.
///
/// Loads configuration from XDG paths, builds a core [`Config`],
/// performs a small async operation, and logs progress with
/// `tracing`.
///
/// # Errors
///
/// Returns a [`miette::Report`] if configuration cannot be loaded
/// or if the core [`Config`] name is invalid.
pub async fn run(args: RunArgs) -> Result<()> {
    // Load layered CLI configuration
    let cli_config = crate::config::CliConfig::load()?;

    // CLI args override config file / defaults
    let name = args
        .name
        .as_deref()
        .filter(|n| !n.is_empty())
        .unwrap_or(&cli_config.name);

    // Build core domain config (validates non-empty name)
    let core_config = gitflow-cli_core::Config::new(name)
        .map_err(|e| miette!("{e}"))?;

    let core_config = if let Some(ref desc) = cli_config.description {
        core_config.with_description(desc.clone())
    } else {
        core_config
    };

    tracing::info!(
        name = %core_config.name(),
        has_description = core_config.description().is_some(),
        "Configuration loaded"
    );

    // Perform a small async operation to demonstrate the runtime
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    tracing::debug!("Async work unit completed");

    // Output summary to stdout
    println!("Config: {core_config:?}");

    tracing::info!("Run command completed successfully");
    Ok(())
}
