#!/usr/bin/env bash
set -euo pipefail

# Release script that automatically switches from dev to main branch
# Usage: ./scripts/release-from-dev.sh [VERSION_TYPE]
# VERSION_TYPE: patch (default), minor, major

VERSION_TYPE="${1:-patch}"

echo "🚀 gf-workflow Release Script"
echo "================================"
echo ""

# Check current branch
CURRENT_BRANCH=$(git branch --show-current)
echo "📍 Current branch: $CURRENT_BRANCH"

if [ "$CURRENT_BRANCH" = "main" ]; then
    echo "✅ Already on main branch, proceeding with release..."
elif [ "$CURRENT_BRANCH" = "dev" ]; then
    echo "🔄 On dev branch, switching to main for release..."

    # Check for uncommitted changes
    if ! git diff --quiet || ! git diff --cached --quiet; then
        echo "❌ Error: You have uncommitted changes on dev branch"
        echo "   Please commit or stash them first"
        exit 1
    fi

    # Switch to main
    git checkout main

    # Pull latest main
    echo "📥 Pulling latest main..."
    git pull origin main

    # Merge dev into main
    echo "🔀 Merging dev into main..."
    git merge dev -m "chore: release - merge dev into main"

    echo "✅ Successfully switched to main and merged dev"
else
    echo "⚠️  Warning: You're on branch '$CURRENT_BRANCH' (not dev or main)"
    echo "   Proceeding anyway, but this is unusual..."
    read -p "   Continue? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

echo ""
echo "📦 Starting release process..."
echo ""

# Run the standard release script
exec bash scripts/release.sh "$VERSION_TYPE"
