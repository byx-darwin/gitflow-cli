# Read-only Skill tool permissions (#343)

The `gf-security-check`, `gf-label-stats`, `gf-pipeline-analyzer`, and `gf-repo-onboarding` skills pre-approve only `Read`, `Grep`, and `Glob`. In Claude Code, their `disallowed-tools: Write, Edit` field removes those direct file-writing tools for the invocation turn. Shell commands remain subject to the host's normal permission rules; `Bash` is deliberately not pre-approved. These fields do not make arbitrary shell commands read-only.

`gf-security-check` and `gf-pipeline-analyzer` return reports to their caller. When `gf-workflow` needs an audit-trail file, the workflow caller writes and archives the report after the read-only skill returns. `gf-repo-onboarding` produces chat output only; a later request to save it is a separate writing task.

`make check-agent-sync` runs `scripts/validate-skill-permissions.py`, which rejects a direct `Write` or `Edit` grant in a Skill whose introduction claims to be read-only. The validator has fixture tests in `scripts/tests/test_validate_skill_permissions.py`. It checks metadata consistency, not runtime shell behavior.

`disallowed-tools` is a [Claude Code frontmatter extension](https://code.claude.com/docs/en/skills#frontmatter-reference). Generic Agent Skills packages only accept `allowed-tools`, so the four Claude Code skills are not directly portable to that upload format without an equivalent deny policy in the target host. Other agents may ignore these permission fields; their own tool permissions still apply.
