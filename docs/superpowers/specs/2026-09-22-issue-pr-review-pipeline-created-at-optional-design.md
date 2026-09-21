# IssueData/PrData/CommentData/ReviewData/PipelineStatus 时间戳字段改为 Option，消除跨平台静默伪造

**Issue:** #380
**Workflow:** wf-2026-09-21-011（待创建）
**Precedent:** #366（`ReleaseData.created_at` 同类修复，`docs/superpowers/specs/2026-09-21-release-created-at-optional-design.md`）

## 1. 问题

`crates/core/src/{issue,pr,review,pipeline,types}.rs` 里 5 个核心类型的时间戳字段
（`IssueData.created_at`/`updated_at`、`PrData.created_at`/`updated_at`、
`ReviewData.submitted_at`、`CommentData.created_at`、
`PipelineStatus.created_at`/`updated_at`）类型都是 `DateTime<Utc>`（非
`Option`）。GitLab 和 GitCode 两个平台适配器在各自的 `From<XxxApiResponse> for
YyyData` 实现里，API 未返回或解析失败时一律回落 `Utc::now()`——与 #366 修复的
`ReleaseData.created_at` 是同一类缺陷：一条真实创建时间未知的记录会显示成"刚刚
创建"，与真实的同期记录在展示层无从分辨。

## 2. 现状调研（已验证，来自 #380 澄清阶段的详尽审计，非推断）

审计方法：不仅 grep 字面量 `Utc::now()`（会漏掉复用闭包/`Utc::now` 裸函数引用
传参的写法，例如 `crates/gitcode/src/pr.rs` 的 `parse_time` 闭包），而是逐个通读
每个 `impl From<...ApiResponse> for <CoreType>` 或等价转换函数。

### 2.1 需要修的站点（Category A，缺失字段回落，本 Issue 范围）

| 核心类型字段 | 平台 | 文件:行 | 具体写法 |
|---|---|---|---|
| `IssueData.created_at`/`updated_at` | GitLab | `crates/gitlab/src/issue.rs:242-278` | `let now = Utc::now();` 复用两次，`api.created_at.unwrap_or(now)` |
| `IssueData.created_at`/`updated_at` | GitCode | `crates/gitcode/src/issue.rs:85-134` | `.unwrap_or_else(Utc::now)`（**注：本站点未被 #395 修复**，#380 之前的澄清评论有误，已在 Issue 上更正） |
| `PrData.created_at`/`updated_at` | GitLab | `crates/gitlab/src/mr.rs:294-325` | 同构 `let now = Utc::now();` 复用模式 |
| `PrData.created_at`/`updated_at` | GitCode | `crates/gitcode/src/pr.rs:156-193` | `parse_time` 闭包（157-159 行）复用两次，`.map_or_else(Utc::now, ...)`；同文件里 `merged_at` 用另一个闭包 `parse_opt_time`（162-164 行）正确透传 `None`，是现成的正例参照 |
| `CommentData.created_at`（issue 评论） | GitLab | `crates/gitlab/src/issue.rs:288-310` | `.unwrap_or_else(Utc::now)` |
| `CommentData.created_at`（issue 评论） | GitCode | `crates/gitcode/src/issue.rs:172-197` | 外层 `map_or_else(Utc::now, ...)`（缺字段）包一层内层 `.unwrap_or_else(\|_\| Utc::now())`（解析失败）——两层最终落到同一个字段，一次修复覆盖两层 |
| `CommentData.created_at`（PR/MR 评论） | GitLab | `crates/gitlab/src/mr.rs:335-356` | 同构，独立的 struct/impl |
| `CommentData.created_at`（PR/MR 评论） | GitCode | `crates/gitcode/src/pr.rs:125-153` | 同构，与 gitcode issue.rs 形状一致 |
| `ReviewData.submitted_at` | GitLab | `crates/gitlab/src/review.rs:141-148`（`request_changes()`）、`crates/gitlab/src/review.rs:206-215`（`submit_review()`） | 两处调用点，均为 `note.created_at.unwrap_or_else(Utc::now)`，`note.created_at: Option<DateTime<Utc>>` |
| `PipelineStatus.created_at`/`updated_at` | GitLab | `crates/gitlab/src/pipeline.rs:170-191` | `.unwrap_or_else(Utc::now)`，`api.created_at`/`updated_at: Option<DateTime<Utc>>` |

