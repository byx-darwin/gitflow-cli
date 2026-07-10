#!/usr/bin/env bash
# Improved release workflow with safety checks and interactive preview
# Usage: bash scripts/release.sh [--quick]
set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Quick mode flag
QUICK_MODE=false
if [[ "${1:-}" == "--quick" ]]; then
    QUICK_MODE=true
fi

# Helper functions
log_info() {
    echo -e "${BLUE}==>${NC} $1"
}

log_success() {
    echo -e "${GREEN}✓${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}⚠${NC} $1"
}

log_error() {
    echo -e "${RED}✗${NC} $1"
}

confirm() {
    if $QUICK_MODE; then
        return 0
    fi
    echo -e "${CYAN}?${NC} $1 [y/N]"
    read -r response
    [[ "$response" =~ ^[Yy]$ ]]
}

# Cleanup function for rollback
cleanup_on_error() {
    local exit_code=$?
    if [ $exit_code -ne 0 ]; then
        echo ""
        log_error "Release failed with exit code $exit_code"
        echo ""
        log_warn "If the release was partially completed, you may need to:"
        echo "  1. git reset --hard HEAD~1  (remove version bump commit)"
        echo "  2. git tag -d <tag>         (remove local tag)"
        echo "  3. git push origin :<tag>   (remove remote tag)"
        echo ""
    fi
    exit $exit_code
}

trap cleanup_on_error EXIT

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."

    if ! command -v cargo &> /dev/null; then
        log_error "cargo not found. Please install Rust."
        exit 1
    fi

    if ! command -v cargo-release &> /dev/null; then
        log_error "cargo-release not found. Run: cargo install cargo-release"
        exit 1
    fi

    if ! command -v git-cliff &> /dev/null; then
        log_error "git-cliff not found. Run: cargo install git-cliff"
        exit 1
    fi

    log_success "Prerequisites OK"
}

# Pre-flight checks
preflight_checks() {
    log_info "Running pre-flight checks..."

    # Check if on main branch
    local current_branch
    current_branch=$(git branch --show-current)
    if [ "$current_branch" != "main" ]; then
        log_error "Must be on 'main' branch. Current: $current_branch"
        exit 1
    fi
    log_success "On main branch"

    # Check working directory is clean
    if [ -n "$(git status --porcelain)" ]; then
        log_error "Working directory is not clean. Commit or stash changes first."
        git status --short
        exit 1
    fi
    log_success "Working directory clean"

    # Check if there are unpushed commits
    if [ "$(git rev-parse HEAD)" != "$(git rev-parse origin/main 2>/dev/null || echo '')" ]; then
        log_warn "There are unpushed commits on main. Make sure they are reviewed."
    fi

    # Run tests
    log_info "Running tests..."
    if ! make test > /dev/null 2>&1; then
        log_error "Tests failed. Fix failures before releasing."
        exit 1
    fi
    log_success "Tests passed"

    # Run clippy
    log_info "Running clippy..."
    if ! make clippy > /dev/null 2>&1; then
        log_error "Clippy check failed. Fix warnings before releasing."
        exit 1
    fi
    log_success "Clippy passed"

    echo ""
}

# Get current version
get_current_version() {
    cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version'
}

# Infer version bump from conventional commits
infer_version_bump() {
    local last_tag="$1"

    # Get commits since last tag
    local commits
    commits=$(git log "${last_tag}..HEAD" --pretty=format:"%s" --no-merges 2>/dev/null || echo "")

    if [ -z "$commits" ]; then
        echo "patch"  # Default to patch if no commits
        return
    fi

    # Check for breaking changes
    if echo "$commits" | grep -qE "(^feat!|BREAKING CHANGE)"; then
        echo "major"
        return
    fi

    # Check for features
    if echo "$commits" | grep -q "^feat"; then
        echo "minor"
        return
    fi

    # Default to patch
    echo "patch"
}

# Calculate next version
calculate_next_version() {
    local current="$1"
    local bump="$2"

    # Split version into parts
    local major minor patch
    IFS='.' read -r major minor patch <<< "$current"

    case "$bump" in
        major)
            echo "$((major + 1)).0.0"
            ;;
        minor)
            echo "${major}.$((minor + 1)).0"
            ;;
        patch)
            echo "${major}.${minor}.$((patch + 1))"
            ;;
        *)
            echo "$current"
            ;;
    esac
}

