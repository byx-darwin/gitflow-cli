# gh 2.98.0 compatibility verification (Issue #227)

The [official GitHub CLI 2.98.0 release](https://github.com/cli/cli/releases/tag/v2.98.0) was tested with the macOS arm64 release archive. The archive's SHA-256 matched the official checksums file, and the extracted binary reported `gh version 2.98.0`. It was run from a temporary directory; the system-installed `gh 2.97.0` was not changed.

| Check | Result |
| --- | --- |
| `make smoke-test-github` with the temporary 2.98.0 binary first in `PATH` | 60 passed, 0 failed, 0 skipped |
| Smoke test API reads | Issue, PR, Release, Label, and Milestone listing all passed |
| `cargo test -p gitflow-github` | 310 adapter tests and 9 doctests passed |

The release adds `gh pr checkout --worktree` and search modes, and fixes a Codespaces port forwarding security issue. These changes do not require a change to the existing `gf` argument mappings. The verified read-only behavior supports adding `2.98.0` to `tested_versions`. No observed incompatibility warrants a higher `min_version` or a new contract fixture. The smoke suite does not exercise write operations.
