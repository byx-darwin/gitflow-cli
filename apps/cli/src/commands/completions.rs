//! Shell completion script generation.
//!
//! Generates tab-completion scripts for bash, zsh, and fish shells
//! using the `clap_complete` crate. Supports outputting to stdout or
//! installing/uninstalling to the appropriate user-level configuration
//! directory.
//!
//! Note: the `install`/`uninstall` helpers use `std::fs` for synchronous
//! file operations. This module is invoked before the `tokio` runtime is
//! constructed (see `main()`), so `tokio::fs` is not available here.

#![allow(
    clippy::disallowed_methods,
    reason = "Completions command runs synchronously before the tokio runtime is constructed"
)]

use std::{io::Write, path::PathBuf};

use clap::Args;

/// Supported shells for completion script generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Shell {
    /// Bourne Again `SHell` (bash).
    Bash,
    /// Z shell (zsh).
    Zsh,
    /// Friendly interactive shell (fish).
    Fish,
}

impl Shell {
    /// Parse a shell variant from the basename of a path (e.g. `"/bin/zsh"` → `Some(Zsh)`).
    ///
    /// Returns `None` if the filename does not match a known shell name.
    #[must_use]
    pub fn from_env_name(s: &str) -> Option<Self> {
        let basename = std::path::Path::new(s)
            .file_name()
            .and_then(|n| n.to_str())?;
        match basename {
            "bash" => Some(Shell::Bash),
            "zsh" => Some(Shell::Zsh),
            "fish" => Some(Shell::Fish),
            _ => None,
        }
    }

    /// Detect the current shell from the `$SHELL` environment variable.
    ///
    /// # Errors
    ///
    /// Returns an error when `$SHELL` is unset, empty, or names an
    /// unsupported shell.
    pub fn detect_from_env() -> miette::Result<Self> {
        let shell_var = std::env::var("SHELL").map_err(|_| {
            miette::miette!(
                "Could not detect shell: $SHELL environment variable is not set. Use `--shell \
                 <bash|zsh|fish>` to specify explicitly."
            )
        })?;
        Self::from_env_name(&shell_var).ok_or_else(|| {
            miette::miette!(
                "Unsupported shell detected from $SHELL={shell_var}. Supported shells: bash, zsh, \
                 fish. Use `--shell <bash|zsh|fish>` to specify explicitly."
            )
        })
    }

    /// Resolve the installation directory for this shell.
    ///
    /// If `home_override` is `Some`, that path is used as the home directory;
    /// otherwise `$HOME` is read from the environment.
    ///
    /// Returns the user-level completion directory:
    /// - bash: `<home>/.local/share/bash-completion/completions/`
    /// - zsh:  `<home>/.zfunc/`
    /// - fish: `<home>/.config/fish/completions/`
    ///
    /// # Errors
    ///
    /// Returns an error when the home path cannot be determined (no override
    /// and `$HOME` is unset).
    pub fn install_dir(self, home_override: Option<&std::path::Path>) -> miette::Result<PathBuf> {
        let home = if let Some(p) = home_override {
            p.to_path_buf()
        } else {
            let home_var = std::env::var("HOME").map_err(|_| {
                miette::miette!(
                    "Could not determine home directory: $HOME environment variable is not set"
                )
            })?;
            PathBuf::from(home_var)
        };
        let dir = match self {
            Shell::Bash => home.join(".local/share/bash-completion/completions"),
            Shell::Zsh => {
                // macOS / Linux 标准 zsh site-functions 目录，已在默认 fpath 中
                let site = PathBuf::from("/usr/local/share/zsh/site-functions");
                if site.exists() {
                    site
                } else {
                    // 回退到用户目录
                    home.join(".local/share/zsh/site-functions")
                }
            }
            Shell::Fish => home.join(".config/fish/completions"),
        };
        Ok(dir)
    }

    /// Return the conventional completion-file name for `gitflow-cli` in this shell.
    #[must_use]
    pub fn completion_filename(self) -> &'static str {
        match self {
            Shell::Bash => "gitflow-cli.bash",
            Shell::Zsh => "_gitflow-cli",
            Shell::Fish => "gitflow-cli.fish",
        }
    }
}

/// Arguments for the completions subcommand.
#[derive(Debug, Args)]
pub struct CompletionsArgs {
    /// Shell to generate completions for.
    ///
    /// Required when generating to stdout. Optional for `--install`/`--uninstall`,
    /// falling back to auto-detection from `$SHELL`.
    #[arg(value_enum, required_unless_present_any = ["install", "uninstall"])]
    pub shell: Option<Shell>,

    /// Install the completion script to the shell's configuration directory.
    #[arg(long, conflicts_with = "uninstall")]
    pub install: bool,

