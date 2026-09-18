# 列表命令分页：消除静默截断

- **Issue**：[#360](https://github.com/byx-darwin/gitflow-cli/issues/360)
- **Workflow**：`wf-2026-09-17-003`（standard 模式）
- **日期**：2026-09-17
- **状态**：设计已确认，待实现

## 1. 问题

`gf issue list --state open` 默认只返回 30 条，且不给任何截断信号。

根因在三个适配器中同构：`args.limit` 为 `None` 时，适配器**根本不拼 `--limit`**，
于是直接继承底层 CLI 的默认值（`gh issue list` 默认 30）。截断发生在 `gh` 进程内，
gf 侧完全无感。

```rust
// crates/github/src/issue.rs:256
let limit_str = args.limit.map(|limit| limit.to_string());
if let Some(ref limit) = limit_str {   // None 时什么都不加
    cmd_args.push("--limit");
    cmd_args.push(limit);
}
```

### 1.1 本仓库实测（2026-09-17）

```
默认(无 --limit): 30 条
--limit 1000:     33 条
被静默丢弃:       #93, #101, #102
```

### 1.2 核查中发现的、Issue 正文未记载的问题

| # | 发现 | 位置 | 性质 |
|---|---|---|---|
| 1 | `--limit` 跨平台语义不一致：github/gitcode 映射到 `--limit`（总条数），gitlab 映射到 `--per-page`（**单页大小**，GitLab API 上限 100） | `crates/gitlab/src/issue.rs:432` | `--limit 1000` 在 GitLab 上静默只回 ≤100 条 —— 第二个静默丢数据的洞 |
| 2 | `gf release list --limit` **被解析后直接丢弃**：`ReleaseCommand::List { .. }` 用 `..` 忽略 `limit`，`provider.list()` 不收参数 | `apps/cli/src/commands/release.rs:188` | 不是截断，是**旗标静默失效**。现有测试（`release.rs:502`）只断言解析结果为 `Some(10)`，未断言其生效 —— 这是该 bug 存活至今的原因 |
| 3 | `gf label list` 硬编码 `--limit 100`，CLI 未暴露任何旗标 | `crates/github/src/label.rs:138` | 同类截断，且用户无法覆盖 |
| 4 | `gf release list` 完全没有 limit 概念，继承 `gh release list` 默认 30 | `crates/github/src/release.rs:141` | 同类截断 |
| 5 | `list_comments`（三平台）：`api` 调用无 `--paginate`，只回第一页 | `crates/github/src/issue.rs:444` 等 | 同根因；`gf-workflow`、`gf-pr-apply-feedback` 是受害方 |
| 6 | `milestone list`：`gh api repos/.../milestones` 无 `--paginate`；且绕过 `runner` 直接用 `tokio::process::Command`，不可测 | `crates/github/src/label.rs:366` | 同根因 + 可测性缺陷 |
| 7 | `gf-issue-triage` SKILL.md 使用**无 `--limit`** 的 `gf issue list --state open` | `skills/gf-issue-triage/SKILL.md:49,72` | 坐实 Issue 推断：历史 triage 报告只覆盖了前 30 条 |
| 8 | `gf-label-stats` 声称用 `--limit` + `--page` 翻页兜底，但 **`--page` 参数在 CLI 中不存在** | `skills/gf-label-stats/SKILL.md:100` | 假兜底。该 skill "知道"要翻页却写了个不存在的参数，且长期无人发现 |

发现 8 是本设计选型的关键实证：**依赖调用方自觉的"提示性保证"在模型驱动的 skill 上会悄悄腐烂。**

## 2. 底层 CLI 能力（已实测，非推断）

| CLI | 版本 | 全量能力 | 关键事实 |
|---|---|---|---|
| `gh` | 2.97.0 | **无**"无限"选项 | `--limit int` = 最多取多少（默认 30），gh 内部自行翻页。"全量"只能表达成一个足够大的数 |
| `glab` | 1.115.0 | 有 `-A/--all` | 另有 `-p/--page`、`-P/--per-page`（默认 30，API 上限 100） |
| `gh api` / `glab api` | — | 均有 `--paginate` | 评论与 milestone 路径有现成解 |
| gitcode | **未安装** | 未知 | 本机 PATH 上的 `gc` 是其他工具，无法实测。只能按其 `--limit` 保守处理 |

**推论**：上限不是设计偏好，是 `gh` 逼出来的硬约束。既然必然存在上限，就必须同时提供触顶信号。

## 3. 选型

采纳「默认全量 + 截断信号」并行（Issue AC#1 的两条都做）：

- **默认全量**是机制性保证 —— 调用方无需改动即被修复
- **截断信号**覆盖两种触顶：默认上限触顶、用户显式 `--limit` 截断

### 3.1 已排除的备选

| 备选 | 排除理由 |
|---|---|
| 仅做截断信号，保留默认 30 | 退回提示性保证：要求每个调用方记得读 `truncated` 并重试。受害方是模型驱动的 skill，发现 8 即其失败实证 |
| 暴露 `--all` 旗标，默认不变 | 同上：要求每个调用方记得加旗标 |
| 上限取 10000 | 收益只在极少数超大仓库兑现，却把"修数据丢失"的改动变成"引入性能回归"的风险。触顶时 `truncated` 会明确提示用户加 `--limit` |
| 在 core 层包 paginating decorator | **不可行**：现签名 `list(args) -> Vec<T>` 无游标概念，decorator 无从发起第二页；改完 trait 后 decorator 亦无存在价值 |
| glab 用 `-A/--all` 取全量 | `--all` 无上界，会先拉完整个项目再由我们截断，cap 形同虚设 |

### 3.2 上限取值

`DEFAULT_LIST_LIMIT = 1000`。本仓库 33 条，量级余量 30 倍。

## 4. 范围：单一机制，六个命令

**修订说明**：初版设计将命令分为「A 族 list 子命令」与「B 族 api 分页」两套机制，
理由是 api 路径只能用 `--paginate`（全有或全无，无法逐页控制）。**该前提经实测证伪**：

```
$ gh api "repos/byx-darwin/gitflow-cli/issues/359/comments?per_page=2&page=1"  → 2 条
$ gh api "repos/byx-darwin/gitflow-cli/issues/359/comments?per_page=1&page=2"  → 第 2 条
```

`gh api` 与 `glab api` 均直接接受 `per_page` / `page` 查询参数，因此 api 路径可以走
与 list 子命令**完全相同**的分页机制。B 族取消，`--paginate` 不再使用，
初版 §4.2 中「知情接受的不一致」（B 族无上限、`truncated` 恒为 false）随之消失。

### 4.1 六个命令的策略矩阵（旗标均已实测）

| 命令 | github | gitlab | gitcode |
|---|---|---|---|
| `issue list` | `SingleShot` — `--limit N` | `Paged{100}` — `--per-page/--page` | `SingleShot` — `--limit N` |
| `pr list` | `SingleShot` — `--limit N` | `Paged{100}` — `--per-page/--page` | `SingleShot` — `--limit N` |
| `release list` | `SingleShot` — `--limit N` | `Paged{100}` — `--per-page/--page` | `SingleShot` — `--limit N` |
| `label list` | `SingleShot` — `--limit N` | `Paged{100}` — `--per-page/--page` | 见 §4.3 |
| `milestone list` | `Paged{100}` — `api ?per_page&page` | `Paged{100}` — `--per-page/--page` | 见 §4.3 |
| `issue comments` | `Paged{100}` — `api ?per_page&page` | `Paged{100}` — `api ?per_page&page` | `Paged{100}` — `api ?per_page&page`（未实测） |

`gh` 侧全部默认 30（`issue/pr/label/release list` 的 `--limit` 默认值均为 30，已实测）；
`glab` 侧 `--per-page` 默认 30，`milestone list` 默认 20。

### 4.2 明确排除并记录

`gf pipeline list` 硬编码 `--limit 30`（`crates/github/src/pipeline.rs:308,392`）。它取的是
**最近 30 次运行**，是有意的时间窗口，不是完整性声明 —— 与本 Issue「自称全量却残缺」
不是同一问题。已核查，不改。

### 4.3 gitcode 的 label / milestone：无法保证，如实记录

`crates/gitcode/src/label.rs:99`（label list）与 `:310`（milestone list）**绕过 `runner`**，
直接用 `tokio::process::Command`，且其分页旗标无从实测（gitcode CLI 在开发环境不可获得，
PATH 上的 `gc` 是其他工具）。本次：

- 两处改走 `runner`，恢复可测性（否则本设计的 argv 断言对其完全失效）
- 策略声明为 `SingleShot`，但**不传 `--limit`**（该旗标是否存在未知，乱传会直接报错）

**后果必须如实说明**：若 gitcode 的这两个命令在服务端有默认上限，
本次改动**无法探测也无法报告**其截断。这是一个已知的、未闭合的缺口，
不得在交付时声称三平台已全覆盖。其余四个命令（issue/pr/release list、comments）
gitcode 侧沿用其既有的 `--limit` / `api` 形态，可正常参与 N+1 探测。

### 4.3.1 gitcode 的 `--limit` 默认值：保守取值，不做通用 clamp

`issue list` / `pr list` / `release list` 的 `cap = limit.unwrap_or(DEFAULT_LIST_LIMIT)`
默认为 1000，N+1 探测会把 `cap + 1 = 1001` 传给 `--limit`。但 gitcode CLI 在本机不可
获得，其 `--limit` 的合法取值范围未经实测；本设计 §4.3 已经假定 100 是 gitcode 的
单页上限（`GITCODE_API_MAX_PER_PAGE`）。若 1001 超出 gitcode `--limit` 的真实上限，
这三个命令会在默认调用下直接报错，对所有 gitcode 用户破坏式回归。

处理方式：gitcode 单独持有 `GITCODE_DEFAULT_LIST_LIMIT = GITCODE_API_MAX_PER_PAGE`
（即 100），只替换这三个命令里 `.unwrap_or(DEFAULT_LIST_LIMIT)` 的默认值来源；
**用户显式传入的 `--limit` 不做任何 clamp**——该值在本分支之前就必须落在 gitcode
允许的范围内（否则本就会报错），本分支不改变这一事实。`label list` / `milestone
list`（见 §4.3）继续不传 `--limit`，不受影响；github、gitlab 不受影响。

**残留风险（已知、已评估、有意不消除）**：N+1 探测必然发送 `cap + 1`，因此默认调用
实际传出的是 `--limit 101` —— 仍然比本设计假定的 gitcode 单页上限 100 **多 1**。
若 `gc issue/pr/release list` 的 `--limit` 与其 `api` 的 `per_page` 共用同一个 100
上限，这三个命令在默认调用下**依然会报错**，而本分支之前它们在用户未指定 limit 时
根本不发 `--limit` 旗标。

风险因此是从「1001，几乎必然越界」降到「101，越界 1」，**未归零**。把默认值取 99
（探测恰好发出 100）可以在同一假设下彻底消除它，但那个上限本身也只是推断，99 不过是
另一个猜测。经评估决定：保留 100，把风险如实记录于此，待 gitcode CLI 可获得时优先
实测其 `--limit` 真实上限，再一次性校准默认值。

## 5. 输出契约

### 5.0 两个类型，两个边界

分页信息在两处出现，职责不同，不可混为一谈：

```rust
// crates/core/src/paging.rs —— provider 的返回类型（内部）
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Paged<T> {
    /// 已截断到 cap 之内的条目。
    pub items: Vec<T>,
    /// 是否因触顶而丢弃了更多条目（N+1 探测的结论）。
    pub truncated: bool,
    /// 实际生效的上限。
    pub limit: u32,
}
```

`Paged<T>` **不实现 `Serialize`**，不会整体落到输出里。CLI 层在打印前把它拆成两半：

```rust
let paged = provider.list(args).await?;
let (items, meta) = paged.into_parts();
let output = CliOutput::success_paged(items, meta, platform, "issue list");
```

`paged.items`（纯数组）落到 `data`，元数据落到信封的 `pagination`。
这样 `data` 的形状完全不变，见 §5.1。

### 5.1 信封加字段，`data` 形状不变

```rust
pub struct CliOutput<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,                          // 仍是数组，不包装
    pub error: Option<CliError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<PaginationMeta>,       // 新增
    pub platform: String,
    pub command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PaginationMeta {
    pub truncated: bool,
    pub returned: usize,
    pub limit: u32,
}
```

`PaginationMeta` 未标 `#[non_exhaustive]`：该属性会禁止结构体字面量构造，而 CLI
层的测试需要直接用字面量构造 `PaginationMeta` 来断言渲染逻辑（见
`apps/cli/src/commands/output.rs` 的测试）。这是实现阶段的有意选择，设计到此为止
不再假定该属性存在。

两条性质：

1. `skip_serializing_if` ⇒ **所有非列表命令的输出与今天逐字节相同**。现有 jq 脚本与
   skill 不受影响；`data[]` 仍是数组，不会变成 `data.items[]`。
2. 四种输出格式（Json / Text / Toon / Auto）**信封上的 `pagination` 键在全部四种格式
   中都会出现**，因为都是从同一个 `serde_json::to_value(信封)` 分流而来
   （`apps/cli/src/commands/output.rs`）。但人类可读信号并不因此均匀：Text 模式把
   信封交给对象格式化器渲染，`pagination` 只会作为一个嵌套对象出现在文本输出里，
   而不是一句警告；Text 模式下真正面向人的截断信号是 §5.3 描述的 stderr 警告，
   与 JSON/Toon/Auto 中作为结构化字段出现的 `pagination` 不是同一回事。这点关键：
   skill 消费的是默认 Auto（本仓库 33 条会走 TOON），键的存在能让受害最深的调用方
   读到信号。

### 5.2 不携带总数

`Paged<T>` 与 `PaginationMeta` 均不携带总数字段。本次范围内的三个平台都不能
廉价提供总数：`gh issue list --json` 的字段集中没有总数，取真实总数需另发一次
GraphQL/search 查询；gitlab、gitcode 同理未实测出廉价途径。保证项是
`truncated` + `returned`，不为总数多打一轮 API。

`Paged<T>` 标记了 `#[non_exhaustive]`，日后若某平台确实能廉价拿到总数，在
`Paged<T>` 上新增一个 `Option` 字段是非破坏性（non-breaking）的加法；
`PaginationMeta` 未标记 `#[non_exhaustive]`（见 §5.1 注）但同样只是加一个
`#[serde(skip_serializing_if = "Option::is_none")]` 字段，不破坏现有 JSON
消费方。不需要现在为它们预留字段。

### 5.3 人类可读警告

`truncated == true` 时向 **stderr** 输出：

```
⚠️  结果被截断：返回 1000 条，仓库中还有更多。用 --limit <N> 提高上限。
```

走 stderr 而非 stdout，保证 stdout 仍可直接喂 jq。

## 6. 分页器：`cli-adapter-utils::paging`

放在 `cli-adapter-utils` —— 与既有的 `CommandRunner`、`EnvSource`（#359 产出）同层，
不引入新概念。

```rust
pub const DEFAULT_LIST_LIMIT: u32 = 1000;

/// 平台的分页能力。用 enum 显式表达，而非靠算术巧合让循环在 gh 上恰好只跑一轮。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FetchStrategy {
    /// 底层 CLI 自行翻页，一次调用即可取至多 N 条（gh、gitcode）。
    SingleShot,
    /// 必须由调用方逐页请求，每页至多 `per_page` 条（glab）。
    Paged { per_page: u32 },
}

#[async_trait]
pub trait ListFetcher: Send + Sync {
    type Item: Send;
    fn strategy(&self) -> FetchStrategy;
    /// `SingleShot`：`page` 恒为 1，`limit` 由 `fetch_capped` 传入 `cap + 1`。
    /// `Paged`：取第 `page` 页（1-based），`limit` 由 `fetch_capped` 取自
    /// `FetchStrategy::Paged { per_page }`，即每页大小。
    async fn fetch(&self, page: u32, limit: u32) -> Result<Vec<Self::Item>>;
}

pub async fn fetch_capped<F: ListFetcher>(f: &F, cap: u32) -> Result<Paged<F::Item>>;
```

### 6.1 算法

要 `cap + 1` 条（N+1 探测）：

- `SingleShot`：一次调用，`limit = cap + 1`
- `Paged`：以 `per_page` 循环，直到累计 ≥ `cap + 1`，或某页短于 `per_page`（已取尽）

收到 `> cap` 条 ⇒ 截到 `cap`、`truncated = true`；否则 `truncated = false`。

N+1 探测是必需的：若只要 `cap` 条并恰好收到 `cap` 条，无法区分
「恰好有 cap 条」与「还有更多」。

### 6.2 gitcode 的未验证性与失效方向

gitcode CLI 本机不可用，声明为 `SingleShot` 并在代码注释标明未验证。

**失效方向是安全的**：若 gitcode 的 `--limit` 实际是页大小，N+1 探测会
**过度上报 `truncated`**，而不会静默丢数据 —— 即"吵但诚实"，与本 Issue 要消灭的
"静默丢失"方向相反。

## 7. 签名变更

唯一的 public API 破坏点。`Vec` 装不下 `truncated`，而 CLI 层看不见底层调用、
无从自行判断，因此绕不开：

```rust
// 前
async fn list(&self, args: ListIssueArgs) -> Result<Vec<IssueData>>;
// 后
async fn list(&self, args: ListIssueArgs) -> Result<Paged<IssueData>>;
```

三平台实现与相关测试同步调整。一并修两个已确认的 bug：

- `ReleaseProvider::list(&self)` → 接收 `ListReleaseArgs`，令 `gf release list --limit`
  真正生效（发现 2）
- `GitHubLabelProvider` 去掉硬编码 `--limit 100`，CLI 暴露 `--limit`（发现 3）

### 7.1 `limit` 语义变更

| | 前 | 后 |
|---|---|---|
| `None` | 随底层默认（30） | 取至 `DEFAULT_LIST_LIMIT` |
| `Some(n)` | 上限 n，无信号 | 上限 n，**同样跑 N+1 探测** |

用户要 10 条而实际有 33 条时也会 `truncated: true` —— 这是"截断信号"那一半的落点。

## 8. Skill 修正

改 `skills/` 下的源，**不改 `.claude/skills/` 副本**（CLAUDE.md 规定），收尾跑
`make check-agent-sync`。

- `skills/gf-issue-triage/SKILL.md`：默认调用即全量，无需加旗标；补一条
  「读 `pagination.truncated`，为真则报告必须声明覆盖不完整」
- `skills/gf-label-stats/SKILL.md`：删除第 100 行的假兜底（`--page` 参数不存在）；
  `--limit 1000` 可去掉（默认已是 1000）

## 9. 测试策略（TDD）

| 层 | 内容 |
|---|---|
| `fetch_capped` 单元测试 | 对 mock `ListFetcher` 跑边界：0 / cap-1 / cap / cap+1 / cap+2 条 × `SingleShot` × `Paged`。纯内存，不起子进程 |
| 适配器 | `MockCommandRunner` 断言**精确 argv**：gh `--limit 1001`、glab `--per-page 100 --page 1..`、api 路径含 `--paginate` |
| 回归护栏 | `gf release list --limit 10` 必须断言该值**抵达 provider**，而非仅断言解析结果 |
| 信封兼容 | 断言非列表命令输出**不含** `pagination` 键 |
| milestone 可测性 | `list_milestones` 改走 `runner`，使其可被 `MockCommandRunner` 断言 |

## 10. 验收对照（Issue #360）

| AC | 落点 |
|---|---|
| ① 默认全量或显式截断信号，二选一 | §3 两条都做 |
| ② 「静默丢数据」不得保留 | §5.1 信号覆盖四种输出格式；§5.3 stderr 警告 |
| ③ `pr list` / `label list` / `release list` 一并核查 | §1.2 发现 2/3/4 已核查，§4 一并修复；`pipeline list` 见 §4.1 记录不改 |
| ④ 自称全量的 skill 显式用全量调用 | §8 |
| ⑤ 可复现验证 | §11 |

## 11. 可复现验证（AC#5）

就用本仓库，无需造数据：

| 场景 | 修复前（已实测） | 修复后应为 |
|---|---|---|
| `gf issue list --state open` | 30 条，静默丢 #93/#101/#102 | 33 条，`truncated: false` |
| `gf issue list --state open --limit 10` | 10 条，无信号 | 10 条，`truncated: true` + stderr 警告 |

## 12. 已知遗留（不在本次范围）

1. `gf pipeline list` 硬编码 30 —— 见 §4.2，判定为有意的时间窗口
2. gitcode 的 `label list` / `milestone list` 截断无法探测 —— 见 §4.3，已知缺口
3. gitcode 适配器全程未经实测验证 —— 见 §6.2，失效方向安全但非零风险
4. 本次范围内三个平台均不携带总数 —— 见 §5.2，`Paged<T>`/`PaginationMeta` 均无
   `total_count` 字段；后续若某平台能廉价提供，可作为非破坏性加法补上
5. gitcode 的 `--limit` 取值范围未经验证，默认值出于保守选择 —— 见 §4.3.1（gitcode
   本机不可获得，无法实测其真实上限；若未来 gitcode CLI 可用，应验证真实上限并
   据此调整默认值）
