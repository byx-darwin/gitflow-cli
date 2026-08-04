# 契约测试 + 兼容性矩阵 + 版本护栏 设计文档

> **Issue:** [#95](https://github.com/byx-darwin/gitflow-cli/issues/95) · 阶段一·第 3-4 周
> **状态:** 已批准
> **日期:** 2026-07-31

## 1. 目标

把子进程耦合从负债变成受控边界：

1. **统一错误层** — 消除三平台 crate 重复定义的错误类型，收敛到 core
2. **版本护栏** — 启动时校验底层 CLI 版本，中文友好错误，不泄漏底层 CLI 内部信息
3. **契约测试** — 用真实 CLI 输出夹具锁定 JSON 反序列化契约，上游格式变更立即报红
4. **兼容性矩阵** — 单一 JSON 源生成 Markdown 文档 + 编译时嵌入代码

## 2. 统一错误层（core）

### 2.1 新增类型

在 `crates/core/src/cli_error.rs` 新增 `PlatformCliError`（注意：与 `output.rs` 中的 `CliError` 不同，后者是 JSON 输出信封类型）：

```rust
/// 统一的底层平台 CLI 错误。
///
/// 各平台 crate 的 `parse_*_error()` 函数返回此类型，
/// 替代现有的 `GhError`、`GlabError`、`GitcodeError`。
///
/// 用户可见信息（`user_message`、`hint`）为中文主导；
/// `raw_stderr` 仅用于 debug 日志，不展示给用户。
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct PlatformCliError {
    /// 用户可见的错误消息（中文主导）。
    pub user_message: String,
    /// 底层 CLI 原始 stderr（仅用于 `tracing::debug!`，不展示给用户）。
    pub raw_stderr: String,
    /// 修复建议（中文）。
    pub hint: Option<String>,
    /// 相关文档链接。
    pub doc_link: Option<String>,
    /// 平台错误代码（如 gh 的 `NOT_FOUND`、gitcode 的 `UNAUTHORIZED`）。
    pub code: Option<String>,
    /// 来源平台。
    pub platform: Platform,
}
```

### 2.2 Display 实现

```rust
impl fmt::Display for PlatformCliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.user_message)?;
        if let Some(ref hint) = self.hint {
            write!(f, "\n\n🔧 修复建议：{hint}")?;
        }
        if let Some(ref link) = self.doc_link {
            write!(f, "\n📖 文档：{link}")?;
        }
        Ok(())
    }
}
```

`raw_stderr` 仅出现在 `Debug` 输出中（derive 自动包含），不会通过 `Display` 泄漏给用户。

### 2.3 CoreError 新变体

```rust
pub enum CoreError {
    // ... 现有变体 ...

    /// 底层平台 CLI 执行错误（结构化）。
    #[error(transparent)]
    Cli(#[from] PlatformCliError),
}
```

`#[non_exhaustive]` 保证新增变体不是 breaking change。

### 2.4 各 crate 迁移

| crate | 现有类型 | 迁移方式 |
|-------|---------|---------|
| `gf-github` | `GhError` + `parse_gh_error()` | `parse_gh_error()` 返回 `PlatformCliError`，删除 `GhError` |
| `gf-gitlab` | `GlabError` + `parse_glab_error()` | 同上 |
| `gf-gitcode` | `GitcodeError` + `parse_gitcode_error()` | 同上 |

各 `parse_*_error()` 的解析逻辑不变（JSON → 回退文本），但：
- `user_message` 映射为中文（常见错误码查表 + 通用回退）
- `raw_stderr` 保留完整 stderr 原文
- `platform` 字段设为对应平台

Provider 方法中的调用从：

```rust
let gh_err = parse_gh_error(&output.stderr);
return Err(CoreError::Platform(format!("{gh_err}")));
```

改为：

```rust
return Err(parse_gh_error(&output.stderr).into());
```

### 2.5 中文错误映射

常见错误码中文映射表（各 crate 内部）：

| 错误码/模式 | 中文 user_message |
|------------|------------------|
| `NOT_FOUND` | "资源不存在" |
| `UNAUTHORIZED` / 401 | "认证失败，请重新登录" |
| `FORBIDDEN` / 403 | "权限不足" |
| `Not logged in` (文本匹配) | "未登录" |
| 回退 | "平台 CLI 执行失败" |

`hint` 统一附加对应平台的登录命令提示（如 "运行 `gh auth login` 重新认证"）。

## 3. 版本护栏（prerequisites.rs）

### 3.1 版本要求更新

| 平台 | 二进制 | 当前最低版本 | 新最低版本 |
|------|-------|------------|----------|
| GitHub | `gh` | 2.0.0 | 2.0.0（不变） |
| GitLab | `glab` | 1.30.0 | 1.30.0（不变） |
| GitCode | `gitcode` | 0.5.9 | **0.6.0** |

### 3.2 PrerequisiteError 中文化

所有 `PrerequisiteError` 变体的 `#[error(...)]` 改为中文主导，新增 `doc_link` 字段：

```rust
#[derive(Debug, thiserror::Error)]
pub enum PrerequisiteError {
    #[error(
        "[[PLATFORM]] 未检测到 {binary}。\n\n\
         📦 安装：{install_cmd}\n\
         📖 文档：{doc_link}\n\n\
         其他安装方式：\n{install_hint}"
    )]
    NotFound {
        binary: String,
        platform: String,
        install_hint: String,
        install_url: String,
        install_cmd: String,
        doc_link: String,
    },

    #[error(
        "[[PLATFORM]] {binary} 版本过低：当前 v{found}，需要 v{required}+。\n\n\
         📦 升级：{install_cmd}\n\
         📖 文档：{doc_link}"
    )]
    VersionTooLow {
        binary: String,
        platform: String,
        found: String,
        required: String,
        install_cmd: String,
        doc_link: String,
    },

    #[error(
        "[[PLATFORM]] {binary} 版本信息解析失败。\n\n\
         📦 重新安装：{install_cmd}\n\
         📖 文档：{doc_link}"
    )]
    VersionParseFailed {
        binary: String,
        platform: String,
        install_cmd: String,
        doc_link: String,
    },

    #[error(
        "[[PLATFORM]] {binary} 未认证。\n\n\
         🔍 原因：{reason}\n\
         🔧 修复：运行 `{hint}` 完成登录"
    )]
    NotAuthenticated {
        binary: String,
        platform: String,
        reason: String,
        hint: String,
    },

    #[error("不支持的平台：{platform}。支持的平台：github、gitlab、gitcode")]
    UnsupportedPlatform { platform: String },
}
```

### 3.3 错误泄漏防护

- `NotAuthenticated.reason` 不再直接传递底层 CLI 的原始 stderr，改为使用 `PlatformCliError.user_message`（已中文化、已脱敏）
- `check()` 中的认证检查桥接：`AuthChecker` 返回的错误经 `PlatformCliError` 过滤后再传入 `PrerequisiteError`

### 3.4 CliRequirement 扩展

```rust
pub struct CliRequirement {
    pub binary: &'static str,
    pub min_version: &'static str,
    pub install_url: &'static str,
    pub install_hint: &'static str,
    pub install_cmd: &'static str,
    pub login_cmd: &'static str,
    pub login_with_token: &'static str,
    pub doc_link: &'static str,  // 新增
}
```

## 4. 契约测试基础设施

### 4.1 夹具文件组织

每平台 crate 下 `tests/fixtures/` 目录，命名规则：`{resource}_{action}_{platform}_v{major_version}.json`

```
crates/github/tests/fixtures/
├── pr_list_github_v2.json
├── issue_list_github_v2.json
└── label_list_github_v2.json

crates/gitlab/tests/fixtures/
├── pr_list_gitlab_v1.json
├── issue_list_gitlab_v1.json
└── label_list_gitlab_v1.json

crates/gitcode/tests/fixtures/
├── pr_list_gitcode_v0.6.json      ← 重命名自 pr_list_gitcode_v0.6.1.json
├── issue_list_gitcode_v0.6.json
└── label_list_gitcode_v0.6.json
```

共 9 个夹具文件（1 个已有 + 8 个新增）。

### 4.2 夹具采集

- **GitHub / GitLab：** 从 `gh api` / `glab api` 的真实输出中采集，或使用官方 API 文档中的示例响应
- **GitCode：** 从 `gitcode` CLI 的真实输出中采集（已有 `pr_list` 先例）
- 采集后手动脱敏（替换真实 token、邮箱等）
- 每个夹具文件至少包含 1 条完整记录，覆盖所有已知字段

### 4.3 测试模式

每个平台 crate 新增契约测试模块（`tests/contract_test.rs` 或 `#[cfg(test)] mod contract_tests`）：

```rust
/// 契约测试：验证 gitcode PR list 的 JSON 输出格式与反序列化模型一致。
///
/// 夹具来源：gitcode v0.6.1 真实 CLI 输出。
/// 若上游 CLI 变更输出格式，此测试将失败，提醒更新 serde 模型。
#[tokio::test]
async fn test_contract_pr_list_gitcode_v0_6() {
    let fixture = include_str!("fixtures/pr_list_gitcode_v0.6.json");
    let runner = MockCommandRunner::success(fixture);
    let provider = GitCodePrProvider::with_runner("owner/repo", runner);

    let prs = provider.list(ListPrArgs::default()).await.expect("反序列化失败");

    assert!(!prs.is_empty());
    let pr = &prs[0];
    // 锁定关键字段存在且类型正确
    assert!(pr.number > 0);
    assert!(!pr.title.is_empty());
    assert!(!pr.state.is_empty());
}
```

### 4.4 CI 集成

契约测试是普通 `cargo test` 的一部分，无需额外 CI 配置。`make test` 即覆盖。

## 5. 兼容性矩阵

### 5.1 单一数据源

`docs/compatibility-matrix.json`：

```json
{
  "schema_version": 1,
  "updated_at": "2026-07-31",
  "gitflow_cli_version": "0.9.0",
  "platforms": [
    {
      "name": "GitHub",
      "identifier": "github",
      "cli_binary": "gh",
      "min_version": "2.0.0",
      "tested_versions": ["2.62.0"],
      "install_url": "https://github.com/cli/cli#installation",
      "doc_link": "https://cli.github.com/manual/",
      "features": {
        "issue": true,
        "pr": true,
        "label": true,
        "milestone": true,
        "release": true,
        "pipeline": true,
        "review": true,
        "auth": true
      }
    },
    {
      "name": "GitLab",
      "identifier": "gitlab",
      "cli_binary": "glab",
      "min_version": "1.30.0",
      "tested_versions": ["1.46.1"],
      "install_url": "https://gitlab.com/gitlab-org/cli#installation",
      "doc_link": "https://gitlab.com/gitlab-org/cli/-/blob/main/docs/",
      "features": {
        "issue": true,
        "pr": true,
        "label": true,
        "milestone": true,
        "release": true,
        "pipeline": true,
        "review": true,
        "auth": true
      }
    },
    {
      "name": "GitCode",
      "identifier": "gitcode",
      "cli_binary": "gitcode",
      "min_version": "0.6.0",
      "tested_versions": ["0.6.1"],
      "install_url": "https://gitcode.com/gitcode-cli/cli",
      "doc_link": "https://gitcode.com/gitcode-cli/cli/blob/main/README.md",
      "features": {
        "issue": true,
        "pr": true,
        "label": true,
        "milestone": true,
        "release": true,
        "pipeline": true,
        "review": true,
        "auth": true
      }
    }
  ]
}
```

### 5.2 编译时嵌入

`crates/core/src/compatibility.rs`：

```rust
/// 编译时嵌入的兼容性矩阵 JSON。
const MATRIX_JSON: &str = include_str!("../../../docs/compatibility-matrix.json");

/// 解析后的平台兼容性信息。
#[derive(Debug, Clone, Deserialize)]
pub struct PlatformCompat {
    pub name: String,
    pub identifier: String,
    pub cli_binary: String,
    pub min_version: String,
    pub tested_versions: Vec<String>,
    pub install_url: String,
    pub doc_link: String,
}

/// 获取所有平台的兼容性信息。
pub fn platform_compatibility() -> Vec<PlatformCompat> { ... }

/// 获取指定平台的兼容性信息。
pub fn platform_requirement(identifier: &str) -> Option<PlatformCompat> { ... }
```

`prerequisites.rs` 的 `requirement_for()` 改为从此数据源读取 `min_version`、`install_url`、`doc_link`，消除硬编码重复。

### 5.3 Markdown 生成

`Makefile` 新增目标：

```makefile
.PHONY: compatibility-matrix
compatibility-matrix: ## 从 JSON 生成兼容性矩阵 Markdown
	cargo run --example gen-compat-matrix
```

`examples/gen-compat-matrix.rs` 读取 JSON 生成 `docs/compatibility-matrix.md`，格式：

```markdown
# 兼容性矩阵

> 自动生成，请勿手动编辑。数据源：`docs/compatibility-matrix.json`
> 更新时间：2026-07-31

| 平台 | CLI 工具 | 最低版本 | 已测试版本 | 功能覆盖 |
|------|---------|---------|-----------|---------|
| GitHub | `gh` | ≥ 2.0.0 | 2.62.0 | issue ✅ pr ✅ label ✅ ... |
| GitLab | `glab` | ≥ 1.30.0 | 1.46.1 | issue ✅ pr ✅ label ✅ ... |
| GitCode | `gitcode` | ≥ 0.6.0 | 0.6.1 | issue ✅ pr ✅ label ✅ ... |
```

## 6. 不做清单

- ❌ 不引入 JSON Schema 验证（`jsonschema` crate）— 当前阶段 fixture 反序列化测试足够
- ❌ 不引入 `insta` 快照测试 — 避免新依赖
- ❌ 不做 CI 定期巡检刷新夹具 — 归入 #96（e2e 实化）
- ❌ 不改变 `CommandRunner` trait 的位置（保留在各 crate）— 归入未来重构
- ❌ 不改变 `AuthChecker` trait — 仅桥接其输出到 `PlatformCliError`

## 7. 验收标准

- [ ] `cargo test` 包含 9 个契约测试且全绿
- [ ] `PrerequisiteError` 所有变体的 Display 输出为中文
- [ ] gitcode 最低版本校验为 0.6.0
- [ ] 底层 CLI 原始 stderr 不出现在用户可见的错误消息中（仅 `tracing::debug!`）
- [ ] `GhError`、`GlabError`、`GitcodeError` 已删除，统一为 `PlatformCliError`
- [ ] `docs/compatibility-matrix.md` 存在且由 `make compatibility-matrix` 生成
- [ ] `docs/compatibility-matrix.json` 是版本要求的单一数据源
- [ ] `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic` 通过