    /// Uninstall the completion script from the shell's configuration directory.
    #[arg(long)]
    pub uninstall: bool,
}

/// Generate, install, or uninstall shell completion scripts.
///
/// `C` is the top-level CLI command type (must implement
/// [`clap::CommandFactory`]).
///
/// Behaviour depends on the flags supplied in `args`:
/// - `--install`: write the script to the shell's config directory.
/// - `--uninstall`: remove the script from the shell's config directory.
/// - neither: write the script to stdout (existing behaviour).
pub fn generate<C: clap::CommandFactory>(args: &CompletionsArgs) -> std::process::ExitCode {
    if args.install {
        match install::<C>(args) {
            Ok(()) => std::process::ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("{e:?}");
                std::process::ExitCode::from(1)
            }
        }
    } else if args.uninstall {
        match uninstall(args) {
            Ok(()) => std::process::ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("{e:?}");
                std::process::ExitCode::from(1)
            }
        }
    } else {
        let Some(shell) = args.shell else {
            eprintln!("Error: shell argument is required when not using --install or --uninstall");
            return std::process::ExitCode::from(1);
        };
        let mut cmd = C::command();
        let name = cmd.get_name().to_string();
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        write_completion(&mut cmd, shell, &name, &mut handle);
        std::process::ExitCode::SUCCESS
    }
}

/// Resolve the shell to use, honouring an explicit argument or falling
/// back to `$SHELL` auto-detection.
fn resolve_shell(explicit: Option<Shell>) -> miette::Result<Shell> {
    explicit.map_or_else(Shell::detect_from_env, Ok)
}

/// Generate a completion script into an arbitrary [`Write`] sink.
fn write_completion<W: Write>(cmd: &mut clap::Command, shell: Shell, name: &str, writer: &mut W) {
    use clap_complete::{generate, shells};
    match shell {
        Shell::Bash => generate(shells::Bash, cmd, name, writer),
        Shell::Zsh => generate(shells::Zsh, cmd, name, writer),
        Shell::Fish => generate(shells::Fish, cmd, name, writer),
    }
}

/// Install the completion script to the shell's configuration directory.
///
/// Creates the target directory if it does not exist. Overwrites any
/// existing completion file.
///
/// # Errors
///
/// - Shell cannot be resolved (no explicit shell and `$SHELL` unset/unsupported).
/// - Home directory cannot be determined.
/// - Filesystem write fails.
fn install<C: clap::CommandFactory>(args: &CompletionsArgs) -> miette::Result<()> {
    let shell = resolve_shell(args.shell)?;
    let dir = shell.install_dir(None)?;
    std::fs::create_dir_all(&dir).map_err(|e| {
        miette::miette!(
            "Failed to create completion directory {}: {e}",
            dir.display()
        )
    })?;

    let file_path = dir.join(shell.completion_filename());
    let mut cmd = C::command();
    let name = cmd.get_name().to_string();
    let mut buf = Vec::new();
    write_completion(&mut cmd, shell, &name, &mut buf);
    let output = String::from_utf8(buf)
        .map_err(|e| miette::miette!("Generated completion is not valid UTF-8: {e}"))?;

    std::fs::write(&file_path, &output).map_err(|e| {
        miette::miette!(
            "Failed to write completion file {}: {e}",
            file_path.display()
        )
    })?;

    let shell_name = shell_name_str(shell);
    println!(
        "✅ Completion installed for {shell_name} → {}",
        file_path.display()
    );
    Ok(())
}

/// Uninstall the completion script from the shell's configuration directory.
///
/// # Errors
///
/// - Shell cannot be resolved.
/// - Home directory cannot be determined.
/// - Completion file does not exist.
/// - Filesystem removal fails.
fn uninstall(args: &CompletionsArgs) -> miette::Result<()> {
    let shell = resolve_shell(args.shell)?;
    let dir = shell.install_dir(None)?;
    let file_path = dir.join(shell.completion_filename());

    if !file_path.exists() {
        let shell_name = shell_name_str(shell);
        return Err(miette::miette!(
            "No completion file found at {} — nothing to uninstall for {shell_name}",
            file_path.display()
        ));
    }

    std::fs::remove_file(&file_path).map_err(|e| {
        miette::miette!(
            "Failed to remove completion file {}: {e}",
            file_path.display()
        )
    })?;

    let shell_name = shell_name_str(shell);
    println!("✅ Completion uninstalled for {shell_name}");
    Ok(())
}

