#!/usr/bin/env bash
# install.sh — gitflow-cli 一键安装脚本
#
# 功能：
#   1. 检查依赖（Rust、Git、平台原生 CLI）
#   2. 编译 release 二进制并安装到 PATH
#   3. 安装 Skills 到 ~/.claude/skills/
#   4. 注册 Stop Hook 到项目 .claude/settings.json
#   5. 验证安装结果
#
# 用法：
#   ./scripts/install.sh [选项]
#
# 选项：
#   --no-build    跳过编译步骤
#   --no-skills   跳过 Skills 安装
#   --no-hooks    跳过 Hook 注册
#   --help        显示帮助信息
#
# 支持平台：macOS、Linux、Windows (Git Bash)
# 依赖 Issue: (#5)

set -euo pipefail

# ---------------------------------------------------------------------------
# 常量
# ---------------------------------------------------------------------------

readonly SCRIPT_NAME="install.sh"
readonly BINARY_NAME="gitflow-cli"
readonly MIN_RUST_VERSION="1.96.0"
readonly SKILLS_SOURCE_DIR="skills"
readonly SKILLS_TARGET_DIR="${HOME}/.claude/skills"
readonly HOOKS_SOURCE_DIR="hooks"
readonly SETTINGS_FILE=".claude/settings.json"

# 嵌套 Stop Hook 配置（对齐 Claude Code 官方 schema）
# matcher 在顶层，hooks 数组包含 type+command 对象
readonly HOOK_CONFIG='{"matcher": "gitflow", "hooks": [{"type": "command", "command": "bash hooks/auto-report-bug.sh"}]}'
readonly REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# ---------------------------------------------------------------------------
# 颜色与输出
# ---------------------------------------------------------------------------

if [[ -t 1 ]] && [[ -z "${NO_COLOR:-}" ]]; then
    readonly C_RESET="\033[0m"
    readonly C_GREEN="\033[0;32m"
    readonly C_RED="\033[0;31m"
    readonly C_YELLOW="\033[0;33m"
    readonly C_BLUE="\033[0;34m"
    readonly C_BOLD="\033[1m"
else
    readonly C_RESET=""
    readonly C_GREEN=""
    readonly C_RED=""
    readonly C_YELLOW=""
    readonly C_BLUE=""
    readonly C_BOLD=""
fi

info()    { printf "${C_GREEN}✅ %s${C_RESET}\n" "$*"; }
warn()    { printf "${C_YELLOW}⚠️  %s${C_RESET}\n" "$*" >&2; }
error()   { printf "${C_RED}❌ %s${C_RESET}\n" "$*" >&2; }
step()    { printf "\n${C_BLUE}${C_BOLD}▶ %s${C_RESET}\n" "$*"; }

# ---------------------------------------------------------------------------
# 平台检测
# ---------------------------------------------------------------------------

detect_os() {
    local uname_s
    uname_s="$(uname -s 2>/dev/null || echo "unknown")"
    case "$uname_s" in
        Darwin*)  echo "macos" ;;
        Linux*)   echo "linux" ;;
        MINGW*|MSYS*|CYGWIN*) echo "windows" ;;
        *)        echo "unknown" ;;
    esac
}

readonly PLATFORM_OS="$(detect_os)"

# 判断命令是否存在（兼容 Windows Git Bash 中 which/where 差异）
has_command() {
    if [[ "$PLATFORM_OS" == "windows" ]]; then
        command -v "$1" &>/dev/null || where "$1" &>/dev/null
    else
        command -v "$1" &>/dev/null
    fi
}

# ---------------------------------------------------------------------------
# 参数解析
# ---------------------------------------------------------------------------

FLAG_NO_BUILD=false
FLAG_NO_SKILLS=false
FLAG_NO_HOOKS=false

parse_args() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
            --no-build)   FLAG_NO_BUILD=true ;;
            --no-skills)  FLAG_NO_SKILLS=true ;;
            --no-hooks)   FLAG_NO_HOOKS=true ;;
            --help|-h)
                print_usage
                exit 0
                ;;
            *)
                error "未知参数: $1"
                print_usage
                exit 1
                ;;
        esac
        shift
    done
}

