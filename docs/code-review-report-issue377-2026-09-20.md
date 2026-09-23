# Code Review Report — Issue #377

> **交付方式：** `local_merge`（`git merge --no-ff`），无 PR 编号可挂载正式 GitHub review verdict。`gf review approve/request-changes` 的 CLI 提交步骤因无 open PR 而不适用。本报告复用 Phase 3 执行阶段已完成的独立代码审查（非自审——由独立 subagent 在实现完成后重新读取 diff 与被镜像的 GitLab 实现做的审查），作为 Phase 4 的书面存档。
> **Merge commit:** `1bf8c6704c30dac955498ed11513a83c5a4ea9b0`
> **Diff range:** `dev(38e6fab)..1bf8c67`（仅 `crates/gitcode/src/label.rs`）
> **分析日期：** 2026-09-20
> **工作流：** `wf-2026-09-20-006`

## 背景

Issue #377：GitCode milestone `due_on` 字段在真实 API 返回纯日期格式（如 `"2026-08-31"`，无时间/时区）时，被 `DateTime::parse_from_rfc3339` 解析失败后静默丢弃为 `None`。修复照搬 `crates/gitlab/src/label.rs:508-521` 已验证正确的降级逻辑：先 RFC3339，失败则退回 `chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d")`，按当天 UTC 零点构造。

## 审查结论（独立复核，非自审）

**Spec compliance: ✅**（Issue #377 四项 AC 逐条核实）
1. 纯日期正确解析——`test_should_default_issue_counts_when_absent_from_real_list_response` 断言 `due_on == Some(2026-08-31T00:00:00Z)`，通过
2. 完整 RFC3339 回归不受影响——`test_should_deserialize_milestone_api_response`（`2026-06-01T00:00:00Z`）与 `test_should_convert_milestone_api_to_data`（`2026-12-01T00:00:00Z`）未改动且仍通过
3. 新测试不依赖网络——纯内存 `serde_json::from_slice` 反序列化，无 I/O
4. GitHub 同类风险已核查——`crates/github/src/label.rs:348-350` 逻辑同样脆弱，但 GitHub REST API 文档承诺 `due_on` 恒为完整 ISO8601，现有测试也全部用完整时间戳，无纯日期证据，结论"不改代码"合理

**Code quality: Approved**
- 无 panic 路径：新增代码全部走 `.ok()`/`.and_then()`/`.map()`，无 `unwrap()`/`expect()`
- 范围精确：仅 `crates/gitcode/src/label.rs` 一个文件
- 相比被镜像的 GitLab 实现更简洁：GitLab 版本有一段永远不可达的死代码兜底（`and_hms_opt(0,0,0).unwrap_or_else(|| ...and_hms_opt(12,0,0)...)`，因为 `and_hms_opt(0,0,0)` 对合法 `NaiveDate` 不可能失败），本次修复用 `.and_then()` 达到相同效果且不带这段死代码——非缺陷，是对被镜像模式的小幅改进
- 无 Critical/Important 发现

**Minor（不阻塞）**：本次未顺手清理 GitLab 里那段死代码兜底——合理，Issue #377 范围内不包括 GitLab，且该死代码本身不产生任何错误行为。

## 本地验证

```
cargo test -p gitflow-gitcode label::                                             # 21/21 passed
cargo clippy -p gitflow-gitcode --all-targets --all-features -- -D warnings -W clippy::pedantic   # 无警告
cargo +nightly fmt -- --check                                                     # 无差异
make test（合并后，dev 分支全量）                                                     # 1560/1560 passed
```

## 结论

**Approve.** 无 Critical/Important 发现，1 条 Minor 不阻塞。Issue #377 的四项 AC 全部收口。因交付方式为 `local_merge`，本报告即为正式书面存档，不额外调用 `gf review` CLI 提交动作。
