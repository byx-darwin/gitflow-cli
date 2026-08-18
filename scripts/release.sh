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

# Mode flags
QUICK_MODE=false
REHEARSE_MODE=false
case "${1:-}" in
    --quick) QUICK_MODE=true ;;
    --rehearse) REHEARSE_MODE=true; QUICK_MODE=true ;;
    --self-test)
        # self-test 无需发布前置,直接运行并退出
        QUICK_MODE=true
        ;;
    "") ;;
    *)
        echo "Usage: bash scripts/release.sh [--quick|--rehearse|--self-test]" >&2
        exit 2
        ;;
esac

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

# ---------------------------------------------------------------------------
# Release artifact validation (pure functions; testable via --self-test)
# ---------------------------------------------------------------------------

# 未被替换的模板变量,如 {version} 或 {{version}}
TEMPLATE_RESIDUE_PATTERN='\{\{?[a-zA-Z_]+\}\}?'
# 合法 tag:vX.Y.Z 或 vX.Y.Z-<prerelease>
VERSION_TAG_PATTERN='^v[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$'
# 合法发布提交主题
RELEASE_COMMIT_PATTERN='^chore: release v[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$'

validate_commit_subject() {
    local subject="$1"
    if [[ "$subject" =~ $TEMPLATE_RESIDUE_PATTERN ]]; then
        log_error "Template residue in commit subject: $subject"
        log_error "Expected: chore: release v1.2.3"
        log_error "This usually means release.toml uses incorrect placeholder syntax."
        log_error "For cargo-release 1.1.3+, use {{version}} (double curly braces)."
        return 1
    fi
    if [[ ! "$subject" =~ $RELEASE_COMMIT_PATTERN ]]; then
        log_error "Malformed release commit subject: $subject"
        return 1
    fi
    return 0
}

validate_tag_name() {
    local tag="$1"
    if [[ "$tag" =~ $TEMPLATE_RESIDUE_PATTERN ]]; then
        log_error "Template residue in tag name: $tag"
        log_error "Expected: v1.2.3"
        log_error "This usually means release.toml uses incorrect placeholder syntax."
        log_error "For cargo-release 1.1.3+, use {{version}} (double curly braces)."
        return 1
    fi
    if [[ ! "$tag" =~ $VERSION_TAG_PATTERN ]]; then
        log_error "Malformed tag name: $tag"
        return 1
    fi
    return 0
}

validate_no_template_residue() {
    local file="$1"
    # Check heading lines (# ## ###) for unsubstituted template variables.
    # Body text (commit messages) may legitimately mention {{version}} when
    # describing template-syntax fixes, so only headings are validated.
    local matches
    matches=$(grep -nE '^#+.*'"$TEMPLATE_RESIDUE_PATTERN" "$file" || true)
    if [ -n "$matches" ]; then
        log_error "Template residue found in $file:"
        echo "$matches" | head -5
        return 1
    fi
    return 0
}

# Classify `gh pr checks` output into passed|failed|pending.
# gh exits 8 while any check is pending or failed, so callers must tolerate a
# non-zero exit from the command itself (see execute_release); this function
# only parses the captured text. `grep -c` with zero matches prints "0" and
# exits 1, so `|| true` keeps the count as "0" without set -e aborting.
ci_checks_state() {
    local checks_output="$1"
    local pending_count failed_count
    pending_count=$(printf '%s\n' "$checks_output" | grep -c "pending" || true)
    failed_count=$(printf '%s\n' "$checks_output" | grep -c "fail" || true)
    if [ "$failed_count" -gt 0 ]; then
        echo "failed"
    elif [ "$pending_count" -eq 0 ]; then
        echo "passed"
    else
        echo "pending"
    fi
}

