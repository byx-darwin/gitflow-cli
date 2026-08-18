#!/usr/bin/env bats

# Bats test suite for the Claude Code Stop Hook: auto-report-bug.sh
#
# The hook reads `.cache/bug-reports/pending.json` in the git repo root,
# shallow-validates it, checks GitHub auth (with a 24h auth cache), and then:
#   - exits silently when there is nothing to report,
#   - renames malformed reports to `<report>.invalid`,
#   - prints a login guide when auth fails,
#   - prints a bug-report banner when auth succeeds,
#   - skips the live `gh` call when the auth cache is still valid.
#
# These tests mock `git` (repo-root discovery) and `gh` (live auth check) so the
# hook runs in a hermetic temp sandbox without a real repo or network access.

setup() {
  # --- Hermetic sandbox root (what `git rev-parse --show-toplevel` returns) ---
  SANDBOX="$BATS_TEST_TMPDIR/sandbox"
  mkdir -p "$SANDBOX"

  # --- Mock `git`: only `rev-parse --show-toplevel` is used by the hook ---
  local bindir="$BATS_TEST_TMPDIR/bin"
  mkdir -p "$bindir"
  cat > "$bindir/git" <<'MOCK'
#!/usr/bin/env bash
if [ "$1" = "rev-parse" ]; then
  printf '%s\n' "$GIT_TOPLEVEL"
fi
MOCK
  chmod +x "$bindir/git"

  # --- Mock `gh`: records every invocation; auth result from $GH_AUTH_STATUS ---
  cat > "$bindir/gh" <<'MOCK'
#!/usr/bin/env bash
printf '%s\n' "$*" >> "$GH_CALL_LOG"
if [ "${GH_AUTH_STATUS:-ok}" = "fail" ]; then
  exit 1
fi
exit 0
MOCK
  chmod +x "$bindir/gh"

  export PATH="$bindir:$PATH"
  export GIT_TOPLEVEL="$SANDBOX"
  export GH_AUTH_STATUS="ok"
  export GH_CALL_LOG="$BATS_TEST_TMPDIR/gh-calls.log"
  : > "$GH_CALL_LOG"

  # --- The hook under test (sibling of this tests/ directory) ---
  HOOK_SCRIPT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/auto-report-bug.sh"

  # --- Locations the hook reads and writes inside the sandbox ---
  PENDING_FILE="$SANDBOX/.cache/bug-reports/pending.json"
  AUTH_CACHE_DIR="$SANDBOX/.cache/auth-cache"
}

# Run the hook with stdin closed to a non-TTY and stdout+stderr captured
# together so assertions are portable across Bats versions.
run_hook() {
  run bash -c '"$1" 2>&1 < /dev/null' hook_script "$HOOK_SCRIPT"
}

# Write a valid pending error report (all fields the hook extracts).
write_pending() {
  mkdir -p "$(dirname "$PENDING_FILE")"
  cat > "$PENDING_FILE" <<'JSON'
{
  "command": "gf issue create",
  "error_code": "E_API",
  "error_message": "rate limit exceeded",
  "platform": "github",
  "timestamp": "2026-08-06T10:00:00Z"
}
JSON
}

@test "no pending.json -> silent exit" {
  run_hook

  [ "$status" -eq 0 ]
  [ -z "$output" ]
  [ ! -e "${PENDING_FILE}.invalid" ]
}

@test "invalid JSON -> renamed to .invalid" {
  mkdir -p "$(dirname "$PENDING_FILE")"
  printf '%s\n' '{"not_an_error_report": true}' > "$PENDING_FILE"

  run_hook

  [ "$status" -eq 0 ]
  [ ! -f "$PENDING_FILE" ]
  [ -f "${PENDING_FILE}.invalid" ]
  [[ "$output" == *"格式异常"* ]]
  [[ "$output" == *"pending.json.invalid"* ]]
}

@test "auth failure -> outputs login guide" {
  write_pending
  export GH_AUTH_STATUS="fail"

  run_hook

  [ "$status" -eq 0 ]
  [[ "$output" == *"GitHub 未登录"* ]]
  [[ "$output" == *"gh auth login"* ]]
  [[ "$output" == *"报告内容"* ]]
}

@test "auth success -> outputs banner and seeds auth cache" {
  write_pending

  run_hook

  [ "$status" -eq 0 ]
  [[ "$output" == *"检测到 gf CLI 错误报告"* ]]
  [[ "$output" == *"gf-autoreport-bug"* ]]
  [ -f "$AUTH_CACHE_DIR/github.ttl" ]
  # Live auth check must have run exactly once.
  [ "$(wc -l < "$GH_CALL_LOG")" -eq 1 ]
}

@test "auth cache valid -> skips gf CLI call" {
  write_pending
  mkdir -p "$AUTH_CACHE_DIR"
  echo $(( $(date +%s) - 60 )) > "$AUTH_CACHE_DIR/github.ttl"

  # If the hook wrongly called gf, auth would fail and this test would catch it.
  export GH_AUTH_STATUS="fail"

  run_hook

  [ "$status" -eq 0 ]
  [[ "$output" == *"cache 命中"* ]]
  [[ "$output" == *"检测到 gf CLI 错误报告"* ]]
  [ ! -s "$GH_CALL_LOG" ]
}

@test "auth success -> banner instruction uses gf-autoreport-bug (no stale gitflow-) and MUST directive" {
  write_pending

  run_hook

  [ "$status" -eq 0 ]
  # The load instruction must reference the current skill name and be directive.
  [[ "$output" == *"gf-autoreport-bug"* ]]
  [[ "$output" == *"MUST load the gf-autoreport-bug skill"* ]]
  # The stale skill name must not appear anywhere in the banner.
  if echo "$output" | grep -q "gitflow-autoreport-bug"; then
    echo "❌ banner still references stale gitflow-autoreport-bug" >&2
    return 1
  fi
}

@test "auth success -> calls gh auth status" {
  write_pending
  GH_AUTH_STATUS="ok"
  run_hook

  [ "$status" -eq 0 ]
  # Live auth check must run exactly once.
  [ "$(wc -l < "$GH_CALL_LOG")" -eq 1 ]
  grep -q "auth status" "$GH_CALL_LOG"
}
