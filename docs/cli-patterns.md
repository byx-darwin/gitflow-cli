# CLI Patterns

This document is the canonical CLI architecture guide for `gitflow-cli`. Every CLI binary in this workspace follows these conventions.

## 1. Argument Parsing

Use `clap` derive API with a top-level `Cli` struct and subcommands via a `Commands` enum.

```rust
use clap::{Parser, Subcommand};

/// gitflow-cli — short description of the tool.
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Turn debugging information on.
    #[arg(short, long, env = "APP_LOG_LEVEL", default_value = "info")]
    pub log_level: String,

    /// Path to the configuration file.
    #[arg(short, long, env = "APP_CONFIG_PATH")]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run the main workflow.
    Run {
        /// Input file path.
        #[arg(short, long)]
        input: PathBuf,
    },
    /// Generate shell completions.
    Completions {
        /// Shell to generate completions for.
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
}
```

Env var integration uses `#[arg(env = "...")]` so CLI flags, env vars, and config files all flow through a single `Cli` struct. The explicit `env` attribute documents the binding at the call site.

## 2. Error Handling

Layer error types by crate boundary:

- **Library crate** (`crates/core`): use `thiserror` for domain error enums.
- **CLI binary** (`apps/cli`): use `miette` for fancy diagnostics with source-code snippets and help text.

```rust
// Library crate — thiserror
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("invalid input: {0}")]
    InvalidInput(String),
}

// CLI binary — miette wrapper
#[derive(Debug, miette::Diagnostic, thiserror::Error)]
pub enum CliError {
    #[error("configuration error")]
    #[diagnostic(code(app::config), help("check your config file at ~/.config/gitflow-cli/config.toml"))]
    Config(#[source] CoreError),

    #[error("I/O error")]
    #[diagnostic(code(app::io))]
    Io(#[source] std::io::Error),
}
```

### Exit Codes

| Code | Meaning           | When                                       |
|------|-------------------|--------------------------------------------|
| 0    | Success           | Normal completion.                         |
| 1    | General error     | Runtime failure (I/O, config, etc.).       |
| 2    | CLI misuse        | Invalid arguments or flags.                |
| 130  | Interrupted       | SIGINT received, graceful shutdown.        |

`clap` handles exit code 2 automatically on parse failure. Return `Ok(())` for 0, and map all other errors to 1 (or 130 for interruption).

```rust
fn main() -> miette::Result<()> {
    let cli = Cli::parse();
    // ... setup ...
    let result = run(cli);
    if let Err(ref e) = result {
        if let Some(CliError::Interrupted) = e.downcast_ref::<CliError>() {
            std::process::exit(130);
        }
    }
    result.map_err(|e| {
        eprintln!("{e:?}");
        std::process::exit(1);
    })
}
```

## 3. Signal Handling

### SIGPIPE

On Unix, set `SIGPIPE` to `SIG_DFL` so broken-pipe writes terminate the process instead of panicking.

```rust
/// Set SIGPIPE to SIG_DFL on Unix. This is a no-op on other platforms.
///
/// Without this, writing to a closed pipe (e.g., `app | head`) would
/// deliver SIGPIPE, which Rust ignores by default, causing an `ErrorKind::BrokenPipe`
/// panic instead of a clean exit.
pub fn reset_sigpipe() {
    #[cfg(unix)]
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
}
```

### Ctrl+C (SIGINT)

Use `tokio::signal` for graceful shutdown.

```rust
use tokio::signal;

async fn run_with_shutdown() -> Result<(), CliError> {
    tokio::select! {
        result = do_work() => result,
        _ = signal::ctrl_c() => {
            tracing::info!("received Ctrl+C, shutting down gracefully");
            // Cleanup: remove temp files, flush buffers, etc.
            Err(CliError::Interrupted)
        }
    }
}
```

### SIGTERM (cleanup)

For longer-running CLI tools, handle `SIGTERM` to clean up temp files.

```rust
#[cfg(unix)]
async fn handle_termination(temp_dir: &TempDir) {
    let mut sigterm = tokio::signal::unix::signal(
        tokio::signal::unix::SignalKind::terminate(),
    )
    .expect("failed to register SIGTERM handler");

    sigterm.recv().await;
    tracing::warn!("received SIGTERM, cleaning up");

    // TempDir drop handles cleanup, but explicit removal is safer
    if let Err(e) = temp_dir.close() {
        tracing::error!("failed to clean up temp dir: {e}");
    }
    std::process::exit(0);
}
```

