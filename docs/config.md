# Configuration

`gitflow-cli` resolves configuration from four layers, each overriding the previous.

## Priority (lowest to highest)

1. **Code defaults** — hardcoded in the `Cli` struct via `#[arg(default_value = "...")]`.
2. **Config file** — `$XDG_CONFIG_HOME/gitflow-cli/config.toml`.
3. **Environment variables** — prefixed with `APP_`.
4. **CLI flags** — highest priority, overriding all other sources.

## Config File Format

Config files use TOML. YAML is supported optionally via the `config` crate for environments that prefer it.

### Example `config.toml`

```toml
# ~/.config/gitflow-cli/config.toml

log_level = "debug"
output_format = "json"

[server]
host = "0.0.0.0"
port = 8080

[features]
enable_cache = true
max_cache_entries = 1000
```

### Loading with the `config` crate

```rust
use config::{Config, File, FileFormat};

fn load_config_file(path: &Path) -> Result<AppConfig, config::ConfigError> {
    let settings = Config::builder()
        .add_source(File::from(path).format(FileFormat::Toml).required(false))
        .build()?;

    settings.try_deserialize::<AppConfig>()
}
```

## XDG Directory Layout

Uses the [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/latest/):

| Variable               | Default (Linux)                           | Purpose                     |
|------------------------|-------------------------------------------|-----------------------------|
| `XDG_CONFIG_HOME`      | `~/.config`                               | Config file location.       |
| `XDG_CACHE_HOME`       | `~/.cache`                                | Cache and temp data.        |
| `XDG_DATA_HOME`        | `~/.local/share`                          | Persistent data.            |

Use the `dirs` crate for cross-platform path discovery:

```rust
fn app_config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("gitflow-cli"))
}

fn config_file_path() -> Option<PathBuf> {
    app_config_dir().map(|d| d.join("config.toml"))
}

fn ensure_config_dir() -> std::io::Result<PathBuf> {
    let dir = dirs::config_dir()
        .ok_or_else(|| std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "could not determine config directory",
        ))?
        .join("gitflow-cli");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}
```

## Environment Variable Overrides

Environment variables use the `APP_` prefix. Each config key maps to `APP_<SECTION>_<KEY>` in screaming snake case.

| Config Key            | Env Var Equivalent           |
|-----------------------|------------------------------|
| `log_level`           | `APP_LOG_LEVEL`              |
| `output_format`       | `APP_OUTPUT_FORMAT`          |
| `server.host`         | `APP_SERVER_HOST`            |
| `server.port`         | `APP_SERVER_PORT`            |
| `features.enable_cache`| `APP_FEATURES_ENABLE_CACHE`  |

### Clap integration

Use `#[arg(env = "...")]` on the `Cli` struct:

```rust
#[derive(Parser)]
pub struct Cli {
    #[arg(long, env = "APP_LOG_LEVEL", default_value = "info")]
    pub log_level: String,

    #[arg(long, env = "APP_CONFIG_PATH")]
    pub config: Option<PathBuf>,
}
```

Clap respects the precedence: CLI flag > env var > default.

## `.env` File Loading (Local Development)

For local development, load `.env` files with `dotenvy`:

```rust
fn load_dotenv() {
    // .env files should never be committed to git
    if let Err(e) = dotenvy::dotenv() {
        if e.not_found() {
            // .env not found is fine; it is optional
            tracing::debug!("no .env file found, skipping");
        } else {
            tracing::warn!("failed to load .env: {e}");
        }
    }
}
```

Call this early in `main`, before parsing CLI args:

```rust
fn main() -> miette::Result<()> {
    load_dotenv();
    // ... Cli::parse() will now see APP_* env vars from .env
}
```

**Security note**: `.env` files must never be committed to version control. Add `.env` to `.gitignore`. Use `.env.example` (checked in) to document expected variables without values.

## Validating Required Config at Startup

Validate all required configuration after merging all sources. Fail fast with a clear error message:

```rust
fn validate_config(cli: &Cli) -> Result<(), CliError> {
    let errors: Vec<String> = Vec::new();

    // Example: require a non-empty value
    if cli.log_level.is_empty() {
        errors.push("log_level must not be empty".into());
    }

    if !errors.is_empty() {
        return Err(CliError::InvalidConfig(errors.join("; ")));
    }

    Ok(())
}
```

Validate at startup, before any work begins. A missing or invalid config should produce a clear error before the tool starts its main operation.

## Full Config Bootstrap Example

```rust
fn bootstrap_config() -> Result<AppConfig, CliError> {
    // 1. Load .env for local development
    load_dotenv();

    // 2. Parse CLI args (which also picks up env vars via #[arg(env = ...)])
    let mut cli = Cli::parse();

    // 3. Load config file (fills in gaps, does not override CLI/env)
    let config_path = cli.config.or_else(|| config_file_path());
    if let Some(path) = config_path {
        if path.exists() {
            let file_config = load_config_file(&path)
                .map_err(|e| CliError::Config(format!("failed to load {path:?}: {e}")))?;
            // Merge — only apply values that aren't already set by CLI/env
            merge_defaults(&mut cli, &file_config);
        }
    }

    // 4. Validate
    validate_config(&cli)?;

    Ok(AppConfig::from(cli))
}
```
