# GitHub/GitCode MilestoneApiResponse 命名方向修复设计（#364 + 审计）

**Status:** Approved（Bounded 路径，无需架构级评审）
**Workflow:** wf-2026-09-19-007

## 背景

`crates/github/src/label.rs` 与 `crates/gitcode/src/label.rs` 的 `MilestoneApiResponse`
都标了 `#[serde(rename_all = "camelCase")]`，但两个平台的 milestone API 实际返回
snake_case 字段名（`due_on`、`closed_issues`、`open_issues`）。三个字段都带
`#[serde(default)]`，于是反序列化失配被静默降级为 `None`/`0`，而不是报错。

根因：`rename_all = "camelCase"` 是项目对**出站**类型（`gf` 输出给调用方的 JSON，如
`MilestoneData`）的约定，被错误套用在了**入站**类型（解析上游 API 响应的
`MilestoneApiResponse`）上——入站类型必须匹配上游自己的命名，与出站方向相反。

## 实测证据

### GitHub（#364 原始报告）

```
$ gh api "repos/rust-lang/rust/milestones?per_page=1" \
    --jq '.[0] | {due_on, closed_issues, open_issues, title}'
{"closed_issues":1238,"due_on":null,"open_issues":45,"title":"1.99.0"}
```

字段名为 snake_case，与 `camelCase` 标注方向相反。

### GitCode（本次审计新增确认，2026-09-19 用真实已登录 CLI 实测）

```
$ gitcode milestone list --repo openharmony/docs --limit 3 --json
[
  {"id": null, "number": 733070, "title": "IT26_OpenHarmony 7.0(Release)",
   "description": "", "state": "active", "due_on": "2026-08-31"},
  ...
]
```

同样是 snake_case 的 `due_on`，且——**关键发现**——真实 `milestone list` 响应里
`closed_issues`/`open_issues` **完全不出现**，不是"值为 0"，是"键不存在"。

## 对 Issue #364 验收标准第 4 条的结论

> 评估这几个字段上的 `#[serde(default)]` 是否应当保留

**结论：保留。** 上面的真实响应证明 `closed_issues`/`open_issues` 在 `milestone list`
端点本就可能缺失，`#[serde(default)]` 是对这个真实情况的合理兜底，不是问题根源。
真正的 bug 只在 `rename_all` 的方向，不在 `default` 上——去掉 `default` 反而会让这个
真实存在的合法缺失场景直接报错，是不对的。

## 修复方案

1. `crates/github/src/label.rs` — 移除 `MilestoneApiResponse` 的
   `#[serde(rename_all = "camelCase")]`。不加该属性时 serde 默认按字段名原样匹配，
   Rust 字段本就是 snake_case，与上游一致。
2. `crates/gitcode/src/label.rs` — 同样移除 `MilestoneApiResponse` 的 `camelCase`。
3. 各补一条不依赖网络的反序列化单测：
   - GitHub：沿用 issue 里给出的真实形状 JSON（非零 `closed_issues`/`open_issues`、
     非空 `due_on`），断言三个字段被正确解析，修复前必须失败。
   - GitCode：用上面实测抓到的真实响应改造（含 `due_on` 非空；`closed_issues`/
     `open_issues` 缺失时应仍能通过 `#[serde(default)]` 解析为 0），并额外补一条
     `closed_issues`/`open_issues` 存在且非零时的正确解析断言。

## 对 Issue #364 验收标准第 3 条的结论（17 类型全量审计）

对 `github`/`gitlab`/`gitcode` 三个 crate 中全部 `*ApiResponse` 类型逐一核对
`rename_all` 标注方向是否与该类型自己所属平台的真实 API 命名一致：

| 类型 | 位置 | 标注 | 结论 |
|---|---|---|---|
| `MilestoneApiResponse` | `github/src/label.rs` | `camelCase` | **有 bug，本次修复** |
| `MilestoneApiResponse` | `gitcode/src/label.rs` | `camelCase` | **有 bug，本次修复**（真实 API 实测确认） |
| `CommitApiResponse` | `github/src/commit.rs` | `camelCase` | 标注方向错，但字段全单词（`sha`/`commit`/`author`/`committer`/`stats`/`files`），无实际影响，不修 |
| `CommitInner`（嵌套） | `github/src/commit.rs` | `camelCase` | 同上，字段全单词，无影响，不修 |
| `CommitApiResponse` | `gitcode/src/commit.rs` | `camelCase` | 同 github 版本，无影响，不修 |
| `CommitInner`（嵌套） | `gitcode/src/commit.rs` | `camelCase` | 同上，不修 |
| `IssueApiResponse` | `gitcode/src/issue.rs` | `lowercase` | serde 的 `lowercase` 只做大小写转换、不处理下划线；字段名本就全小写（`created_at`/`html_url`），属于空操作，无实际影响，不修（可选的表述清理，不在本次范围） |
| `CommitApiResponse`/`CommentApiResponse` | `gitlab/src/commit.rs` | `snake_case` | 显式标注且与 GitLab API 一致，正确 |
| 其余 11 处（`gitlab`: review/release/label/mr/issue/pipeline；`gitcode`: label(Label非Milestone)/release/pr/issue 的其余类型） | — | 无 `rename_all`，serde 默认按字段名原样匹配，均与已验证的 GitHub/GitLab/GitCode REST snake_case 约定一致，正确 |

**总结论：17 个入站类型中，2 个有真实 bug（本次修复），2 处标注方向错但因字段全单词而
无实际影响（不修，标注仍具误导性但改动收益低于风险，YAGNI），其余 13 处正确。**

## 测试策略

两处修复各自独立文件、独立 struct，互不重叠，符合 batch 合并标准。

- 单元测试：两条新增反序列化测试（见上）
- 回归：`cargo test -p gitflow-github -p gitflow-gitcode`
- 交付前：`make lint`

## 验收标准（对齐 Issue #364）

- [ ] `MilestoneApiResponse`（github）能正确解析 `due_on`/`closed_issues`/`open_issues`
- [ ] `MilestoneApiResponse`（gitcode）能正确解析 `due_on`；`closed_issues`/`open_issues`
      缺失时仍能通过 `default` 解析为 0，存在时能正确解析非零值
- [ ] 新增不依赖网络的反序列化单测，覆盖两处，修复前必须失败
- [ ] 17 类型审计结论已记录在案（见上表）
- [ ] `#[serde(default)]` 保留的理由已记录在案（见上）
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::pedantic` 无新增告警
