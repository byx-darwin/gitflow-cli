# Milestone 挂载 Issue/PR 支持

**Issue:** #357
**Workflow:** wf-2026-09-21-002

## 1. 问题

`gf milestone` 提供完整 CRUD，`MilestoneData` 也完整建模了 `closed_issues`/`open_issues` 进度计数器，但**没有任何命令能把 Issue 或 PR 挂到 milestone 上**。`gf issue create`/`edit`、`gf issue list`、`gf pr create` 均无 `--milestone` 参数，`IssueData`/`PrData` 也没有 milestone 字段。后果：milestone 的 `openIssues`/`closedIssues` 永远为 0，功能只对"用别的工具往里加票"的场景有意义。

## 2. 平台能力实测（已验证，非推断）

| 能力 | `gh` | `glab` | `gitcode` |
|---|---|---|---|
| `issue create` 挂 milestone | `-m/--milestone <title>` | `-m/--milestone <全局ID或title>` | `-m/--milestone <number>` |
| `issue edit` 挂/取消 milestone | `-m <title>` 挂；`--remove-milestone` 取消 | `-m <title>`；传 `""`/`0` 取消 | `-m <number>` 挂；无原生"取消"语义，需确认 |
| `pr`/`mr create` 挂 milestone | `-m/--milestone <title>` | `-m/--milestone <全局ID或title>` | **无此参数** |
| `pr`/`mr edit` 挂 milestone | — | — | `-m/--milestone <number>` |
| `issue list` 按 milestone 过滤 | `-m <number或title>` | `-m <id>` | `-m <title或number>` |
| `pr list` 按 milestone 过滤 | — | — | `-m <title>` |

**关键不一致**：
- GitHub/GitLab 都用**标题**匹配（GitLab 文档写"全局 ID 或 title"，但 GitLab 的"全局 ID"与本项目 `MilestoneData.number` 承载的**项目内 iid** 是两个不同的数字空间，直接传 `number` 有传错的风险；统一改传 title 规避这个歧义）。
- GitCode 只认**数字编号**，且 `pr create` 完全没有挂载入口。
- 三者的 list 过滤参数语义也不完全对齐（GitHub 两者皆可，GitLab 只認 id，GitCode 两者皆可但两个字段名不同）。

## 3. 设计决策

### 3.1 `MilestoneRef`：轻量引用，不嵌入完整 `MilestoneData`

新增 `crates/core/src/types.rs`：

```rust
/// A lightweight reference to a milestone attached to an Issue or PR.
///
/// Deliberately does not embed the full `MilestoneData` (due date, progress
/// counters) — that would duplicate data already served by `gf milestone
/// list`/`view` and cost an extra API call on every issue/PR fetch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MilestoneRef {
    /// The milestone's number (platform-native numbering).
    pub number: u64,
    /// The milestone's title.
    pub title: String,
}
```

`IssueData`/`PrData` 各加一个 `#[serde(skip_serializing_if = "Option::is_none")] pub milestone: Option<MilestoneRef>` 字段（未挂载时不出现在 JSON 里，符合 `#[serde(skip_serializing_if = "Option::is_none")]` 既有惯例）。

### 3.2 统一标识符解析器

新增 `crates/core/src/label.rs`：

```rust
/// Resolves a user-supplied milestone identifier (`--milestone <NUMBER|TITLE>`)
/// against the repository's milestone list, matching by number first (if the
/// identifier parses as `u64`) then by exact title.
///
/// # Errors
///
/// Returns an error if no milestone matches, or if the identifier is a
/// non-numeric string that doesn't exactly match any milestone's title.
pub async fn resolve_milestone_identifier(
    provider: &dyn MilestoneProvider,
    identifier: &str,
) -> Result<MilestoneRef> {
    let milestones = provider.list(None).await?;
    if let Ok(number) = identifier.parse::<u64>() {
        if let Some(m) = milestones.items.iter().find(|m| m.number == number) {
            return Ok(MilestoneRef { number: m.number, title: m.title.clone() });
        }
    }
    milestones
        .items
        .iter()
        .find(|m| m.title == identifier)
        .map(|m| MilestoneRef { number: m.number, title: m.title.clone() })
        .ok_or_else(|| CoreError::Platform(format!(
            "milestone '{identifier}' not found (matched by neither number nor exact title)"
        )))
}
```

每个平台的 `issue.rs`/`pr.rs` 在需要挂载/取消/过滤时，各自构造一个同仓库的 `XxxMilestoneProvider` 调用它。解析结果按平台原生需要的形式使用：

- GitHub/GitLab：用 `MilestoneRef.title`（规避 GitLab id/iid 歧义）
- GitCode：用 `MilestoneRef.number`

List 过滤同样先解析再传原生形式，保持三平台行为一致、可预测（代价：每次 list 过滤多一次 milestone 列表调用，可接受）。

### 3.3 三态取消挂载（仅 Issue 侧；PR 侧只需 create）

`CreateIssueArgs`/`CreatePrArgs` 新增 `pub milestone: Option<String>`（创建时的原始标识符，`None` = 不挂载）。

