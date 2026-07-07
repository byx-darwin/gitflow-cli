#!/bin/bash
# Skills Refactor - Quick Resume Script
# Run this in a new Claude session to resume work

cd /Users/byx/Documents/workspace/github.com/byx-darwin/gitflow-cli

echo '=== Step 1: 验证状态 ==='
git checkout refactor/skills-superpowers
git log --oneline | grep -c 'refactor.*skill'
echo 'refactor commits on branch'

echo ''
echo '=== Step 2: 检查哪些 skills 还有 trigger format ==='
MISSING=''
for skill in auth autoreport-bug commit issue-create issue-review issue-triage issue label-milestone label-stats pipeline-analyzer pr-apply-feedback pr-create pr-inline-review pr-review pr precommit quality regression release-helper release repo-onboarding repo review security-check weekly-report workflow; do
  if ! grep -q 'Use when' "skills/gitflow-/SKILL.md" 2>/dev/null; then
    MISSING="$MISSING $skill"
  fi
done
if [ -z "$MISSING" ]; then
  echo 'All 26 skills have trigger format'
else
  echo "MISSING:$MISSING"
fi

echo ''
echo '=== Step 3: 查看进度 ==='
cat .superpowers/sdd/progress-refactor.md 2>/dev/null || echo 'No progress file'

echo ''
echo '=== Step 4: 获取 resume token ==='
echo 'Token available in session history'