print_usage() {
    cat <<EOF
${BINARY_NAME} 一键安装脚本

用法: ${SCRIPT_NAME} [选项]

选项:
  --no-build    跳过编译步骤（适用于已编译或 Homebrew 安装场景）
  --no-skills   跳过 Skills 安装
  --no-hooks    跳过 Stop Hook 注册
  --help, -h    显示此帮助信息

示例:
  ${SCRIPT_NAME}                   # 完整安装
  ${SCRIPT_NAME} --no-build        # 跳过编译，仅安装 Skills + Hooks
  ${SCRIPT_NAME} --no-skills       # 仅编译安装二进制 + 注册 Hooks

支持平台: macOS、Linux、Windows (Git Bash)
EOF
}

# ---------------------------------------------------------------------------
# 版本比较
# 返回 0 表示 version_a >= version_b
# ---------------------------------------------------------------------------

version_gte() {
    local version_a="$1"
    local version_b="$2"
    # 比较主.次.补丁
    local a_major a_minor a_patch b_major b_minor b_patch
    IFS='.' read -r a_major a_minor a_patch <<< "$version_a"
    IFS='.' read -r b_major b_minor b_patch <<< "$version_b"
    a_major="${a_major:-0}"; a_minor="${a_minor:-0}"; a_patch="${a_patch:-0}"
    b_major="${b_major:-0}"; b_minor="${b_minor:-0}"; b_patch="${b_patch:-0}"

    if (( a_major > b_major )); then return 0; fi
    if (( a_major < b_major )); then return 1; fi
    if (( a_minor > b_minor )); then return 0; fi
    if (( a_minor < b_minor )); then return 1; fi
    if (( a_patch >= b_patch )); then return 0; fi
    return 1
}

# ---------------------------------------------------------------------------
# Step 1: 检查依赖
# ---------------------------------------------------------------------------

check_dependencies() {
    step "Step 1/5: 检查依赖"
    local missing=false

    # 检查 Rust
    if has_command rustc; then
        local rust_version_full rust_version
        rust_version_full="$(rustc --version 2>/dev/null | sed 's/rustc //' | awk '{print $1}')"
        rust_version="${rust_version_full}"
        if version_gte "$rust_version" "$MIN_RUST_VERSION"; then
            info "Rust toolchain: rustc ${rust_version} (>= ${MIN_RUST_VERSION})"
        else
            error "Rust 版本过低: rustc ${rust_version}，需要 >= ${MIN_RUST_VERSION}"
            echo "  请运行: rustup update stable"
            missing=true
        fi
    else
        error "未找到 Rust toolchain"
        echo "  请安装: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        missing=true
    fi

    # 检查 Git
    if has_command git; then
        local git_version
        git_version="$(git --version 2>/dev/null | awk '{print $3}')"
        info "Git: ${git_version}"
    else
        error "未找到 Git"
        case "$PLATFORM_OS" in
            macos)   echo "  请安装: brew install git 或 xcode-select --install" ;;
            linux)   echo "  请安装: sudo apt-get install git / sudo yum install git" ;;
            windows) echo "  请安装: https://git-scm.com/download/win" ;;
            *)       echo "  请安装 Git 后重试" ;;
        esac
        missing=true
    fi

    # 检查 cargo
    if has_command cargo; then
        info "Cargo: $(cargo --version 2>/dev/null | awk '{print $2}')"
    else
        error "未找到 Cargo（Rust 工具链不完整）"
        missing=true
    fi

    # 原生 CLI 自动检测（按平台）
    detect_native_cli

    if [[ "$missing" == "true" ]]; then
        echo ""
        error "依赖检查失败，请先安装缺失的工具后重试"
        exit 1
    fi
}

