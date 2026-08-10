#!/usr/bin/env bash
# validate-skill-commands.sh — Validate that gf CLI commands referenced in skill docs exist.
#
# Usage: ./scripts/validate-skill-commands.sh [--verbose]
# Exit codes: 0 = all references valid, 1 = mismatches found, 2 = internal error
#
# This script:
# 1. Extracts all valid gf subcommands by recursively scanning --help output
# 2. Scans skills/*/SKILL.md for `gf <command>` patterns
# 3. Reports references to non-existent commands
# 4. Excludes skill files with a [PLANNED] banner

set -euo pipefail

VERBOSE="${1:-}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SKILLS_DIR="$REPO_ROOT/skills"

# Determine the gf binary path
GF_BIN="${GF_BIN:-$REPO_ROOT/target/debug/gf}"
if [[ ! -x "$GF_BIN" ]]; then
    GF_BIN="$(command -v gf 2>/dev/null || true)"
fi
if [[ -z "${GF_BIN:-}" || ! -x "$GF_BIN" ]]; then
    echo "ERROR: gf binary not found. Set GF_BIN or build with 'cargo build'." >&2
    exit 2
fi

# Temp files
VALID_CMDS_FILE="$(mktemp)"
trap 'rm -f "$VALID_CMDS_FILE"' EXIT

# Known top-level gf commands (includes planned but not-yet-implemented ones)
TOP_LEVEL_COMMANDS="issue pr release review auth label milestone commit pipeline workflow skills update run repo"

# Words that indicate prose, not commands (filter these out)
PROSE_WORDS="is are was were be been being have has had do does did will would could should may might can shall must the a an this that these those not no all each every some any none more most less least very just only also still already even"

# --- Step 1: Extract all valid gf commands ---

extract_commands() {
    local cmd_prefix="$1"
    local help_output

    if [[ -z "$cmd_prefix" ]]; then
        help_output="$("$GF_BIN" --help 2>&1)" || true
    else
        help_output="$("$GF_BIN" $cmd_prefix --help 2>&1)" || true
    fi

    local in_commands=false
    while IFS= read -r line; do
        if [[ "$line" =~ ^Commands: ]]; then
            in_commands=true
            continue
        fi
        if $in_commands; then
            if [[ -z "$line" || ! "$line" =~ ^[[:space:]] ]]; then
                break
            fi
            local subcmd
            subcmd="$(echo "$line" | awk '{print $1}')"
            if [[ -n "$subcmd" && "$subcmd" != "help" ]]; then
                local full_cmd
                if [[ -z "$cmd_prefix" ]]; then
                    full_cmd="$subcmd"
                else
                    full_cmd="$cmd_prefix $subcmd"
                fi
                echo "$full_cmd" >> "$VALID_CMDS_FILE"
                extract_commands "$full_cmd"
            fi
        fi
    done <<< "$help_output"
}

extract_commands ""

cmd_count="$(wc -l < "$VALID_CMDS_FILE" | tr -d ' ')"
if [[ "$cmd_count" -eq 0 ]]; then
    echo "ERROR: No commands extracted from gf --help" >&2
    exit 2
fi

if [[ "$VERBOSE" == "--verbose" ]]; then
    echo "=== Valid gf commands ($cmd_count) ==="
    sort "$VALID_CMDS_FILE" | sed 's/^/  /'
    echo ""
fi

# Helper: check if a word is a prose word
is_prose_word() {
    local word="$1"
    for pw in $PROSE_WORDS; do
        if [[ "$word" == "$pw" ]]; then
            return 0
        fi
    done
    return 1
}

# Helper: check if a command path (or prefix) is valid
is_valid_command() {
    local cmd_path="$1"

    # Exact match
    if grep -qFx "$cmd_path" "$VALID_CMDS_FILE" 2>/dev/null; then
        return 0
    fi

    # Check if it's a valid prefix (command exists but has subcommands we didn't specify)
    # e.g., "gf issue" is valid even though "gf issue list" is the full command
    if grep -q "^${cmd_path} " "$VALID_CMDS_FILE" 2>/dev/null; then
        return 0
    fi

    return 1
}

# --- Step 2: Scan skill files for gf command references ---

mismatches=0
files_scanned=0
refs_checked=0

check_skill_file() {
    local skill_file="$1"

    # Skip files with [PLANNED] banner (check first 10 lines)
    if head -10 "$skill_file" | grep -q '\[PLANNED\]'; then
        if [[ "$VERBOSE" == "--verbose" ]]; then
            echo "SKIP: $skill_file (marked [PLANNED])"
        fi
        return
    fi

    files_scanned=$((files_scanned + 1))

    local line_num=0
    while IFS= read -r line; do
        line_num=$((line_num + 1))

        # Skip YAML frontmatter
        if [[ "$line" =~ ^--- ]]; then
            continue
        fi

        # Extract potential gf command references
        # Pattern: `gf` followed by 1-3 hyphenated-lowercase words
        local refs
        refs="$(echo "$line" | grep -oE 'gf[[:space:]]+[a-z][a-z-]*([[:space:]]+[a-z][a-z-]*){0,2}' 2>/dev/null || true)"

        if [[ -z "$refs" ]]; then
            continue
        fi

        while IFS= read -r ref; do
            [[ -z "$ref" ]] && continue
            # Strip leading "gf "
            local cmd_path="${ref#gf }"
            cmd_path="$(echo "$cmd_path" | sed 's/[[:space:]]*$//')"

            # Split into words
            local word1 word2 word3
            word1="$(echo "$cmd_path" | awk '{print $1}')"
            word2="$(echo "$cmd_path" | awk '{print $2}')"
            word3="$(echo "$cmd_path" | awk '{print $3}')"

            # Filter: first word must be a known top-level command
            local is_top_level=false
            for tc in $TOP_LEVEL_COMMANDS; do
                if [[ "$word1" == "$tc" ]]; then
                    is_top_level=true
                    break
                fi
            done
            if ! $is_top_level; then
                continue
            fi

            # Filter: subsequent words must NOT be prose words
            if [[ -n "$word2" ]] && is_prose_word "$word2"; then
                continue
            fi
            if [[ -n "$word3" ]] && is_prose_word "$word3"; then
                continue
            fi

            refs_checked=$((refs_checked + 1))

            # Validate: check if the command path exists
            if ! is_valid_command "$cmd_path"; then
                # Also check 2-word prefix (for cases like "gf issue label <n>" where we captured 3 words)
                if [[ -n "$word3" ]] && is_valid_command "$word1 $word2"; then
                    # The 2-word prefix is valid, so this is a valid command with extra args
                    continue
                fi
                echo "MISMATCH: $skill_file:$line_num — 'gf $cmd_path' does not exist"
                mismatches=$((mismatches + 1))
            fi
        done <<< "$refs"
    done < "$skill_file"
}

# Find all SKILL.md files
for sf in "$SKILLS_DIR"/*/SKILL.md; do
    [[ -f "$sf" ]] || continue
    check_skill_file "$sf"
done

# --- Step 3: Report ---

echo ""
echo "=== Validation Summary ==="
echo "Commands in CLI: $cmd_count"
echo "Files scanned:   $files_scanned"
echo "Refs checked:    $refs_checked"
echo "Mismatches:      $mismatches"

if [[ $mismatches -gt 0 ]]; then
    echo ""
    echo "FAILED: $mismatches command reference(s) do not exist in the CLI."
    exit 1
else
    echo ""
    echo "PASSED: All skill command references are valid."
    exit 0
fi
