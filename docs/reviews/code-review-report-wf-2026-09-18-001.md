# Phase 4 交付后代码审查 — wf-2026-09-18-001（Issue #365）

**范围**：合并提交 `e77407d`（`--no-ff`，13 个提交，`feat/365-gitcode-list-pagination` → `dev`）
的交付后审查，交付区间 `5aded99..e77407d`。本审查不重复推导各任务级对抗审查、分支级审查
已经覆盖的内容，聚焦于：(1) 合并集成正确性，(2) 五条 gitcode list 路径是否真的消除了静默
截断，(3) 新引入的中间类型与 api 路径的健壮性，(4) 对已记录遗留项的诚实性核验。

仓库：`/Users/xs/Documents/workspce/github.com/byx-darwin/gitflow-cli`，分支 `dev`。
本次审查过程中为验证第 6 节的结论，曾在 `crates/gitcode/src/release.rs` 临时追加一个探针
测试并运行，随后已完整还原，`git status --porcelain` 为空，工作区无残留。

---

## 1. 交付了什么，为什么

Issue #360 声称消除了 `gf` 列表命令的静默截断。**在 gitcode 上它没有做到**：缺陷仍然存在，
只是把阈值从平台默认的 30 挪到了 100，并且依然报告 `truncated: false`。#365 是对这一
「已宣称修复、实际未修复」的返工。

#360 之所以选错方向，不是因为疏忽，而是因为当时**根本拿不到 gitcode CLI**，其设计文档
§4.3 / §4.3.1 里所有「无法验证，故保守处理」的记载，都是把推断当成事实写进了设计。
本次首次获得可用且已登录的 `gitcode-cli 0.12.0`，用实测把三条假设逐条推翻：

| #360 的假设 | #365 的实测事实 |
|---|---|
| gitcode 的 `--limit` 与 `gh` 同义，CLI 内部自动翻页取满 N | 只发一次 API 调用；`--per-page` 默认取 `--limit`，不翻页 |
| `--limit` 可能有范围限制，传 1001 会报错 | 不报错；真实行为是 `per_page` 被服务端**静默封顶**在 100 |
| `label list` / `milestone list` 是否支持分页旗标无从得知，故一律不传 | 两者都支持 `-L/--limit`、`--page`、`--per-page`，且页间不重叠 |

本次交付因此包含：

- `issue` / `pr` / `label` / `milestone` 四条 list 从 `FetchStrategy::SingleShot` 改为
  `FetchStrategy::Paged { per_page }`，显式传 `--per-page` / `--page`，**一律不再传 `--limit`**
- `release list` 改走 `gitcode api /repos/{repo}/releases?per_page&page`（实测该 CLI 子命令
  **没有任何分页旗标**，只有 `-L/--limit`），响应经私有中间类型 `ReleaseApiResponse` +
  `From` 映射到 core 的 `ReleaseData`
- 删除 crate 级常量 `GITCODE_DEFAULT_LIST_LIMIT`，五条路径统一回落 `DEFAULT_LIST_LIMIT`（1000）
- 补齐 argv 级回归护栏、一条「诚实报告未截断」的短页测试、一条针对公开仓库
  `openharmony/docs` 的只读 e2e，并让 `e2e-gitcode` 的 `noauth` 用例对配置文件登录态密封
- 作废两份设计文档中已被证伪的记载（#360 设计 §4.3 / §4.3.1 / §12，以及本次新设计 §3.2 的
  原判断）

## 2. 驱动这次改动的实测事实

取自 `docs/superpowers/specs/2026-09-18-gitcode-pagination-fix-design.md` §2，环境
`gitcode-cli 0.12.0`，样本 `openharmony/docs`（公开、issue 数 >200）：

- 缺陷链条（改动前的 gitcode `issue list`）：`cap = GITCODE_DEFAULT_LIST_LIMIT = 100` →
  `SingleShot` 的 N+1 探测发出 `--limit 101` → `per_page` 默认取 `--limit` 被静默封顶 →
  实回 100 条 → `truncated = (100 > 100) = false`。200+ issue 的仓库丢掉一半以上而不报警。
- 旗标矩阵：`issue/pr/label/milestone list` 均有 `--page` / `--per-page`；**`release list`
  三者皆无**，这与 #365 正文 AC#1 的假定不符，设计 §2.3 已如实标注该偏差。
