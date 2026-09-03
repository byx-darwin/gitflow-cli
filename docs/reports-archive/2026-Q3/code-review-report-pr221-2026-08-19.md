## PR Review — #221 (wf-2026-08-19-002)

> ⚠️ 自审限制：作者与评审账号均为 `byx-darwin`，以 comment 记录，不构成正式 approve。

**变更面：** `crates/gitlab/src/issue.rs` / `mr.rs` / `review.rs` / `pipeline.rs`，修复嵌套 group 项目路径 `%2F` 编码不全导致 404 的同根因 bug（共 5 处）。

6 维度评估：

| 维度 | 判定 | 说明 |
|---|---|---|
| Correctness | ✅ | 全部 5 处手拼 `glab api` 路径改用全量 `encode_project_path(&self.repo)`；3 段路径 `group/subgroup/project` → `/projects/group%2Fsubgroup%2Fproject/...`，符合 GitLab API 对项目路径单段全量编码的要求 |
| Reliability | ✅ | 复用已有 `pub(crate) encode_project_path`（commit.rs:190），消除 `split_once` 部分编码逻辑；repo 形状由构造保证，畸形 repo 由 glab 自身报错 |
| Security | ✅ | 无敏感变更；路径仅做 URL 编码，不涉及 shell 拼接（argv-form） |
| Maintainability | ✅ | DRY：5 处重复的 split_once+部分编码收敛到单一共享辅助；doc 注释同步更新为 `{repo-encoded}` 语义 |
| Test coverage | ✅ | 新增 5 个嵌套 group 路径测试（`MockCommandRunner` 断言传给 glab 的 argv）；`cargo test -p gitflow-gitlab` 238；`make test` 1322；clippy pedantic / nightly fmt 干净 |
| Documentation | ✅ | 设计文档 + 实施计划已入库；PR body 含 `Closes #219` |

**非阻塞观察：**
1. 移除 `split_once` 校验分支意味着对无 `/` 的畸形 repo 不再返回本地 `Invalid repo format` 错误，改由 glab 返回底层错误——符合 issue 建议的修复方向，属预期行为变化。
2. 提交分组含一个 `style: fmt` 提交，不影响交付正确性。

**Verification：** `cargo test -p gitflow-gitlab` 238 · `make test` 1322 · clippy pedantic · nightly fmt · CI 2/4 latest run success（2 running，monitor 跟踪中）

Closes #219
