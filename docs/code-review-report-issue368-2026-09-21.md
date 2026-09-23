# Code Review Report — Issue #368

> **交付方式：** `local_merge`（`git merge --no-ff`），无 PR 编号可挂载正式 GitHub review verdict。本报告复用 Phase 3 执行阶段已完成的独立代码审查，作为 Phase 4 的书面存档。
> **Merge commit:** `4bba6b50c9c8fac7addb043ad6ca267656d29e1a`
> **Diff range:** `dev(0b7bcbd)..4bba6b5`（`crates/gitcode/src/release.rs`、`docs/superpowers/specs/2026-09-18-gitcode-pagination-fix-design.md`）
> **分析日期：** 2026-09-21
> **工作流：** `wf-2026-09-20-007`

## 背景

Issue #368：gitcode `release list` 的 api 路径非空响应形状从未经过真实服务端验证，此前两次教训（#360、F1）都证明"想当然"在这条路径上不可靠。用自有测试仓库 `byx-darwin/NexaTrade` 创建两个真实 release，实测 `gitcode api /repos/{repo}/releases?per_page=N&page=M`。

## 审查结论（独立复核）

**Spec compliance: ✅**（Issue #368 六项 AC 逐条核实）
1. 找到/创建带 release 的自有仓库——`byx-darwin/NexaTrade`，创建 2 个真实 release
2. 实测真实响应——直接打 `gitcode api` 端点，非模拟
3. 字段核对——`id`/`draft`/`html_url`/`url`/`published_at` 完全缺失（非 `null`）、`author.id` 字符串编码、`created_at` 带时区偏移，均已记录
4. 若有出入修正映射——**无出入，无需修改**，该结论有依据：`ReleaseApiResponse` 每个字段都是 `Option<T>` + `#[serde(default)]`，"缺失"与"`null`"在 serde 语义下走同一条退化路径；新增测试 `test_should_deserialize_real_gitcode_release_api_response` 用真实捕获 payload 验证，跑通
5. `page` 参数生效性——手动实测 `page=1`/`page=2` 各 `per_page=1` 返回不重叠记录，如实记入 commit message 与设计文档，未伪造成自动化测试
6. 设计文档 §7 更新——以追加方式记入闭合结论，未篡改或删除此前 F1 两轮订正的历史记录

**Code quality: Approved**
- 范围精确：仅测试文件 + 设计文档，未触碰 `crates/github`/`crates/gitlab`
- 无 panic 路径：变更为纯测试新增，`.expect()` 均在测试断言内，符合项目惯例
- 测试真实性：新测试的断言检查的是"退化到默认值"的具体行为（`id==0`、`!draft`、`url.is_empty()`、`published_at.is_none()`），不是"不 panic 就算过"
- `cargo clippy -p gitflow-gitcode --all-targets --all-features -- -D warnings` 无告警

**Minor（不阻塞）**：新测试 fixture 里 `prerelease` 字段是真实存在的值（`false`），不是缺失场景，该项断言检查的是正常路径而非退化路径——不影响其余字段的退化路径验证有效性。

## 收尾说明

清理测试 release 时发现 `gitcode release delete`（及裸 `api -X DELETE`）返回 `HTTP 405: Request method 'DELETE' not supported`——平台本身不支持该操作，非本项目工具链缺陷。两个测试 release 已如实记录在 Issue 评论中，保留在 `byx-darwin/NexaTrade` 上（名称已标注测试用途，无害）。

## 结论

**Approve.** 无 Critical/Important 发现，1 条 Minor 不阻塞。Issue #368 的六项 AC 全部收口。因交付方式为 `local_merge`，本报告即为正式书面存档，不额外调用 `gf review` CLI 提交动作。
