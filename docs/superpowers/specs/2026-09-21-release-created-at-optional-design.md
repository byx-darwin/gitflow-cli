# ReleaseData.created_at 改为 Option，消除静默伪造创建时间

**Issue:** #366
**Workflow:** wf-2026-09-21-001

## 1. 问题

`ReleaseData.created_at`（`crates/core/src/release.rs`）类型是 `DateTime<Utc>`（非
`Option`）。gitlab 与 gitcode 两个平台适配器在各自的
`From<XxxApiResponse> for ReleaseData` 实现里，API 未返回该字段时一律回落
`Utc::now()`：

- `crates/gitlab/src/release.rs:179`：`created_at: api.created_at.unwrap_or(now)`
- `crates/gitcode/src/release.rs:180`：`created_at: api.created_at.unwrap_or_else(Utc::now)`

后果：一个真实创建时间缺失的 release，会被显示成"今天创建"，且与真实的今日
release 在展示层无从分辨——这是静默产生错误数据，与 #360/#365 系列要消灭的静默
截断属同一类缺陷，只是发生在展示层而非分页层。

## 2. 现状调研（已验证，非推断）

- `published_at`（同一个 `ReleaseData` 结构体里，紧邻 `created_at` 下方）已经是
  `Option<DateTime<Utc>>`，并挂 `#[serde(skip_serializing_if = "Option::is_none")]`
  ——`None` 时字段整个从 JSON 里消失。这是现成的同构先例。
- **GitHub 侧没有任何 fallback 逻辑**：`crates/github/src/release.rs` 的
  `list()`/`view()` 直接把 `gh release list --json ...`/`gh release view --json ...`
  的输出反序列化成 `ReleaseData` 本身（`serde_json::from_slice::<ReleaseData>`），
  不经过任何中间 struct。这意味着 Issue AC 里"github 适配器不再回落"这一条对
  GitHub 而言是自动满足的（本来就没有可删的 fallback 代码），只需要确认类型变更
  后编译通过、且反序列化时字段缺失不再报错（当前是必填字段，若 `gh` 输出缺失
  `createdAt`，`serde_json::from_slice` 会直接失败；改成 `Option` 后会优雅退化）。
- **展示层没有专门的 `created_at` 渲染代码**：`apps/cli/src/commands/release.rs`
  完全不引用 `created_at`/`createdAt`。CLI 的 text/auto 输出走
  `apps/cli/src/commands/output.rs` 的通用 `serde_json::Value` 渲染器
  （`print_text_object`），JSON key 缺失时该渲染器**不会**为那一行生成任何输出
  ——这与本设计"整个字段消失"的选择天然一致，`apps/cli` 不需要改任何代码。
- **兼容性调用点核查**：`grep -rln "createdAt" skills/ docs/` 未命中任何 release
  相关文档或 skill；`grep -rn "\.created_at\b"` 未发现任何针对 release 的排序或
  比较逻辑。`crates/gitlab/src/mr.rs` 里同名的 `created_at` 属于 merge request，
  是完全独立的字段，不在本次改动范围内。
- 现有测试里，`crates/core/src/release.rs:190` 与
  `crates/gitcode/src/release.rs:894,1058` 各有一条直接断言 `created_at` 具体值
  的测试，字段类型变更后需要同步改成 `Option` 比较。未发现任何专门测试"缺失时
  回退到当前时间"这个行为本身的用例（该行为原本就是本次要删除的缺陷，不存在需
  要迁移的正向测试）。

## 3. 设计决策

### 3.1 `None` 时的序列化行为

**决策：省略字段（`#[serde(skip_serializing_if = "Option::is_none")]`），与
`published_at` 完全同构。**

备选方案（保留字段但序列化为显式 `null`，配合 `output.rs` 现成的
`Value::Null → "-"` 渲染逻辑）被否决：虽然能在 text 模式下显式展示"-"，但会让
同一个结构体里两个语义相同的"可能缺失的时间字段"（`created_at`/`published_at`）
用两种不同的序列化策略处理 `None`，制造不必要的内部不一致。省略字段同样满足
"不伪造时间"这个核心诉求（消失 ≠ 伪造），且是 DRY 的选择。

### 3.2 语义化版本号

`gitflow-core` 发布在 crates.io，`ReleaseData.created_at` 从必填字段变为可选字段
是一次公开类型的破坏性变更（依赖方若手动构造 `ReleaseData` 或做穷尽模式匹配会
编译失败；JSON 反序列化方向本身是兼容的，向后兼容旧数据）。

**决策：本次提交不手动修改 `Cargo.toml` 的 `version` 字段**，在 commit message
中包含 `BREAKING CHANGE:` 段落，交给下次 `make release` 的 conventional-commits
版本推导（见 `CLAUDE.md` → Release 工作流）自然产出 major 版本提升。理由：版本号
提升与"什么时候发布"是独立决策，不应该被单个 Issue 修复的时间点绑架；`make
release` 已有现成的、经过验证的推导流程。

## 4. 改动范围

| 文件 | 改动 |
|---|---|
| `crates/core/src/release.rs` | `created_at` 类型改为 `Option<DateTime<Utc>>`，加 `skip_serializing_if`；相关测试改用 `Option` 比较 |
| `crates/gitlab/src/release.rs` | 删除 `unwrap_or(now)`，直接透传 `api.created_at`；新增"缺失时为 None"回归测试 |
| `crates/gitcode/src/release.rs` | 删除 `unwrap_or_else(Utc::now)`，直接透传 `api.created_at`；新增同类回归测试；现有断言具体值的测试同步改 `Option` |
| `crates/github/src/release.rs` | 无生产代码改动；确认字段类型变更后编译通过、缺失字段时反序列化不报错（补一条验证测试） |
| `apps/cli/` | 不改动 |

## 5. 测试策略

1. **RED**：先在 gitlab、gitcode 两侧各写一条"API 响应省略 created_at 字段 → `ReleaseData.created_at` 为 `None`"的测试，此时应因编译错误（字段类型不匹配）或断言失败而红。
2. **GREEN**：核心类型改 `Option` + 两个适配器删除 fallback，使上述测试通过。
3. 同步修正 `crates/core/src/release.rs`、`crates/gitcode/src/release.rs` 里断言具体值的既有测试。
4. GitHub 侧补一条"响应省略 createdAt → 不报错，字段为 None"的测试，钉住"类型变更后自动兼容"这个结论，而不是只凭读代码推断。
5. 全量 `make test` + `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic`。

## 6. 不做的事

- 不改 `crates/gitlab/src/mr.rs` 的同名字段（不同功能，范围外）。
- 不在本次提交调整 `Cargo.toml` 版本号（见 3.2）。
- 不新增任何 CLI 渲染代码（见现状调研，通用渲染器已经处理正确）。