/// Return the user-facing lowercase name for a shell.
const fn shell_name_str(shell: Shell) -> &'static str {
    match shell {
        Shell::Bash => "bash",
        Shell::Zsh => "zsh",
        Shell::Fish => "fish",
    }
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory as _;

    use super::*;

    #[test]
    fn test_should_detect_bash_from_full_path() {
        assert_eq!(Shell::from_env_name("/bin/bash"), Some(Shell::Bash));
    }

    #[test]
    fn test_should_detect_zsh_from_full_path() {
        assert_eq!(Shell::from_env_name("/usr/bin/zsh"), Some(Shell::Zsh));
    }

    #[test]
    fn test_should_detect_fish_from_bare_name() {
        assert_eq!(Shell::from_env_name("fish"), Some(Shell::Fish));
    }

    #[test]
    fn test_should_return_none_for_unknown_shell() {
        assert_eq!(Shell::from_env_name("/bin/tcsh"), None);
        assert_eq!(Shell::from_env_name(""), None);
    }

    #[test]
    fn test_should_return_correct_bash_filename() {
        assert_eq!(Shell::Bash.completion_filename(), "gitflow-cli.bash");
    }

    #[test]
    fn test_should_return_correct_zsh_filename() {
        assert_eq!(Shell::Zsh.completion_filename(), "_gitflow-cli");
    }

    #[test]
    fn test_should_return_correct_fish_filename() {
        assert_eq!(Shell::Fish.completion_filename(), "gitflow-cli.fish");
    }

    #[test]
    fn test_should_return_correct_bash_install_dir() {
        let home = std::path::Path::new("/tmp/gitflow-cli-test-home");
        let dir = Shell::Bash
            .install_dir(Some(home))
            .expect("override provided");
        assert_eq!(
            dir,
            PathBuf::from("/tmp/gitflow-cli-test-home/.local/share/bash-completion/completions")
        );
    }

    #[test]
    fn test_should_return_correct_zsh_install_dir() {
        let home = std::path::Path::new("/tmp/gitflow-cli-test-home");
        let dir = Shell::Zsh
            .install_dir(Some(home))
            .expect("override provided");
        assert_eq!(dir, PathBuf::from("/tmp/gitflow-cli-test-home/.zfunc"));
    }

    #[test]
    fn test_should_return_correct_fish_install_dir() {
        let home = std::path::Path::new("/tmp/gitflow-cli-test-home");
        let dir = Shell::Fish
            .install_dir(Some(home))
            .expect("override provided");
        assert_eq!(
            dir,
            PathBuf::from("/tmp/gitflow-cli-test-home/.config/fish/completions")
        );
    }

    #[test]
    fn test_should_generate_bash_completion_contains_function() {
        let mut cmd = crate::Cli::command();
        let mut output = Vec::new();
        write_completion(&mut cmd, Shell::Bash, "gitflow-cli", &mut output);
        let output = String::from_utf8(output).expect("completion should be valid UTF-8");
        assert!(
            output.contains("complete -F"),
            "bash completion should contain `complete -F` invocation"
        );
    }

    #[test]
    fn test_should_generate_zsh_completion_contains_compdef() {
        let mut cmd = crate::Cli::command();
        let mut output = Vec::new();
        write_completion(&mut cmd, Shell::Zsh, "gitflow-cli", &mut output);
        let output = String::from_utf8(output).expect("completion should be valid UTF-8");
        assert!(
            output.contains("#compdef"),
            "zsh completion should contain `#compdef` directive"
        );
    }

    #[test]
    fn test_should_generate_fish_completion_contains_complete() {
        let mut cmd = crate::Cli::command();
        let mut output = Vec::new();
        write_completion(&mut cmd, Shell::Fish, "gitflow-cli", &mut output);
        let output = String::from_utf8(output).expect("completion should be valid UTF-8");
        assert!(
            output.contains("complete -c"),
            "fish completion should contain `complete -c` invocation"
        );
    }

    #[test]
    fn test_should_generate_non_empty_output_for_all_shells() {
        for shell in [Shell::Bash, Shell::Zsh, Shell::Fish] {
            let mut cmd = crate::Cli::command();
            let mut output = Vec::new();
            write_completion(&mut cmd, shell, "gitflow-cli", &mut output);
            assert!(
                !output.is_empty(),
                "completion for {shell:?} should not be empty"
            );
        }
    }

    #[test]
    fn test_should_produce_different_output_per_shell() {
        let mut cmd_bash = crate::Cli::command();
        let mut bash_out = Vec::new();
        write_completion(&mut cmd_bash, Shell::Bash, "gitflow-cli", &mut bash_out);

        let mut cmd_zsh = crate::Cli::command();
        let mut zsh_out = Vec::new();
        write_completion(&mut cmd_zsh, Shell::Zsh, "gitflow-cli", &mut zsh_out);

        assert_ne!(bash_out, zsh_out, "bash and zsh completions should differ");
    }
}
