# Design: `e2e-gitlab` / `e2e-gitcode` 覆盖对齐 `e2e-github`

- **Issue**: [#291](https://github.com/byx-darwin/gitflow-cli/issues/291)（源自 [#284](https://github.com/byx-darwin/gitflow-cli/issues/284) 多角色评估 v2 P2）
- **Workflow**: `wf-2026-09-03-001`
- **Date**: 2026-09-03
- **Mode**: full

## Context

`e2e-github` 目前有 4 个测试文件（`auth`/`issue`/`noauth`/`pr`，共 246 行），`e2e-core` 提供
`TtyRunner`/`TestConfig`/`TestFixture` 共享 harness（357 行）。GitLab、GitCode 两个平台没有对应
的 crate，三平台可信度不对等——这是当前最容易被用户踩中的空洞。

`e2e-github` 之所以能直接对当前仓库跑 `issue list --platform github`，是因为 CI checkout 的就是
`gitflow-cli` 自身（origin 天然指向 GitHub），属于"自举（dogfooding）"。GitLab、GitCode 没有这个
天然条件：

1. 本仓库的 `git remote origin` 永远指向 GitHub。
2. `gf` 的仓库解析（`apps/cli/src/main.rs::resolve_platform`）无条件从 `git remote get-url origin`
   提取 `owner/repo`，且没有全局 `--repo` flag——只有 `issue create`/`pr create` 这类命令有局部
   `--repo` 覆盖，`list`/`close`/`reopen` 都没有。
3. 因此要让 `issue list --platform gitlab` 查到正确仓库，必须让执行时的工作目录本身的
   `git remote origin` 指向目标 GitLab/GitCode 仓库。

同时，GitLab、GitCode 的凭据环境变量名与 GitHub 不同：

| 平台 | provider 实际读取的 env var | 来源 |
|------|------------------------------|------|
| GitHub | `GH_TOKEN`（`e2e-core::TestConfig::gh_env()` 已支持） | `crates/e2e-core/src/config.rs` |
| GitLab | `GL_TOKEN` | `crates/gitlab/src/auth.rs:158,205,218` |
| GitCode | `GITCODE_TOKEN` | `crates/gitcode/src/auth.rs:217,248` |

`TestConfig` 已经预留了 `gitlab_token`/`gitcode_token` 字段（从 `E2E_GITLAB_TOKEN`/
`E2E_GITCODE_TOKEN` 读取）但从未被使用——本次工作正是把它们接上。

**确认事项（用户已明确）**：当前没有可用于 dogfooding 的真实 GitLab.com / GitCode.com 测试仓库，
也没有配置对应的 CI Secrets。本次范围是把 crate/harness 基础设施完整落地，遵循
`E2E_GITHUB_TOKEN` 缺失时自动 skip 的既有约定；真实仓库与 Secrets 留给后续 Issue。

## Goal

新增 `crates/e2e-gitlab`、`crates/e2e-gitcode`，测试覆盖深度对齐 `e2e-github`
（`auth`/`issue`/`noauth`/`pr` 四类只读实测场景），复用并按需扩展 `e2e-core` 的
harness/config/fixture 机制，CI 的 `e2e-tests.yml` 纳入两个新 crate 的定时回归。

## Non-Goals

- 不新增或修改任何 GitHub Secrets（`E2E_GITLAB_TOKEN`/`E2E_GITCODE_TOKEN`/
  `E2E_TEST_REPO_GITLAB`/`E2E_TEST_REPO_GITCODE` 均先留空，由后续基础设施 Issue 配置）
- 不改动生产 CLI 代码（`apps/cli`、`crates/gitlab`、`crates/gitcode` 均不动）——本次是纯测试侧方案
- 不改变 `e2e-core` 现有对 GitHub 路径的行为（`mode()`/`gh_env()`/`TestConfig::from_env*`
  向后兼容，`e2e-github` 现有测试不受影响）
- 不解决"真实凭据下、真实仓库里跑通"的端到端验证——这依赖尚不存在的基础设施，只能在
  本地/CI 上验证"无凭据/无仓库时优雅 skip"这一路径

## Approach

### 1. `e2e-core` 扩展（新增能力，不破坏现有 API）

**`TestConfig`**（`crates/e2e-core/src/config.rs`）新增：

- 字段：`gitlab_test_repo: Option<String>`（读 `E2E_TEST_REPO_GITLAB`）、
  `gitcode_test_repo: Option<String>`（读 `E2E_TEST_REPO_GITCODE`）
- `gl_env(&self) -> Vec<(String, String)>`：有 `gitlab_token` 则返回
  `[("GL_TOKEN", token)]`，否则空——对齐 `gh_env()` 的结构与命名风格
- `gitcode_env(&self) -> Vec<(String, String)>`：同上，映射到 `GITCODE_TOKEN`
- `has_gitlab_auth(&self) -> bool` / `has_gitcode_auth(&self) -> bool`：对齐
  `has_github_auth()`
- `gitlab_mode(&self) -> TestMode` / `gitcode_mode(&self) -> TestMode`：对齐 `mode()`，
  分别基于 `has_gitlab_auth()`/`has_gitcode_auth()` 派生
- `from_env()`/`from_env_lenient()` 同步读取上述两个新环境变量（可选，缺省为 `None`）

**`TtyRunner`**（`crates/e2e-core/src/tty.rs`）新增：

- `pub fn dir(&mut self, path: impl Into<PathBuf>) -> &mut Self`：覆盖执行时的工作目录
  （当前 `working_dir` 只能在 `new()` 时取进程自身 cwd，无 setter）

**新增辅助函数** `crates/e2e-core/src/fixture.rs`（或新增 `scratch.rs` 模块，视实现时体积决定）：

- `pub fn scratch_repo_dir(remote_url: &str) -> Result<tempfile::TempDir, FixtureError>`：
  在系统临时目录下 `git init` 一个空仓库并 `git remote add origin <remote_url>`，返回
  `TempDir`（析构时自动清理）。供 `issue.rs`/`pr.rs` 测试构造"remote 指向目标平台仓库"的
  执行环境。`e2e-gitlab`/`e2e-gitcode` 通过 `dev-dependency` 引入 `e2e-core`，但它们调用的
  是 `e2e-core` 的公开 API（而非 `#[cfg(test)]` 内部代码），因此 `tempfile` 需加到
  `e2e-core` 自身的 `[dependencies]`（而非 `[dev-dependencies]`）——workspace 已有
  `tempfile = "3"`，直接复用同一版本声明即可，无需引入新版本。

### 2. `crates/e2e-gitlab`、`crates/e2e-gitcode`

`Cargo.toml` 与 `e2e-github` 同构：

```toml
[package]
name = "e2e-gitlab"  # / "e2e-gitcode"
version.workspace = true
edition.workspace = true
publish = false
release = false
license = "MIT"

[dev-dependencies]
e2e-core = { path = "../e2e-core" }
serde_json = "1.0"
tokio = { version = "1", features = ["full", "test-util"] }

[lints]
workspace = true
```

两者均加入根 `Cargo.toml` 的 `[workspace.members]`。

**测试文件**（每个 crate 4 个，命名与结构对齐 `e2e-github`）：

- **`auth.rs`** — `test_should_report_logged_in_with_real_credentials`：用
  `gitlab_mode()`/`gitcode_mode()` 判断 `TestMode::Authenticated`，非认证态 `eprintln!` +
  `return`（skip）。沿用 `Interactive`/`NonInteractive` 双模式循环，注入 `gl_env()`/
  `gitcode_env()`，断言 `auth status --platform gitlab/gitcode --output json` 返回
  `success: true` 且 `data.loggedIn: true`。

- **`noauth.rs`** — 未认证错误路径，无需凭据，任何环境可跑：
  - `test_should_fail_with_login_guidance_when_status_checked_unauthenticated`
  - `test_should_fail_with_login_guidance_when_listing_issues_unauthenticated`
  - 用 `env_remove("GL_TOKEN")` / `env_remove("GITCODE_TOKEN")` 清除继承的 token。
    **与 `e2e-github` 的差异**：GitHub 版本额外用 `GH_CONFIG_DIR` 指向空目录屏蔽 `gh` 的
    `hosts.yml` 状态；GitLab/GitCode 的 provider 是纯 env-var 读取（无本地配置文件状态，
    见 `crates/gitlab/src/auth.rs`、`crates/gitcode/src/auth.rs`），因此 `env_remove` 单独
    即可保证确定性，无需额外的空目录隔离。
  - **关键前提（Phase 2 规划阶段发现并已确认方案）**：`GitLabAuthProvider::status()` /
    `GitCodeAuthProvider::status()` 始终真实 spawn 外部 `glab`/`gc` 二进制（不像
    `is_authenticated()` 那样有 `GL_TOKEN`/`GITCODE_TOKEN` 的环境变量短路）。GitHub Actions
    `ubuntu-latest` 预装 `gh`，但不预装 `glab`/`gc`，二进制缺失会报
    `Failed to spawn glab auth status: No such file or directory`，而非"未登录+登录指引"，
    使断言失真。因此 CI job 需要新增安装步骤（见下方"CI"小节），本地开发者运行这两个 crate
    的测试前也需要自行安装 `glab`/`gc`。
  - 断言 stderr/stdout 组合包含登录指引文案（具体文案以两个 provider 的实际错误输出为准，
    实现阶段核对 `crates/gitlab/src/auth.rs`、`crates/gitcode/src/auth.rs` 的错误信息，
    该文案由 `glab`/`gc` 二进制自身产生，`gf` 侧仅透传/包装 stdout+stderr）。

- **`issue.rs`** — `test_should_list_open_issues_with_valid_schema`：
  - `gitlab_mode()`/`gitcode_mode()` 非 `Authenticated`，或对应 `*_test_repo` 为空 → skip
  - 用 `scratch_repo_dir(&format!("https://gitlab.com/{repo}.git"))`（或 gitcode 对应 URL 模板）
    构造工作目录，`TtyRunner::dir()` 指向该目录
  - 执行 `issue list --platform gitlab/gitcode --state open --output json`，schema 断言对齐
    `e2e-github`（`success: true`，`data` 为数组，每项 `number`/`title` 类型校验）

- **`pr.rs`** — 同上模式，查询 `closed` 状态。**与 `e2e-github` 的差异**：`e2e-github` 断言
  `!items.is_empty()`（利用本仓库自身已有已合并 PR 的确定性）；GitLab/GitCode 测试仓库在
  基础设施到位前身份未知（可能是全新空仓库），因此本次断言放宽为"若 `items` 非空则逐项校验
  schema"，不强制非空。后续基础设施 Issue 若确认测试仓库有稳定的历史 PR/MR，可再收紧为强
  断言。

### 3. CI（`.github/workflows/e2e-tests.yml`）

新增 `e2e-gitlab`、`e2e-gitcode` 两个 job，结构与现有 `e2e-github` job 一致（checkout → 装
toolchain → `cargo-nextest` → build release → 加 PATH → 跑测试 → 汇报运行模式 → 上传结果），
差异仅在于：

- **新增二进制安装步骤**（`e2e-github` 无需此步，因 `ubuntu-latest` 预装 `gh`）：
  - `e2e-gitlab` job：`go install gitlab.com/gitlab-org/cli/cmd/glab@latest`，并将 Go bin 目录
    （`$(go env GOPATH)/bin`）加入 `$GITHUB_PATH`（`ubuntu-latest` 镜像预装 Go 工具链，无需
    额外安装 Go 本身）
  - `e2e-gitcode` job：`pip install gitcode-cli`（对齐 `docs/cli-compatibility.md` 记录的官方
    安装方式，`ubuntu-latest` 预装 Python3/pip）
  - 不涉及任何 Secrets，纯公开工具安装；这一步使 `noauth.rs` 能验证真实的"未登录 + 登录指引"
    路径，而非"二进制缺失"报错
- 跑的 crate：`cargo nextest run -p e2e-core -p e2e-gitlab --all-features`（GitCode 同理）
- 注入的 env：GitLab job 用 `E2E_GITLAB_TOKEN`/`E2E_TEST_REPO_GITLAB`；GitCode job 用
  `E2E_GITCODE_TOKEN`/`E2E_TEST_REPO_GITCODE`（均为 `secrets.*`，当前未配置，值为空，测试据此
  优雅 skip）
- `Report run mode` step 按各自 token 是否非空汇报 authenticated/unauthenticated
- `Upload test results` 的 artifact name 改为 `e2e-results-gitlab`/`e2e-results-gitcode`

`schedule`（每周一 02:00 UTC）、`workflow_dispatch`、`push`/`pull_request` 的 `paths` 触发条件
不变——三个 job 共用同一批触发器。

## Data Flow

```
CI job (e2e-gitlab)
  └─ cargo nextest run -p e2e-core -p e2e-gitlab
       ├─ auth.rs   → TestConfig::gitlab_mode() → [skip | 真实 gf auth status 调用]
       ├─ noauth.rs → env_remove(GL_TOKEN) → gf auth status（预期失败+登录指引）
       ├─ issue.rs  → gitlab_mode()+test_repo 双重检查 → [skip | scratch_repo_dir + gf issue list]
       └─ pr.rs     → 同 issue.rs，查询 closed 状态
```

`e2e-gitcode` 数据流完全对称，仅 provider/env var 名称不同。

## Testing

- `cargo nextest run -p e2e-core -p e2e-gitlab -p e2e-gitcode --all-features`（本地，无凭据环境下
  应全部 pass：`auth`/`issue`/`pr` skip，`noauth` 正常执行并通过）
- `cargo test -p e2e-core`：验证新增的 `gl_env()`/`gitcode_env()`/`gitlab_mode()`/
  `gitcode_mode()`/`scratch_repo_dir()`/`TtyRunner::dir()` 单元测试（每个新增函数至少一条
  成功路径 + 一条边界路径，遵循仓库 TDD 约定）
- `cargo nextest run -p e2e-core -p e2e-github --all-features`：回归验证现有 GitHub 路径未被破坏
- `make lint` / `cargo clippy --all-targets --all-features -p e2e-core -p e2e-gitlab -p e2e-gitcode
  -- -D warnings -W clippy::pedantic`
- CI 层面：新增两个 job 首次跑绿即视为通过（预期均为 unauthenticated skip 模式）

## Open Questions / Follow-ups（记入后续 Issue，不在本次范围）

1. 配置真实 GitLab.com / GitCode.com 测试仓库，并在仓库 Secrets 中配置
   `E2E_GITLAB_TOKEN`/`E2E_GITCODE_TOKEN`/`E2E_TEST_REPO_GITLAB`/`E2E_TEST_REPO_GITCODE`
2. 待真实仓库确认有稳定历史 MR/PR 后，收紧 `pr.rs` 的非空断言
3. ~~`noauth.rs` 的登录指引文案断言需要在实现阶段对照 provider 实际错误信息二次确认~~
   （已在规划阶段确认：CI 新增 `glab`/`gc` 二进制安装步骤后，`noauth.rs` 走真实"未登录"路径，
   文案以 `crates/gitlab/src/auth.rs`/`crates/gitcode/src/auth.rs` 解析的 provider 输出为准，
   实现阶段仍需核对具体断言字符串）
4. 本地开发者若要运行 `e2e-gitlab`/`e2e-gitcode` 测试，需自行安装 `glab`/`gc`
   CLI（CI 已自动安装，仅影响本地手动运行场景）
