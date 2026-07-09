# E2E 非交互式测试框架设计文档

**Issue**: #71
**日期**: 2026-07-09
**状态**: 设计中
**模式**: full (四阶段工作流)

---

## 1. 背景与目标

### 1.1 问题陈述

在 v0.6.0 全量测试中，GitCode `pr merge` 缺少 `--yes` 参数的问题没有被发现，因为测试只在交互模式下进行。同类问题可能仍存在于其他命令中。

需要建立非交互式 E2E 测试框架，模拟 CI/Agent 环境（无 TTY）运行每个 gitflow-cli 命令，确保所有命令在非交互模式下也能正常工作。

### 1.2 目标

- 建立 Rust 集成测试框架，同时测试交互模式（有 TTY）和非交互模式（无 TTY）
- 覆盖所有平台的认证命令和破坏性命令
- 分阶段交付：Phase 1 (GitHub) → Phase 2 (GitCode) → Phase 3 (GitLab)
- 集成到 CI workflow，自动运行并生成测试报告

### 1.3 非目标

- 不测试只读命令的所有边界情况（由单元测试覆盖）
- 不替换现有的 smoke test 脚本
- 不解决 GitLab 认证问题（Phase 3 的前置条件）

---

## 2. 架构设计

### 2.1 Workspace 结构扩展

```
gitflow-cli/
├── crates/
│   ├── core/              # 现有
│   ├── github/            # 现有
│   ├── gitcode/           # 现有
│   ├── gitlab/            # 现有
│   ├── e2e-core/          # 🆕 共享测试工具
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── tty.rs     # TTY 控制（portable-pty 封装）
│   │   │   ├── assertions.rs  # 自定义断言宏
│   │   │   ├── cleanup.rs     # 测试资源清理
│   │   │   └── config.rs      # 测试配置（环境变量读取）
│   │   └── Cargo.toml
│   ├── e2e-github/        # 🆕 Phase 1
│   │   ├── tests/
│   │   │   ├── auth.rs
│   │   │   ├── issue.rs
│   │   │   ├── pr.rs
│   │   │   └── label.rs
│   │   └── Cargo.toml
│   ├── e2e-gitcode/       # 🆕 Phase 2
│   └── e2e-gitlab/        # 🆕 Phase 3
└── .github/workflows/
    └── e2e-tests.yml      # 🆕 E2E 测试 workflow
```

### 2.2 依赖关系

```toml
# crates/e2e-core/Cargo.toml
[dependencies]
portable-pty = "0.8"       # TTY 控制
assert_cmd = "2.0"         # 命令断言
predicates = "3.0"         # 谓词断言
tokio = { version = "1", features = ["full"] }
serde_json = "1.0"         # 解析 JSON/TOON 输出
thiserror = "1.0"
tracing = "0.1"

# crates/e2e-github/Cargo.toml
[dev-dependencies]
e2e-core = { path = "../e2e-core" }
tokio = { version = "1", features = ["full", "test-util"] }
```

### 2.3 测试执行流程

```
┌─────────────────────────────────────────────────┐
│  Test Runner (cargo nextest)                    │
└────────────────┬────────────────────────────────┘
                 │
        ┌────────▼────────┐
        │  e2e-core       │
        │  ┌───────────┐  │
        │  │ TtyRunner │  │  ← 封装 portable-pty
        │  └───────────┘  │
        │  ┌───────────┐  │
        │  │ Assertions│  │  ← 自定义断言
        │  └───────────┘  │
        │  ┌───────────┐  │
        │  │ Cleanup   │  │  ← 测试后清理
        │  └───────────┘  │
        └────────┬────────┘
                 │
    ┌────────────┼────────────┐
    ▼            ▼            ▼
┌────────┐  ┌────────┐  ┌────────┐
│ GitHub │  │ GitCode│  │ GitLab │
│ Tests  │  │ Tests  │  │ Tests  │
└────────┘  └────────┘  └────────┘
```

---

## 3. TTY 控制机制

### 3.1 核心挑战

