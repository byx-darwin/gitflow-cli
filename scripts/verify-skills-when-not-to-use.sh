#!/usr/bin/env bash
# Verify all skills have "When NOT to Use" section
# Exit code 0 = all skills have the section, 1 = missing

set -euo pipefail

SKILLS_DIR="skills"
MISSING=()

for skill_dir in "$SKILLS_DIR"/gf-*/; do
    skill_name=$(basename "$skill_dir")
    skill_file="$skill_dir/SKILL.md"

    if [ ! -f "$skill_file" ]; then
        echo "⚠️  $skill_name: SKILL.md not found"
        continue
    fi

    # Check for "When NOT to Use" section (case-insensitive)
    if grep -qi "when not to use" "$skill_file"; then
        echo "✅ $skill_name: has 'When NOT to Use' section"
    else
        echo "❌ $skill_name: MISSING 'When NOT to Use' section"
        MISSING+=("$skill_name")
    fi
done

echo ""
echo "=== Summary ==="
if [ ${#MISSING[@]} -eq 0 ]; then
    echo "✅ All skills have 'When NOT to Use' section"
    exit 0
else
    echo "❌ ${#MISSING[@]} skill(s) missing 'When NOT to Use':"
    for skill in "${MISSING[@]}"; do
        echo "   - $skill"
    done
    exit 1
fi