## 4. Config Layering

Config is resolved in priority order (lowest to highest):

1. **Code defaults** — set via `#[arg(default_value = "...")]` on the `Cli` struct.
2. **XDG config file** — `$XDG_CONFIG_HOME/gitflow-cli/config.toml`.
3. **Environment variables** — `APP_`-prefixed, bound via `#[arg(env = "...")]`.
4. **CLI flags** — highest priority.

Use the `dirs` crate for XDG path discovery:

```rust
fn default_config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("gitflow-cli").join("config.toml"))
}
```

Merge logic:

```rust
fn load_config() -> Result<Cli, CliError> {
    let mut cli = Cli::parse(); // CLI flags (highest priority)

    // Env vars override defaults (handled by clap #[arg(env = ...)])

    // Config file provides base values
    if let Some(config_path) = cli.config.or_else(default_config_path) {
        if config_path.exists() {
            let contents = std::fs::read_to_string(&config_path)?;
            let file_config: toml::Value = toml::from_str(&contents)?;
            // Merge file_config into cli, only filling in None fields
            // ...
        }
    }

    Ok(cli)
}
```

## 5. Output and Color

Logging should adapt to the output target:

```rust
use tracing_subscriber::{fmt, EnvFilter, layer::SubscriberExt};

fn setup_logging(cli: &Cli) {
    let is_terminal = atty::is(atty::Stream::Stdout);
    let use_color = match (std::env::var("NO_COLOR"), std::env::var("CLICOLOR")) {
        (Ok(_), _) => false,
        (_, Ok(ref v)) if v == "0" => false,
        _ => is_terminal,
    };

    let subscriber = tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(&cli.log_level)));

    if is_terminal {
        subscriber
            .with(fmt::layer()
                .with_ansi(use_color)
                .with_target(false))
            .init();
    } else {
        subscriber
            .with(fmt::layer()
                .json()
                .with_current_span(false))
            .init();
    }
}
```

Rules:

- Respect `NO_COLOR` (any value disables color) and `CLICOLOR=0` (explicit disable).
- Use human-readable format when stdout is a TTY, JSON when piped.
- Never use `println!` or `eprintln!` for operational output; use `tracing` for structured logging and reserve stderr for diagnostics.

## 6. Temp Files

Always use the `tempfile` crate. For secret-bearing files, use restrictive permissions (`0o600`).

```rust
use tempfile::{NamedTempFile, Builder};

/// Create a temp file with restrictive permissions for sensitive data.
fn create_secret_temp_file() -> std::io::Result<NamedTempFile> {
    Builder::new()
        .prefix("gitflow-cli-")
        .suffix(".tmp")
        .permissions(std::fs::Permissions::from_mode(0o600))
        .tempfile()
}
```