需要同时测试两种模式：
- **交互模式**：有 TTY，命令可以提示用户确认
- **非交互模式**：无 TTY（stdin 重定向），命令必须自动处理或失败

### 3.2 TtyRunner 抽象

```rust
// crates/e2e-core/src/tty.rs

use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::process::ExitStatus;

/// TTY 模式
#[derive(Debug, Clone, Copy)]
pub enum TtyMode {
    /// 有 TTY（交互模式）
    Interactive,
    /// 无 TTY（非交互模式，stdin 重定向）
    NonInteractive,
}

/// TTY 测试运行器
pub struct TtyRunner {
    mode: TtyMode,
    working_dir: PathBuf,
    env_vars: HashMap<String, String>,
}

impl TtyRunner {
    pub fn new(mode: TtyMode) -> Self {
        Self {
            mode,
            working_dir: std::env::current_dir().unwrap(),
            env_vars: HashMap::new(),
        }
    }

    /// 设置环境变量
    pub fn env<K, V>(&mut self, key: K, value: V) -> &mut Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.env_vars.insert(key.into(), value.into());
        self
    }

    /// 执行命令并返回输出
    pub async fn run(&self, args: &[&str]) -> Result<CommandOutput> {
        match self.mode {
            TtyMode::Interactive => self.run_with_pty(args).await,
            TtyMode::NonInteractive => self.run_without_pty(args).await,
        }
    }

    /// 有 TTY 模式：使用 portable-pty
    async fn run_with_pty(&self, args: &[&str]) -> Result<CommandOutput> {
        let pty_system = native_pty_system();
        let pty_pair = pty_system.openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let mut cmd = CommandBuilder::new("gitflow-cli");
        cmd.args(args);
        cmd.cwd(&self.working_dir);

        for (k, v) in &self.env_vars {
            cmd.env(k, v);
        }

        let child = pty_pair.slave.spawn_command(cmd)?;
        let output = pty_pair.master.read_to_string()?;
        let status = child.wait()?;

        Ok(CommandOutput {
            stdout: output,
            stderr: String::new(),
            status,
        })
    }

    /// 无 TTY 模式：stdin 重定向到 /dev/null
    async fn run_without_pty(&self, args: &[&str]) -> Result<CommandOutput> {
        use tokio::process::Command;

        let mut cmd = Command::new("gitflow-cli");
        cmd.args(args);
        cmd.current_dir(&self.working_dir);
        cmd.stdin(std::process::Stdio::null()); // 关键：重定向 stdin
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        for (k, v) in &self.env_vars {
            cmd.env(k, v);
        }

        let output = cmd.output().await?;

        Ok(CommandOutput {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            status: output.status,
        })
    }
}

/// 命令输出
#[derive(Debug)]
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
    pub status: ExitStatus,
}
```

### 3.3 设计决策

1. **`portable-pty`**：跨平台伪终端库，Windows 也支持
2. **`stdin: null`**：模拟 CI 环境的标准方式
3. **统一接口**：`TtyRunner` 屏蔽底层差异，测试代码简洁

---

## 4. 测试数据管理

### 4.1 核心挑战

1. **只读命令**（list, view, status）：可以安全执行
2. **破坏性命令**（close, merge, delete）：会修改真实数据，需要测试后恢复
3. **平台凭证**：需要 GitHub/GitLab/GitCode 的访问令牌
4. **测试仓库**：需要专用的测试仓库，避免影响真实项目

### 4.2 测试数据工厂

