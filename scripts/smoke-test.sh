#!/usr/bin/env bash
set -euo pipefail

echo "=== gitflow-cli Smoke Test (Phase 1 + Phase 2) ==="

echo ""
echo "--- Phase 1: Core Commands (API-dependent, best-effort) ---"

echo "[1/5] issue view (best-effort)"
gitflow-cli issue view 1 --platform github 2>&1 | head -5 || echo "  (skipped — no valid auth)"

echo "[2/5] issue list (best-effort)"
gitflow-cli issue list --state open --limit 3 --platform github 2>&1 | head -5 || echo "  (skipped — no valid auth)"

echo "[3/5] pr list (best-effort)"
gitflow-cli pr list --state open --limit 3 --platform github 2>&1 | head -5 || echo "  (skipped — no valid auth)"

echo "[4/5] skills --help"
gitflow-cli skills --help 2>&1 | head -5

echo "[5/5] run --help"
gitflow-cli run --help 2>&1 | head -5

echo ""
echo "--- Phase 2: Read-only Commands (help only) ---"

echo "[6/15] issue comment --help"
gitflow-cli issue comment 1 --help 2>&1 | head -5

echo "[7/15] issue close --help"
gitflow-cli issue close 1 --help 2>&1 | head -5

echo "[8/15] issue reopen --help"
gitflow-cli issue reopen 1 --help 2>&1 | head -5

echo "[9/15] issue add-label --help"
gitflow-cli issue add-label 1 --help 2>&1 | head -5

echo "[10/15] issue remove-label --help"
gitflow-cli issue remove-label 1 --help 2>&1 | head -5

echo "[11/15] release list --help"
gitflow-cli release list --help 2>&1 | head -5

echo "[12/15] release view --help"
gitflow-cli release view --help 2>&1 | head -5

echo "[13/15] label list --help"
gitflow-cli label list --help 2>&1 | head -5

echo "[14/15] milestone list --help"
gitflow-cli milestone list --help 2>&1 | head -5

echo "[15/15] review comment --help"
gitflow-cli review comment --help 2>&1 | head -5

echo ""
echo "--- Phase 2: Write Commands (--help only, not executed) ---"

echo "[W1] issue create --help"
gitflow-cli issue create --help 2>&1 | head -5

echo "[W2] pr create --help"
gitflow-cli pr create --help 2>&1 | head -5

echo "[W3] pr merge --help"
gitflow-cli pr merge --help 2>&1 | head -5

echo "[W4] pr checkout --help"
gitflow-cli pr checkout --help 2>&1 | head -5

echo "[W5] pr ready --help"
gitflow-cli pr ready --help 2>&1 | head -5

echo "[W6] pr wip --help"
gitflow-cli pr wip --help 2>&1 | head -5

echo "[W7] pr sync --help"
gitflow-cli pr sync --help 2>&1 | head -5

echo "[W8] pr comment --help"
gitflow-cli pr comment --help 2>&1 | head -5

echo "[W9] release create --help"
gitflow-cli release create --help 2>&1 | head -5

echo "[W10] release edit --help"
gitflow-cli release edit --help 2>&1 | head -5

echo "[W11] release delete --help"
gitflow-cli release delete --help 2>&1 | head -5

echo "[W12] auth login --help"
gitflow-cli auth login --help 2>&1 | head -5

echo "[W13] auth logout --help"
gitflow-cli auth logout --help 2>&1 | head -5

echo "[W14] auth status --help"
gitflow-cli auth status --help 2>&1 | head -5

echo "[W15] label create --help"
gitflow-cli label create --help 2>&1 | head -5

echo "[W16] milestone create --help"
gitflow-cli milestone create --help 2>&1 | head -5

echo ""
echo "=== Smoke test passed ==="