run_self_test() {
    local failures=0

    expect_pass() {
        local desc="$1"; shift
        if "$@" >/dev/null 2>&1; then
            log_success "$desc"
        else
            log_error "$desc (expected pass, got fail)"
            failures=$((failures + 1))
        fi
    }

    expect_fail() {
        local desc="$1"; shift
        if "$@" >/dev/null 2>&1; then
            log_error "$desc (expected fail, got pass)"
            failures=$((failures + 1))
        else
            log_success "$desc"
        fi
    }

    echo ""
    log_info "Running release validation self-test..."

    expect_pass "commit subject: well-formed" validate_commit_subject "chore: release v1.0.0"
    expect_pass "commit subject: prerelease" validate_commit_subject "chore: release v1.0.0-rc.1"
    expect_fail "commit subject: template residue" validate_commit_subject "chore: release v{{version}}"
    # NEW: single-brace residue detection
    expect_fail "commit subject: single-brace residue" validate_commit_subject "chore: release v{version}"
    expect_fail "commit subject: malformed" validate_commit_subject "release 1.0.0"

    expect_pass "tag: well-formed" validate_tag_name "v1.0.0"
    expect_pass "tag: prerelease" validate_tag_name "v1.0.0-rc.1"
    expect_fail "tag: template residue" validate_tag_name "v{{version}}"
    expect_fail "tag: single-brace residue" validate_tag_name "v{version}"
    expect_fail "tag: missing v prefix" validate_tag_name "1.0.0"

    local tmp
    tmp=$(mktemp)
    printf '## v{{version}}\n' > "$tmp"
    expect_fail "changelog: template residue" validate_no_template_residue "$tmp"
    printf '## v{version}\n' > "$tmp"
    expect_fail "changelog: single-brace residue" validate_no_template_residue "$tmp"
    printf '## 1.0.0 - 2026-07-31\n' > "$tmp"
    expect_pass "changelog: clean" validate_no_template_residue "$tmp"
    printf -- '- **(release)** use {{version}} template syntax - ([abc1234](https://github.com/x/y/commit/abc123))\n' > "$tmp"
    expect_pass "changelog: commit msg with {{version}} excluded" validate_no_template_residue "$tmp"
    printf -- '- Merge pull request #159: fix(release) use {{version}} template syntax for cargo-release 1.1.3\n' >> "$tmp"
    expect_pass "changelog: merge msg with {{version}} excluded" validate_no_template_residue "$tmp"
    rm -f "$tmp"

    # ci_checks_state: `gh pr checks` output (NAME<TAB>STATE<TAB>...) → passed|failed|pending
    expect_pass "ci state: all pass → passed" test "$(ci_checks_state $'lint\tpass\t0\nbuild\tpass\t0')" = "passed"
    expect_pass "ci state: some pending → pending" test "$(ci_checks_state $'lint\tpass\t0\nbuild\tpending\t0')" = "pending"
    expect_pass "ci state: failed + pending → failed" test "$(ci_checks_state $'lint\tfail\t0\nbuild\tpending\t0')" = "failed"
    expect_fail "ci state: failed not misread as passed" test "$(ci_checks_state $'lint\tfail\t0')" = "passed"
    expect_fail "ci state: pending not misread as passed" test "$(ci_checks_state $'build\tpending\t0')" = "passed"

    echo ""
    if [ "$failures" -eq 0 ]; then
        log_success "Self-test passed"
        return 0
    fi
    log_error "Self-test failed: $failures case(s)"
    return 1
}

if [[ "${1:-}" == "--self-test" ]]; then
    trap - EXIT
    if run_self_test; then exit 0; else exit 1; fi
