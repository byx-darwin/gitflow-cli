#!/usr/bin/env bash
# scripts/_common.sh — thin wrapper that sources the canonical skills/_common.sh
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../skills/_common.sh"