Cleanup in signal handlers ensures temp files are removed on interruption (see [Signal Handling](#3-signal-handling) above). `TempDir` and `NamedTempFile` automatically clean up on `Drop`, but explicit cleanup in signal handlers covers the edge case where the process exits before `Drop` runs.

## 7. Shell Completions

Generate shell completions via `clap_complete`. Prefer a hidden subcommand pattern:

```rust
use clap_complete::Shell;

#[derive(Subcommand)]
pub enum Commands {
    /// Generate shell completions.
    #[command(hide = true)]
    Completions {
        /// Shell to generate completions for.
        #[arg(value_enum)]
        shell: Shell,
    },
    // ... other subcommands
}

fn handle_completions(shell: Shell, cmd: &mut clap::Command) {
    let name = cmd.get_name().to_string();
    clap_complete::generate(shell, cmd, &name, &mut std::io::stdout());
}
```

Installation paths:

```bash
# Bash
gitflow-cli completions bash | sudo tee /usr/share/bash-completion/completions/gitflow-cli

# Zsh
gitflow-cli completions zsh > ~/.zsh/completions/_gitflow-cli

# Fish
gitflow-cli completions fish > ~/.config/fish/completions/gitflow-cli.fish
```

See [Shell Completions](./shell-completions.md) for the full guide.

## 8. Input Validation

### SafePath Newtype

Wrap file path arguments in a newtype that validates safety at parse time:

```rust
/// A validated, safe filesystem path.
///
/// Guarantees:
/// - Not empty
/// - Does not contain NUL bytes
/// - Does not contain `..` traversal segments
/// - Is not an absolute path (unless allow-absolute is enabled)
#[derive(Debug, Clone)]
pub struct SafePath(PathBuf);

impl SafePath {
    /// Maximum allowed byte length for a path component.
    const MAX_LEN: usize = 4096;

    pub fn new(path: impl AsRef<Path>) -> Result<Self, CliError> {
        let path = path.as_ref();
        if path.as_os_str().is_empty() {
            return Err(CliError::InvalidInput("path must not be empty".into()));
        }
        if path.as_os_str().len() > Self::MAX_LEN {
            return Err(CliError::InvalidInput(format!(
                "path exceeds maximum length of {} bytes", Self::MAX_LEN
            )));
        }
        for component in path.components() {
            match component {
                std::path::Component::ParentDir => {
                    return Err(CliError::InvalidInput(
                        "path traversal detected: '..' is not allowed".into(),
                    ));
                }
                std::path::Component::Prefix(_) | std::path::Component::RootDir => {
                    return Err(CliError::InvalidInput(
                        "absolute paths are not allowed".into(),
                    ));
                }
                std::path::Component::Normal(_) | std::path::Component::CurDir => {}
            }
        }
        Ok(Self(path.to_path_buf()))
    }
}
```

### Clap Value Parser

Integrate `SafePath` with clap's value parser:

```rust
#[derive(Parser)]
pub struct Cli {
    /// Input file (relative path only).
    #[arg(short, long, value_parser = parse_safe_path)]
    pub input: SafePath,
}

fn parse_safe_path(s: &str) -> Result<SafePath, String> {
    SafePath::new(s).map_err(|e| e.to_string())
}
```

### Stdin Size Limits

When reading from stdin, enforce a byte limit to prevent memory exhaustion:

```rust
const MAX_STDIN_BYTES: usize = 100 * 1024 * 1024; // 100 MiB

fn read_stdin() -> Result<String, CliError> {
    let mut buffer = Vec::with_capacity(MAX_STDIN_BYTES);
    let mut handle = std::io::stdin().lock();
    handle
        .take(MAX_STDIN_BYTES as u64)
        .read_to_end(&mut buffer)?;
    if buffer.len() == MAX_STDIN_BYTES && handle.bytes().next().is_some() {
        return Err(CliError::InvalidInput(format!(
            "stdin exceeds maximum of {MAX_STDIN_BYTES} bytes"
        )));
    }
    Ok(String::from_utf8(buffer)?)
}
```

## 9. Version Embedding

Use the `built` crate to embed build metadata at compile time. Create `apps/cli/build.rs`:

```rust
fn main() {
    built::write_built_file().expect("failed to acquire build-time information");
}
```

Include the generated file in `main.rs`:

```rust
pub mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

/// Construct a detailed version string.
#[must_use]
pub fn long_version() -> String {
    format!(
        "{} {} ({} {}) [git:{}]",
        built_info::PKG_NAME,
        built_info::PKG_VERSION,
        built_info::TARGET,
        built_info::PROFILE,
        built_info::GIT_VERSION.unwrap_or("unknown"),
    )
}
```

Wire into clap:

```rust
#[derive(Parser)]
#[command(
    version = long_version(),
    about = "gitflow-cli CLI tool"
)]
pub struct Cli { /* ... */ }
```

The `--version` flag will output something like:

```
gitflow-cli 0.1.0 (aarch64-apple-darwin release) [git:abcdef1-dirty]
```

## 10. Async Runtime

For CLI tools, `tokio` with the `current_thread` runtime is usually correct. The CLI workflow is sequential: parse args, load config, do work, report results.

```rust
fn main() -> miette::Result<()> {
    reset_sigpipe();

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build tokio runtime");

    rt.block_on(async {
        let cli = Cli::parse();
        setup_logging(&cli);
        run(cli).await
    })
}
```

Switch to `multi_thread` only when:

- The tool performs high-concurrency I/O (e.g., batch-downloading hundreds of files).
- CPU-bound work benefits from `spawn_blocking` parallelism on a thread pool.

Most CLI tools are I/O-bound with sequential stages: read input, transform, write output. `current_thread` has lower overhead and simpler task-local state management than `multi_thread`.
