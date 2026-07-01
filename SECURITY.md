# Security Policy

## Supported Versions

Only the latest release is actively supported with security patches.

## Reporting a Vulnerability

**Do not open a public issue.** Instead, report vulnerabilities privately:

1. Email the maintainers directly (do not use public channels).
2. Allow up to 7 days for an initial response.
3. We will coordinate disclosure after a fix is released.

## Security Expectations

This repository is a Rust workspace template. Users are responsible for following the security guidelines in `CLAUDE.md`, including:

- Never committing `.env` files or secrets.
- Running `cargo audit` and `cargo deny check` on dependency changes.
- Applying the principle of least privilege in CI permissions.
- Validating all external input at trust boundaries.