fi

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

    # Check if on dev or main branch
    local current_branch
    current_branch=$(git branch --show-current)
    if [ "$current_branch" != "dev" ] && [ "$current_branch" != "main" ]; then
        log_error "Must be on 'dev' or 'main' branch. Current: $current_branch"
        exit 1
    fi
    log_success "On $current_branch branch"

    # If on main, switch to dev
    if [ "$current_branch" = "main" ]; then
        log_info "Switching to dev branch..."
        git checkout dev
        git pull origin dev
        log_success "Switched to dev"
    fi

    # Check working directory is clean
    if [ -n "$(git status --porcelain)" ]; then
        log_error "Working directory is not clean. Commit or stash changes first."
        git status --short
        exit 1
    fi
    log_success "Working directory clean"

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

    # Run format check
    log_info "Running format check..."
    if ! make fmt > /dev/null 2>&1; then
        log_error "Format check failed. Run 'cargo +nightly fmt' to fix."
        exit 1
    fi
    log_success "Format check passed"

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

    # Run cargo release dry-run; capture output and enforce real exit status.
    local dry_run_log
    dry_run_log=$(mktemp)
    set +e
    cargo release version "${RELEASE_BUMP}" --dry-run --workspace \
        >"$dry_run_log" 2>&1
    local dry_run_rc=$?
    set -e
    head -20 "$dry_run_log"

    # Fail the rehearsal if cargo release itself errored.
    if [ "$dry_run_rc" -ne 0 ]; then
        log_error "cargo release dry-run failed (exit $dry_run_rc). Fix errors before releasing."
        rm -f "$dry_run_log"
        exit 1
    fi

    # Detect template residue (single {var} or double {{var}} braces).
    # Any unsubstituted token in the dry-run output means templates did NOT get replaced.
    local residue_found=false
    if grep -qE "$TEMPLATE_RESIDUE_PATTERN" "$dry_run_log"; then
        residue_found=true
    fi
    rm -f "$dry_run_log"
    if [ "$residue_found" = true ]; then
        log_error "cargo release dry-run output contains unsubstituted template tokens."
        log_error "Verify release.toml templates match the installed cargo-release version."
        exit 1
    fi

    echo ""

    if ! confirm "Proceed with actual release?"; then
        log_warn "Release cancelled by user."
        exit 0
    fi

    echo ""
}

# Check CI status
check_ci_status() {
    log_info "Checking GitHub CI status..."

    if ! command -v gh &> /dev/null; then
        log_warn "gh CLI not found. Skipping CI check."
        log_warn "Install gh: https://cli.github.com/"
        if confirm "Continue without CI check?"; then
            return 0
        else
            log_error "Aborted by user"
            exit 1
        fi
    fi

    # Get the latest commit SHA
    local commit_sha
    commit_sha=$(git rev-parse HEAD)

    # Check CI status for this commit
    local ci_status
    ci_status=$(gh run list --commit "$commit_sha" --limit 1 --json conclusion --jq '.[0].conclusion' 2>/dev/null || echo "not_found")

    case "$ci_status" in
        "success")
            log_success "GitHub CI passed ✓"
            return 0
            ;;
        "failure"|"cancelled"|"timed_out")
            log_error "GitHub CI failed: $ci_status"
            log_error "Fix CI issues before releasing"
            echo ""
            echo "View CI status:"
            echo "  gh run list --commit $commit_sha"
            echo "  https://github.com/$(git remote get-url origin | sed 's/.*github.com[:/]\(.*\)\.git/\1/')/actions"
            exit 1
            ;;
        "in_progress"|"queued"|"waiting")
            log_warn "GitHub CI is still running: $ci_status"
            log_warn "Please wait for CI to complete"
            if confirm "Continue anyway?"; then
                return 0
            else
                log_info "Waiting for CI... (press Ctrl+C to abort)"
                while true; do
                    sleep 10
                    ci_status=$(gh run list --commit "$commit_sha" --limit 1 --json conclusion --jq '.[0].conclusion' 2>/dev/null || echo "not_found")
                    case "$ci_status" in
                        "success")
                            log_success "GitHub CI passed ✓"
                            return 0
                            ;;
                        "failure"|"cancelled"|"timed_out")
                            log_error "GitHub CI failed: $ci_status"
                            exit 1
                            ;;
                    esac
                    echo -n "."
                done
            fi
            ;;
        *)
            log_warn "CI status unknown: $ci_status"
            if confirm "Continue without CI check?"; then
                return 0
            else
                log_error "Aborted by user"
                exit 1
            fi
            ;;
    esac
}

