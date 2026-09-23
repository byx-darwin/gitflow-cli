#!/usr/bin/env bash
# validate-skill-links.sh — Validate that relative references in skill docs resolve.
#
# Usage: ./scripts/validate-skill-links.sh [--root <dir>] [--verbose]
# Exit codes: 0 = all references resolve, 1 = broken references found, 2 = usage/env error
#
# Scope — only navigational references the skill runtime actually follows:
#   1. Markdown link syntax `[text](path)` with a relative target
#   2. Backticked `references/*.md` progressive-disclosure load targets
#   3. Backticked cross-skill `gf-quality/references/*.md` load targets
# Paths merely *mentioned* (a skill's own output path, a conditional
# prerequisite) are not references and are deliberately out of scope.
#
# Resolution roots, first hit wins:
#   1. the directory containing the referring file
#   2. the skill root (`<root>/<skill-name>/`)
#   3. the shared skills root (`<root>/`) for cross-skill references
# Root 2 is required because `references/detector.md` refers to sibling
# references as `references/<lang>.md`, i.e. relative to the skill root.

set -euo pipefail

ROOT=""
VERBOSE=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --root)
            [[ $# -ge 2 ]] || { echo "ERROR: --root needs a value" >&2; exit 2; }
            ROOT="$2"; shift 2 ;;
        --verbose) VERBOSE=1; shift ;;
        *) echo "ERROR: unknown option: $1" >&2; exit 2 ;;
    esac
done

if [[ -z "$ROOT" ]]; then
    ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/skills"
fi
ROOT="${ROOT%/}"

if [[ ! -d "$ROOT" ]]; then
    echo "ERROR: skills root not found: $ROOT" >&2
    exit 2
fi

BROKEN=()
CHECKED=0

is_skippable() {
    case "$1" in
        ''|http://*|https://*|mailto:*|\#*|/*) return 0 ;;
        *'<'*|*'*'*|*' '*) return 0 ;;
    esac
    return 1
}

check_ref() {
    local file="$1" ref="$2" kind="$3" dir rel skill_root
    dir="$(dirname "$file")"
    rel="${file#"$ROOT"/}"
    skill_root="$ROOT/${rel%%/*}"

    CHECKED=$((CHECKED + 1))
    if [[ -f "$dir/$ref" || -f "$skill_root/$ref" || -f "$ROOT/$ref" ]]; then
        [[ $VERBOSE -eq 1 ]] && echo "  ok   ($kind) $rel -> $ref"
        return 0
    fi
    BROKEN+=("$rel -> $ref  [$kind]")
    return 0
}

while IFS= read -r file; do
    while IFS= read -r ref; do
        is_skippable "$ref" && continue
        check_ref "$file" "$ref" link
    done < <(grep -oE '\]\([^)]+\)' "$file" 2>/dev/null | sed -E 's/^\]\(//; s/\)$//' || true)

    while IFS= read -r ref; do
        is_skippable "$ref" && continue
        check_ref "$file" "$ref" load
    done < <(grep -oE '`references/[A-Za-z0-9_.-]+\.md`' "$file" 2>/dev/null | tr -d '`' || true)

    while IFS= read -r ref; do
        is_skippable "$ref" && continue
        check_ref "$file" "$ref" shared-load
    done < <(grep -oE '`gf-quality/references/[A-Za-z0-9_./-]+\.md`' "$file" 2>/dev/null | tr -d '`' || true)
done < <(find "$ROOT" -name '*.md' -type f | sort)

echo ""
echo "=== Skill Doc Link Summary ==="
if [[ ${#BROKEN[@]} -eq 0 ]]; then
    echo "✅ All $CHECKED skill doc reference(s) resolve"
    exit 0
fi

echo "❌ ${#BROKEN[@]} broken reference(s) out of $CHECKED checked:"
for entry in "${BROKEN[@]}"; do
    echo "   - $entry"
done
exit 1
