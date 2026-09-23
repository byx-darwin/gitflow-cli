#!/usr/bin/env bash
# validate-skill-links-test.sh — fixture-based tests for validate-skill-links.sh
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TARGET="$SCRIPT_DIR/../validate-skill-links.sh"
PASS=0
FAIL=0

fixture_root=""
setup_fixture() {
    fixture_root="$(mktemp -d)"
}
teardown_fixture() {
    [ -n "$fixture_root" ] && rm -rf "$fixture_root"
    fixture_root=""
}

expect_exit() { # $1=expected code, $2=test name
    local expected="$1" name="$2" actual
    bash "$TARGET" --root "$fixture_root" >/dev/null 2>&1
    actual=$?
    if [ "$actual" -eq "$expected" ]; then
        echo "✅ $name"
        PASS=$((PASS + 1))
    else
        echo "❌ $name (expected exit $expected, got $actual)"
        FAIL=$((FAIL + 1))
    fi
}

# --- Case 1: all references resolve -> exit 0 ---
setup_fixture
mkdir -p "$fixture_root/gf-demo/references"
cat > "$fixture_root/gf-demo/SKILL.md" <<'EOF'
See [checklist](references/rust.md) for details.
Load `references/rust.md` when Cargo.toml is present.
EOF
touch "$fixture_root/gf-demo/references/rust.md"
expect_exit 0 "resolving link and load target pass"
teardown_fixture

# --- Case 2: broken backticked load target -> exit 1 ---
setup_fixture
mkdir -p "$fixture_root/gf-demo/references"
cat > "$fixture_root/gf-demo/references/detector.md" <<'EOF'
| `Gemfile` | Ruby | `references/ruby.md` |
EOF
expect_exit 1 "broken load target detected"
teardown_fixture

# --- Case 3: broken markdown link -> exit 1 ---
setup_fixture
mkdir -p "$fixture_root/gf-demo"
cat > "$fixture_root/gf-demo/SKILL.md" <<'EOF'
See [taxonomy](../references/missing.md).
EOF
expect_exit 1 "broken markdown link detected"
teardown_fixture

# --- Case 4: non-reference paths are skipped -> exit 0 ---
setup_fixture
mkdir -p "$fixture_root/gf-demo"
cat > "$fixture_root/gf-demo/SKILL.md" <<'EOF'
Visit [site](https://example.com/x.md) or [anchor](#section).
Write output to `/tmp/report.md` — absolute, not a reference.
Template path `docs/report-<YYYY-MM-DD>.md` uses a placeholder.
Glob `code-review-report-*.md` is not a path.
Run `rm -f /tmp/report.md` to clean up.
EOF
expect_exit 0 "absolute/placeholder/glob/command paths skipped"
teardown_fixture

# --- Case 5: load target resolves against skill root, not container dir -> exit 0 ---
setup_fixture
mkdir -p "$fixture_root/gf-demo/references"
cat > "$fixture_root/gf-demo/references/detector.md" <<'EOF'
| `Cargo.toml` | Rust | `references/rust.md` |
EOF
touch "$fixture_root/gf-demo/references/rust.md"
expect_exit 0 "load target resolves against skill root"
teardown_fixture

echo ""
echo "=== Summary: $PASS passed, $FAIL failed ==="
[ "$FAIL" -eq 0 ]
