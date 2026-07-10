# release: Improved Release Workflow
## Status: Draft

## Context

The current `make release` workflow works but lacks safety checks and user feedback. This plan proposes improvements to make the release process more robust and user-friendly.

## Current Flow Issues

1. No pre-flight validation (tests, clippy, working directory)
2. No version preview (current → next)
3. No changelog preview for user review
4. No dry-run integration before actual release
5. No rollback mechanism on failure

## Proposed Improvements

### Phase 1: Pre-flight Checks
- Verify working directory is clean
- Verify on `main` branch
- Run `make test` and ensure all pass
- Run `make clippy` and ensure no warnings
- Check if there are unpushed commits

### Phase 2: Version Preview
- Show current version: `cargo pkgid | cut -d# -f2`
- Infer next version based on commits (feat→minor, fix→patch, breaking→major)
- Show proposed version
- Ask user to confirm or override

### Phase 3: Changelog Preview
- Generate changelog to temp file
- Display changelog content
- Ask user to review and confirm

### Phase 4: Dry-run
- Run `cargo release --dry-run` first
- Show what will happen
- Ask final confirmation

### Phase 5: Execute Release
- Execute `cargo release` commands
- Generate and commit final changelog
- Create and push tag
- Verify CI starts

### Phase 6: Post-release
- Show release URL
- Remind about GitHub Release (if not auto-created)
- Suggest next steps (Homebrew update, etc.)

## Implementation Plan

### Step 1: Create `scripts/release.sh` wrapper
A bash script that orchestrates the improved flow:
- Pre-flight checks
- Version inference and preview
- Changelog preview
- Dry-run
- Execute release
- Post-release feedback

### Step 2: Update Makefile
```makefile
release: ## Interactive release with safety checks
	@bash scripts/release.sh

release-quick: ## Quick release without previews (for automation)
	@bash scripts/release.sh --quick
```

### Step 3: Add version inference logic
Parse conventional commits since last tag:
- `feat!` or `BREAKING CHANGE` → major
- `feat` → minor
- `fix`/`refactor`/`perf` → patch

### Step 4: Add pre-flight checks
```bash
# Check working directory
if [ -n "$(git status --porcelain)" ]; then
  echo "Error: Working directory not clean"
  exit 1
fi

# Check branch
if [ "$(git branch --show-current)" != "main" ]; then
  echo "Error: Must be on main branch"
  exit 1
fi

# Run tests
make test || { echo "Tests failed"; exit 1; }

# Run clippy
make clippy || { echo "Clippy failed"; exit 1; }
```

### Step 5: Add version inference
```bash
CURRENT_VERSION=$(cargo pkgid | cut -d# -f2)
LAST_TAG=$(git describe --tags --abbrev=0)
COMMITS=$(git log ${LAST_TAG}..HEAD --pretty=format:"%s")

# Count commit types
FEAT_COUNT=$(echo "$COMMITS" | grep -c "^feat" || true)
FIX_COUNT=$(echo "$COMMITS" | grep -c "^fix" || true)
BREAKING=$(echo "$COMMITS" | grep -c "BREAKING CHANGE" || true)

# Infer version bump
if [ "$BREAKING" -gt 0 ]; then
  BUMP="major"
elif [ "$FEAT_COUNT" -gt 0 ]; then
  BUMP="minor"
else
  BUMP="patch"
fi
```

## Critical Files
- `Makefile` (lines 139-159): Current release targets
- `release.toml`: cargo-release configuration
- `cliff.toml`: git-cliff configuration
- `scripts/release.sh`: New wrapper script (to create)

## Validation
- [ ] Pre-flight checks catch dirty working directory
- [ ] Version inference correctly identifies bump type
- [ ] Changelog preview shows correct content
- [ ] Dry-run shows what will happen
- [ ] Actual release creates correct tag
- [ ] GitHub Release is created (or instructions shown)
- [ ] Rollback works if mid-release failure

## Rollback Plan
If release fails mid-way:
1. `git reset --hard HEAD~1` (remove version bump commit)
2. `git tag -d <tag>` (remove tag)
3. `git push origin :<tag>` (remove remote tag)
4. Inform user of manual steps needed
