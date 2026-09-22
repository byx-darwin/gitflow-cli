# GitCode CLI 0.11.1 compatibility verification (Issue #188)

The [GitCode CLI v0.11.1 release](https://github.com/gitcode-cli/cli/releases/tag/v0.11.1) provided a macOS arm64 archive and checksum file. The archive's SHA-256 matched the published checksum, and the extracted binary reported `gc version 0.11.1`. A temporary `gitcode` alias was used because `gf` invokes that command name. Neither alias nor binary was installed system-wide.

| Check | Result |
| --- | --- |
| GitCode read-only smoke test from an ephemeral repository with a GitCode remote | 60 passed, 0 failed, 0 skipped |
| Smoke test API reads | Issue, PR, Release, Label, and Milestone listing all passed |
| `cargo test -p gitflow-gitcode` | 252 adapter tests and 8 doctests passed |

The release notes describe an installation and symlink migration fix; no command mapping change was identified. The verified behavior supports adding `0.11.1` to `tested_versions`. No observed incompatibility warrants a higher `min_version` or a new contract fixture. The smoke suite does not exercise write operations.