# Execute release
execute_release() {
    log_info "Executing release v${RELEASE_VERSION}..."

    # Step 1: Create release branch from dev
    local release_branch="release/v${RELEASE_VERSION}"
    log_info "Step 1/8: Creating release branch: $release_branch"
    git checkout -b "$release_branch"

    # Step 2: Bump version
    log_info "Step 2/8: Bumping version..."
    cargo release version "${RELEASE_BUMP}" --execute --workspace --no-confirm

    # Step 3: Commit version
    log_info "Step 3/8: Committing version bump..."
    cargo release commit --execute --no-confirm

    # Gate: validate the release commit subject (blocks v{{version}}-class incidents)
    local commit_subject
    commit_subject=$(git log -1 --pretty=%s)
    if ! validate_commit_subject "$commit_subject"; then
        log_error "Release commit validation failed. Rolling back bump commit."
        git reset --hard HEAD~1
        git checkout dev
        git branch -D "$release_branch"
        trap - EXIT
        exit 1
    fi
    log_success "Release commit subject validated"

    # Step 4: Generate changelog
    log_info "Step 4/8: Generating CHANGELOG.md..."
    git cliff -o CHANGELOG.md

    # Step 5: Commit changelog
    log_info "Step 5/8: Committing changelog..."
    git add CHANGELOG.md
    git commit -m "chore: update CHANGELOG.md for v${RELEASE_VERSION}" || true

    # Gate: no template residue in the generated changelog
    if ! validate_no_template_residue CHANGELOG.md; then
        log_error "CHANGELOG.md contains unsubstituted template variables. Aborting."
        git checkout dev
        git branch -D "$release_branch"
        trap - EXIT
        exit 1
    fi
    log_success "CHANGELOG.md validated"

    # Step 6: Push release branch
    log_info "Step 6/8: Pushing release branch..."
    git push -u origin "$release_branch"

    # Step 7: Create PR to main
    log_info "Step 7/8: Creating PR to main..."
    local pr_url
    pr_url=$(gh pr create \
        --base main \
        --head "$release_branch" \
        --title "chore: release v${RELEASE_VERSION}" \
        --body "## Release v${RELEASE_VERSION}

Automated release PR created by release script.

### Changes
- Version bump: v${RELEASE_VERSION}
- Updated CHANGELOG.md

### Commits since last release
$(git log --oneline "$(git describe --tags --abbrev=0 2>/dev/null || echo 'HEAD')..HEAD" | head -20)

Ready for release! 🚀" 2>&1)

    if [ -z "$pr_url" ]; then
        log_error "Failed to create PR"
        exit 1
    fi

    local pr_number
    pr_number=$(echo "$pr_url" | grep -oE '[0-9]+$')
    log_success "PR created: $pr_url"

    # Step 8: Wait for CI and merge
    log_info "Step 8/8: Waiting for CI checks..."
    local max_wait=600  # 10 minutes
    local waited=0

    while true; do
        # gh pr checks exits 8 while any check is pending/failed; tolerate that
        # here so set -euo pipefail does not abort the loop before checks settle.
        local checks_output
        checks_output=$(gh pr checks "$pr_number" 2>&1 || true)
        local state
        state=$(ci_checks_state "$checks_output")

        if [ "$state" = "failed" ]; then
            log_error "CI checks failed!"
            echo "$checks_output" | grep "fail"
            log_error "Fix CI issues and retry"
            log_info "PR: $pr_url"
            exit 1
        fi

        if [ "$state" = "passed" ]; then
            log_success "All CI checks passed!"
            break
        fi

        if [ $waited -ge $max_wait ]; then
            log_error "Timeout waiting for CI"
            log_info "Please check manually: $pr_url"
            exit 1
        fi

        echo -n "."
        sleep 30
        waited=$((waited + 30))
    done

    echo ""
    log_info "Merging PR..."
    gh pr merge "$pr_number" --squash --delete-branch

    # Switch to main and pull
    log_info "Switching to main..."
    git checkout main
    git pull origin main

    # Create and push tag
    log_info "Creating tag v${RELEASE_VERSION}..."
    git tag "v${RELEASE_VERSION}"
    git push origin "v${RELEASE_VERSION}"

    # Wait for GitHub Actions to build binaries
    log_info "Waiting for GitHub Actions to build binaries..."
    sleep 30

    # Publish to crates.io
    if confirm "Publish to crates.io?"; then
        log_info "Publishing to crates.io..."
        # Stage skills for crates.io package
        make stage-skills-for-publish
        # Publish using cargo release
        cargo release publish --execute --no-confirm --registry crates-io --allow-dirty || {
            log_error "Failed to publish to crates.io"
            log_warn "You can retry manually: make stage-skills-for-publish && cargo release publish --execute --no-confirm --registry crates-io --allow-dirty"
        }
        # Clean up staged skills
        make clean-staged-skills
        log_success "Published to crates.io"
    else
        log_warn "Skipping crates.io publish"
    fi

    # Update Homebrew formula
    if confirm "Update Homebrew formula?"; then
        log_info "Updating Homebrew formula..."

        # Wait for GitHub release to be created
        log_info "Waiting for GitHub release to be created..."
        local release_url="https://github.com/byx-darwin/gitflow-cli/releases/download/v${RELEASE_VERSION}"
        local max_attempts=20
        local attempt=0

        while [ $attempt -lt $max_attempts ]; do
            if curl -s -I "${release_url}/gf-aarch64-apple-darwin.tar.gz" | grep -q "200"; then
                log_success "GitHub release binaries ready"
                break
            fi
            echo -n "."
            sleep 15
            attempt=$((attempt + 1))
        done

        if [ $attempt -ge $max_attempts ]; then
            log_warn "Timeout waiting for GitHub release binaries"
            log_warn "Please update Homebrew formula manually later"
        else
            # Download and calculate SHA256 for each platform
            local tmp_dir=$(mktemp -d)
            cd "$tmp_dir"

            local platforms=(
                "aarch64-apple-darwin"
                "x86_64-apple-darwin"
                "aarch64-unknown-linux-gnu"
                "x86_64-unknown-linux-gnu"
            )

            declare -A sha256_sums

            for platform in "${platforms[@]}"; do
                local tarball="gf-${platform}.tar.gz"
                log_info "Downloading $tarball..."
                curl -sL -o "$tarball" "${release_url}/${tarball}"
                local sha256=$(shasum -a 256 "$tarball" | awk '{print $1}')
                sha256_sums[$platform]=$sha256
                log_success "$platform: $sha256"
            done

            cd - > /dev/null
            rm -rf "$tmp_dir"

            # Update Homebrew formula in homebrew-tap repository
            local tap_repo="../homebrew-tap"
            log_info "Updating Homebrew formula in homebrew-tap..."

            # Clone or update homebrew-tap repository
            if [ ! -d "$tap_repo" ]; then
                log_info "Cloning homebrew-tap repository..."
                git clone git@github.com:byx-darwin/homebrew-tap.git "$tap_repo"
            else
                log_info "Updating homebrew-tap repository..."
                cd "$tap_repo"
                git checkout main
                git pull origin main
                cd - > /dev/null
            fi

            local formula_file="$tap_repo/Formula/gf.rb"
            log_info "Updating $formula_file..."

            # Create updated formula
            cat > "$formula_file" <<EOF
class Gf < Formula
  desc "Multi-platform Git forge CLI — unified interface for GitHub, GitLab, and GitCode"
  homepage "https://github.com/byx-darwin/gitflow-cli"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "${release_url}/gf-${RELEASE_VERSION}-aarch64-apple-darwin.tar.gz"
      sha256 "${sha256_sums[aarch64-apple-darwin]}"
    else
      url "${release_url}/gf-${RELEASE_VERSION}-x86_64-apple-darwin.tar.gz"
      sha256 "${sha256_sums[x86_64-apple-darwin]}"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "${release_url}/gf-${RELEASE_VERSION}-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "${sha256_sums[aarch64-unknown-linux-gnu]}"
    else
      url "${release_url}/gf-${RELEASE_VERSION}-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "${sha256_sums[x86_64-unknown-linux-gnu]}"
    end
  end

  # gh CLI 是运行时依赖（GitHub 平台需要）
  # glab 和 gc 是可选的（GitLab/GitCode 平台需要）
  depends_on "gh"

  def install
    bin.install "gf"

    # 安装 Shell 补全
    generate_completions_from_executable(bin/"gf", "completions")
  end

  test do
    system "#{bin}/gf", "--version"
    system "#{bin}/gf", "--help"
  end
end
EOF

            # Commit and push Homebrew formula update
            cd "$tap_repo"
            git add "$formula_file"
            git commit -m "feat: update gf formula to v${RELEASE_VERSION}"
            git push origin main
            cd - > /dev/null
            log_success "Homebrew formula updated in homebrew-tap"
        fi
    else
        log_warn "Skipping Homebrew formula update"
    fi

    # Sync main back to dev
    log_info "Syncing main back to dev..."
    git checkout dev
    git merge main -m "chore: sync main back to dev after release v${RELEASE_VERSION}"
    git push origin dev

    # Cleanup
    log_info "Cleaning up..."
    git branch -D "$release_branch" 2>/dev/null || true

    echo ""
    log_success "Release v${RELEASE_VERSION} completed!"
    log_info "GitHub Release will be created automatically by CI"
    log_info "PR: $pr_url"
    log_info "Tag: v${RELEASE_VERSION}"
}

# Post-release info
post_release() {
    echo ""
    echo -e "${GREEN}=== Release Complete ===${NC}"
    echo ""
    echo "Tag:            v${RELEASE_VERSION}"
    echo "Release URL:    https://github.com/byx-darwin/gf/releases/tag/v${RELEASE_VERSION}"
    echo ""
    echo "Next steps:"
    echo "  • GitHub Release should be auto-created by CI"
    echo "  • Monitor CI: gh run list --limit 1"
    echo "  • Update Homebrew formula if needed: make package"
    echo ""
}

# Rehearsal report (dry-run drill; prints the mandatory checklist)
print_rehearsal_report() {
    echo ""
    echo -e "${GREEN}╔════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║   Release Rehearsal Report (dry-run)   ║${NC}"
    echo -e "${GREEN}╚════════════════════════════════════════╝${NC}"
    echo ""
    echo "  ✅ Prerequisites (cargo / cargo-release / git-cliff)"
    echo "  ✅ On main branch, working tree clean"
    echo "  ✅ Tests passed (cargo nextest)"
    echo "  ✅ Clippy passed"
    echo "  ✅ Version preview: v${RELEASE_VERSION}"
    echo "  ✅ cargo release dry-run succeeded"
    echo "  ✅ Validation self-test:"
    if run_self_test > /dev/null 2>&1; then
        echo "     validators green (commit subject / tag / changelog residue)"
    else
        log_error "Validation self-test failed during rehearsal"
        exit 1
    fi
    echo ""
    log_success "Rehearsal passed. No changes were made (dry-run only)."
    log_info "Run 'bash scripts/release.sh' to perform the actual release."
}

# Main flow
main() {
    echo ""
    echo -e "${CYAN}╔════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║   gf Release Workflow         ║${NC}"
    echo -e "${CYAN}╚════════════════════════════════════════╝${NC}"
    echo ""

    check_prerequisites
    preflight_checks
    show_version_preview
    preview_changelog

    # dry-run 是强制步骤:--quick 仅跳过交互确认,不跳过 dry-run
    dry_run

    if $REHEARSE_MODE; then
        print_rehearsal_report
        exit 0
    fi

    execute_release
    post_release
}

main "$@"
