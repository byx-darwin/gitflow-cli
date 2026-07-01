#!/usr/bin/env bash
set -euo pipefail

echo "=== gitflow-cli Smoke Test (Phase 1 - GitHub) ==="

echo "[1/3] issue view"
gitflow-cli issue view 1 --platform github 2>&1 | head -5

echo "[2/3] issue list"
gitflow-cli issue list --state open --limit 3 --platform github 2>&1 | head -5

echo "[3/3] pr list"
gitflow-cli pr list --state open --limit 3 --platform github 2>&1 | head -5

echo "=== Smoke test passed ==="