- `--per-page` 优先于 `--limit`：`--limit 5 --per-page 3` 返回 3 条；单传 `--per-page 100`
  不受 CLI 默认 `--limit 30` 影响。
- `per_page` 超限静默封顶而非报错：`--per-page 101` 与 `per_page=1001` 均实回 100 条。
- `--json` 是布尔旗标而非 `gh` 那样的字段选择器：`label list --json "name,color"` 的位置
  参数被静默忽略。这使 argv 中删除 `LABEL_FIELDS` / `RELEASE_FIELDS` 有实测依据，不是猜测。

统一的页大小公式落在五处，完全一致：
`let per_page = cap.saturating_add(1).min(crate::GITCODE_API_MAX_PER_PAGE);`

## 3. 合并集成

- `e77407d^1` = `5aded99`（合并前的 `dev`），`e77407d^2` = `f5a0596`（分支尖端）。
  `git merge-base 5aded99 f5a0596` 返回 `5aded99` 本身，`git log 5aded99..f5a0596` 之外
  `git log f5a0596..5aded99` 为空 —— 分支起自 `dev` 的精确尖端，这是一次可快进但出于历史
  可读性用 `--no-ff` 执行的合并，没有发生任何冲突消解，也不存在被覆盖的并行工作。
- 全树 `git grep` 冲突标记（`<<<<<<<` / `>>>>>>>`）无命中。
- `git diff --stat 5aded99..e77407d` 为 12 个文件、+2640/−277，与提交信息声明的爆炸半径
  一致：5 个 gitcode 源文件 + `runner.rs` 测试基建 + 2 个 e2e 测试文件 + 3 份文档 + `docs/index.md`。
  **没有越界改动**：`apps/cli`、`crates/core`、`crates/github`、`crates/gitlab` 一行未动，
  这与「本次只修 gitcode 平台」的声明相符。

**结论**：合并集成干净、范围精确。

## 4. 代码质量评估

**正面**：

- **策略选择有据可依，且与仓库内既有正确实现同构**。`release list` 的 api 路径与本 crate
  已交付且正确的 `issue comments`（`crates/gitcode/src/issue.rs:594`）以及
  `crates/gitlab/src/release.rs` 的 `ReleaseApiResponse` 完全同构，是照搬既有模式而非新发明。
- **注释记录的是实测，不是推断**。`crates/gitcode/src/lib.rs` 里 `GITCODE_API_MAX_PER_PAGE`
  的文档注释被整体重写，且没有停在「已实测可用」这种舒适结论上，而是继续枚举了三种
  「端点不遵守分页约定」的偏离形态及各自后果，并明确点出**第三种（`per_page` 被静默调低）
  与第一种（两者都被忽略）会产生完全相同的「短页」现象，无法仅凭该观测区分**，因此
  「短页本身不能证明数据已取尽」。这是本次交付里质量最高的一段文字：它没有把
  `fetch_capped` 的短页终止条件包装成安全保证。
- **`url` 字段的语义统一被显式处理**。`api.html_url.or(api.url)` 保证 `list` 与 `view` 两条
  路径返回的 `url` 含义一致（网页地址而非 API self-link），并有两条测试分别钉住优先级与回退。
- **`--limit` 被有意剔除而非顺手保留**。设计 §3.1 给出的理由是「`--per-page` 已经决定单页
  大小，总量由 `fetch_capped` 的 `cap` 把关，再传一个 `--limit` 只会引入第二个语义不明的
  截断点」，测试里也有一条 `!first.iter().any(|a| a == "--limit")` 的断言把它钉死。
- **常量删除的时序被写进了计划的全局约束**：`GITCODE_DEFAULT_LIST_LIMIT` 的删除必须排在
  最后一个消费者之后，否则中间态会出现 `dead_code` 告警而被 `-D warnings` 拒绝。

**任务级对抗审查抓到的关键缺陷（应当被显著记录）**：

`release list` 的初版设计判断是「类型复用，不新增中间类型」，推理依据是 `ReleaseData`
的 Rust 字段名（`tag_name` / `created_at`）本身就是 snake_case，因此能直接吃下 gitcode 的
snake_case REST 响应。**这个推理混淆了 Rust 标识符与 serde 线上名**：`ReleaseData` 标注了
`#[serde(rename_all = "camelCase")]`，实际接受的字段名是 `tagName` / `createdAt` /
`publishedAt`。若按原设计实现，`gf release list --platform gitcode` 在**任何有 release 的
仓库上都会以 `missing field 'tagName'` 直接失败** —— 这不是边缘情形，是该命令的主路径。

