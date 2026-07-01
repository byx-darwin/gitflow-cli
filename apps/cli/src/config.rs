//! Layered configuration management.
//!
//! Configuration is loaded from multiple sources in priority order
//! (lowest to highest):
//! 1. Code defaults
//! 2. XDG config file (`TOML` format)
//! 3. Environment variables (`CLI_` prefix)

use miette::{IntoDiagnostic, Result, WrapErr, miette};
use serde::{Deserialize, Serialize};

/// Maximum length for the config name field.
const MAX_NAME_LEN: usize = 256;
/// Maximum length for the description field.
const MAX_DESCRIPTION_LEN: usize = 4096;

/// CLI application configuration loaded from layered sources.
///
/// Fields marked `#[serde(default)]` fall back to [`Default`] when
/// the config file omits them.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CliConfig {
    /// Application name (used to construct the core [`Config`]).
    pub name: String,
    /// Optional description for the current run.
    pub description: Option<String>,
    /// Enable verbose / debug output.
    pub verbose: bool,
    /// Tracing-filter log level (e.g. `info`, `debug`, `trace`).
    pub log_level: String,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            name: default_name(),
            description: None,
            verbose: false,
            log_level: default_log_level(),
        }
    }
}

impl CliConfig {
    /// Validates all fields after deserialization.
    fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(miette!("config name must not be empty"));
        }
        if self.name.len() > MAX_NAME_LEN {
            return Err(miette!("config name exceeds max length of {MAX_NAME_LEN}B"));
        }
        if let Some(ref desc) = self.description {
            if desc.len() > MAX_DESCRIPTION_LEN {
                return Err(miette!(
                    "config description exceeds max length of {MAX_DESCRIPTION_LEN}B"
                ));
            }
        }
        // Validate log_level is a valid tracing directive
        self.log_level
            .parse::<tracing_subscriber::filter::Directive>()
            .map_err(|e| miette!("invalid log_level '{}': {e}", self.log_level))?;
        Ok(())
    }

    /// Load configuration from all layers.
    ///
    /// Layers are applied in order: code defaults, then `XDG` config
    /// file (`gitflow-cli/config.toml` under the user's config
    /// directory), and finally environment variables with the `CLI_`
    /// prefix.
    ///
    /// # Errors
    ///
    /// Returns an error if the config file exists but cannot be read
    /// or contains invalid `TOML`.
    #[allow(
        clippy::disallowed_methods,
        reason = "Sync FS access at startup before async runtime is created"
    )]
    pub fn load() -> Result<Self> {
        // NOTE: Replace `CLI_` with your tool's env var prefix (e.g., `MYTOOL_`).
        // Search for CLI_ in this file and replace all occurrences.
        let mut config = Self::default();

        // Layer 1: XDG config file
        if let Some(config_dir) = dirs::config_dir() {
            let config_file = config_dir.join("gitflow-cli").join("config.toml");

            if config_file.exists() {
                const MAX_CONFIG_SIZE: u64 = 1_048_576; // 1 MB

                let metadata = std::fs::metadata(&config_file)
                    .map_err(|e| miette!("failed to read config file metadata: {e}"))?;
                if metadata.len() > MAX_CONFIG_SIZE {
                    return Err(miette!("config file exceeds max size of 1 MB"));
                }

                let content = std::fs::read_to_string(&config_file)
                    .into_diagnostic()
                    .wrap_err_with(|| {
                        format!("Failed to read config file: {}", config_file.display())
                    })?;

                if !content.trim().is_empty() {
                    config = toml::from_str(&content)
                        .into_diagnostic()
                        .wrap_err("Failed to parse config file")?;
                }
            }
        }

        // Layer 2: Environment variables (CLI_ prefix)
        if let Ok(val) = std::env::var("CLI_NAME") {
            config.name = val;
        }
        if let Ok(val) = std::env::var("CLI_DESCRIPTION") {
            config.description = Some(val);
        }
        if std::env::var("CLI_VERBOSE").is_ok() {
            config.verbose = true;
        }
        if let Ok(val) = std::env::var("CLI_LOG_LEVEL") {
            config.log_level = val;
        }

        config.validate()?;

        Ok(config)
    }
}

fn default_name() -> String {
    "gitflow-cli".to_string()
}

fn default_log_level() -> String {
    "info".to_string()
}
