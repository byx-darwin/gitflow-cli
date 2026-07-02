#!/usr/bin/env bash
# _common.sh — 共享函数库
#
# 为所有 gitflow Skill 脚本提供标准化的错误捕获、pending.json 写入、
# 平台检测等共享函数。
#
# 用法：在 Skill 脚本顶部 source 此文件
#   SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
#   source "$SCRIPT_DIR/_common.sh"

set -euo pipefail

# ---------------------------------------------------------------------------
# ERR trap — 脚本异常退出时自动报告错误
# ---------------------------------------------------------------------------

# on_error <line_number>
# 由 ERR trap 调用，记录脚本执行失败的错误信息
on_error() {
    local line_number="${1:-unknown}"
    report_error \
        "${_CURRENT_SKILL:-unknown}" \
        "$(detect_platform)" \
        "SKILL_ERROR" \
        "Script failed at line ${line_number}"
}

trap 'on_error $LINENO' ERR

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
# report_error <command> <platform> <error_code> <error_message>
# 将错误信息写入 .cache/bug-reports/pending.json
# ---------------------------------------------------------------------------
report_error() {
    local command="${1:?report_error: command is required}"
    local platform="${2:?report_error: platform is required}"
    local error_code="${3:?report_error: error_code is required}"
    local error_message="${4:?report_error: error_message is required}"

    # 确保在 git 仓库中，避免写入文件系统根目录
    local git_root
    git_root="$(git rev-parse --show-toplevel 2>/dev/null || echo "")"
    if [ -z "$git_root" ]; then
        echo "[_common.sh] 警告: 不在 git 仓库中，跳过错误报告" >&2
        return 1
    fi

    # 确保缓存目录存在
    local cache_dir="${git_root}/.cache/bug-reports"
    mkdir -p "$cache_dir"

    local pending_file="${cache_dir}/pending.json"
    local error_id
    error_id="$(generate_error_id)"
    local timestamp
    timestamp="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

    # JSON 转义所有用户可控字段
    local esc_command esc_error_code esc_error_message esc_platform
    esc_command="$(json_escape "$command")"
    esc_platform="$(json_escape "$platform")"
    esc_error_code="$(json_escape "$error_code")"
    esc_error_message="$(json_escape "$error_message")"

    # 写入 JSON
    printf '{
  "error_id": "%s",
  "command": "%s",
  "platform": "%s",
  "error_code": "%s",
  "error_message": "%s",
  "timestamp": "%s"
}\n' \
        "$error_id" \
        "$esc_command" \
        "$esc_platform" \
        "$esc_error_code" \
        "$esc_error_message" \
        "$timestamp" \
        > "$pending_file"

    echo "[_common.sh] 错误已记录到 ${pending_file}" >&2
}

# ---------------------------------------------------------------------------
# generate_error_id
# 生成唯一错误 ID（时间戳 + 进程号 hash）
# ---------------------------------------------------------------------------
generate_error_id() {
    local timestamp
    timestamp="$(date +%s%N 2>/dev/null || date +%s)"
    local pid=$$
    local raw="${timestamp}-${pid}"

    # 使用 sha256sum 或 shasum 生成 hash（兼容 Linux/macOS）
    if command -v sha256sum &>/dev/null; then
        echo "$raw" | sha256sum | cut -c1-32
    elif command -v shasum &>/dev/null; then
        echo "$raw" | shasum -a 256 | cut -c1-32
    else
        # fallback: 使用时间戳和 PID 的简单组合
        echo "err-${timestamp}-${pid}"
    fi
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
# set_skill_name <name>
# 设置当前 Skill 名称（用于 ERR trap 中的错误报告）
# ---------------------------------------------------------------------------
set_skill_name() {
    _CURRENT_SKILL="${1:?set_skill_name: skill name is required}"
}

# ---------------------------------------------------------------------------
# 初始化提示
# ---------------------------------------------------------------------------
echo "[_common.sh] 共享函数库已加载" >&2