detect_native_cli() {
    # 根据 git remote 检测平台，提示对应的 CLI
    local remote_url
    remote_url="$(cd "$REPO_ROOT" && git remote get-url origin 2>/dev/null || echo "")"

    local detected_platform="unknown"
    case "$remote_url" in
        *github.com*)    detected_platform="github" ;;
        *gitlab.com*)    detected_platform="gitlab" ;;
        *gitcode.com*)   detected_platform="gitcode" ;;
        *gitee.com*)     detected_platform="gitee" ;;
    esac

    case "$detected_platform" in
        github)
            if has_command gh; then
                info "GitHub CLI (gh): $(gh --version 2>/dev/null | head -1 | awk '{print $3}')"
            else
                warn "未检测到 GitHub CLI (gh)，github 平台操作将受限"
                echo "  安装: brew install gh (macOS) / https://cli.github.com/"
            fi
            ;;
        gitlab)
            if has_command glab; then
                info "GitLab CLI (glab): $(glab --version 2>/dev/null | awk '{print $3}')"
            else
                warn "未检测到 GitLab CLI (glab)，gitlab 平台操作将受限"
                echo "  安装: brew install glab (macOS) / https://gitlab.com/gitlab-org/cli"
            fi
            ;;
        gitcode)
            if has_command gitcode; then
                info "GitCode CLI: 已安装"
            else
                warn "未检测到 GitCode CLI，gitcode 平台操作将受限"
            fi
            ;;
        *)
            info "平台原生 CLI: 未检测到特定平台（跳过检测）"
            ;;
    esac
}

# ---------------------------------------------------------------------------
# Step 2: 编译并安装二进制
# ---------------------------------------------------------------------------

build_and_install() {
    step "Step 2/5: 编译并安装二进制"

    if [[ "$FLAG_NO_BUILD" == "true" ]]; then
        info "跳过编译（--no-build）"
        # 验证已有二进制
        if has_command "$BINARY_NAME"; then
            local existing_version
            existing_version="$("${BINARY_NAME}" --version 2>/dev/null || echo '未知版本')"
            info "使用已有二进制: ${existing_version}"
            return 0
        else
            warn "未找到已有的 ${BINARY_NAME} 二进制，--no-build 模式下需要预先安装"
            return 0
        fi
    fi

    cd "$REPO_ROOT"

    # 检查是否为 Homebrew 安装路径
    if [[ -d "/opt/homebrew/opt/${BINARY_NAME}" ]] || [[ -d "/usr/local/opt/${BINARY_NAME}" ]]; then
        warn "检测到 Homebrew 安装路径，建议使用: brew install ${BINARY_NAME}"
        warn "继续从源码编译将覆盖 Homebrew 版本"
    fi

    info "开始编译 release 版本..."
    cargo build --release

    local binary_path="target/release/${BINARY_NAME}"
    if [[ "$PLATFORM_OS" == "windows" ]]; then
        binary_path="target/release/${BINARY_NAME}.exe"
    fi

    if [[ ! -f "$binary_path" ]]; then
        error "编译产物未找到: ${binary_path}"
        exit 1
    fi

    # 确定安装位置
    local install_dir=""
    if [[ -n "${CARGO_HOME:-}" ]] && [[ -d "${CARGO_HOME}/bin" ]]; then
        install_dir="${CARGO_HOME}/bin"
    elif [[ -d "${HOME}/.cargo/bin" ]]; then
        install_dir="${HOME}/.cargo/bin"
    elif [[ -d "/usr/local/bin" ]] && [[ -w "/usr/local/bin" ]]; then
        install_dir="/usr/local/bin"
    elif [[ "$PLATFORM_OS" == "windows" ]]; then
        # Windows 下使用 ~/bin 或 CARGO_HOME
        install_dir="${HOME}/bin"
        mkdir -p "$install_dir"
    else
        install_dir="/usr/local/bin"
        if [[ ! -d "$install_dir" ]]; then
            error "安装目录不存在: ${install_dir}"
            echo "  请创建目录: sudo mkdir -p ${install_dir}"
            exit 1
        fi
        if [[ ! -w "$install_dir" ]]; then
            error "无写入权限: ${install_dir}"
            echo "  请使用 sudo 运行本脚本，或手动设置权限: sudo chmod o+w ${install_dir}"
            exit 1
        fi
    fi

    info "安装二进制到: ${install_dir}"
    mkdir -p "$install_dir"
    cp "$binary_path" "${install_dir}/${BINARY_NAME}"
    if [[ "$PLATFORM_OS" == "windows" ]]; then
        cp "target/release/${BINARY_NAME}.exe" "${install_dir}/${BINARY_NAME}.exe"
    fi
    chmod +x "${install_dir}/${BINARY_NAME}"

    info "编译并安装完成"
}