值得警醒的是：**这与导致 #365 本身的错误是同一类** —— 把一个未经验证的假设当作事实写进
设计。#360 假设 gitcode 的 `--limit` 与 `gh` 同义；本次假设 `ReleaseData` 能吃 snake_case。
两者都是「看起来显然，所以没查」。区别只在于这一次被任务级对抗审查在落地前拦住了，而
上一次没有。修复方式（私有 `ReleaseApiResponse` + `From`）是正确的，并且实现者额外补了
`test_should_reject_snake_case_api_payload_when_using_core_release_data` 这条**成因测试**，
用两条断言（字段名不匹配、数字型 author id 类型不匹配）把「必须有中间类型」这一事实钉住，
防止日后有人把它当冗余抽象删掉。这是正确的应对：不只修缺陷，还给缺陷的成因立碑。

**规范符合性**：全部生产代码无 `unwrap()` / `expect()`（`expect` 仅出现在 `#[cfg(test)]`
的 mock 互斥锁上，并已按规范补 `# Panics` 段）；日志全部走 `tracing::debug!` 且以结构化
字段传 `repo` / `cap` / `per_page`；所有新增类型 derive `Debug`；`#[allow]` 均带 `reason =`。
`cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic` 在合并提交上通过。

## 5. 测试覆盖评估

本区间新增 17 个测试（16 个单测 + 1 条 e2e），命名全部遵循 `test_should_<expected_behavior>`。

**测试基建先行**：`SequencedMockCommandRunner` 此前签名为 `(_program, _args)`，**丢弃 argv**，
任何「断言 argv 含 `--per-page` / `--page`」的验收条款都无从落地。Task 1 先照搬 GitLab 侧
同名结构体（`crates/gitlab/src/runner.rs:161`）补上 `recorded_calls()`，使两侧实现一致。
`run_with_stdin` 委托给 `run`，因此同样被记录，文档注释的描述与实现相符。

**最值得称道的一点：测试参数的取值区间被论证过，而不是随手取的**。计划的全局约束明确写出：
`fetch_capped` 的 `want = cap + 1`，而 `per_page = min(cap+1, 100)`；当 `cap < 100` 时
`per_page` 恰为 `cap+1`，首页一次就满足 `want`，循环**只发一次调用** —— 页号递增无从观测，
断言 `calls[1]` 更会索引越界。因此所有翻页测试的 `cap` 必须取 100。中间提交
`bccde9c docs(plans): fix paging tests that could never observe page increment` 正是对这一
问题的修正。**一个只会发出一次调用的「翻页测试」是零判别力的**，这类测试比没有测试更危险，
本次在计划阶段就把它拦下了。

**子串断言被系统性替换为整串相等**。`"per_page=1001".contains("per_page=100")` 与
`"per_page=100".contains("page=1")` 都为真，因此子串断言既检测不出页大小钳位失效，也检测
不出页号错误。`issue comments` 既有的那条断言（`issue.rs:1092` 附近）也被顺手修正为
`assert_eq!(api_path, ".../comments?per_page=100&page=1")`。这不是本次 Issue 要求的，但它
修的正是「本测试得名的那个缺陷」，属于正确的顺手加固。

**诚实性方向的覆盖**：`test_should_report_no_truncation_when_gitcode_issue_list_ends_on_short_page`
（`cap=150`，首页 100 条 + 次页 20 条 = 120 条）同时验证三件事：跨页条目不丢失、短页时停止
翻页不多探一次、总数未达 cap 时**诚实报告 `truncated = false`**。之前的测试只覆盖「该报
截断时报了」，这条补上了反方向 —— 而 #365 的缺陷恰恰是反方向失真。

**e2e**：`crates/e2e-gitcode/tests/pagination.rs` 打公开只读仓库 `openharmony/docs`，断言
`items.len() > 100 || truncated`，并在模块注释里明确写出「本测试在 #365 修复前必然失败」
以及覆盖 `E2E_TEST_REPO_GITCODE` 时的陷阱（指向 issue 数 ≤100 的仓库会产生假通过，因为
无法区分「确实不足 100」与「恰在 100 处被截断」）。这个陷阱说明写得准确。

