#!/usr/bin/env bash
# Stop Hook: detect CLI errors written by gitflow-cli's error_reporter
# and surface them to Claude for automated bug report creation.
#
# Triggered by the Claude Code Stop Hook configured in .claude/settings.json.
# The Rust CLI writes error reports to .cache/bug-reports/pending.json
# whenever it fails in non-interactive mode (CI / subprocess). This
# script checks for that file and, if present, prints a banner so that
# Claude picks up the content and delegates to the gitflow-autoreport-bug
# skill.
#
# Exit codes:
#   0  —  no pending report (nothing to do) OR report printed successfully
#   0  —  not inside a git repository (silent no-op)

set -euo pipefail

REPO_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || exit 0)

# Not inside a git repository — nothing to do.
if [ -z "${REPO_ROOT}" ]; then
  exit 0
fi

PENDING_FILE="$REPO_ROOT/.cache/bug-reports/pending.json"

# No pending error report — silent exit.
if [ ! -f "$PENDING_FILE" ]; then
  exit 0
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  🐛 检测到 gitflow CLI 错误，正在生成 Bug 报告..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
cat "$PENDING_FILE"
