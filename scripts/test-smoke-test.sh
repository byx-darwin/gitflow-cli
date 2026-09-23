#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"
mkdir -p .cache
TEST_DIR="$(mktemp -d .cache/smoke-script-test.XXXXXX)"
trap 'rm -rf "$TEST_DIR"' EXIT

FAKE_GF="$TEST_DIR/gf"
cat > "$FAKE_GF" <<'EOF'
#!/usr/bin/env bash
if [[ "${1:-}" == "--version" ]]; then
    echo "gf test"
    exit 0
fi
for arg in "$@"; do
    if [[ "$arg" == "--help" ]]; then
        exit 0
    fi
done
echo "unrecognized subcommand" >&2
exit 1
EOF
chmod +x "$FAKE_GF"

git init -q "$TEST_DIR/repository"

# Best-effort mode may skip unavailable API operations.
GF_SMOKE_BINARY="$FAKE_GF" scripts/smoke-test.sh --read-only > "$TEST_DIR/best-effort.log"

# Explicit repository mode must convert the same compatibility error to failure.
if GF_SMOKE_BINARY="$FAKE_GF" scripts/smoke-test.sh \
    --read-only --api-repo-dir "$TEST_DIR/repository" > "$TEST_DIR/strict.log" 2>&1; then
    echo "expected explicit repository smoke test to fail" >&2
    exit 1
fi
grep -q '测试失败' "$TEST_DIR/strict.log"

# External repository paths must stay below the invocation directory.
if GF_SMOKE_BINARY="$FAKE_GF" scripts/smoke-test.sh \
    --read-only --api-repo-dir /tmp > "$TEST_DIR/absolute.log" 2>&1; then
    echo "expected absolute repository path to be rejected" >&2
    exit 1
fi
grep -q "不含 '..' 的相对路径" "$TEST_DIR/absolute.log"

if GF_SMOKE_BINARY="$FAKE_GF" scripts/smoke-test.sh \
    --read-only --api-repo-dir "$TEST_DIR/../repository" > "$TEST_DIR/parent.log" 2>&1; then
    echo "expected parent traversal repository path to be rejected" >&2
    exit 1
fi
grep -q "不含 '..' 的相对路径" "$TEST_DIR/parent.log"

echo "smoke-test script checks passed"
