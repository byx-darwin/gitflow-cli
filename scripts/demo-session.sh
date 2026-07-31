#!/usr/bin/env bash
# 被 asciinema 录制的只读演示会话。需先 `cargo build`。
set -euo pipefail
BIN="${GITFLOW_BIN:-./target/debug/gitflow-cli}"
run() { printf '\n\033[32m$ %s\033[0m\n' "$*"; "$@" || true; sleep 1; }
run "$BIN" --version
run "$BIN" skills list
run "$BIN" issue list --output toon