**GitHub 侧零命中**：`IssueData`/`PrData`/`ReviewData` 的 GitHub 路径要么是
`serde_json::from_slice` 直接反序列化（缺字段直接失败，fail-loud，本来就是对
的），要么走 `parse_api_datetime` 辅助函数（`crates/github/src/issue.rs` ~699
行，`CommentData` 共用），后者回落 `UNIX_EPOCH` + `tracing::warn!`——这是**解析
失败**场景（字段必填、值存在但格式不对），不是缺字段场景，与本 Issue 的缺陷
形态不同，已拆到 #401 单独处理，不在本次改动范围内。

### 2.2 排除的站点（Category B，合法用法，仅加注释）

- `crates/gitlab/src/review.rs:180-189`（`approve()`）：`submitted_at: Utc::now()`
  无条件赋值。`glab mr approve` 不返回时间戳，但"批准"这个动作本身就是同步地
  在这一刻发生——不是在回填一个未知的历史时间，语义上是对的。**决策：加一行
  注释说明原因，代码不改。**

### 2.3 拆分出去的站点（Category C，已建 #401，不在本次范围）

`crates/github/src/review.rs:248-267`（`submitted_at` 解析失败回落，且缺
`tracing::warn!` 日志）、`crates/github/src/pipeline.rs:53-69`（`GhRun::into_status`
解析失败回落，生产路径）、`crates/gitcode/src/pipeline.rs:39-67`（`GcRun::into_status`
同构但整个函数 `#[allow(dead_code)]`，无生产调用方）——这三处都是"字段必填、值存在
但解析失败"，与本 Issue"字段本身缺失"是不同的 bug 形态，已拆到 #401。

### 2.4 下游消费者影响面排查

全仓库排查 `\.created_at\b`/`\.updated_at\b`/`\.submitted_at\b` 的读取点（不含
测试、不含字段定义/构造本身），确认只有一处需要跟着改：

- `crates/gitlab/src/pipeline.rs:349`：`recent.iter().filter(|p| p.created_at >= cutoff)`
- `crates/gitlab/src/pipeline.rs:388`：`(p.updated_at - p.created_at).num_seconds()`

`apps/cli/` 没有任何专门渲染这 5 个类型时间戳字段的代码——CLI 的 text/auto 输出
走 `apps/cli/src/commands/output.rs` 的通用 `serde_json::Value` 渲染器
（`print_text_object`），JSON key 缺失时该渲染器不会为那一行生成任何输出，与
"字段整个消失"的设计选择天然一致，`apps/cli` 不需要改代码（与 #366 的调研结论
一致）。

## 3. 设计决策

### 3.1 `None` 时的序列化行为

**决策：省略字段（`#[serde(skip_serializing_if = "Option::is_none")]`），与
#366 的 `ReleaseData.created_at`/`published_at` 完全同构。** 5 个类型统一同一
策略，不制造内部不一致。

### 3.2 `PipelineStatus` 下游统计口径（gitlab/pipeline.rs 独有影响）

**决策：未知时间戳一律"排除出统计口径"，而不是当 0 或当"现在"处理：**

- `created_at` 为 `None` 的记录：排除出 cutoff 时间窗过滤结果（时间未知，无法
  判断是否在窗口内，保守排除而非猜测归入）
- `created_at`/`updated_at` 任一为 `None` 的记录：不计入平均耗时的样本集合
  （同样是排除而非猜测归零，避免拉低/拉高 `avg_duration_secs`）

这两处都是"未知数据不参与统计"，与 #366"缺失 ≠ 伪造"的原则一致，只是这里的
落点是聚合统计而非单条展示。

### 3.3 Category B 处理

`crates/gitlab/src/review.rs:189` 加注释说明该 `Utc::now()` 是同步动作的真实
时间戳，非缺失字段的伪造，避免未来被误"修复"。代码逻辑不变。

### 3.4 语义化版本号

`gitflow-core` 发布在 crates.io，5 个字段从必填变可选是一次破坏性变更（依赖方
手动构造或穷尽模式匹配会编译失败；JSON 反序列化方向本身向后兼容）。**决策：
本次提交不手动修改 `Cargo.toml` 的 `version` 字段**，在 commit message 中包含
`BREAKING CHANGE:` 段落，交给下次 `make release` 的 conventional-commits 版本
推导自然产出 major 版本提升（与 #366 完全同构的决策，理由同样适用：版本号提升
与"什么时候发布"是独立决策）。

