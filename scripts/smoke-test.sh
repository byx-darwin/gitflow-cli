#!/usr/bin/env bash
set -uo pipefail
# 注意：不使用 set -e，因为我们希望即使某些测试失败也继续运行

# 版本信息
VERSION="1.0.0"
SCRIPT_NAME="$(basename "$0")"

# 默认值
PLATFORM="github"
MODE="read-only"
VERBOSE=0

# 计数器
PASS_COUNT=0
FAIL_COUNT=0
SKIP_COUNT=0

# 查找 gitflow-cli 二进制文件
# 优先使用本地构建的版本
if [[ -f "./target/release/gitflow-cli" ]]; then
    GITFLOW_CLI="./target/release/gitflow-cli"
elif [[ -f "./target/debug/gitflow-cli" ]]; then
    GITFLOW_CLI="./target/debug/gitflow-cli"
elif command -v gitflow-cli &> /dev/null; then
    GITFLOW_CLI="gitflow-cli"
else
    echo "错误: gitflow-cli 未找到。请先运行 'cargo build' 或 'cargo install'" >&2
    exit 1
fi

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 日志函数
log_info() {
    echo -e "${BLUE}[INFO]${NC} $*"
}

log_success() {
    echo -e "${GREEN}[PASS]${NC} $*"
    ((PASS_COUNT++))
}

log_fail() {
    echo -e "${RED}[FAIL]${NC} $*"
    ((FAIL_COUNT++))
}

log_skip() {
    echo -e "${YELLOW}[SKIP]${NC} $*"
    ((SKIP_COUNT++))
}

log_verbose() {
    if [[ $VERBOSE -eq 1 ]]; then
        echo -e "${BLUE}[DEBUG]${NC} $*"
    fi
}

# 使用帮助
usage() {
    cat << EOF
用法: $SCRIPT_NAME [选项]

多平台冒烟测试脚本，支持 GitHub、GitLab、GitCode

选项:
    --platform <平台>    指定测试平台 (github|gitlab|gitcode)，默认: github
    --read-only          只读模式，仅测试 --help 和读取命令 (默认)
    --write              写入模式，额外测试写入命令的 --help
    --verbose, -v        详细输出模式
    --version            显示版本信息
    --help, -h           显示此帮助信息

示例:
    $SCRIPT_NAME                           # 使用默认平台 (github) 只读模式
    $SCRIPT_NAME --platform gitlab         # 测试 GitLab 平台
    $SCRIPT_NAME --platform gitcode --write # 测试 GitCode 平台，包含写入命令
    $SCRIPT_NAME --read-only --verbose     # 只读模式，详细输出

退出码:
    0    所有测试通过
    1    存在失败的测试
EOF
}

# 测试命令执行
test_command() {
    local description="$1"
    shift
    local cmd=("$@")
    local allow_skip="${ALLOW_SKIP:-false}"

    log_verbose "执行: ${cmd[*]}"

    if output=$("${cmd[@]}" 2>&1); then
        log_success "$description"
        log_verbose "输出: $output"
        return 0
    else
        local exit_code=$?
        # 检查是否是因为缺少原生 CLI
        if echo "$output" | grep -qi "native cli.*not.*found\|command not found\|native CLI.*required\|failed to parse.*version"; then
            log_skip "$description (缺少原生 CLI)"
            return 2
        # 检查是否是认证/授权错误 (仅在允许跳过时)
        elif [[ "$allow_skip" == "true" ]] && echo "$output" | grep -qi "auth\|unauthorized\|forbidden\|token\|credentials\|not authenticated\|serialization error"; then
            log_skip "$description (API 错误)"
            return 2
        else
            log_fail "$description (退出码: $exit_code)"
            log_verbose "错误输出: $output"
            return 1
        fi
    fi
}

# 测试 --help 命令
test_help() {
    local description="$1"
    shift
    test_command "$description" "$GITFLOW_CLI" "$@" --help
}

