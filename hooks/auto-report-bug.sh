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

# Skip if running in an interactive terminal — the CLI's error_reporter
# already suppresses pending.json in interactive mode, but guard against
# edge cases where the file was left behind.
if [ -t 1 ] || [ -t 0 ]; then
  # stdout or stdin is a TTY → interactive session, skip
  exit 0
fi

# Read and validate the pending report.
PENDING_CONTENT=$(cat "$PENDING_FILE")

# Quick JSON sanity check — require at least "error_code" field.
# NOTE: This is a shallow validation using grep; `jq` would be preferred
# for robust JSON parsing but is not guaranteed to be available in all
# environments. The error_reporter writes controlled-format JSON so this
# heuristic is sufficient.
if ! echo "$PENDING_CONTENT" | grep -q '"error_code"'; then
  # Invalid format — rename to .invalid and exit.
  mv "$PENDING_FILE" "${PENDING_FILE}.invalid"
  exit 0
fi

# Extract key fields for the prompt banner.
COMMAND=$(echo "$PENDING_CONTENT" | grep -o '"command"[[:space:]]*:[[:space:]]*"[^"]*"' | head -1 | sed 's/.*: *"//;s/"$//')
ERROR_CODE=$(echo "$PENDING_CONTENT" | grep -o '"error_code"[[:space:]]*:[[:space:]]*"[^"]*"' | head -1 | sed 's/.*: *"//;s/"$//')
PLATFORM=$(echo "$PENDING_CONTENT" | grep -o '"platform"[[:space:]]*:[[:space:]]*"[^"]*"' | head -1 | sed 's/.*: *"//;s/"$//')

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  🐛 检测到 gitflow CLI 错误报告"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "  命令:   ${COMMAND:-unknown}"
echo "  平台:   ${PLATFORM:-unknown}"
echo "  错误码: ${ERROR_CODE:-unknown}"
echo ""
echo "  原始报告:"
echo "$PENDING_CONTENT"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  请加载 gitflow-autoreport-bug Skill 执行自动 Bug 报告流程。"
echo "  Skill 路径: skills/gitflow-autoreport-bug/SKILL.md"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