## 4. 改动范围

| 文件 | 改动 |
|---|---|
| `crates/core/src/issue.rs` | `created_at`/`updated_at` 改 `Option<DateTime<Utc>>` + `skip_serializing_if`；相关测试改 `Option` 比较 |
| `crates/core/src/pr.rs` | 同上 |
| `crates/core/src/review.rs` | 仅 `ReviewData.submitted_at` 改 `Option`。`ReviewCommentData.created_at`（core/review.rs:75，行内/文件级审查评论类型）**不在本次范围**——复核确认它从未被任何平台 crate 构造（`grep -rln "ReviewCommentData" crates/*/src/*.rs` 只命中 `core/lib.rs` 的 re-export 和 `core/review.rs` 自己的测试），是尚未接入生产路径的类型，没有可修的 fallback 站点 |
| `crates/core/src/pipeline.rs` | `created_at`/`updated_at` 改 `Option` |
| `crates/core/src/types.rs` | `CommentData.created_at` 改 `Option`（issue/PR 评论共用） |
| `crates/gitlab/src/issue.rs` | 删除两处 `unwrap_or(now)`/`unwrap_or_else(Utc::now)`，直接透传；新增缺失字段回归测试 |
| `crates/gitlab/src/mr.rs` | 同上（PrData + CommentData 两个 impl） |
| `crates/gitlab/src/review.rs` | 删除两处 `unwrap_or_else(Utc::now)`（request_changes/submit_review）；`approve()` 加说明注释，代码不改 |
| `crates/gitlab/src/pipeline.rs` | 删除转换处的 `unwrap_or_else(Utc::now)`；调整 `report()` 里 cutoff 过滤与耗时计算为"None 排除"语义；新增回归测试 |
| `crates/gitcode/src/issue.rs` | 删除 `IssueData` 转换（85-134 行）与 `CommentData` 转换（172-197 行）里的 `Utc::now()` 回落 |
| `crates/gitcode/src/pr.rs` | 删除 `PrData` 转换（`parse_time` 闭包）与 `CommentData` 转换里的 `Utc::now()` 回落；参照同文件已有的 `parse_opt_time` 写法 |
| `crates/github/src/{issue,pr,review,pipeline}.rs` | 无生产代码改动；各补一条"字段变 Option 后编译通过、缺失时反序列化不报错"的验证测试 |
| `apps/cli/` | 不改动 |

## 5. 测试策略

沿用 #366 的 RED→GREEN 模式，按类型分批：

1. **RED**：GitLab + GitCode 各类型先写"API 响应省略该字段 → 核心类型对应字段
   为 `None`"的回归测试，此时因字段类型不匹配或断言失败而红。
2. **GREEN**：核心类型改 `Option` + 对应适配器删除 fallback，使测试转绿。
3. 同步修正现有"断言具体时间值"的测试为 `Option` 比较（`crates/core/src/{issue,pr,review}.rs`、
   `crates/core/src/types.rs`、`crates/gitcode/src/{issue,pr}.rs` 里已有的若干处）。
4. GitHub 侧各类型补一条"响应省略该字段 → 不报错，核心类型字段为 None"的验证
   测试，钉住"类型变更后自动兼容"这个结论。
5. `PipelineStatus` 额外补两条聚合口径测试：`created_at=None` 的记录不进入
   cutoff 过滤结果；`created_at`/`updated_at` 任一为 `None` 时不计入耗时样本。
6. 全量 `make test` + `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic`。

## 6. 不做的事

- 不处理 Category C（解析失败回落）——已拆到 #401，独立设计决策。
- 不改 `crates/gitlab/src/review.rs:189` 的 `approve()` 逻辑，只加注释。
- 不在本次提交调整 `Cargo.toml` 版本号（见 3.4）。
- 不新增任何 CLI 渲染代码（通用渲染器已经处理正确，见 2.4）。
- 不处理 `crates/core/src/pr.rs` 的 `merged_at`（已是 `Option`，两个平台都已
  正确透传，不在缺陷范围内）。
- 不处理 `ReviewCommentData.created_at`（core/review.rs:75）——该类型从未被任何
  平台 crate 构造，不存在需要修的 fallback 站点（见改动范围表）。