**订正（写 plan 前复核代码发现）**：AC 明确只要求 `gf issue create`/`gf issue edit`/`gf issue list --milestone` 三个入口，以及 `gf pr create` 一个入口——**不要求 PR 侧的 edit 或 list 过滤**。且 `gf pr edit` 命令目前在本仓库根本不存在（`crates/core/src/pr.rs` 没有 `EditPrArgs`，`apps/cli/src/commands/pr.rs` 没有 `Edit` 变体）。为一个 Issue 附带新增一个全新的 `gf pr edit` 命令是明显的范围蔓延，按 YAGNI 原则不做。因此：

- `EditIssueArgs` 新增三态字段：

```rust
/// 三态：不传字段（`None`）表示不改动；`Some(None)` 表示取消挂载；
/// `Some(Some(identifier))` 表示设置为该 milestone（`NUMBER` 或 `TITLE`，
/// 由 `resolve_milestone_identifier` 解析）。
pub milestone: Option<Option<String>>,
```

- **不新增 `EditPrArgs`**，PR 侧只有 `CreatePrArgs.milestone: Option<String>`（创建时设置，不支持事后修改——若未来需要，应作为独立 Issue 提出"新增 `gf pr edit` 命令"，而不是夹带在本次里）。
- **不新增 PR 列表的 milestone 过滤**（AC 未要求）。

CLI 层（`apps/cli`）对应两个互斥 flag（仅 `gf issue edit`）：

```
--milestone <NUMBER|TITLE>   # 设置
--remove-milestone           # 取消
```

同时给出两者视为用户输入错误（`miette::miette!("--milestone 与 --remove-milestone 不能同时指定")`）。

`ListIssueArgs` 新增 `pub milestone: Option<String>`（原始标识符，`None` = 不过滤）。`gf pr create` 只需 `--milestone <NUMBER|TITLE>` 一个新 flag，无对应的取消/过滤入口。

### 3.4 GitCode PR 创建时挂载：create + edit 两步

`gitcode pr create` 没有 `--milestone`。`GitCodePrProvider::create()` 若 `args.milestone` 非空：先正常 `pr create`，成功后立即调用 `pr edit <number> --milestone <resolved.number>`。第二步失败时返回明确错误（信息包含"PR #N 已创建，但挂载 milestone 失败"），不静默吞掉、不回滚已创建的 PR（回滚需要额外的删除调用，超出本次范围，且 PR 本身是合法产物，只是缺一个挂载）。

### 3.5 GitCode issue edit 取消挂载语义未知，需要在实现阶段实测确认

调研阶段未能确认 `gitcode issue edit` 是否支持"取消 milestone"（无 `--remove-milestone` 等价物，`--milestone 0` 是否代表取消未经验证）。实现阶段需要对着真实 GitCode 仓库实测（复用 `byx-darwin/NexaTrade` 或已记忆的 GitLab 测试仓库同等角色的 GitCode 测试仓库）确认，若不支持则如实记录限制、不假装支持。

## 4. 改动范围

| 文件 | 改动 |
|---|---|
| `crates/core/src/types.rs` | 新增 `MilestoneRef` |
| `crates/core/src/issue.rs` | `IssueData.milestone`、`CreateIssueArgs.milestone`、`EditIssueArgs.milestone`（三态）、`ListIssueArgs.milestone` |
| `crates/core/src/pr.rs` | `PrData.milestone`、`CreatePrArgs.milestone`（仅创建，无 edit/list 过滤，见 3.3 订正） |
| `crates/core/src/label.rs` | 新增 `resolve_milestone_identifier` |
| `crates/github/src/issue.rs`、`pr.rs` | 挂载/取消/过滤，均用 title |
| `crates/gitlab/src/issue.rs`、`mr.rs` | 同上 |
| `crates/gitcode/src/issue.rs`、`pr.rs` | 挂载/取消/过滤，均用 number；PR create 走 create+edit 两步 |
| `apps/cli/src/commands/issue.rs`、`pr.rs` | `--milestone`/`--remove-milestone` CLI flag |

## 5. 测试策略

- `resolve_milestone_identifier` 的单元测试：按 number 命中、按 title 命中、两者都不命中报错、number 优先于 title（如果标识符恰好既是某个milestone的number又是另一个的title——边界情况需要明确定义优先级并测试）。
- 每个平台 issue/pr 的 create/edit/list 各补一条"挂载成功"与"取消成功"（若平台支持）测试。
- 端到端验证（复用已有测试仓库）：创建 milestone → 挂载一个 issue/PR → `gf milestone list` 的 `openIssues`/`closedIssues` 反映真实数量——这是 AC 里"计数器不再恒为 0"的判据，仅靠单测不够，必须实测。

## 6. 不做的事

- 不改 `MilestoneData` 本身（`due_on`/`closed_issues`/`open_issues` 已经建模正确，问题只在于没有写入路径）。
- 不实现"批量挂载"（一次性把多个 Issue 挂到同一个 milestone）——AC 未要求，YAGNI。
- 不处理 GitCode PR 挂载失败后的自动回滚（见 3.4）。
