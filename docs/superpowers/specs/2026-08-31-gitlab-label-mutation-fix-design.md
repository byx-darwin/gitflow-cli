# GitLab Issue 标签增删修复设计

**Issue:** #270
**类型:** bounded fix（单文件，已有代码流程）

## 问题

`crates/gitlab/src/issue.rs` 中 `add_labels`/`remove_label` 调用的是
`glab issue edit --add-label` / `--remove-label`。`glab`（验证版本 1.115.0）
没有 `issue edit` 子命令，该调用会静默 fallback 到 `glab issue` 顶层帮助文本，
退出码 0 —— 导致 GitLab 平台的标签增删实际上是 no-op，但调用方（包括
`gf-issue-triage`）会误判为成功。

此问题在 #266（PR #269）实现 `edit()` 方法时被发现并记录为跟进项，
本设计即该跟进的实施方案。

## 变更范围

仅 `crates/gitlab/src/issue.rs` 的两个方法，不涉及 GitHub/GitCode 适配器、
trait 定义或 CLI 层。

### 1. `add_labels()`（issue.rs:561-618）

命令从：
```
glab issue edit <n> --repo <repo> --add-label <labels>
```
改为：
```
glab issue update <n> --repo <repo> --label <labels>
```
沿用现有逗号拼接多标签的行为（`labels.join(",")`）。

### 2. `remove_label()`（issue.rs:627-653）

命令从：
```
glab issue edit <n> --repo <repo> --remove-label <label>
```
改为：
```
glab issue update <n> --repo <repo> --unlabel <label>
```

### 3. 文档/日志同步更新

- 两个方法上的 doc comment（当前写的是 `glab issue edit --add-label`/`--remove-label`）
- `extract_missing_labels_from_error` 上引用 `glab issue edit --add-label` 的注释
- `debug!` 日志文案

## 测试策略（TDD）

现有测试（`test_should_return_platform_error_when_glab_fails_for_add_labels`/
`remove_label`、`test_should_auto_create_label_and_retry_on_add_labels_glab` 等）
只断言成功/失败结果，不断言具体命令参数——这正是该 bug 能溜过审查的原因。

新增测试（仿照 `test_should_send_title_and_description_flags_for_edit` 的写法），
断言 `recorded_calls()` 命中：
- `add_labels`: `["issue", "update", <n>, "--repo", <repo>, "--label", <labels>]`
- `remove_label`: `["issue", "update", <n>, "--repo", <repo>, "--unlabel", <label>]`

先跑 RED（对旧实现失败），再改实现使其 GREEN。自动创建缺失标签的重试逻辑
（`ensure_label_exists` + retry）保持不变，仅底层命令参数变化。

## 影响

- 修复 GitLab 平台标签增删操作的真实生效性（此前是 no-op）
- 不影响 GitHub/GitCode 适配器
- 不改变 `IssueProvider` trait 签名或调用方代码