# Show version preview
show_version_preview() {
    log_info "Analyzing version bump..."

    local current_version
    current_version=$(get_current_version)

    local last_tag
    last_tag=$(git describe --tags --abbrev=0 2>/dev/null || echo "v0.0.0")

    local inferred_bump
    inferred_bump=$(infer_version_bump "$last_tag")

    local next_version
    next_version=$(calculate_next_version "$current_version" "$inferred_bump")

    echo ""
    echo -e "${CYAN}Version Preview${NC}"
    echo "  Current:  v${current_version}"
    echo "  Last tag: ${last_tag}"
    echo "  Inferred: ${inferred_bump} bump"
    echo -e "  ${GREEN}Next:     v${next_version}${NC}"
    echo ""

    # Show commit summary
    local commit_count
    commit_count=$(git log "${last_tag}..HEAD" --oneline --no-merges 2>/dev/null | wc -l | tr -d ' ')
    local feat_count
    feat_count=$(git log "${last_tag}..HEAD" --pretty=format:"%s" --no-merges 2>/dev/null | grep -c "^feat" || echo "0")
    local fix_count
    fix_count=$(git log "${last_tag}..HEAD" --pretty=format:"%s" --no-merges 2>/dev/null | grep -c "^fix" || echo "0")

    echo -e "${CYAN}Commits since ${last_tag}:${NC} ${commit_count} total"
    echo "  Features: ${feat_count}"
    echo "  Fixes:    ${fix_count}"
    echo ""

    # Ask for version confirmation
    local selected_version="$next_version"
    local selected_bump="$inferred_bump"

    if ! $QUICK_MODE; then
        echo "Choose version bump:"
        echo "  1) Major (breaking changes) → v$(calculate_next_version "$current_version" "major")"
        echo "  2) Minor (new features)     → v$(calculate_next_version "$current_version" "minor")"
        echo "  3) Patch (bug fixes)        → v$(calculate_next_version "$current_version" "patch")"
        echo "  4) Custom version"
        echo ""

        read -r -p "Select [1-4] (default: 2): " choice
        choice=${choice:-2}

        case "$choice" in
            1)
                selected_bump="major"
                selected_version=$(calculate_next_version "$current_version" "major")
                ;;
            2)
                selected_bump="minor"
                selected_version=$(calculate_next_version "$current_version" "minor")
                ;;
            3)
                selected_bump="patch"
                selected_version=$(calculate_next_version "$current_version" "patch")
                ;;
            4)
                read -r -p "Enter custom version (e.g., 1.2.3): " selected_version
                ;;
            *)
                log_error "Invalid choice"
                exit 1
                ;;
        esac
    fi

    echo ""
    log_info "Selected version: v${selected_version}"

    # Export for use in later steps
    export RELEASE_VERSION="$selected_version"
    export RELEASE_BUMP="$selected_bump"
}

# Generate and preview changelog
preview_changelog() {
    log_info "Generating changelog preview..."

    # Generate changelog to temp file
    local temp_changelog
    temp_changelog=$(mktemp)

    git cliff -o "$temp_changelog" 2>/dev/null || {
        log_warn "git-cliff failed. Continuing without preview."
        rm -f "$temp_changelog"
        return 0
    }

    echo ""
    echo -e "${CYAN}=== Changelog Preview ===${NC}"
    head -50 "$temp_changelog"
    local total_lines
    total_lines=$(wc -l < "$temp_changelog")
    if [ "$total_lines" -gt 50 ]; then
        echo ""
        echo "... (showing first 50 of $total_lines lines)"
    fi
    echo -e "${CYAN}=========================${NC}"
    echo ""

    rm -f "$temp_changelog"

    if ! confirm "Changelog looks good?"; then
        log_warn "Aborting release."
        exit 0
    fi

    echo ""
}

# Dry run
dry_run() {
    log_info "Running dry-run..."

    echo ""
    echo -e "${CYAN}=== Dry Run ===${NC}"
    echo "This will:"
    echo "  1. Bump version to v${RELEASE_VERSION}"
    echo "  2. Commit version change"
    echo "  3. Generate CHANGELOG.md"
    echo "  4. Commit changelog"
    echo "  5. Create tag v${RELEASE_VERSION}"
    echo "  6. Push to origin/main with tags"
    echo -e "${CYAN}===============${NC}"
    echo ""

    # Run cargo release dry-run
    cargo release version "${RELEASE_BUMP}" --dry-run --workspace 2>&1 | head -20 || true

    echo ""

    if ! confirm "Proceed with actual release?"; then
        log_warn "Release cancelled by user."
        exit 0
    fi

    echo ""
}

# Execute release
execute_release() {
    log_info "Executing release v${RELEASE_VERSION}..."

    # Step 1: Bump version
    log_info "Step 1/5: Bumping version..."
    cargo release version "${RELEASE_BUMP}" --execute --workspace --no-confirm

    # Step 2: Commit version
    log_info "Step 2/5: Committing version bump..."
    cargo release commit --execute --no-confirm

    # Step 3: Generate changelog
    log_info "Step 3/5: Generating CHANGELOG.md..."
    git cliff -o CHANGELOG.md

    # Step 4: Commit changelog
    log_info "Step 4/5: Committing changelog..."
    git add CHANGELOG.md
    git commit -m "chore: update CHANGELOG.md for v${RELEASE_VERSION}" || true

    # Step 5: Create tag and push
    log_info "Step 5/5: Creating tag and pushing..."
    cargo release tag --execute --workspace --no-confirm
    git push origin main --tags

    echo ""
    log_success "Release v${RELEASE_VERSION} completed!"
}

# Post-release info
post_release() {
    echo ""
    echo -e "${GREEN}=== Release Complete ===${NC}"
    echo ""
    echo "Tag:            v${RELEASE_VERSION}"
    echo "Release URL:    https://github.com/byx-darwin/gitflow-cli/releases/tag/v${RELEASE_VERSION}"
    echo ""
    echo "Next steps:"
    echo "  • GitHub Release should be auto-created by CI"
    echo "  • Monitor CI: gh run list --limit 1"
    echo "  • Update Homebrew formula if needed: make package"
    echo ""
}

# Main flow
main() {
    echo ""
    echo -e "${CYAN}╔════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║   gitflow-cli Release Workflow         ║${NC}"
    echo -e "${CYAN}╚════════════════════════════════════════╝${NC}"
    echo ""

    check_prerequisites
    preflight_checks
    show_version_preview
    preview_changelog

    if ! $QUICK_MODE; then
        dry_run
    fi

    execute_release
    post_release
}

main "$@"
