# Shell Completions

`gitflow-cli` supports tab-completion generation for Bash, Zsh, and Fish via `clap_complete`.

## Quick Generate

```bash
make completions
```

This outputs completion scripts for all supported shells into `completions/` in the project root.

## Subcommand

The CLI exposes a hidden `completions` subcommand:

```bash
gitflow-cli completions bash
gitflow-cli completions zsh
gitflow-cli completions fish
```

Use `--help` to see all options:

```bash
gitflow-cli completions --help
```

## Installation

### Bash

#### User-level (recommended)

```bash
mkdir -p ~/.local/share/bash-completion/completions
gitflow-cli completions bash > ~/.local/share/bash-completion/completions/gitflow-cli

# Add to ~/.bashrc:
# source ~/.local/share/bash-completion/completions/gitflow-cli
```

#### System-level

```bash
gitflow-cli completions bash | sudo tee /usr/share/bash-completion/completions/gitflow-cli
```

### Zsh

```bash
# Create completions directory on fpath
mkdir -p ~/.zsh/completions

# Generate
gitflow-cli completions zsh > ~/.zsh/completions/_gitflow-cli

# Add to ~/.zshrc (before compinit):
# fpath=(~/.zsh/completions $fpath)
# autoload -Uz compinit && compinit
```

Make sure the completions file starts with `#compdef gitflow-cli` and `compinit` runs after `fpath` is set.

### Fish

```bash
mkdir -p ~/.config/fish/completions
gitflow-cli completions fish > ~/.config/fish/completions/gitflow-cli.fish
```

Fish auto-loads completions from `~/.config/fish/completions/`. No additional configuration is needed.

## CI Verification

Verify that committed completions match the current CLI definition. Add to CI:

```makefile
# Makefile
.PHONY: check-completions
check-completions:
	cargo run -- completions bash > /tmp/completions.bash
	diff -q completions/gitflow-cli.bash /tmp/completions.bash || \
		(echo "Completions are out of date. Run 'make completions'." && exit 1)
```

```yaml
# .github/workflows/build.yml
- name: Check shell completions
  run: make check-completions
```

This ensures CLI changes that add, remove, or rename flags and subcommands are reflected in the checked-in completions.

## How It Works

`clap_complete` introspects the `clap::Command` definition at runtime and emits a shell script that registers completion functions. The shell calls back into the binary with hidden flags (e.g., `--generate-completions`) to discover available subcommands, flags, and value completions.

For this to work:

1. The binary must be in `$PATH` (for the completion script to find it).
2. The `completions` subcommand must exist in the `Commands` enum.
3. Use `#[command(hide = true)]` on the completions variant to keep `--help` output clean.

## Implementation Reference

```rust
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;

#[derive(Parser)]
#[command(version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate shell completions.
    #[command(hide = true)]
    Completions {
        /// Shell to generate completions for.
        #[arg(value_enum)]
        shell: Shell,
    },
}

fn handle_completions(shell: Shell) {
    let mut cmd = Cli::command();
    let name = cmd.get_name().to_string();
    clap_complete::generate(shell, &mut cmd, name, &mut std::io::stdout());
}
```

## Troubleshooting

| Symptom                                      | Fix                                                        |
|----------------------------------------------|------------------------------------------------------------|
| "command not found" after installing         | Ensure the binary is in `$PATH`.                           |
| Completions show nothing                     | Restart the shell or `source` the completion file.         |
| Completions error with "unknown flag"        | The binary version does not match the completion script. Regenerate. |
| Zsh: `compinit` warnings                     | Make sure `fpath` is set before `compinit` in `.zshrc`.    |
| Fish: no completions                         | Check that the file is named `gitflow-cli.fish` and is in `~/.config/fish/completions/`. |
