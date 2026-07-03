#!/usr/bin/env bash
# sync-readme-check.sh — Stop Hook: check if README.md's directory
# structure section matches the actual repo structure.
# Outputs a reminder when they diverge.

set -euo pipefail

REPO_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || exit 0)
[ -z "$REPO_ROOT" ] && exit 0
cd "$REPO_ROOT" || exit 0

# Get actual top-level directories (excluding hidden, .git, target)
get_actual_dirs() {
  find . -maxdepth 1 -type d \
    ! -name '.' \
    ! -name '.*' \
    ! -name 'target' \
    | sed 's|^\./||' \
    | sort
}

# Extract directory names from README's Structure section
get_readme_dirs() {
  local file="$1"
  awk '/^## Structure/,/^## [^S]/' "$file" 2>/dev/null \
    | grep -E '^\||├── |└── ' \
    | sed 's/|//g' \
    | grep -oE '[a-zA-Z0-9_-]+/' \
    | sed 's|/||' \
    | sort -u || true
}

# Get directories from skills/ subdirectory
get_skill_names() {
  find skills -maxdepth 1 -type d \
    ! -name 'skills' \
    | sed 's|skills/||' \
    | sort
}

# Compare top-level structure
actual_dirs=$(get_actual_dirs)
readme_dirs=$(get_readme_dirs "README.md")

missing=$(comm -23 <(echo "$actual_dirs") <(echo "$readme_dirs") 2>/dev/null || true)
extra=$(comm -13 <(echo "$actual_dirs") <(echo "$readme_dirs") 2>/dev/null || true)

if [ -n "$missing" ] || [ -n "$extra" ]; then
  echo ""
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo "  📝 README 目录结构可能需要更新"
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  if [ -n "$missing" ]; then
    echo "  缺少的目录:"
    while IFS= read -r dir; do
      [ -n "$dir" ] && echo "    - $dir"
    done <<< "$missing"
  fi
  if [ -n "$extra" ]; then
    echo "  多余的目录:"
    while IFS= read -r dir; do
      [ -n "$dir" ] && echo "    - $dir"
    done <<< "$extra"
  fi
  echo ""
  echo "  手动检查 README.md 的 Structure 章节是否需要更新"
  echo ""
fi

exit 0
