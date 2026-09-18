# gitcode 列表命令分页：用实测替换推断，消除残留的静默截断

- **Issue**：[#365](https://github.com/byx-darwin/gitflow-cli/issues/365)
- **Workflow**：`wf-2026-09-18-001`（standard 模式）
- **日期**：2026-09-18
- **状态**：设计已确认，待实现
- **前序设计**：[2026-09-17-list-pagination-design.md](./2026-09-17-list-pagination-design.md)（#360）

## 1. 问题

#360 声称消除 `gf` 列表命令的静默截断。**在 gitcode 上它没有做到**：缺陷仍然存在，
只是阈值从平台默认的 30 挪到了 100，且依然报告 `truncated: false`。

交付后的 gitcode `issue list` 路径：

1. `cap = GITCODE_DEFAULT_LIST_LIMIT = 100`（`crates/gitcode/src/lib.rs:75`）
2. `FetchStrategy::SingleShot` 令 N+1 探测发出 `--limit 101`
   （`crates/gitcode/src/issue.rs:306`）
3. gitcode 的 `--per-page` 默认取 `--limit`，即 101 → 被 API 静默封顶 → 实回 100 条
4. `fetch_capped` 判定 `truncated = items.len() > cap` = `100 > 100` = **false**

结果：gitcode 上一个有 200+ issue 的仓库，`gf issue list` 返回 100 条并报告
`truncated: false`，调用方无从知晓丢了一半以上。`pr list`（`pr.rs:227,234`）与
`release list`（`release.rs:148,154`）同构，同样受影响。

### 1.1 根因：#360 的策略选择基于三个当时无法证伪的假设

#360 全程无法获得 gitcode CLI，其设计文档 §4.3 / §4.3.1 中所有「无法验证，故保守
处理」的记载，本次都可以用实测替换 —— 而结论是当时的保守处理**方向选错了**。

| #360 当时的假设 | 实测事实 |
|---|---|
| `--limit` 与 `gh` 同义：CLI 内部自动翻页取满 N | 只发一次 API 调用；`--per-page` 默认取 `--limit`，不加 `--paginate` 不翻页 |
| `--limit` 可能有范围限制，传 1001 会报错 | 不报错；真正的行为是 `per_page` 被静默封顶在 100 |
| `label list` / `milestone list` 是否支持 `--limit` 无从得知，故一律不传 | 两者都支持 `-L/--limit`、`--page`、`--per-page` |

值得一提的对照：#360 的 Task 7 为 gitcode 的 `issue comments` 选的**正是**
`Paged { per_page }`（`crates/gitcode/src/issue.rs:594`），那条路径是**正确**的。
错的只是几个 list 子命令。

## 2. 底层 CLI 能力（已实测，非推断）

环境：`gitcode-cli 0.12.0`（`pip install gitcode-cli`，落在
`~/Library/Python/3.14/bin/gitcode`），已登录 `gitcode.com`。
样本：`openharmony/docs`（公开仓库，issue 数 >200）。

### 2.1 旗标矩阵

| 子命令 | `-L/--limit` | `--page` | `--per-page` | `--paginate` |
|---|---|---|---|---|
| `issue list` | ✅ 默认 30 | ✅ | ✅（默认取 `--limit`） | ❌ |
| `pr list` | ✅ 默认 30 | ✅ | ✅ | ✅ |
| **`release list`** | ✅ 默认 30 | **❌** | **❌** | **❌** |
| `label list` | ✅ 默认 30 | ✅ 默认 1 | ✅ | ❌ |
| `milestone list` | ✅ 默认 30 | ✅ 默认 1 | ✅ | ❌ |

### 2.2 行为实测

```
# CLI 层翻页真实生效，编号不重叠
issue list --per-page 3 --page 1  → 109956,109955,109954
issue list --per-page 3 --page 2  → 109953,109952,109951

# --per-page 优先于 --limit：单传 --per-page 100 未被默认 limit 30 截断
issue list --per-page 100 --page 1 → 100 条
issue list --limit 5 --per-page 3 --page 1 → 3 条

# per_page 静默封顶 100（不报错）
issue list --per-page 101 --page 1 → 100 条

# label 分页同样生效且不重叠
label list --per-page 2 --page 1 → severity/high, severity/critical
label list --per-page 2 --page 2 → kind/defect, 40d9348...

# api 端点可用，支持前导斜杠形式（与 issue comments 现有写法一致）
api "/repos/openharmony/docs/issues?per_page=2&page=1" → 109957,109956
api "repos/openharmony/docs/releases?per_page=2&page=1" → []（该仓库无 release，非报错）
```

### 2.3 与 Issue #365 正文的一处偏差

#365 的验收条款 AC#1 假定 `release list` 与 `issue list` / `pr list` 同样支持
`--page` / `--per-page`。**实测不支持** —— 该子命令只有 `-L/--limit`。本设计因此为
`release list` 单列一条路径（见 §3.2），其余两处按 AC#1 原意处理。

### 2.4 附带发现：`--json` 是布尔旗标

`gitcode label list --json "name,color"` 的第二个位置参数被**静默忽略**（返回全部
字段）。gitcode 的 `--json` 是布尔旗标，而非 `gh` 的字段选择器。因此
`LABEL_FIELDS`（`label.rs:80`）与 `RELEASE_FIELDS`（`release.rs:20`）作为 argv 参数
在 gitcode 侧是死参数。

## 3. 设计

### 3.1 策略矩阵变更

| 命令 | 现状 | 改为 | 依据 |
|---|---|---|---|
| `issue list` | `SingleShot` + `--limit` | `Paged{per_page}` + `--per-page` / `--page`，**不再传 `--limit`** | §2.2：`--per-page` 优先于 `--limit` |
| `pr list` | `SingleShot` + `--limit` | 同上（不用 `--paginate`，需受控翻页） | 同上 |
| `release list` | `SingleShot` + `--limit` | 改走 `gitcode api` + `Paged{per_page}`，见 §3.2 | §2.1：CLI 无分页旗标 |
| `label list` | `SingleShot`，不传任何旗标 | `Paged{per_page}` + `--per-page` / `--page` | §2.1 / §2.2 |
| `milestone list` | 同上 | 同上 | 同上 |

统一页大小公式，与 GitLab 侧及 `issue comments` 同构：

```rust
let per_page = cap.saturating_add(1).min(crate::GITCODE_API_MAX_PER_PAGE);
```

不传 `--limit` 是有意的：`--per-page` 已经决定单页大小，总量由 `fetch_capped` 的
`cap` 把关，再传一个 `--limit` 只会引入第二个语义不明的截断点。

### 3.2 `release list` 改走 api 端点

`gitcode api /repos/{repo}/releases?per_page={}&page={}`，策略 `Paged{per_page}`，
与已交付且正确的 `issue comments` 路径（`issue.rs:594`）完全同构。

**类型复用，不新增中间类型**：`ReleaseData` 的主字段名已是 snake_case
（`tag_name` / `created_at` / `published_at` / `draft` / `prerelease`），与 GitCode
REST 响应同构；`id` / `body` / `author` / `url` 均已 `#[serde(default)]` 或 `Option`，
api 返回的多余字段不影响反序列化。gitcode CLI 的 Go 二进制中可见
`json:"tag_name"` / `json:"prerelease"` 等 snake_case tag，佐证两侧形状一致。
实现时用一份 fixture 单测钉住该形状。

`release` 的其他方法（`create` / `view` / 资源上传下载）**不动**，仍走 CLI 子命令。

**已知验证缺口**：本机未能找到带 release 的公开 gitcode 仓库（探测了
`openharmony/docs`、`openharmony/build`、`openharmony/third_party_node`、
`openharmony/interface_sdk-js`、`openharmony/kernel_linux_5.10`、`apache/dubbo`，
均为空列表）。因此端到端只验证到「端点存在、空列表不报错」，非空响应的字段形状
由 fixture 单测覆盖，未经真实服务端确认。此项如实记入 §7 遗留。

### 3.3 常量与默认值

- **删除** `GITCODE_DEFAULT_LIST_LIMIT`（`lib.rs:75`）。改为 `Paged` 之后该保守默认
  不再需要：页大小已被 `min(GITCODE_API_MAX_PER_PAGE)` 钳住，`cap` 不再直接出现在
  任何 argv 里，`cap = 1000` 不会再产生 `--limit 1001` 这类越界探测值。五处 `cap`
  统一回落 `DEFAULT_LIST_LIMIT`（1000），与 github / gitlab 一致。
- **保留** `GITCODE_API_MAX_PER_PAGE = 100`，并把其文档注释中「是否真的支持
  `per_page`/`page` 未经实测」改写为 §2.2 的实测结论。
- 移除本次重写的两条 list argv 中被静默忽略的 `LABEL_FIELDS` / `RELEASE_FIELDS`
  位置参数（见 §2.4）。**仅限这两条 argv**，其余调用点（`label.rs:221`、
  `release.rs:95,216,688`）不在本次范围内。

### 3.4 测试基建：让 argv 断言成为可能

gitcode 的 `SequencedMockCommandRunner`（`crates/gitcode/src/runner.rs:210`）当前
**丢弃 argv**（签名为 `_program`、`_args`），无法支撑「断言各 list 的 argv 含
`--per-page` 与 `--page`」这条验收条款。

按 GitLab 侧**已有的同名结构体**（`crates/gitlab/src/runner.rs:161`）补上
`recorded: Arc<Mutex<Vec<(String, Vec<String>)>>>` 与 `recorded_calls()`。这是照搬
仓库内既有模式，非新发明；两侧实现届时保持一致。

## 4. 测试策略（TDD）

每个 list 子命令一组测试，RED 阶段必须先失败：

1. **首个响应必须是满页**（`per_page` 条）。否则翻页循环在第一次调用后就因短页
   终止，测试对「是否真的翻页」无判别力 —— 这是本轮回归护栏的核心要求。
2. 断言每次调用的 argv 含 `--per-page` 与 `--page`；`release list` 断言 api path
   含 `per_page=` 与 `page=`。
3. 断言页号在翻页中 1 → 2 递增。
4. 断言 `truncated == true` 且返回条数 `== cap`。
5. `release list`：额外一份 fixture 单测，钉住 api 响应 → `ReleaseData` 的字段映射。

## 5. 端到端验证

新增 `crates/e2e-gitcode/tests/pagination.rs`：

- 默认只读公开仓库 `openharmony/docs`（issue 数 >200，已实测），可由
  `E2E_TEST_REPO_GITCODE` 覆盖；**只读，不写入**。
- 断言 `gf issue list --limit 150` 要么返回 150 条、要么返回上限内条数并报告
  `truncated: true`。
- **修复前该断言必然失败**：今天返回 100 条且 `truncated: false`。

这同时回答 Issue #365 的 AC#7：gitcode 不必再停留在「仅 mock 验证」状态。

## 6. 前序设计文档的修订

`docs/superpowers/specs/2026-09-17-list-pagination-design.md`：

- **§4.1** 策略矩阵的 gitcode 列按 §3.1 更新
- **§4.3** 用 §2.1 / §2.2 的实测结论替换「无法保证，如实记录」
- **§4.3.1** 整节重写：`GITCODE_DEFAULT_LIST_LIMIT` 已删除，其「残留风险」段落
  （`--limit 101` 可能越界）已被实测证伪 —— 真实行为是静默封顶而非报错，删除该记载
- **§12** 移除第 2 条（label/milestone 截断无法探测）、第 3 条（gitcode 全程未实测）、
  第 5 条（`--limit` 取值范围未验证）；新增 `release list` 无分页旗标的实测记录

## 7. 已知遗留

1. `release list` 的 api 非空响应字段形状未经真实服务端确认 —— 见 §3.2，
   由 fixture 单测覆盖，本机无带 release 的 gitcode 仓库可验
2. `LABEL_FIELDS` / `RELEASE_FIELDS` 在 gitcode 侧其余调用点仍是死参数 —— 见 §3.3，
   本次只清理重写到的两条 argv
3. `gf pipeline list` 硬编码 `--limit 30` —— 沿用 #360 §4.2 的判定：有意的时间窗口，
   不是完整性声明，不改

## 8. 验收对照（Issue #365）

| AC | 本设计的处理 |
|---|---|
| 1. 三个 list 改 `Paged` + `--page`/`--per-page` | §3.1；`release list` 因实测无分页旗标改走 api（§3.2），偏差已在 §2.3 说明 |
| 2. `label list` / `milestone list` 启用分页 | §3.1 |
| 3. 移除 `GITCODE_DEFAULT_LIST_LIMIT` | §3.3 |
| 4. 回归护栏：argv 含 `--per-page`/`--page`、页号递增、首响应满页 | §3.4 + §4 |
| 5. 端到端验证，修复前必须失败 | §5 |
| 6. 设计文档 §4.3 / §4.3.1 / §12 更新 | §6（另含 §4.1 矩阵） |
| 7. 评估 `crates/e2e-gitcode` 纳入常规验证 | §5：纳入，默认只读公开仓库 |
