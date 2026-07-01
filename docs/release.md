# Release Process

This document describes the release workflow for `{{ project-name }}`.

## Prerequisites

- Push access to the main repository.
- `cargo-release` and `git-cliff` installed (run `make install-tools`).
- `GITHUB_TOKEN` set for GitHub API access (changelog generation, release uploads).

## Release Checklist

1. **Ensure the branch is clean and up to date**

   ```bash
   git checkout master
   git pull --rebase origin master
   git status  # should be clean
   ```

2. **Run the full CI gate locally**

   ```bash
   make build
   make test
   make lint
   cargo audit
   cargo deny check
   ```

3. **Check that changelog is current**

   ```bash
   git cliff --unreleased --preview
   ```

   Verify all merged changes are represented and well-phrased.

4. **Bump version and update changelog**

   Use `cargo-release` to handle version bump, tag, and push:

   ```bash
   make release
   ```

   This runs `cargo release patch` (or `minor`/`major` as appropriate), which:
   - Bumps the version in `Cargo.toml` files.
   - Updates inter-crate dependency versions.
   - Commits the version bump.
   - Creates a signed tag.
   - Pushes the commit and tag.

5. **Verify CI release build**

   The CI pipeline triggers on tags matching `v*`. Verify the build passes and binaries are attached to the GitHub Release.

6. **Publish to crates.io (if applicable)**

   ```bash
   cargo publish -p core
   ```

   Wait for `core` to be available before publishing dependent crates.

7. **Update Homebrew formula (if applicable)**

   See [Homebrew](#homebrew) below.

## Conventional Commits

This project uses [Conventional Commits](https://www.conventionalcommits.org/) for machine-readable changelogs.

| Type       | Usage                                      | Semver Bump |
|------------|--------------------------------------------|-------------|
| `feat:`    | New feature                                | MINOR       |
| `fix:`     | Bug fix                                    | PATCH       |
| `docs:`    | Documentation only                         | none        |
| `style:`   | Formatting, no code change                 | none        |
| `refactor:`| Code change, no feature/fix                | PATCH       |
| `perf:`    | Performance improvement                    | PATCH       |
| `test:`    | Adding or fixing tests                     | none        |
| `chore:`   | CI, deps, tooling                          | none        |
| `revert:`  | Revert a previous commit                   | PATCH       |

Breaking changes: add `!` after the type/scope or add `BREAKING CHANGE:` in the footer. This triggers a MAJOR bump.

Examples:

```
feat: add support for JSON output
fix(config): handle missing config directory gracefully
docs: add CLI patterns guide
chore(deps): bump clap to 4.5
feat!: drop support for legacy TOML format
```

## git-cliff Configuration

`git-cliff` generates the changelog from conventional commit history. Minimal `cliff.toml`:

```toml
[changelog]
header = "# Changelog\n"
body = """
{% for group, commits in commits | group_by(attribute="group") %}
## {{ group | upper_first }}
{% for commit in commits %}
- {{ commit.message | upper_first }}
{%- endfor %}
{% endfor %}
"""
trim = true

[git]
conventional_commits = true
filter_unconventional = true
split_commits = false
commit_parsers = [
    { message = "^feat", group = "Features" },
    { message = "^fix", group = "Bug Fixes" },
    { message = "^docs", group = "Documentation" },
    { message = "^perf", group = "Performance" },
    { message = "^refactor", group = "Refactor" },
    { message = "^style", group = "Styling" },
    { message = "^test", group = "Testing" },
    { message = "^chore", group = "Chores" },
    { message = "^revert", group = "Reverts" },
]
```

Generate a preview:

```bash
git cliff --unreleased --preview
```

## cargo-release Workflow

`cargo-release` is configured in `release.toml` at the workspace root:

```toml
# release.toml
consolidate-commits = true
pre-release-commit-message = "chore: release {{ version }}"
tag-message = "{{ version }}"
tag-name = "v{{ version }}"
push = true
sign-commit = true
sign-tag = true
```

Release commands:

```bash
# Patch release (0.1.0 -> 0.1.1)
cargo release patch --execute

# Minor release (0.1.1 -> 0.2.0)
cargo release minor --execute

# Major release (0.2.0 -> 1.0.0)
cargo release major --execute

# Dry run (no changes)
cargo release patch
```

## Homebrew Formula

### Option 1: Manual formula

After a GitHub Release with attached binaries, update the Homebrew formula:

```ruby
# Formula/{{ project-name }}.rb
class {{ project-name | capitalize }} < Formula
  desc "Short description of {{ project-name }}"
  homepage "https://github.com/byx-darwin/rust-lib-template"
  url "https://github.com/byx-darwin/rust-lib-template/releases/download/v0.1.0/{{ project-name }}-aarch64-apple-darwin.tar.gz"
  sha256 "<sha256>"
  version "0.1.0"
  license "MIT"

  def install
    bin.install "{{ project-name }}"
  end

  test do
    system "#{bin}/{{ project-name }} --version"
  end
end
```

### Option 2: cargo-dist

Use `cargo-dist` for automated release building and Homebrew formula generation. Configure `dist.toml`:

```toml
# dist.toml
[dist]
targets = ["aarch64-apple-darwin", "x86_64-apple-darwin", "x86_64-unknown-linux-gnu"]
installers = ["homebrew"]
tap = "byx-darwin/homebrew-tap"
```

Then release with:

```bash
cargo dist build
cargo dist plan
```

## cargo-binstall Metadata

Add `[package.metadata.binstall]` to `apps/cli/Cargo.toml` so users can install with `cargo binstall`:

```toml
[package.metadata.binstall]
pkg-url = "{ repo }/releases/download/v{ version }/{ name }-{ target }.tar.gz"
bin-dir = "{ name }-{ version }-{ target }/{ bin }{ binary-ext }"
pkg-fmt = "tgz"
```

Users can then install via:

```bash
cargo binstall {{ project-name }}
```

## Post-Release

- Verify the GitHub Release has correct binaries and changelog.
- Announce on relevant channels.
- Update downstream projects that depend on the library crates.