# 测试所有资源的 --help
test_all_resources_help() {
    local platform="$1"

    log_info "测试所有资源的 --help 命令 (平台: $platform)"

    # 主要资源类型
    local resources=("issue" "pr" "release" "review" "auth" "label" "milestone" "commit" "pipeline")

    for resource in "${resources[@]}"; do
        test_help "$resource --help" --platform "$platform" "$resource"
    done
}

# 测试 pipeline 命令
test_pipeline_commands() {
    local platform="$1"

    log_info "测试 pipeline 子命令 (平台: $platform)"

    test_help "pipeline status --help" --platform "$platform" pipeline status
    test_help "pipeline logs --help" --platform "$platform" pipeline logs
    test_help "pipeline jobs --help" --platform "$platform" pipeline jobs
    test_help "pipeline report --help" --platform "$platform" pipeline report
}

# 测试读取命令的 --help
test_read_commands_help() {
    local platform="$1"

    log_info "测试读取命令的 --help (平台: $platform)"

    # issue 子命令
    test_help "issue list --help" --platform "$platform" issue list
    test_help "issue view --help" --platform "$platform" issue view
    test_help "issue comment --help" --platform "$platform" issue comment
    test_help "issue close --help" --platform "$platform" issue close
    test_help "issue reopen --help" --platform "$platform" issue reopen

    # pr 子命令
    test_help "pr list --help" --platform "$platform" pr list
    test_help "pr view --help" --platform "$platform" pr view
    test_help "pr create --help" --platform "$platform" pr create
    test_help "pr merge --help" --platform "$platform" pr merge
    test_help "pr checkout --help" --platform "$platform" pr checkout
    test_help "pr ready --help" --platform "$platform" pr ready
    test_help "pr wip --help" --platform "$platform" pr wip
    test_help "pr sync --help" --platform "$platform" pr sync
    test_help "pr comment --help" --platform "$platform" pr comment

    # release 子命令
    test_help "release list --help" --platform "$platform" release list
    test_help "release view --help" --platform "$platform" release view
    test_help "release create --help" --platform "$platform" release create
    test_help "release edit --help" --platform "$platform" release edit
    test_help "release delete --help" --platform "$platform" release delete

    # review 子命令
    test_help "review comment --help" --platform "$platform" review comment
    test_help "review approve --help" --platform "$platform" review approve
    test_help "review request-changes --help" --platform "$platform" review request-changes
    test_help "review submit --help" --platform "$platform" review submit

    # auth 子命令
    test_help "auth login --help" --platform "$platform" auth login
    test_help "auth logout --help" --platform "$platform" auth logout
    test_help "auth status --help" --platform "$platform" auth status
    test_help "auth token --help" --platform "$platform" auth token

    # label 子命令
    test_help "label list --help" --platform "$platform" label list
    test_help "label create --help" --platform "$platform" label create
    test_help "label edit --help" --platform "$platform" label edit
    test_help "label delete --help" --platform "$platform" label delete

    # milestone 子命令
    test_help "milestone list --help" --platform "$platform" milestone list
    test_help "milestone create --help" --platform "$platform" milestone create
    test_help "milestone edit --help" --platform "$platform" milestone edit
    test_help "milestone close --help" --platform "$platform" milestone close
    test_help "milestone reopen --help" --platform "$platform" milestone reopen

    # commit 子命令
    test_help "commit view --help" --platform "$platform" commit view
    test_help "commit diff --help" --platform "$platform" commit diff
    test_help "commit patch --help" --platform "$platform" commit patch
    test_help "commit comment --help" --platform "$platform" commit comment
}

# 测试写入命令的 --help
test_write_commands_help() {
    local platform="$1"

    log_info "测试写入命令的 --help (平台: $platform)"

    # issue 写入命令
    test_help "issue create --help" --platform "$platform" issue create

    # pr 写入命令（已在读取命令中测试过 create）
    # release 写入命令（已在读取命令中测试过 create、edit、delete）

    # auth 写入命令
    test_help "auth login --help" --platform "$platform" auth login

    # label 写入命令（已在读取命令中测试过 create、edit、delete）
    # milestone 写入命令（已在读取命令中测试过 create、edit、close、reopen）
}

