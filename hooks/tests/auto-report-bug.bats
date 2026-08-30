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
  #     label list result from $GH_LABEL_LIST_OUTPUT (unset -> default
  #     "auto-report"; explicitly set to "" -> empty, simulating a missing
  #     label — note this must use `${VAR-default}`, NOT `${VAR:-default}`,
  #     since the colon form also substitutes on an explicitly empty value);
  #     $GH_LABEL_LIST_FAIL=true simulates the `gh label list` command itself
  #     failing (network/auth/API error), as opposed to it succeeding with a
  #     non-matching list ---
  cat > "$bindir/gh" <<'MOCK'
#!/usr/bin/env bash
printf '%s\n' "$*" >> "$GH_CALL_LOG"
if [ "$1" = "label" ] && [ "$2" = "list" ]; then
  if [ "${GH_LABEL_LIST_FAIL:-false}" = "true" ]; then
    exit 1
  fi
  printf '%s\n' "${GH_LABEL_LIST_OUTPUT-auto-report}"
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
  echo $(( $(date +%s) - 60 )) > "$AUTH_CACHE_DIR/label-auto-report.ttl"

  # If the hook wrongly called gh, auth would fail and this test would catch it.
  export GH_AUTH_STATUS="fail"

  run_hook

  [ "$status" -eq 0 ]
  [[ "$output" == *"cache 命中"* ]]
  [[ "$output" == *"检测到 gf CLI 错误报告"* ]]
  # Both the auth cache and the label cache are warm, so no live `gh` call
  # of any kind (neither `auth status` nor `label list`) should happen.
  if [ -s "$GH_CALL_LOG" ]; then
    echo "❌ gh was unexpectedly called despite valid auth + label caches:" >&2
    cat "$GH_CALL_LOG" >&2
    return 1
  fi
}

@test "label check is cached -> second run within TTL skips live gh label list" {
  write_pending

  # First run: no label cache yet, so `gh label list` must be called once
  # and the result cached.
  run_hook
  [ "$status" -eq 0 ]
  [[ "$output" == *"检测到 gf CLI 错误报告"* ]]
  grep -q "label list" "$GH_CALL_LOG"
  [ -f "$AUTH_CACHE_DIR/label-auto-report.ttl" ]

  # Second run within the label-cache TTL must not call `gh label list`
  # again, even though the auth cache from run 1 is also still warm.
  : > "$GH_CALL_LOG"
  run_hook
  [ "$status" -eq 0 ]
  [[ "$output" == *"检测到 gf CLI 错误报告"* ]]
  if grep -q "label list" "$GH_CALL_LOG"; then
    echo "❌ label list was called again despite a valid label cache" >&2
    return 1
  fi
}

@test "auth success -> banner instruction uses gf-autoreport-bug (no stale gitflow-) and is honest about unattended skip" {
  write_pending

  run_hook

  [ "$status" -eq 0 ]
  # The load instruction must reference the current skill name and be directive
  # about what a human needs to do — but it must NOT claim the skill will
  # itself file the Issue unattended (it defaults to skip when non-interactive).
  [[ "$output" == *"gf-autoreport-bug"* ]]
  [[ "$output" == *"交互式重新触发 gf-autoreport-bug skill"* ]]
  if [[ "$output" == *"MUST load the gf-autoreport-bug skill"* ]]; then
    echo "❌ banner still uses the misleading 'MUST load' directive (it implies unattended auto-filing)" >&2
    return 1
  fi
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
  if ! { [[ "$output" == *"auto-report"*"label"* ]] || [[ "$output" == *"标签"* ]]; }; then
    echo "❌ missing-label warning did not mention 'auto-report'+'label' or '标签'" >&2
    return 1
  fi
  if [[ "$output" == *"检测到 gf CLI 错误报告"* ]]; then
    echo "❌ banner was emitted despite the auto-report label being missing" >&2
    return 1
  fi
  [ -f "$PENDING_FILE" ]
}

@test "gh label list command failure -> warns it is unavailable, not that the label is missing" {
  mkdir -p "$(dirname "$PENDING_FILE")"
  cat > "$PENDING_FILE" <<'JSON'
{"id":"abc","command":"issue list","platform":"github","error_code":"500","error_message":"boom","timestamp":"2026-08-30T00:00:00Z"}
JSON
  export GH_AUTH_STATUS="ok"
  export GH_LABEL_LIST_FAIL="true"

  run_hook

  [ "$status" -eq 0 ]
  # A transient `gh label list` failure (network/auth/API) must not be
  # presented as a confident "the label is missing, here's how to create
  # it" — that message is reserved for an exact-match failure on a
  # successful `gh` call.
  [[ "$output" == *"无法确认"* ]]
  if [[ "$output" == *"仓库缺少 auto-report 标签"* ]]; then
    echo "❌ transient gh failure was misreported as a confirmed missing label" >&2
    return 1
  fi
  if [[ "$output" == *"gh label create"* ]]; then
    echo "❌ transient gh failure suggested 'gh label create', which would itself fail" >&2
    return 1
  fi
  if [[ "$output" == *"检测到 gf CLI 错误报告"* ]]; then
    echo "❌ banner was emitted despite the label check being unavailable" >&2
    return 1
  fi
  [ -f "$PENDING_FILE" ]
}

@test "exact label match required -> fuzzy substring match alone does not pass the check" {
  mkdir -p "$(dirname "$PENDING_FILE")"
  cat > "$PENDING_FILE" <<'JSON'
{"id":"abc","command":"issue list","platform":"github","error_code":"500","error_message":"boom","timestamp":"2026-08-30T00:00:00Z"}
JSON
  export GH_AUTH_STATUS="ok"
  # Only a fuzzy-matching label exists, not the exact "auto-report" label.
  export GH_LABEL_LIST_OUTPUT="auto-report-triage"

  run_hook

  [ "$status" -eq 0 ]
  [[ "$output" == *"仓库缺少 auto-report 标签"* ]]
  if [[ "$output" == *"检测到 gf CLI 错误报告"* ]]; then
    echo "❌ banner was emitted despite only a fuzzy-matching label existing" >&2
    return 1
  fi
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
  [[ "$output" == *"检测到 gf CLI 错误报告"* ]]
  [[ "$output" == *"交互式重新触发 gf-autoreport-bug skill"* ]]
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
