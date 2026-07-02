# Task 5 报告: apps/cli — Shell 补全增强

**状态**: DONE
**提交数**: 0 (实现完成,待合并时统一提交)

## 实现内容

### 1. 增强 `apps/cli/src/commands/completions.rs`

**新增 `Shell` 方法**:
- `from_env_name(s: &str) -> Option<Self>`: 从路径名解析 shell 类型 (如 `/bin/zsh` → `Zsh`)
- `detect_from_env() -> miette::Result<Self>`: 从 `$SHELL` 环境变量自动检测 shell
- `install_dir(self, home_override: Option<&Path>) -> miette::Result<PathBuf>`: 返回各 shell 的安装目录
  - bash: `<home>/.local/share/bash-completion/completions/`
  - zsh:  `<home>/.zfunc/`
  - fish: `<home>/.config/fish/completions/`
- `completion_filename(self) -> &'static str`: 返回对应 shell 的补全文件名
  - bash: `gitflow-cli.bash`
  - zsh:  `_gitflow-cli` (遵循 zsh 命名惯例)
  - fish: `gitflow-cli.fish`

**新增 CLI 参数**:
- `CompletionsArgs.shell` 改为 `Option<Shell>`,使用 `required_unless_present_any = ["install", "uninstall"]`
- `--install`: 安装补全脚本到对应 shell 配置目录
- `--uninstall`: 卸载补全脚本
- `--install` 与 `--uninstall` 通过 `conflicts_with` 互斥

**重构**:
- `generate<C>` 函数根据 `install`/`uninstall` 标志分派到不同逻辑分支
- 新增私有辅助函数 `resolve_shell`, `write_completion`, `install`, `uninstall`, `shell_name_str`
- 安装/卸载流程均通过 `miette` 报告清晰的错误信息

**关键设计决策**:
- 保留 `std::fs::*` 进行文件操作 (completions 命令在 tokio runtime 构造之前执行),通过 `#[allow(clippy::disallowed_methods, reason = "...")]` 抑制告警
- `Shell` 枚举派生 `Copy` 以允许 `self` 而非 `&self` 的参数风格 (满足 pedantic clippy)
- `install_dir` 接受 `home_override` 参数,避免测试中修改环境变量 (Rust 2024 中 `set_var` 为 unsafe)

### 2. 创建 `scripts/generate-completions.sh`

- 预生成 bash/zsh/fish 三种 shell 的补全文件到 `completions/` 目录
- 按优先级定位二进制: `target/release/` → `target/debug/` → `$PATH` 中的 `gitflow-cli`
- 彩色输出 (info/success/error),中文提示
- 输出使用说明 (bash source / zsh fpath / fish 复制)

### 3. 更新 Makefile

- `completions`: 改为调用 `scripts/generate-completions.sh`
- 新增 `completions-install`: 运行 `cargo run -- completions --install`
- 新增 `completions-uninstall`: 运行 `cargo run -- completions --uninstall`
- `install`: 增加 `completions-install` 依赖,使 `make install` 同时安装补全脚本
- 更新 `.PHONY` 列表

### 4. 测试

**单元测试** (15 个,在 `completions.rs` 内):
- `test_should_detect_bash_from_full_path`
- `test_should_detect_zsh_from_full_path`
- `test_should_detect_fish_from_bare_name`
- `test_should_return_none_for_unknown_shell`
- `test_should_return_correct_{bash,zsh,fish}_filename` (3 个)
- `test_should_return_correct_{bash,zsh,fish}_install_dir` (3 个)
- `test_should_generate_{bash,zsh,fish}_completion_contains_*` (3 个)
- `test_should_generate_non_empty_output_for_all_shells`
- `test_should_produce_different_output_per_shell`

**集成测试** (11 个,在 `apps/cli/tests/completions_test.rs`):
- `test_should_generate_{bash,zsh,fish}_completion_to_stdout`
- `test_should_require_shell_without_flags`
- `test_should_reject_install_and_uninstall_together`
- `test_should_install_completion_for_explicit_shell` (使用 tempdir 隔离)
- `test_should_uninstall_existing_completion` (先 fixture 写入再卸载验证)
- `test_should_fail_uninstall_when_file_missing`
- `test_should_auto_detect_shell_from_env` (覆盖 `$SHELL` 环境变量)
- `test_should_reject_unsupported_shell_from_env`
- `test_should_fail_when_shell_env_is_missing`

## 测试结果

- CLI crate: 133 个单元测试 + 11 个补全集成测试 + 14 个其他集成测试 = 全部通过 ✅
- 工作区: 所有 crate 的全部单元测试和文档测试通过 ✅
- `cargo fmt --check` (nightly): 通过 ✅
- `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic`: 通过 ✅

## 修改的文件

- `apps/cli/src/commands/completions.rs` — 核心实现 (重写,从 45 行扩展到 ~420 行)
- `apps/cli/tests/completions_test.rs` — 新增集成测试文件 (11 个测试)
- `scripts/generate-completions.sh` — 新增补全生成脚本 (可执行)
- `Makefile` — 新增 `completions-install`/`completions-uninstall` 目标,`completions` 改用脚本,`install` 集成补全

## 自审发现

1. **clippy pedantic 合规**: 最初版本触发 7 个 pedantic 告警 (`std::fs::*` 禁用、`&self` 传值优化、`let...else` 模式)。均已修复:
   - 添加带 reason 的 `#[allow(clippy::disallowed_methods)]`
   - `&self` 改为 `self` (因为 `Shell` 是 `Copy` 枚举)
   - `match` 改为 `let...else`

2. **Rust 2024 兼容性**: `std::env::set_var` 在 Rust 2024 中为 unsafe (因多线程竞争)。通过重构 `install_dir` 接受 `home_override` 参数,完全避免测试中修改环境变量。

3. **集成测试隔离**: 使用 `tempfile::tempdir()` 创建临时 HOME 目录,确保测试不会污染用户真实 shell 配置。

4. **zsh 文件名约定**: 使用 `_gitflow-cli` (不带 `.zsh` 后缀),遵循 zsh 补全系统的命名约定。

## 注意事项

- 无阻塞性问题
- 实现完整,符合任务 brief 全部 4 个 step 的要求
- 所有 CLI 参数行为已通过集成测试验证 (stdout 生成、install、uninstall、auto-detect、error paths)
- 脚本在 macOS (zsh) 和 bash 环境下手工测试通过