# ---------------------------------------------------------------------------
# Step 3: 安装 Skills
# ---------------------------------------------------------------------------

install_skills() {
    step "Step 3/5: 安装 Skills"

    if [[ "$FLAG_NO_SKILLS" == "true" ]]; then
        info "跳过 Skills 安装（--no-skills）"
        return 0
    fi

    local skills_source="${REPO_ROOT}/${SKILLS_SOURCE_DIR}"
    if [[ ! -d "$skills_source" ]]; then
        warn "Skills 目录不存在: ${skills_source}，跳过安装"
        return 0
    fi

    mkdir -p "$SKILLS_TARGET_DIR"

    local installed=0
    local skipped=0

    for skill_dir in "$skills_source"/*/; do
        [[ -d "$skill_dir" ]] || continue
        local skill_name
        skill_name="$(basename "$skill_dir")"

        # 跳过非 gitflow 前缀的目录（如 _common.sh 所在的父目录不会被遍历）
        local target_path="${SKILLS_TARGET_DIR}/${skill_name}"

        if [[ -d "$target_path" ]]; then
            warn "冲突: Skill '${skill_name}' 已存在于 ${target_path}，跳过"
            (( skipped++ )) || true
            continue
        fi

        # 复制整个 skill 目录
        cp -r "$skill_dir" "$target_path"
        (( installed++ )) || true
        info "安装 Skill: ${skill_name}"
    done

    # 复制 _common.sh 共享库
    if [[ -f "${skills_source}/_common.sh" ]]; then
        cp "${skills_source}/_common.sh" "${SKILLS_TARGET_DIR}/_common.sh"
        info "安装共享库: _common.sh"
    fi

    echo ""
    info "Skills 安装完成: 新增 ${installed} 个，跳过 ${skipped} 个（冲突）"
}

# ---------------------------------------------------------------------------
# Step 4: 注册 Hooks
# ---------------------------------------------------------------------------

register_hooks() {
    step "Step 4/5: 注册 Stop Hook"

    if [[ "$FLAG_NO_HOOKS" == "true" ]]; then
        info "跳过 Hook 注册（--no-hooks）"
        return 0
    fi

    local settings_target="${REPO_ROOT}/${SETTINGS_FILE}"
    local settings_dir
    settings_dir="$(dirname "$settings_target")"

    # 确保 hooks 源文件存在
    local hook_script="${REPO_ROOT}/${HOOKS_SOURCE_DIR}/auto-report-bug.sh"
    if [[ ! -f "$hook_script" ]]; then
        warn "Hook 脚本不存在: ${hook_script}，跳过注册"
        return 0
    fi

    # 确保 hook 脚本可执行
    chmod +x "$hook_script"

    # 确保 .claude 目录存在
    mkdir -p "$settings_dir"

    # 检查 settings.json 中是否已有 Stop hook 配置
    if [[ -f "$settings_target" ]]; then
        # 检查是否已包含 auto-report-bug hook
        if grep -q "auto-report-bug" "$settings_target" 2>/dev/null; then
            info "Stop Hook 已配置（auto-report-bug），跳过注册"
            return 0
        fi

        # settings.json 存在但没有 hook — 需要合并
        # 使用简单方式：检查是否有 hooks 键，然后追加
        if grep -q '"hooks"' "$settings_target" 2>/dev/null; then
            # 已有 hooks 字段但没匹配 — 需要手动合并
            warn "settings.json 已有 hooks 配置，但缺少 auto-report-bug"
            warn "请手动合并以下配置到: ${settings_target}"
            echo "  ${HOOK_CONFIG}"
            return 0
        fi
    fi

    # 仅当目标文件不存在或为空时写入，否则提示手动合并
    if [[ -f "$settings_target" ]] && [[ -s "$settings_target" ]]; then
        warn "settings.json 已存在且包含其他配置，不会自动覆盖"
        warn "请手动将以下 Hook 配置合并到: ${settings_target}"
        echo ""
        echo '  在现有 JSON 对象中添加以下 "hooks" 键：'
        echo ""
        cat <<MERGE_EOF
    "hooks": {
      "Stop": [
        ${HOOK_CONFIG}
      ]
    }
MERGE_EOF
        echo ""
        return 0
    fi

    cat > "$settings_target" <<SETTINGS_EOF
{
  "hooks": {
    "Stop": [
      ${HOOK_CONFIG}
    ]
  }
}
SETTINGS_EOF

    info "Stop Hook 已注册到: ${settings_target}"
}

# ---------------------------------------------------------------------------
# Step 5: 验证
# ---------------------------------------------------------------------------

verify_installation() {
    step "Step 5/5: 验证安装"

    local ok=true

    # 验证二进制
    if has_command "$BINARY_NAME"; then
        local version_output
        version_output="$("${BINARY_NAME}" --version 2>/dev/null || echo '版本未知')"
        info "${BINARY_NAME} 已安装: ${version_output}"

        # 尝试 auth status
        if "${BINARY_NAME}" auth status &>/dev/null 2>&1; then
            info "${BINARY_NAME} auth status: 正常"
        else
            warn "${BINARY_NAME} auth status: 未认证或命令不可用（非致命）"
        fi
    else
        error "未找到 ${BINARY_NAME} 命令，请确保安装目录在 PATH 中"
        ok=false
    fi

    # 统计安装的 Skills 数量
    local skill_count=0
    if [[ -d "$SKILLS_TARGET_DIR" ]]; then
        # 统计 gitflow- 开头的目录数
        skill_count=$(find "$SKILLS_TARGET_DIR" -maxdepth 1 -type d -name "gitflow-*" 2>/dev/null | wc -l | tr -d ' ')
        info "已安装 Skills: ${skill_count} 个"
    else
        warn "Skills 目录不存在: ${SKILLS_TARGET_DIR}"
    fi

    # 验证 hook
    local settings_target="${REPO_ROOT}/${SETTINGS_FILE}"
    if [[ -f "$settings_target" ]] && grep -q "auto-report-bug" "$settings_target" 2>/dev/null; then
        info "Stop Hook: 已配置"
    else
        warn "Stop Hook: 未配置（运行 ${SCRIPT_NAME} 不带 --no-hooks 可注册）"
    fi

    if [[ "$ok" == "true" ]]; then
        echo ""
        info "🎉 gitflow-cli 安装完成！"
    else
        echo ""
        error "安装过程中出现错误，请检查上方日志"
        return 1
    fi
}

# ---------------------------------------------------------------------------
# 主流程
# ---------------------------------------------------------------------------

main() {
    parse_args "$@"

    echo ""
    printf "${C_BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${C_RESET}\n"
    printf "${C_BOLD}  gitflow-cli 一键安装脚本${C_RESET}\n"
    printf "${C_BOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${C_RESET}\n"
    echo ""
    info "平台: ${PLATFORM_OS}"
    info "仓库根目录: ${REPO_ROOT}"

    if [[ "$FLAG_NO_BUILD" == "true" ]]; then info "标志: 跳过编译"; fi
    if [[ "$FLAG_NO_SKILLS" == "true" ]]; then info "标志: 跳过 Skills 安装"; fi
    if [[ "$FLAG_NO_HOOKS" == "true" ]]; then info "标志: 跳过 Hook 注册"; fi

    check_dependencies
    build_and_install
    install_skills
    register_hooks
    verify_installation
}

main "$@"
