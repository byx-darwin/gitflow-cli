#!/usr/bin/env bash
# 用 asciinema + svg-term 将真实会话渲染为 docs/assets/demo.svg。
# 依赖：asciinema、svg-term-cli（npm i -g svg-term-cli）。
set -euo pipefail
ROOT="$(git rev-parse --show-toplevel)"
OUT="${ROOT}/docs/assets/demo.svg"
CAST="$(mktemp /tmp/gf-demo.XXXXXX.cast)"
mkdir -p "${ROOT}/docs/assets"
cargo build -p gf
GF_BIN="${ROOT}/target/debug/gf" \
  asciinema rec "$CAST" --command "bash ${ROOT}/scripts/demo-session.sh" --overwrite
svg-term "$CAST" --out "$OUT" --window --width 90 --height 20
rm -f "$CAST"
echo "Wrote ${OUT}"
