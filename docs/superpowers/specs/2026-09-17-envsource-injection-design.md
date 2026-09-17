# EnvSource 注入消除测试级进程环境竞态 — 设计

- **Issue**: [#359](https://github.com/byx-darwin/gitflow-cli/issues/359)
- **Workflow**: `wf-2026-09-17-002`（standard 模式）
- **日期**: 2026-09-17
- **状态**: 待实施

## 背景

`crates/gitlab/src/auth.rs` 的两个单元测试用 `temp_env::with_var` 设置 `GL_TOKEN`。
`temp_env` 修改的是**进程级**环境变量，而 `cargo test` 在单进程内多线程并行执行同
crate 的测试，于是并发运行的其他测试会读到这个变量：`GitLabAuthProvider::token()`
的 env 短路分支先于注入的 `MockCommandRunner` 命中，断言随之失败。

本地复现（2026-09-17，本仓库 `dev` @ `78a3ae4`）：

```
$ cargo test -p gitflow-gitlab --lib
thread 'auth::tests::test_should_error_when_stdout_has_no_token_line' panicked at
  crates/gitlab/src/auth.rs:436:9: assertion failed: result.is_err()
test result: FAILED. 259 passed; 1 failed
```

失败的**具体测试与条数随线程调度漂移**（Issue 记录的是
`test_should_extract_token_from_auth_status_show_token` 与 3~4 条），这是竞态的典型特征。
`cargo nextest` 因每个测试独立进程而不受影响，所以 `make test` 绿、`cargo test` 红。

### 影响面测绘

| crate | 生产代码读取 env 的位置 | `temp_env` 测试 | 今日是否撞车 |
|---|---|---|---|
| gitlab | `token()`:158（`GL_TOKEN`）、`token()`:164（`GITLAB_HOST`）、`is_authenticated()`:205、`check_status()`:218 | 2 处（:347, :356） | **是** |
| github | `is_authenticated()`:225、`check_status()`:238 | 2 处（:466, :475） | 否 |
| gitcode | `is_authenticated()`:217、`check_status()`:248 | 2 处（:397, :406） | 否 |

github 与 gitcode 的 `token()` 直接调用 `gh auth token` / `gc auth token`，不读 env，
因此今天不会与并行测试撞车 —— 但污染源（进程级 env 写入）同构，属于同源风险。

### 连带后果

- `cargo test`（CLAUDE.md 与多数 Rust 工作流的默认命令）恒红
- `cargo llvm-cov` 内部调用 `cargo test` 而非 nextest，覆盖率闸门被此竞态挡住，
  必须加 `--ignore-run-fail` 才能出数。经核查，该参数仅出现在上一轮 workflow 的
  计划/设计文档中，**Makefile 与 CI 均未硬编码**，即闸门命令本身是干净的、只是跑不通
- 次生缺陷 1：gitlab 的 8 处 `token()` 测试隐式依赖宿主环境未设置 `GL_TOKEN`。
  任何在 shell 中 `export GL_TOKEN=...` 的开发者，即便没有并行竞态也必红
- 次生缺陷 2：`token()`:164 读取 `GITLAB_HOST` 并据此追加 `--hostname` 参数，而
  `test_should_extract_token_from_auth_status_show_token` 断言记录到的参数恰为
  `["auth", "status", "--show-token"]`。任何导出了 `GITLAB_HOST` 的开发者会让该测试失败。
  该读取在 Phase 2 计划阶段才被发现，初版设计遗漏

## 目标与非目标

**目标**

1. 消除测试对进程级环境变量的写入，从源头而非串行化层面解决竞态
2. 使受影响测试对宿主环境免疫
3. 三个 adapter crate 同构处理，消除同源风险
4. 不引入新的运行时依赖，不破坏现有调用方

**非目标**

- 不改变任何 env 变量的**语义**（`GL_TOKEN` / `GH_TOKEN` / `GITCODE_TOKEN` 的优先级
  与判定逻辑保持原样）
- 不重构 `AuthProvider` / `AuthChecker` trait 本身
- 不引入 `--test-threads=1` 或 `serial_test` 等串行化手段

## 方案选型

| 方案 | 做法 | 取舍 |
|---|---|---|
| A — 抽出纯函数 | 把 env 判定逻辑剥成私有纯函数，测试直接喂值 | 最小改动、零 API 变更；但 env 分支本身退化为未覆盖的一行 wrapper |
| B — `serial_test` | 给 env 测试打 `#[serial]` | **陷阱**：`#[serial]` 只在标记者之间互斥，必须同时标记所有会读到该 env 的测试，漏一个即静默复发，新增测试同样会无声重开缺口。属掩盖而非解决 |
| **C — EnvSource 注入（选定）** | 为 provider 增加 env 来源泛型参数，测试注入假 env | 最彻底：进程级 env 读取被收敛到唯一实现；代价是三个 crate 的 public 类型签名新增泛型参数 |

选定 **C**。B 被 Issue 验收标准第 3 条实质排除（「修复方式不依赖 `--test-threads=1`，
那是掩盖而非解决」的同一理由适用于任何串行化手段）；A 虽最省事，但留下未覆盖分支，
且无法解决「开发者 shell 中已导出 token 则测试必红」这一次生缺陷。

## 设计

### 1. 抽象归属

新增 `EnvSource` 于 `gitflow-cli-adapter-utils`，与既有的 `CommandRunner` 并列 ——
该 crate 的存在意义正是承载三个 adapter 共享的进程交互抽象。

```rust
/// Read-only access to process environment variables.
///
/// Abstracts environment lookups so tests can inject deterministic values
/// without mutating process-global state, which races with parallel tests.
pub trait EnvSource: std::fmt::Debug + Send + Sync {
    /// Return the value of `key`, or `None` if it is unset or not valid UTF-8.
    fn var(&self, key: &str) -> Option<String>;
}

/// Default implementation reading the real process environment.
#[derive(Debug, Clone, Default)]
pub struct RealEnv;

impl EnvSource for RealEnv {
    fn var(&self, key: &str) -> Option<String> {
        std::env::var(key).ok()
    }
}
```

同步 trait：`AuthChecker::is_authenticated` / `check_status` 本身是同步的，
env 查询也无需异步。`std::env::var` 不在 `clippy.toml` 的 disallowed 列表中。

### 2. 类型签名演化

三个 provider 同构改造（以 gitlab 为例）：

```rust
pub struct GitLabAuthProvider<R: CommandRunner = RealCommandRunner, E: EnvSource = RealEnv> {
    runner: R,
    env: E,
}
```

向后兼容由**带默认值的类型参数**保证：

| 现有写法 | 变更后解析为 | 破坏 |
|---|---|---|
| `GitLabAuthProvider::new()` | `<RealCommandRunner, RealEnv>` | 否 |
| `GitLabAuthProvider::with_session(&s)` | 同上 | 否 |
| `GitLabAuthProvider::with_runner(mock)` | `<MockCommandRunner, RealEnv>` | 否 |
| `GitLabAuthProvider<MockCommandRunner>`（显式标注） | `<MockCommandRunner, RealEnv>` | 否 |

`apps/cli` 共 9 处构造点（`commands/auth.rs:59-61`、`commands/doctor.rs:164-166`、
`commands/prerequisites.rs:301-303`）全部走 `::new()` 并 box 成
`dyn AuthProvider` / `dyn AuthChecker`，对本次变更完全透明。

新增构造函数：

```rust
// 既有的 with_runner 收窄到 RealEnv 特化块，签名与返回类型对调用方不变
impl<R: CommandRunner> GitLabAuthProvider<R, RealEnv> {
    #[must_use]
    pub fn with_runner(runner: R) -> Self {
        Self { runner, env: RealEnv }
    }
}

impl<R: CommandRunner, E: EnvSource> GitLabAuthProvider<R, E> {
    /// Create a provider with both a custom runner and a custom environment source.
    #[must_use]
    pub fn with_runner_and_env(runner: R, env: E) -> Self {
        Self { runner, env }
    }
}
```

两个 impl 块可共存：方法名不同，且前者是后者的特化形式。

两个 trait impl 的泛型头相应扩展：

```rust
impl<R: CommandRunner + 'static, E: EnvSource + 'static> AuthProvider for GitLabAuthProvider<R, E>
impl<R: CommandRunner, E: EnvSource>                     AuthChecker  for GitLabAuthProvider<R, E>
```

生产代码中 8 处 `std::env::var(...)`（gitlab 4、github 2、gitcode 2）改为
`self.env.var(...)`。**进程级 env 的读取从此只存在于 `RealEnv` 一处。**

gitlab 的第 4 处是 `token()`:164 的 `GITLAB_HOST`（拼成 `--hostname` 参数），
在初版设计中被遗漏 —— 见下节「次生缺陷」第 2 条。

### 3. 测试替身

`MockEnv` 置于同一 crate，由 feature 门控，不进入生产 API 表面：

```toml
# crates/cli-adapter-utils/Cargo.toml
[features]
test-util = []
```

```toml
# crates/{gitlab,github,gitcode}/Cargo.toml
[dev-dependencies]
gitflow-cli-adapter-utils = { workspace = true, features = ["test-util"] }
```

工作区使用 `resolver = "3"`，dev-dependencies 的 feature 不会渗入 `cargo build`
对 lib 的构建，`test-util` 仅在测试 target 打开。

备选是每个 crate 各写一份 `#[cfg(test)] struct MockEnv`（照搬现有 `MockCommandRunner`
的 per-crate `#[cfg(test)]` 惯例）。此处选共享版：三份实现将完全字面相同，
而 adapter-utils 的职责正是消除这类重复。

### 4. 测试改造范围

| crate | 改造 |
|---|---|
| gitlab | 2 个 env 测试 → `with_runner_and_env(mock_runner, MockEnv::with("GL_TOKEN", "test_token"))`；**8 处 `token()` 调用点所在测试**（`auth.rs` 的 :421, :434, :503, :515, :528, :551, :565, :629）→ 注入 `MockEnv::empty()` |
| github | 2 个 env 测试 → 注入 `MockEnv::with("GH_TOKEN", "test_token")` |
| gitcode | 2 个 env 测试 → 注入 `MockEnv::with("GITCODE_TOKEN", "test_token")` |

gitlab 那 8 处注入空 env 一并修掉上文的次生缺陷，使其对宿主环境免疫。

### 5. 依赖清理

`temp-env` 的 5 处声明全部移除：

- `Cargo.toml:42`（workspace 依赖定义）
- `crates/gitlab/Cargo.toml:29`、`crates/github/Cargo.toml:29`、`crates/gitcode/Cargo.toml:30`
- `apps/cli/Cargo.toml:64` —— 该处声明后 `apps/cli` 内零使用，是既有死依赖

## 错误处理

不引入新的错误路径。`EnvSource::var` 返回 `Option<String>`，把
`std::env::var` 的 `NotPresent` 与 `NotUnicode` 两种错误一并折叠为 `None` ——
这与被替换的原代码行为一致（原代码即为 `std::env::var(...).ok()` 与
`std::env::var(...).is_ok()`）。

## 测试策略

遵循 TDD（RED → GREEN → REFACTOR）。

**RED**：首个测试写为「`MockCommandRunner` 返回 `glpat-abcdef`、`MockEnv::empty()`，
断言 `token()` 返回 `glpat-abcdef`」。由于 `with_runner_and_env` 与 `EnvSource`
尚不存在，它以**编译失败**形式 RED —— 这正是待引入 API 的第一个消费者。

**GREEN**：按上文顺序落地 `EnvSource` / `RealEnv` / `MockEnv` → 三个 provider 的
字段与泛型 → 生产代码 7 处 env 读取改写 → 测试改造。

**REFACTOR**：移除 `temp-env` 依赖，`make lint`。

新增的公开项（`EnvSource`、`RealEnv`）需满足工作区 `missing_docs` 与
`missing_debug_implementations` 要求。

## 风险

| 风险 | 评估 |
|---|---|
| `gitflow-cli-adapter-utils` 已发布至 crates.io，新增 public trait | 纯增量（无移除、无签名变更），属 minor 级兼容变更 |
| 三个 adapter crate 结构体新增泛型参数 | 带默认值，对所有现存用法透明；已逐一核对 9 处构造点 |
| `test-util` feature 意外渗入生产构建 | resolver = "3" 下 dev-dependencies 的 feature 不参与 lib 构建；交付前以 `cargo tree -e features` 核验 |

## 验收标准映射

| Issue #359 验收项 | 满足方式 |
|---|---|
| `cargo test -p gitflow-gitlab --lib` 连续 5 次全绿 | 不再有任何测试写入进程 env，竞态源头消失；交付前实测 5 次 |
| `cargo llvm-cov --workspace --fail-under-lines 80` 无需 `--ignore-run-fail` | 同上。另一处噪声（`e2e-gitcode` 的 `gc` 撞名）已由 `78a3ae4` 修复，两处一并清零 |
| 修复方式不依赖 `--test-threads=1` | 未引入任何串行化手段 |
| gitcode / github 同类写法一并评估 | 采纳「改为不依赖进程级 env 的注入方式」，三个 crate 同构处理 |

## 参考

- Issue [#359](https://github.com/byx-darwin/gitflow-cli/issues/359)
- 上游语境：[覆盖率度量口径统一设计](./2026-09-17-coverage-metric-unification-design.md)（本问题在其 Phase 3 覆盖率闸门处复现）