**`noauth` 密封**：原模块注释断言「`GitCodeAuthProvider` 没有本地配置文件状态，`env_remove`
单独即可保证确定性」——该前提已不成立，gitcode CLI 把登录态存在 `~/.config/gc/auth.json`，
致使两条用例在已登录的开发机上恒红（CI 无登录态故一直没暴露）。实测排除了
`XDG_CONFIG_HOME`（被忽略）与 `HOME`（pip wrapper 崩溃）两种手段后，改用
`GC_TOKEN=<无效值>`，并在注释里如实写出其代价：依赖一次真实的 token 校验请求，**离线环境
下报错形态可能不同**。这是合格的诚实标注。

**覆盖缺口**（见第 6、7 节）：`release list` 的非空响应形状**仅由 fixture 覆盖**；e2e 只
验证到 issue 一条路径，`pr` / `label` / `milestone` / `release` 四条改动路径没有任何真实
服务端验证。

**验证结果（合并提交处）**：

| 检查项 | 结果 |
|---|---|
| `cargo nextest run --workspace` | 1545 passed / 0 failed |
| `cargo +nightly fmt --check` | 通过 |
| `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic` | 通过 |

## 6. 本次审查发现的新问题

### F1（重要）`#[serde(default)]` 不覆盖类型不匹配，设计 §7 与代码注释的「整体退化」承诺不成立

**结论先行**：设计文档 §7 遗留 1 写道 ——

> `ReleaseApiResponse` 每个字段都标注了 `#[serde(default)]`，因此即便字段形状与假设有出入，
> 也只是该字段退化为默认值，**不会导致整个响应反序列化失败**

`crates/gitcode/src/release.rs` 的类型文档注释同样写着「形状不匹配时应可预期地退化而非
半途失败」。**这两句话是错的。** `#[serde(default)]` 只在字段**缺失**时生效；字段**存在但
类型不符**（或为 `null`）时，serde 直接报错，整个数组的反序列化随之失败，`gf release list`
整条命令返回错误。

本次审查用临时探针实测确认（探针已移除）：

| 输入 | 结果 |
|---|---|
| `{"id":"12","tag_name":"v1"}` | `invalid type: string "12", expected u64` |
| `{"id":null,"tag_name":"v1","draft":null}` | `invalid type: null, expected u64` |
| `{"tag_name":"v1","created_at":"2026-01-01 00:00:00"}` | 反序列化失败（非 RFC3339） |

**为什么这不是理论风险，而是这个平台上的高概率事件**：gitcode 的 REST 响应**已被本仓库
证实会把数字 ID 编码为 JSON 字符串**。证据有二，都在本次改动内：

1. `crates/gitcode/src/issue.rs:27` 的 `IssueApiResponse.number` 类型就是 `String`，本次新增的
   fixture `issue_page_json` 也确实以 `"number":"{n}"`（带引号）构造 —— 说明 issue 端点的
   编号是字符串。
2. 本次为 `ReleaseUserApi.id` **专门**挂了 `deserialize_u64_or_string_to_string`，注释写明
   「gitcode api 的 `id` 可能是数字也可能是字符串」，并配了
   `test_should_accept_string_author_id_from_api` 覆盖字符串型 id。

也就是说，实现者已经**明确知道** gitcode 会把 id 编码为字符串，并对 author 的 id 做了防护，
却把 release 自身的 `id` 留作裸 `u64`。一旦 `/repos/{repo}/releases` 返回 `"id": "12"`，
整条 `release list` 直接失败 —— 而这一路径恰恰是**唯一没有任何真实服务端验证**的路径
（§7 遗留 1 自己承认：本机找不到带 release 的公开 gitcode 仓库）。

**为什么现有测试没有发现**：`test_should_degrade_predictably_for_minimal_api_release_object`
只喂了 `{"tag_name": "v0.0.1"}`，即**字段全部缺失**的场景 —— 这正是 `#[serde(default)]`
唯一能兜住的情形。测试验证的是 `default` 的定义域，而文档声称的是 `default` 之外的值域。

**这是第三次同类错误**。#360 假设「`--limit` 与 `gh` 同义」；本次初版假设「`ReleaseData` 能吃
snake_case」；现在是假设「`#[serde(default)]` 能兜住形状偏差」。三次都是把一个未验证的
推断直接写成结论性断言，而且这一次它被写在了**专门用于记录诚实缺口的「已知遗留」章节里**
—— 一个本应降低风险的段落，反而提供了虚假的安全感。

**建议修法**（按优先级）：

