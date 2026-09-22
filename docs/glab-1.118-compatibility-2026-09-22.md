# glab 1.118.0 compatibility verification (Issue #402)

The installed `glab --version` reported `1.118.0 (570955d42)`. The [upstream release](https://gitlab.com/gitlab-org/cli/-/releases/v1.118.0) identifies the same version. No CLI installation or authentication change was needed.

Verification against the current local `gf` build:

| Check | Result |
| --- | --- |
| `cargo test -p gitflow-gitlab` | 302 adapter tests passed; 9 doctests passed |
| GitLab smoke test from the GitHub checkout | 55 passed, 5 API checks skipped because that checkout has no GitLab remote |
| GitLab smoke test from an ephemeral repository with an authenticated GitLab remote | 60 passed, 0 failed, 0 skipped |
| Read-only API operations in that ephemeral repository | Issue, PR, Release, Label, and Milestone listing succeeded |

The ephemeral repository contained only a temporary Git remote; no project content was changed. The smoke suite exercises command help and five read-only API operations. Adapter contract tests exercise the `glab` argument and response mappings with fixtures. These results justify adding `1.118.0` to `tested_versions`; they do not establish that every write operation or every GitLab instance has been tested. No observed incompatibility requires a `min_version` increase or a new contract fixture.
