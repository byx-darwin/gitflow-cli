# Pre-commit Hook Template

> Source for `gitflow-precommit` hook setup path. Written to `.git/hooks/pre-commit` only after explicit user confirmation.

```bash
#!/usr/bin/env bash
set -euo pipefail

echo "Running pre-commit checks..."

echo "[1/3] Format check"
if ! cargo +nightly fmt -- --check 2>&1; then
    echo "FAIL: format. Run 'cargo +nightly fmt'."
    exit 1
fi

echo "[2/3] Clippy"
if ! cargo clippy --all-targets --all-features -- -D warnings 2>&1; then
    echo "FAIL: clippy. Fix warnings."
    exit 1
fi

echo "[3/3] Test"
if ! cargo test --workspace 2>&1; then
    echo "FAIL: test. Fix failures."
    exit 1
fi

echo "All pre-commit checks passed."
```