1. 给 `ReleaseApiResponse.id` 挂 `deserialize_u64_or_string_to_string` 的 u64 版本（或先解析
   为字符串再 `parse().unwrap_or(0)`，与 `IssueApiResponse.number` 的既有做法一致）；
2. 把 `tag_name` / `draft` / `prerelease` 改为对 `null` 容忍（`Option<T>` + `unwrap_or_default`，
   而非 `#[serde(default)] T`）—— Gitee 血统的 API 用 `null` 表达「无此概念」是常见做法；
3. **立即修正设计 §7 遗留 1 与 `release.rs` 的类型文档注释**：删掉「不会导致整个响应反序列化
   失败」这句，改写为「`#[serde(default)] `仅覆盖字段缺失；类型不符或 `null` 会使整条
   `release list` 失败，且该路径无真实服务端覆盖」。即使 1、2 两条因范围原因暂不做，
   第 3 条也必须做 —— 留一个错误的安全承诺，比留一个如实标注的缺口危险得多。
4. 补一条测试，喂入字符串型 `id` 与 `null` 字段，把修好后的实际容忍边界钉住。

**严重度定级说明**：定为「重要」而非「严重」，因为它不影响已合并代码在**当前可观测行为**
下的正确性（本机找不到带 release 的 gitcode 仓库，因而没有已知的复现环境）；但它是一条
主命令路径上的硬失败风险，且被一段错误的文档掩盖，不宜按「次要」处理。

除 F1 外，本次审查未发现新的正确性、安全或静默失败问题。

## 7. 延续的已知缺口

| # | 缺口 | 评估 |
|---|---|---|
| 1 | `release list` 的 api **非空**响应字段形状未经真实服务端确认 —— 本机探测了 `openharmony/docs`、`openharmony/build`、`openharmony/third_party_node`、`openharmony/interface_sdk-js`、`openharmony/kernel_linux_5.10`、`apache/dubbo`，均为空列表 | 缺口本身如实记录、探测过程可追溯。但其**缓解措施的描述是错的**，见 F1 —— 缺口的实际严重度高于文档所述 |
| 2 | `ReleaseApiResponse` → `ReleaseData` 的 `created_at` 在 api 未返回该字段时回退 `Utc::now()` | 定级准确：仅用于展示，且做法照搬自 gitlab 侧同构实现，不是本次新引入的债。诚实的修法是把 core 层 `ReleaseData::created_at` 改成 `Option<DateTime<Utc>>` 并在 github / gitlab / gitcode 三平台一并改动，范围超出 #365，有意不做 |
| 3 | **本次交付完全绕过 CI**：仓库的 `ci.yml` 有意排除对 `dev` 的 push，local_merge 路径不触发任何流水线 | 这是三条缺口里影响面最大的一条。1545 条测试与三项静态检查全部只在本机跑过，**没有任何独立环境复核**。叠加缺口 4，e2e 的实际有效覆盖比表面看到的更窄 |
| 4 | e2e `pagination.rs` 在探测失败（未装 gitcode CLI 或未登录）时静默 `return` 视作 skip | 设计上是有意的（否则 CI 恒红），但后果是：在任何没有登录态的环境里这条测试**恒为空跑并报告通过**。叠加缺口 3（根本不跑 CI），该 e2e 的实际守护范围仅限于「开发者本机、已登录、手动运行」 |
| 5 | e2e 只覆盖 `issue list` 一条路径；`pr` / `label` / `milestone` / `release` 四条同样改动过的路径无真实服务端验证 | 未在设计 §7 中单列。`pr` / `label` / `milestone` 的分页行为有直接实测支撑（§2.2 记录了 `label list --per-page 2 --page 1/2` 不重叠），风险较低；`release` 见 F1 |
| 6 | `LABEL_FIELDS` / `RELEASE_FIELDS` 在 gitcode 侧其余调用点仍是死参数（`label.rs:236`、`release.rs:179,307`） | 如实记录于设计 §3.3 / §7 遗留 3，本次只清理重写到的两条 argv。这些调用点（如 `fetch_label`）还同时**绕过 `runner` 直接用 `tokio::process::Command`**，argv 无法被测试观测 —— 属先于本次的既有债，不在 #365 范围 |
| 7 | `noauth` e2e 依赖一次真实 token 校验请求，离线环境下报错形态可能不同 | 已在模块注释中如实标注为该手段的代价 |

## 8. 后续建议

