#!/usr/bin/env bash
# generate-completions.sh — 为 bash/zsh/fish 预生成 shell 补全脚本
#
# 功能：
#   生成 gitflow-cli 三种 shell 的 tab 补全文件，输出到 `completions/`
#   目录，供打包分发或手工 source 使用。
#
# 二进制解析优先级：
#   1. `target/release/gitflow-cli`   (release 构建产物)
#   2. `target/debug/gitflow-cli`     (debug 构建产物)
#   3. `gitflow-cli`                  (PATH 中可执行的安装版本)
#
# 用法：
#   ./scripts/generate-completions.sh
#   make completions
#
# 依赖：bash 5+, gitflow-cli 二进制（任意来源）

set -euo pipefail

# ---------------------------------------------------------------------------
# 常量
# ---------------------------------------------------------------------------

readonly SCRIPT_NAME="generate-completions.sh"
readonly BINARY_NAME="gitflow-cli"
readonly OUTPUT_DIR="completions"

# ---------------------------------------------------------------------------
# 工具函数
# ---------------------------------------------------------------------------

info() { printf '\033[1;34m[i]\033[0m %s\n' "$*"; }
success() { printf '\033[1;32m✔\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m⚠\033[0m %s\n' "$*" >&2; }
die() { printf '\033[1;31m✖\033[0m %s\n' "$*" >&2; exit 1; }

# ---------------------------------------------------------------------------
# 解析项目根目录（脚本所在的上一级目录）
# ---------------------------------------------------------------------------

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

# ---------------------------------------------------------------------------
# 定位可执行文件
# ---------------------------------------------------------------------------

resolve_binary() {
    if [[ -x "$PROJECT_ROOT/target/release/$BINARY_NAME" ]]; then
        echo "$PROJECT_ROOT/target/release/$BINARY_NAME"
    elif [[ -x "$PROJECT_ROOT/target/debug/$BINARY_NAME" ]]; then
        echo "$PROJECT_ROOT/target/debug/$BINARY_NAME"
    elif command -v "$BINARY_NAME" >/dev/null 2>&1; then
        command -v "$BINARY_NAME"
    else
        die "未找到 $BINARY_NAME 二进制。请先运行 \`cargo build\` 或安装到 PATH。"
    fi
}

# ---------------------------------------------------------------------------
# 生成单个 shell 的补全文件
# ---------------------------------------------------------------------------

generate_for_shell() {
    local shell_name="$1"
    local filename="$2"
    local binary="$3"
    local output_path="$OUTPUT_DIR/$filename"

    if "$binary" completions "$shell_name" >"$output_path"; then
        success "已生成 $shell_name 补全 → $output_path"
    else
        die "生成 $shell_name 补全失败"
    fi
}

# ---------------------------------------------------------------------------
# 主流程
# ---------------------------------------------------------------------------

main() {
    info "开始为 $BINARY_NAME 生成 shell 补全脚本..."

    local binary
    binary="$(resolve_binary)"
    info "使用二进制：$binary"

    mkdir -p "$OUTPUT_DIR"

    generate_for_shell "bash" "gitflow-cli.bash" "$binary"
    generate_for_shell "zsh"  "_gitflow-cli"     "$binary"
    generate_for_shell "fish" "gitflow-cli.fish" "$binary"

    echo ""
    success "所有 shell 补全已生成到 ./$OUTPUT_DIR/"
    echo ""
    echo "使用方式："
    echo "  bash: source $OUTPUT_DIR/gitflow-cli.bash"
    echo "  zsh:  将 $OUTPUT_DIR 加入 fpath，例如 fpath=($OUTPUT_DIR \$fpath)"
    echo "  fish: cp $OUTPUT_DIR/gitflow-cli.fish ~/.config/fish/completions/"
}

main "$@"