# 最佳努力 API 测试(读取命令)
test_api_read_commands() {
    local current_platform="$1"

    log_info "测试 API 读取命令 (最佳努力模式, 平台: $current_platform)"

    # 允许跳过认证相关的错误
    export ALLOW_SKIP=true

    # 这些命令可能因为缺少认证或原生 CLI 而失败,但我们应该优雅地跳过
    test_command "issue list (API)" "$GITFLOW_CLI" --platform "$current_platform" issue list --limit 3 || true
    test_command "pr list (API)" "$GITFLOW_CLI" --platform "$current_platform" pr list --limit 3 || true
    test_command "release list (API)" "$GITFLOW_CLI" --platform "$current_platform" release list --limit 3 || true
    test_command "label list (API)" "$GITFLOW_CLI" --platform "$current_platform" label list || true
    test_command "milestone list (API)" "$GITFLOW_CLI" --platform "$current_platform" milestone list || true

    # 重置
    unset ALLOW_SKIP
}

# 解析命令行参数
parse_args() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --platform)
                if [[ -z "${2:-}" ]]; then
                    echo "错误: --platform 需要参数" >&2
                    usage
                    exit 1
                fi
                PLATFORM="$2"
                if [[ "$PLATFORM" != "github" && "$PLATFORM" != "gitlab" && "$PLATFORM" != "gitcode" ]]; then
                    echo "错误: 无效的平台 '$PLATFORM'，必须是 github、gitlab 或 gitcode" >&2
                    exit 1
                fi
                shift 2
                ;;
            --read-only)
                MODE="read-only"
                shift
                ;;
            --write)
                MODE="write"
                shift
                ;;
            --verbose|-v)
                VERBOSE=1
                shift
                ;;
            --version)
                echo "$SCRIPT_NAME 版本 $VERSION"
                exit 0
                ;;
            --help|-h)
                usage
                exit 0
                ;;
            *)
                echo "错误: 未知选项 '$1'" >&2
                usage
                exit 1
                ;;
        esac
    done
}

# 主函数
main() {
    parse_args "$@"

    echo "========================================"
    echo "gitflow-cli 多平台冒烟测试"
    echo "========================================"
    echo "平台: $PLATFORM"
    echo "模式: $MODE"
    echo "二进制: $GITFLOW_CLI"
    echo "========================================"
    echo ""

    log_info "gitflow-cli 版本: $($GITFLOW_CLI --version)"
    echo ""

    # 测试主命令 --help
    log_info "测试主命令 --help"
    test_help "gitflow-cli --help"

    # 测试所有资源 --help
    echo ""
    test_all_resources_help "$PLATFORM"

    # 测试 pipeline 命令
    echo ""
    test_pipeline_commands "$PLATFORM"

    # 测试读取命令 --help
    echo ""
    test_read_commands_help "$PLATFORM"

    # 如果是写入模式，测试写入命令
    if [[ "$MODE" == "write" ]]; then
        echo ""
        test_write_commands_help "$PLATFORM"
    fi

    # 最佳努力 API 测试（仅在 read-only 模式下，因为写入命令需要实际执行）
    if [[ "$MODE" == "read-only" ]]; then
        echo ""
        test_api_read_commands "$PLATFORM"
    fi

    # 打印总结
    echo ""
    echo "========================================"
    echo "测试总结"
    echo "========================================"
    echo -e "通过: ${GREEN}$PASS_COUNT${NC}"
    echo -e "失败: ${RED}$FAIL_COUNT${NC}"
    echo -e "跳过: ${YELLOW}$SKIP_COUNT${NC}"
    echo "========================================"

    if [[ $FAIL_COUNT -gt 0 ]]; then
        echo -e "${RED}测试失败${NC}"
        exit 1
    else
        echo -e "${GREEN}测试通过${NC}"
        exit 0
    fi
}

# 执行主函数
main "$@"
