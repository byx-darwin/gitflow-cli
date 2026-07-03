#!/usr/bin/env bash
# Stop Hook: detect CLI errors and surface them for automated reporting.
#
# Triggered by the Claude Code Stop Hook configured in .claude/settings.json.
# The Rust CLI writes error reports to .cache/bug-reports/pending.json
# whenever it fails in non-interactive mode (CI / subprocess). This script:
#   1. Checks for pending.json
#   2. Shallow-validates JSON
#   3. Uses auth cache (24h TTL) to avoid redundant auth checks
#   4. Outputs a banner that triggers the gitflow-autoreport-bug skill
#
# Exit codes: 0 always (silent no-op when nothing to do)

set -euo pipefail

REPO_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || exit 0)
[ -z "$REPO_ROOT" ] && exit 0

PENDING_FILE="$REPO_ROOT/.cache/bug-reports/pending.json"

# No pending error report — silent exit.
if [ ! -f "$PENDING_FILE" ]; then
  exit 0
fi

# Interactive terminal guard — skip if in TTY.
if [ -t 1 ] || [ -t 0 ]; then
  exit 0
fi

# Read pending report content.
PENDING_CONTENT=$(cat "$PENDING_FILE")

# Shallow JSON validation — require at least "error_code" field.
if ! echo "$PENDING_CONTENT" | grep -q '"error_code"'; then
  mv "$PENDING_FILE" "${PENDING_FILE}.invalid"
  echo "⚠️  pending.json 格式异常，已重命名为 pending.json.invalid" >&2
  exit 0
fi

# Extract key fields for the prompt banner.
COMMAND=$(echo "$PENDING_CONTENT" | grep -o '"command"[[:space:]]*:[[:space:]]*"[^"]*"' | head -1 | sed 's/.*: *"//;s/"$//')
ERROR_CODE=$(echo "$PENDING_CONTENT" | grep -o '"error_code"[[:space:]]*:[[:space:]]*"[^"]*"' | head -1 | sed 's/.*: *"//;s/"$//')
PLATFORM=$(echo "$PENDING_CONTENT" | grep -o '"platform"[[:space:]]*:[[:space:]]*"[^"]*"' | head -1 | sed 's/.*: *"//;s/"$//')
TIMESTAMP=$(echo "$PENDING_CONTENT" | grep -o '"timestamp"[[:space:]]*:[[:space:]]*"[^"]*"' | head -1 | sed 's/.*: *"//;s/"$//')

# Auth cache check (24h TTL).
CACHE_FILE="$REPO_ROOT/.cache/auth-cache/${PLATFORM}.ttl"
AUTH_CACHE_TTL=86400
AUTH_STATUS="未知"

if [ -f "$CACHE_FILE" ]; then
  CACHED_TIME=$(cat "$CACHE_FILE")
  NOW=$(date +%s 2>/dev/null || python3 -c "import time; print(int(time.time()))")
  AGE=$(( NOW - CACHED_TIME ))
  if [ "$AGE" -lt "$AUTH_CACHE_TTL" ]; then
    AUTH_STATUS="✅ cache 命中（age: ${AGE}s）"
  fi
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  🐛 检测到 gitflow CLI 错误报告"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "  命令:   ${COMMAND:-unknown}"
echo "  平台:   ${PLATFORM:-unknown}"
echo "  错误码: ${ERROR_CODE:-unknown}"
echo "  时间:   ${TIMESTAMP:-unknown}"
echo "  认证:   ${AUTH_STATUS}"
echo ""
echo "  原始报告:"
echo "$PENDING_CONTENT"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "  请加载 gitflow-autoreport-bug Skill 执行自动 Bug 报告流程。"
echo "  Skill 路径: skills/gitflow-autoreport-bug/SKILL.md"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

exit 0
