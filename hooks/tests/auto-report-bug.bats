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

  # --- Mock `gh`: records every invocation; auth result from $GH_AUTH_STATUS;
  #     label list result from $GH_LABEL_LIST_OUTPUT ---
  cat > "$bindir/gh" <<'MOCK'
#!/usr/bin/env bash
printf '%s\n' "$*" >> "$GH_CALL_LOG"
if [ "$1" = "label" ] && [ "$2" = "list" ]; then
  printf '%s\n' "${GH_LABEL_LIST_OUTPUT:-auto-report}"
  exit 0
fi
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
  # Use $BATS_TEST_DIRNAME, not ${BASH_SOURCE[0]}: Bats preprocesses the
  # .bats file into a temp copy before executing it, so BASH_SOURCE[0]
  # resolves to that temp location, not this file's real directory.
  HOOK_SCRIPT="$(cd "$BATS_TEST_DIRNAME/.." && pwd)/auto-report-bug.sh"

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
  # Live auth check must have run exactly once, plus the label pre-check.
  [ "$(wc -l < "$GH_CALL_LOG")" -eq 2 ]
  grep -q "auth status" "$GH_CALL_LOG"
  grep -q "label list" "$GH_CALL_LOG"
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
  # The cached auth check must skip the live `gh auth status` call, but the
  # label pre-check still runs (it depends on auth succeeding, not on how).
  if grep -q "auth status" "$GH_CALL_LOG"; then
    echo "❌ auth status was unexpectedly called despite a valid auth cache" >&2
    return 1
  fi
  grep -q "label list" "$GH_CALL_LOG"
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

@test "warns and does not emit banner when auto-report label is missing" {
  mkdir -p "$(dirname "$PENDING_FILE")"
  cat > "$PENDING_FILE" <<'JSON'
{"id":"abc","command":"issue list","platform":"github","error_code":"500","error_message":"boom","timestamp":"2026-08-30T00:00:00Z"}
JSON
  export GH_AUTH_STATUS="ok"
  export GH_LABEL_LIST_OUTPUT=""

  run_hook

  [ "$status" -eq 0 ]
  [[ "$output" == *"auto-report"*"label"* ]] || [[ "$output" == *"标签"* ]]
  [[ "$output" != *"MUST load the gf-autoreport-bug skill"* ]]
  [ -f "$PENDING_FILE" ]
}

@test "emits banner when auto-report label exists" {
  mkdir -p "$(dirname "$PENDING_FILE")"
  cat > "$PENDING_FILE" <<'JSON'
{"id":"abc","command":"issue list","platform":"github","error_code":"500","error_message":"boom","timestamp":"2026-08-30T00:00:00Z"}
JSON
  export GH_AUTH_STATUS="ok"
  export GH_LABEL_LIST_OUTPUT="auto-report"

  run_hook

  [ "$status" -eq 0 ]
  [[ "$output" == *"MUST load the gf-autoreport-bug skill"* ]]
}

@test "auth success -> calls gh auth status" {
  write_pending
  GH_AUTH_STATUS="ok"
  run_hook

  [ "$status" -eq 0 ]
  # Live auth check must run exactly once, plus the label pre-check.
  [ "$(wc -l < "$GH_CALL_LOG")" -eq 2 ]
  grep -q "auth status" "$GH_CALL_LOG"
}