```rust
// crates/e2e-core/src/fixture.rs

use serde::{Deserialize, Serialize};

/// 测试资源类型
#[derive(Debug, Clone)]
pub enum TestResource {
    Issue { number: u64 },
    Pr { number: u64 },
    Label { name: String },
    Milestone { number: u64 },
    Release { tag: String },
}

/// 测试固件管理器
pub struct TestFixture {
    platform: Platform,
    repo: String,
    created_resources: Vec<TestResource>,
}

impl TestFixture {
    pub fn new(platform: Platform) -> Self {
        Self {
            platform,
            repo: std::env::var("E2E_TEST_REPO")
                .expect("E2E_TEST_REPO must be set"),
            created_resources: Vec::new(),
        }
    }

    /// 创建测试 Issue
    pub async fn create_issue(&mut self, title: &str) -> Result<u64> {
        let runner = TtyRunner::new(TtyMode::NonInteractive);
        let output = runner
            .run(&["issue", "create", "--title", title, "--output", "json"])
            .await?;

        let issue: Issue = serde_json::from_str(&output.stdout)?;
        self.created_resources.push(TestResource::Issue { number: issue.number });
        Ok(issue.number)
    }

    /// 创建测试 Label
    pub async fn create_label(&mut self, name: &str) -> Result<()> {
        let runner = TtyRunner::new(TtyMode::NonInteractive);
        runner
            .run(&["label", "create", "--name", name, "--color", "ff0000"])
            .await?;
        self.created_resources.push(TestResource::Label { name: name.to_string() });
        Ok(())
    }

    /// 清理所有创建的资源
    pub async fn cleanup(&mut self) -> Result<()> {
        let runner = TtyRunner::new(TtyMode::NonInteractive);

        for resource in self.created_resources.drain(..) {
            match resource {
                TestResource::Issue { number } => {
                    let _ = runner.run(&["issue", "close", &number.to_string()]).await;
                }
                TestResource::Label { name } => {
                    let _ = runner.run(&["label", "delete", "--name", &name]).await;
                }
                TestResource::Pr { number } => {
                    let _ = runner.run(&["pr", "close", &number.to_string()]).await;
                }
                // ... 其他资源类型
            }
        }
        Ok(())
    }
}

impl Drop for TestFixture {
    fn drop(&mut self) {
        // 即使测试失败也尝试清理
        if !self.created_resources.is_empty() {
            tracing::warn!("TestFixture dropped with {} resources not cleaned up",
                          self.created_resources.len());
            // 注意：这里不能调用 async cleanup，只能记录警告
            // 实际清理应该在测试的 teardown 阶段完成
        }
    }
}
```

### 4.3 环境变量配置

```bash
# .env.example（不提交，仅本地开发）
E2E_TEST_REPO=byx-darwin/e2e-test-repo
E2E_GITHUB_TOKEN=ghp_xxxx
E2E_GITCODE_TOKEN=xxxx
E2E_GITLAB_TOKEN=xxxx

# CI 中通过 GitHub Secrets 注入
```

### 4.4 测试仓库要求

**专用测试仓库 `byx-darwin/e2e-test-repo` 需要：**
- ✅ 空仓库或仅有 README
- ✅ 启用 Issues、PR、Releases 功能
- ✅ 测试令牌有完整权限（repo, issue, pr 等）
- ✅ 定期清理（可以每周手动或通过脚本）

---

## 5. CI 集成

### 5.1 GitHub Actions Workflow

