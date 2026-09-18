# Refactor Report — gitflow-cli
生成时间：2026-09-18 · 目标范围：paging.rs 及 gf-smell 同批候选 · 应用手法数：1 · 建议手法数：1

## 已应用（等价，自动应用）
### RF-001 · Extract Function 提炼函数 · commit d4c14d48197dbb8b0af2c47b0d674de5ac015656
- 位置：crates/core/src/paging.rs（提炼前 `fetch_capped` 内 `FetchStrategy::Paged` 分支的分页循环）→ 新增私有函数
  `fetch_all_pages<T, F, Fut>(per_page: u32, want: usize, fetch: &F) -> Result<Vec<T>>`
- 提炼范围：`per_page == 0` 校验、`let mut page = 1; loop { .. }` 分页循环（两个 `break` 与
  `page.checked_add` 溢出错误）整体搬入新函数；`fetch_capped` 的 `SingleShot` 分支、`want` 计算、
  截断逻辑与 `Paged` 结构体构造保持原位不变
- 校验：编译 ✓（`make check`，`cargo clippy -p gitflow-core --all-targets --all-features -- -D warnings -W clippy::pedantic` 无新增告警）测试 ✓（`make test`：1549/1549 通过，含 `paging` 模块 15 个测试全绿，其中
  `test_should_stop_paging_on_short_page`、`test_should_walk_pages_until_cap_plus_one_collected`、
  `test_should_probe_one_past_cap_when_cap_is_a_multiple_of_per_page`、
  `test_should_stop_paging_when_exhausted_exactly_at_page_boundary`、
  `test_should_preserve_item_order_across_pages`、`test_should_reject_zero_per_page` 六个用例直接覆盖
  `FetchStrategy::Paged` 分支，构成本次提炼的行为基线）
- 附带清理：提炼后 `fetch_capped` 内 `let mut items: Vec<T> = Vec::new();` 后接 `match` 分别赋值的写法
  会触发 `unused_assignments` 告警（两条分支都变成整体覆盖赋值），已改写为
  `let mut items: Vec<T> = match strategy { .. };`，纯语法改写、返回值与两条分支结果完全一致，无行为变化

## 建议（条件等价 / 可能变更，未自动应用）
### RF-101 · Extract Function 提炼函数 · 条件等价
- 位置：apps/cli/src/commands/pr.rs:214、issue.rs:175、release.rs:138、label.rs:264
- 证据强度：Observed（阅读 pr.rs:214 完整函数体已确认 4 个内联分支的具体位置）
- 置信度：Medium
- 严重度：Medium
- 风险说明：这 4 处 `match` 臂内联了参数解析与三路分支逻辑，提炼前需逐臂确认
  是否存在提前 return 或跨臂共享的局部变量（Slide Statements 的同类风险）；
  与本次 paging.rs 处理的纯分页循环不同，不能直接套用同一提炼边界
- 前置条件：逐处人工核实臂内是否有跨语句的隐含顺序依赖；未验证
- 建议：待用户确认后逐处应用 Extract Function，每处单独校验、单独 commit

**本次 dogfooding 运行发现的前置阻塞项（记录以免遗失）**：原计划的 Task 4 目标是
`apps/cli/src/commands/label.rs::handle_label`（provider 选择 `match`），但实地核实
（Step 1 gate）发现该函数、以及同批 `pr.rs::handle_pr`、`issue.rs::handle_issue`、
`release.rs::handle_release`、`label.rs::handle_milestone` 这 4 个 CLI 命令分发函数在
仓库中**完全没有测试覆盖**——`apps/cli/tests/*_test.rs` 仅对这些命令做了 `--help`/clap
解析层面的断言，未见任何用例真正驱动 `handle_*` 本体、走到 `platform` 选择分支。按
`skills/gf-refactor/SKILL.md` 的「When NOT to Refactor」硬性规则（无测试覆盖的代码路径
一律拒绝重构，不得擅自补测试），原目标被判定 BLOCKED 并已改用有真实测试覆盖的
`crates/core/src/paging.rs::fetch_capped` 作为本次 dogfooding 的提炼对象（见上方 RF-001）。
这 5 个 `handle_*` 分发函数（含 `label.rs::handle_label` 本身）目前仍是重构候选中的
覆盖缺口，需要先补测试才能被本 skill 安全处理。

## 已排除
（无 — 本次范围仅取自 gf-smell 既有报告的候选，未新增排除项）
