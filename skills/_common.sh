#!/usr/bin/env bash
# _common.sh — 共享函数库
#
# 为所有 gf Skill 脚本提供 JSON 转义、平台检测等共享函数。
#
# 用法：在 Skill 脚本顶部 source 此文件
#   SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
#   source "$SCRIPT_DIR/_common.sh"

set -euo pipefail

# ---------------------------------------------------------------------------
# json_escape <string>
# 转义字符串中的 JSON 特殊字符（双引号、反斜杠、换行、制表符、回车）
# ---------------------------------------------------------------------------
json_escape() {
    local s="$1"
    s="${s//\\/\\\\}"   # 反斜杠 -> \\
    s="${s//\"/\\\"}"   # 双引号 -> \"
    s="${s//$'\n'/\\n}" # 换行 -> \n
    s="${s//$'\r'/\\r}" # 回车 -> \r
    s="${s//$'\t'/\\t}" # 制表符 -> \t
    printf '%s' "$s"
}

# ---------------------------------------------------------------------------
# detect_platform
# 从 git remote URL 检测当前平台（github / gitlab / gitcode / gitee / bitbucket / unknown）
# ---------------------------------------------------------------------------
detect_platform() {
    local remote_url
    remote_url=$(git remote get-url origin 2>/dev/null || echo "")

    if [ -z "$remote_url" ]; then
        echo "unknown"
        return 0
    fi

    case "$remote_url" in
        *github.com*)   echo "github" ;;
        *gitlab.com*)   echo "gitlab" ;;
        *gitcode.com*)  echo "gitcode" ;;
        *gitee.com*)    echo "gitee" ;;
        *bitbucket.org*) echo "bitbucket" ;;
        *git.n.xiaomi.com*) echo "xiaomi-git" ;;
        *)              echo "unknown" ;;
    esac
}

# ---------------------------------------------------------------------------
# cd_to_git_root
# 切换到 git 仓库根目录
# ---------------------------------------------------------------------------
cd_to_git_root() {
    local root
    root=$(git rev-parse --show-toplevel 2>/dev/null)
    if [ -z "$root" ]; then
        echo "错误: 当前目录不在 git 仓库中" >&2
        return 1
    fi
    cd "$root"
}

# ---------------------------------------------------------------------------
# check_prerequisites <cli_name> [cli_name ...]
# 检查所需 CLI 工具是否可用，缺失时报错退出
# ---------------------------------------------------------------------------
check_prerequisites() {
    if [ $# -eq 0 ]; then
        echo "用法: check_prerequisites <cli_name> [cli_name ...]" >&2
        return 1
    fi

    local missing=()
    for cmd in "$@"; do
        if ! command -v "$cmd" &>/dev/null; then
            missing+=("$cmd")
        fi
    done

    if [ ${#missing[@]} -gt 0 ]; then
        echo "错误: 缺少必要的 CLI 工具: ${missing[*]}" >&2
        echo "请先安装这些工具后再运行此脚本。" >&2
        return 1
    fi
}

# ---------------------------------------------------------------------------
# 初始化提示
# ---------------------------------------------------------------------------
echo "[_common.sh] 共享函数库已加载" >&2
