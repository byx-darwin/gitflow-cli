//! Shell completion script generation.
//!
//! Generates tab-completion scripts for bash, zsh, and fish shells
//! using the `clap_complete` crate.

use clap::Args;

/// Supported shells for completion script generation.
#[derive(Debug, Clone, clap::ValueEnum)]
pub enum Shell {
    /// Bourne Again SHell (bash).
    Bash,
    /// Z shell (zsh).
    Zsh,
    /// Friendly interactive shell (fish).
    Fish,
}

/// Arguments for the completions subcommand.
#[derive(Debug, Args)]
pub struct CompletionsArgs {
    /// Shell to generate completions for.
    #[arg(value_enum)]
    pub shell: Shell,
}

/// Generate and print a shell completion script to stdout.
///
/// `C` is the top-level CLI command type (must implement
/// [`clap::CommandFactory`]).
pub fn generate<C: clap::CommandFactory>(args: &CompletionsArgs) -> std::process::ExitCode {
    use clap_complete::generate;
    use clap_complete::shells;

    let mut cmd = C::command();
    let name = cmd.get_name().to_string();
    let stdout = &mut std::io::stdout();

    match &args.shell {
        Shell::Bash => generate(shells::Bash, &mut cmd, &name, stdout),
        Shell::Zsh => generate(shells::Zsh, &mut cmd, &name, stdout),
        Shell::Fish => generate(shells::Fish, &mut cmd, &name, stdout),
    }

    std::process::ExitCode::SUCCESS
}