```yaml
# .github/workflows/e2e-tests.yml

name: E2E Tests

on:
  # 触发条件
  push:
    branches: [main]
  pull_request:
    branches: [main]
    paths:
      - 'crates/**'
      - 'apps/**'
      - '.github/workflows/e2e-tests.yml'
  # 定时运行（每天凌晨 2 点）
  schedule:
    - cron: '0 2 * * *'
  # 手动触发
  workflow_dispatch:

concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true

permissions:
  contents: read

jobs:
  # 阶段 1：GitHub 平台测试（Phase 1）
  e2e-github:
    name: E2E Tests (GitHub)
    runs-on: ubuntu-latest
    timeout-minutes: 30

    steps:
      - uses: actions/checkout@v4

      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2

      - name: Install cargo-nextest
        uses: taiki-e/install-action@cargo-nextest

      - name: Build release binary
        run: cargo build --release

      - name: Add to PATH
        run: echo "${{ github.workspace }}/target/release" >> $GITHUB_PATH

      - name: Run E2E tests
        env:
          E2E_TEST_REPO: ${{ secrets.E2E_TEST_REPO }}
          E2E_GITHUB_TOKEN: ${{ secrets.E2E_GITHUB_TOKEN }}
          RUST_LOG: info
        run: |
          cargo nextest run -p e2e-github --all-features

      - name: Upload test results
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: e2e-results-github
          path: target/nextest/
          retention-days: 7

  # 阶段 2：GitCode 平台测试（Phase 2）
  e2e-gitcode:
    name: E2E Tests (GitCode)
    runs-on: ubuntu-latest
    timeout-minutes: 30
    # 暂时禁用，Phase 2 启用
    if: false

    steps:
      # ... 类似 e2e-github

  # 阶段 3：GitLab 平台测试（Phase 3）
  e2e-gitlab:
    name: E2E Tests (GitLab)
    runs-on: ubuntu-latest
    timeout-minutes: 30
    # 暂时禁用，Phase 3 启用
    if: false

    steps:
      # ... 类似 e2e-github

  # 测试报告汇总
  e2e-report:
    name: E2E Test Report
    runs-on: ubuntu-latest
    needs: [e2e-github, e2e-gitcode, e2e-gitlab]
    if: always()

    steps:
      - name: Download all artifacts
        uses: actions/download-artifact@v4
        with:
          pattern: e2e-results-*
          path: results/

      - name: Generate test report
        run: |
          echo "## E2E Test Report" > report.md
          # ... 生成报告逻辑
```

### 5.2 GitHub Secrets 配置

| Secret 名称 | 描述 | 示例值 |
|------------|------|--------|
| `E2E_TEST_REPO` | GitHub 测试仓库 | `byx-darwin/e2e-test-repo` |
| `E2E_GITHUB_TOKEN` | GitHub 访问令牌 | `ghp_xxxx` |
| `E2E_GITCODE_TEST_REPO` | GitCode 测试仓库 | `byx-darwin/e2e-test-repo` |
| `E2E_GITCODE_TOKEN` | GitCode 访问令牌 | `xxxx` |
| `E2E_GITLAB_TEST_REPO` | GitLab 测试仓库 | `xyun.git.nyuncloud.com/...` |
| `E2E_GITLAB_TOKEN` | GitLab 访问令牌 | `glpat-xxxx` |

### 5.3 CI 执行流程

```
┌─────────────────────────────────────────────────┐
│              PR 推送到 main                      │
└────────────────┬────────────────────────────────┘
                 │
                 ▼
        ┌────────────────┐
        │ 检查 paths 过滤 │  ← 只有关键文件变更才触发
        └────────┬───────┘
                 │
        ┌────────▼────────┐
        │ 并行运行 E2E    │
        │ • e2e-github    │
        │ • e2e-gitcode   │  ← Phase 2 启用
        │ • e2e-gitlab    │  ← Phase 3 启用
        └────────┬────────┘
                 │
        ┌────────▼────────┐
        │ 上传测试结果    │  ← artifacts 保留 7 天
        └────────┬────────┘
                 │
        ┌────────▼────────┐
        │ 生成测试报告    │
        └─────────────────┘
```

---

## 6. 分阶段交付计划

### 6.1 Phase 1：GitHub 平台（3-5 天）

**交付物**：
- ✅ `crates/e2e-core/` — 共享测试工具
- ✅ `crates/e2e-github/` — GitHub 平台 E2E 测试
- ✅ `.github/workflows/e2e-tests.yml` — CI workflow
- ✅ 专用测试仓库 `byx-darwin/e2e-test-repo`

**测试覆盖**：

| 命令类别 | 命令 | 交互模式 | 非交互模式 | 优先级 |
|---------|------|---------|-----------|-------|
| Auth | `auth status` | ✅ | ✅ | P0 |
| Auth | `auth token` | ✅ | ✅ | P0 |
| Issue | `issue list` | ✅ | ✅ | P1 |
| Issue | `issue view` | ✅ | ✅ | P1 |
| Issue | `issue create` | ✅ | ✅ | P1 |
| Issue | `issue close` | ✅ | ✅ | P0 |
| Issue | `issue reopen` | ✅ | ✅ | P0 |
| PR | `pr list` | ✅ | ✅ | P1 |
| PR | `pr view` | ✅ | ✅ | P1 |
| PR | `pr merge` | ✅ | ✅ | P0 |
| Label | `label list` | ✅ | ✅ | P2 |
| Label | `label create` | ✅ | ✅ | P2 |
| Label | `label delete` | ✅ | ✅ | P2 |

