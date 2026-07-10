#!/usr/bin/env bash
# Demo script to show the release workflow without actually releasing
# This simulates the interactive flow

set -euo pipefail

echo ""
echo "=== Release Workflow Demo ==="
echo ""
echo "This demo shows what 'make release' will do."
echo ""

# Show what the script will detect
echo "Step 1: Pre-flight Checks"
echo "  ✓ Checks if on main branch"
echo "  ✓ Checks if working directory is clean"
echo "  ✓ Runs tests (make test)"
echo "  ✓ Runs clippy (make clippy)"
echo ""

# Show version inference
CURRENT_VERSION=$(cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version')
LAST_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "v0.0.0")

echo "Step 2: Version Analysis"
echo "  Current version: v${CURRENT_VERSION}"
echo "  Last tag: ${LAST_TAG}"

# Count commits
COMMIT_COUNT=$(git log "${LAST_TAG}..HEAD" --oneline --no-merges 2>/dev/null | wc -l | tr -d ' ')
FEAT_COUNT=$(git log "${LAST_TAG}..HEAD" --pretty=format:"%s" --no-merges 2>/dev/null | grep -c "^feat" || echo "0")
FIX_COUNT=$(git log "${LAST_TAG}..HEAD" --pretty=format:"%s" --no-merges 2>/dev/null | grep -c "^fix" || echo "0")

echo "  Commits since last tag: ${COMMIT_COUNT}"
echo "    - Features: ${FEAT_COUNT}"
echo "    - Fixes: ${FIX_COUNT}"

# Infer bump
if git log "${LAST_TAG}..HEAD" --pretty=format:"%s" --no-merges 2>/dev/null | grep -qE "(^feat!|BREAKING CHANGE)"; then
    BUMP="major"
elif git log "${LAST_TAG}..HEAD" --pretty=format:"%s" --no-merges 2>/dev/null | grep -q "^feat"; then
    BUMP="minor"
else
    BUMP="patch"
fi

echo "  Inferred bump: ${BUMP}"

# Calculate next version
IFS='.' read -r major minor patch <<< "${CURRENT_VERSION}"
case "$BUMP" in
    major) NEXT="$((major + 1)).0.0" ;;
    minor) NEXT="${major}.$((minor + 1)).0" ;;
    patch) NEXT="${major}.${minor}.$((patch + 1))" ;;
esac

echo "  Next version: v${NEXT}"
echo ""

echo "Step 3: Interactive Version Selection"
echo "  You'll be prompted to choose:"
echo "    1) Major (breaking changes) → v1.0.0"
echo "    2) Minor (new features)     → v${NEXT}"
echo "    3) Patch (bug fixes)        → v0.7.1"
echo "    4) Custom version"
echo ""

echo "Step 4: Changelog Preview"
echo "  Shows first 50 lines of generated CHANGELOG.md"
echo "  You can review and confirm"
echo ""

echo "Step 5: Dry Run"
echo "  Shows what will happen:"
echo "    • Bump version to v${NEXT}"
echo "    • Commit version change"
echo "    • Generate CHANGELOG.md"
echo "    • Commit changelog"
echo "    • Create tag v${NEXT}"
echo "    • Push to origin/main with tags"
echo ""

echo "Step 6: Execute Release"
echo "  After confirmation, actually performs the release"
echo ""

echo "=== Ready to Release? ==="
echo ""
echo "To actually release, run: make release"
echo ""
