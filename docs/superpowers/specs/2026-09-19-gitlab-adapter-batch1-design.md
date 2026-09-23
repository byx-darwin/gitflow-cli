# GitLab 适配器批次1修复设计（#372 / #362 / #375）

**Status:** Approved（Bounded 路径，无需架构级评审）
**Workflow:** wf-2026-09-19-006

## 背景

三个独立发现于近期 dogfooding 的 GitLab 适配器缺陷，均已在 Issue 中给出根因、复现步骤与验收标准：

- **#372** `gf label create` 在 GitLab 上——原报告称标签名被当位置参数传给 `glab`，**经核实该部分已被更早的提交修复**（`crates/gitlab/src/label.rs` 现已使用 `--name`/`--color`）。剩余待修范围缩小为：`--color` 十六进制值的 `#` 前缀归一化，以及 `apps/cli/src/commands/label.rs` 帮助文案与实际要求不符。
- **#362** `gf auth status --platform gitlab`：失败路径不输出 JSON（其他平台会）；多 host 场景被折叠为单一布尔，导致已认证的 host 被误报为未认证。
- **#375** `gf issue create`（GitLab）在未提供 `--body` 时 100% 失败，因为 `--description` 只在有 body 时才被传给 `glab`。

## 方案

### 1. `crates/gitlab/src/label.rs`（#372）

- `create()` 中对 `args.color` 做归一化：缺少 `#` 前缀时自动补上，再传给 `glab label create --color`。
- 同步修正 `apps/cli/src/commands/label.rs:34` 的帮助文案，明确 `#` 前缀是否必需（以实测结果为准）。
- 新增单测：验证传入 `d73a4a`（无 `#`）时，实际传给 `glab` 的参数是 `#d73a4a`。

### 2. `crates/gitlab/src/auth.rs` + `crates/core/src/auth.rs`（#362）

- `status()`：将 `glab auth status` 的 stdout+stderr 解析为按 host 分组的列表，替代当前"整段文本包含 not logged in"式的全局判断。
- 整体结论（方案A，已与用户确认）：**任一 host 解析出已登录，整体 `logged_in = true`**——不引入 repo remote → host 的精确匹配（`GitLabAuthProvider` 目前不持有 repo 上下文，引入需要改动 provider 构造签名及三处调用点 `apps/cli/src/commands/{auth,prerequisites,doctor}.rs`，超出本批次范围）。
- 失败路径不再直接 `Err(...)` 提前返回（除非 CLI 完全无法执行/无法解析），统一返回 `Ok(AuthStatus{...})`，交由 CLI 层正常序列化为 JSON。
- `crates/core/src/auth.rs`：为 `AuthStatus` 新增可选字段
  ```rust
  #[serde(default, skip_serializing_if = "Vec::is_empty")]
  pub hosts: Vec<HostAuthStatus>,
  ```
  新增结构体 `HostAuthStatus { host: String, logged_in: bool, user: Option<String> }`。纯新增、向后兼容；GitHub/GitCode 保持返回空 `hosts`（它们本就单 host，不受影响）。
- 新增测试：三平台 JSON 结构一致性测试；GitLab 多 host mock 场景（一台已登录一台未登录 → 整体 `logged_in=true` 且 `hosts` 含两条记录）。

### 3. `crates/gitlab/src/issue.rs`（#375）

- `create()`：把 `if let Some(body) = &args.body { push "--description"; push body }` 改为无条件 `push "--description"`，值为 `args.body.as_deref().unwrap_or("")`。
- 新增测试：`body: None` 时断言实际传给 `glab` 的参数列表（对照既有 `Some(body)` 路径的测试）。

## 范围外

- `docs/specs/phase4-dogfooding-checklist.md` 的 `--labels`/`gf release create --notes` 笔误（#375 报告中提到）——纯文档缺陷，留待单独开 issue，不混入本次代码修复。

## 测试策略

三处均为独立文件、独立 trait 方法调用，无交叉依赖。按 Smart Subagent Batching 打分（3 个文件、无跨模块边界、无破坏性公开 API 变更、无迁移）属于 simple，采用主 agent 批量实现 + 单次 review。

- 单元测试：三处各自的 mock CommandRunner 测试（归一化参数、JSON 序列化结构、多 host 折叠逻辑）。
- 回归：`cargo test -p gitflow-gitlab -p gitflow-core`。
- 交付前：`make lint`（fmt + clippy）。

## 验收标准（汇总自三个 Issue）

- [ ] `gf label create "<name>" --color "<hex，无 # 前缀>"` 在 GitLab 平台成功创建标签
- [ ] `gf label create --help` 中 color 参数说明与实际要求一致
- [ ] `gf auth status --platform gitlab --output json` 在成功与失败两条路径上都输出 JSON，结构与 GitHub/GitCode 对齐
- [ ] 多 host 场景下，当前已认证的 host 不再被折叠进"未认证"结论；`hosts` 字段可见每台 host 状态
- [ ] 三平台 `auth status` JSON 结构一致性有测试覆盖
- [ ] `gf issue create --title "<any>"`（无 `--body`）在 GitLab 上成功
- [ ] 新增回归测试覆盖 `body: None` 的 `glab issue create` 调用参数，且不破坏 `Some(body)` 路径