**验收标准**：
- ✅ 所有 P0 测试通过（auth, issue close/reopen, pr merge）
- ✅ CI workflow 正常运行
- ✅ 测试后资源自动清理
- ✅ 测试报告自动生成

### 6.2 Phase 2：GitCode 平台（2-3 天）

**交付物**：
- ✅ `crates/e2e-gitcode/` — GitCode 平台 E2E 测试
- ✅ 扩展 CI workflow（启用 `e2e-gitcode` job）
- ✅ 专用测试仓库 `byx-darwin/e2e-test-repo`（GitCode）

**测试覆盖**：
- 与 Phase 1 相同的命令集合
- 重点关注 Issue #70 中暴露的 `pr merge --yes` 问题

**验收标准**：
- ✅ 所有 P0 测试通过
- ✅ `pr merge` 非交互模式正常工作
- ✅ CI workflow 正常运行

### 6.3 Phase 3：GitLab 平台（2-3 天）

**前置条件**：
- 需要先解决 GitLab 认证问题（全量测试报告中 GitLab 通过率 0%）

**交付物**：
- ✅ `crates/e2e-gitlab/` — GitLab 平台 E2E 测试
- ✅ 扩展 CI workflow（启用 `e2e-gitlab` job）
- ✅ 专用测试仓库（GitLab）

**测试覆盖**：
- 与 Phase 1 相同的命令集合
- 重点关注 `glab` CLI 的兼容性问题

**验收标准**：
- ✅ 所有 P0 测试通过
- ✅ `auth status` 和 `auth token` 正常工作
- ✅ CI workflow 正常运行

---

## 7. 风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|---------|
| GitLab 认证问题未解决 | Phase 3 阻塞 | 先调查 `glab` CLI 认证机制，必要时单独创建 Issue |
| 测试令牌权限不足 | 无法创建/删除资源 | 提前验证令牌权限，准备多个备选令牌 |
| 测试仓库被污染 | 测试结果不准确 | 定期清理脚本，测试前检查仓库状态 |
| `portable-pty` 跨平台兼容性问题 | Windows/macOS 测试失败 | 提前在三个平台验证，准备降级方案（纯 stdin 重定向） |

---

## 8. 后续优化（不在本次范围内）

- 🔮 测试覆盖率报告集成（codecov 或类似工具）
- 🔮 测试重试机制（网络超时自动重试）
- 🔮 测试并行化（同一平台内多个测试并行）
- 🔮 测试结果可视化 Dashboard

---

## 9. 验收标准

### 9.1 功能验收

- ✅ 所有 P0 测试通过（auth, issue close/reopen, pr merge）
- ✅ CI workflow 正常运行
- ✅ 测试后资源自动清理
- ✅ 测试报告自动生成

### 9.2 代码质量

- ✅ 通过 `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic`
- ✅ 通过 `cargo +nightly fmt -- --check`
- ✅ 所有公共 API 有文档注释
- ✅ 测试覆盖率 > 80%

### 9.3 文档

- ✅ 设计文档已审查并批准
- ✅ 实现计划已创建
- ✅ 测试仓库已配置

---

## 10. 附录

### 10.1 相关文档

- Issue #70: GitCode `pr merge` 非交互式模式失败
- 全量测试报告: `docs/test-report-gitflow-cli-full-test-2026-07-08.md`

### 10.2 技术栈

- Rust 2024 edition
- `cargo nextest` 测试运行器
- `portable-pty` TTY 控制
- `assert_cmd` + `predicates` 断言
- GitHub Actions CI/CD

---

**文档版本**: 1.0
**最后更新**: 2026-07-09
**作者**: Claude Code (gitflow-workflow)