1. **（承接 F1，最高优先）** 修正设计 §7 遗留 1 与 `release.rs` 类型注释中关于
   `#[serde(default)]` 的错误承诺，并给 `ReleaseApiResponse.id` 加上字符串/数字双容忍。
   这两处一起改，工作量很小，但它消除的是一条主命令路径上被文档掩盖的硬失败风险。
2. **`created_at` 的核心类型修正**：把 `ReleaseData::created_at` 改为 `Option<DateTime<Utc>>`，
   在 github / gitlab / gitcode 三平台同步落地，消除 `Utc::now()` 兜底这一「用当前时间冒充
   创建时间」的展示层谎言。这是一次跨 crate 的破坏性改动（`gitflow-core` 已发布至 crates.io），
   需要 `BREAKING CHANGE` footer，建议与下一次 major 版本一并安排。
3. **关注 gitcode 侧的子进程扇出**：改为 `Paged` 之后，默认 `cap = DEFAULT_LIST_LIMIT = 1000`、
   `per_page = 100`，意味着在超大仓库上 `gf pr cleanup` 与按名称解析 label 的路径最多会**串行
   spawn 11 次子进程**。这比之前「只取首页却声称完整」**更正确**，但目前**既无超时也无进度
   输出**，用户会看到一段无反馈的长时间挂起。建议为分页循环加一个整体超时与 stderr 上的进度
   提示（例如每页输出一次 `debug!` 之外的可见进度）。
4. **把 `dev` 纳入 CI 触发，或为 local_merge 路径补一道等效闸门**（缺口 3）。本次 1545 条测试
   全部只在单台开发机上跑过；缺口 4 又让唯一的真实服务端 e2e 在无登录态环境里空跑。两者
   叠加后，「全绿」这一结论的独立性实际上为零。
5. **修正 `cargo-fmt` pre-commit hook 的行为**：该 hook 当前**只打印 diff 而不应用格式化**，
   本次交付中先后有四名实现者被它绊住（看到 diff 却不知道需要自己再跑一次 `cargo +nightly fmt`）。
   四次重复踩同一个坑，说明这不是个人疏忽而是工具行为的可用性缺陷。注意 `.pre-commit-config.yaml`
   的 hook 类型与阶段配置受项目规约保护，**变更需先征得用户确认**。
6. **给 e2e 的 skip 路径加显性信号**（缺口 4）：至少让空跑在测试输出中留下无法被误读为
   「已验证」的标记，避免「CI 绿」被当成「gitcode 分页已在真实服务端验证」。

## 9. 结论

**可以保持合并状态并随下一次发布出货。** 合并集成干净（可快进、无冲突、无并行工作被覆盖），
范围精确限定在 gitcode 平台，`apps/cli` 与 core / github / gitlab 一行未动。五条 list 路径
的静默截断在 gitcode 上被真正消除，且每一条策略选择都有实测而非推断支撑 —— 这正是 #365
相对 #360 的根本改进。测试质量高于平均水准：翻页测试的取值区间被论证过（而非随手取到一个
零判别力的区间），子串断言被系统性替换为整串相等，并补上了「诚实报告未截断」这一反方向覆盖。

本次审查发现一个新问题 F1：设计 §7 与代码注释关于 `#[serde(default)]` 能保证「整体退化而非
失败」的承诺是错的，而 `ReleaseApiResponse.id` 的裸 `u64` 在 gitcode 这个已知会把 ID 编码为
字符串的平台上，构成一条主命令路径的硬失败风险；该路径又恰好是本次唯一完全没有真实服务端
覆盖的路径。它不影响当前可观测行为，但应在下一次触碰该文件时优先修掉，**文档的修正尤其
不应推迟** —— 一段错误的安全承诺，比一个如实标注的缺口更危险。

值得作为流程教训记录的是：本次交付中出现的三个问题（#360 的 `--limit` 假设、初版的
`ReleaseData` 复用假设、以及 F1 的 `serde(default)` 假设）**属于同一类错误：把未经验证的
推断写成结论性断言**。第一个造成了这个 Issue，第二个被任务级对抗审查在落地前拦住，第三个
藏在专门用于记录诚实缺口的章节里直到本次审查。对抗审查有效，但覆盖面还没有延伸到文档中的
断言本身 —— 建议后续把「文档里每一条形如『不会 / 必然 / 保证』的断言是否有实测或类型系统
支撑」显式列入审查清单。
