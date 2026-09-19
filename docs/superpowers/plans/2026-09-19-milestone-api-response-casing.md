# MilestoneApiResponse 命名方向修复 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 修复 GitHub 与 GitCode 两个平台 `MilestoneApiResponse` 的 `rename_all` 方向错误，使 `due_on`/`closed_issues`/`open_issues` 不再被静默归零/置空。

**Architecture:** 两处改动分别落在 `crates/github/src/label.rs` 与 `crates/gitcode/src/label.rs`，各自独立文件、独立 struct，互不重叠；同时更新两处文件里现有的、用了错误 camelCase JSON fixture 的反序列化测试（否则修复后这些既有测试会因为 fixture 本身写错而变红）。

**Tech Stack:** Rust 2024 / serde_json

**Spec:** `docs/superpowers/specs/2026-09-19-milestone-api-response-casing-design.md`

## Global Constraints

- 保留 `due_on`/`closed_issues`/`open_issues` 上的 `#[serde(default)]`（设计文档结论：这三个字段在真实 API 响应里确实可能缺失，`default` 是合理兜底，bug 只在命名方向）
- 不引入新的 clippy 告警：`cargo clippy --workspace --all-targets --all-features -- -D warnings -W clippy::pedantic`
- 每个任务完成后运行对应 crate 的 `cargo test`

---

### Task 1: GitHub `MilestoneApiResponse` 命名方向修复（#364 本体）

**Files:**
- Modify: `crates/github/src/label.rs:322-335`（`MilestoneApiResponse` 定义）
- Modify: `crates/github/src/label.rs`（`test_should_deserialize_milestone_api_response`、`test_should_deserialize_closed_milestone`、`test_should_deserialize_milestone_list` 三条既有测试的 JSON fixture）

**Interfaces:**
- Consumes: 无外部依赖，仅改类型内部字段匹配规则
- Produces: `MilestoneApiResponse` 字段名与 `gh api milestones` 真实响应（snake_case）匹配；`impl From<MilestoneApiResponse> for MilestoneData` 不变

- [ ] **Step 1: 把既有测试的 JSON fixture 改成真实 snake_case 形状（此时代码还未修，测试应变红）**

修改 `crates/github/src/label.rs` 中的 `test_should_deserialize_milestone_api_response`：

```rust
    #[test]
    fn test_should_deserialize_milestone_api_response() {
        let json = br#"{
            "number": 1,
            "title": "v1.0 Release",
            "description": "First stable release",
            "state": "open",
            "due_on": "2026-06-01T00:00:00Z",
            "closed_issues": 10,
            "open_issues": 5
        }"#;

        let api: MilestoneApiResponse =
            serde_json::from_slice(json).expect("valid MilestoneApiResponse");
        let data: MilestoneData = api.into();

        assert_eq!(data.number, 1);
        assert_eq!(data.title, "v1.0 Release");
        assert_eq!(data.description, Some("First stable release".into()));
        assert_eq!(data.state, State::Open);
        assert!(data.due_on.is_some());
        assert_eq!(data.closed_issues, 10);
        assert_eq!(data.open_issues, 5);
    }
```

修改 `test_should_deserialize_closed_milestone`：

```rust
    #[test]
    fn test_should_deserialize_closed_milestone() {
        let json = br#"{
            "number": 2,
            "title": "v0.9 Beta",
            "description": null,
            "state": "closed",
            "due_on": null,
            "closed_issues": 20,
            "open_issues": 0
        }"#;

        let api: MilestoneApiResponse =
            serde_json::from_slice(json).expect("valid MilestoneApiResponse");
        let data: MilestoneData = api.into();

        assert_eq!(data.state, State::Closed);
        assert!(data.description.is_none());
        assert!(data.due_on.is_none());
```

（该测试函数余下的断言行不变，只改上面这段 JSON 与前几行。）

修改 `test_should_deserialize_milestone_list`（只改 JSON，断言不变）：

```rust
    #[test]
    fn test_should_deserialize_milestone_list() {
        let json = br#"[
            {"number": 1, "title": "v1.0", "description": null, "state": "open", "due_on": null, "closed_issues": 0, "open_issues": 3},
            {"number": 2, "title": "v0.9", "description": "Beta", "state": "closed", "due_on": "2026-01-01T00:00:00Z", "closed_issues": 15, "open_issues": 0}
        ]"#;
```

- [ ] **Step 2: 运行测试确认失败（fixture 已是真实形状，代码仍是错误的 camelCase 期待，字段应静默变默认值，导致断言失败）**

Run: `cargo test -p gitflow-github test_should_deserialize_milestone_api_response test_should_deserialize_closed_milestone -- --nocapture`
Expected: FAIL —— `test_should_deserialize_milestone_api_response` 断言 `data.due_on.is_some()` 应失败（实际因未命中 `due_on` 键而是 `None`），`assert_eq!(data.closed_issues, 10)` 应失败（实际为 `0`）

- [ ] **Step 3: 移除错误的 `rename_all` 属性**

修改 `crates/github/src/label.rs` 第 322 行附近：

```rust
/// `gh api milestones` 返回的 JSON 结构。
#[derive(Debug, Clone, Deserialize)]
struct MilestoneApiResponse {
    number: u64,
    title: String,
    #[serde(default)]
    description: Option<String>,
    state: String,
    #[serde(default)]
    due_on: Option<String>,
    #[serde(default)]
    closed_issues: u64,
    #[serde(default)]
    open_issues: u64,
}
```

（去掉 `#[serde(rename_all = "camelCase")]` 这一行，其余字段与 `#[serde(default)]` 全部保留不变。）

- [ ] **Step 4: 运行测试确认通过**

Run: `cargo test -p gitflow-github label:: -- --nocapture`
Expected: PASS（改过的 3 条 + 该文件其余全部 label/milestone 测试）

- [ ] **Step 5: 提交**

```bash
git add crates/github/src/label.rs
git commit -m "fix(github): MilestoneApiResponse match upstream snake_case, stop silently zeroing due_on/issue counts (#364)"
```

---

### Task 2: GitCode `MilestoneApiResponse` 命名方向修复（审计新增发现）

**Files:**
- Modify: `crates/gitcode/src/label.rs:308-321`（`MilestoneApiResponse` 定义）
- Modify: `crates/gitcode/src/label.rs`（`test_should_deserialize_milestone_api_response`、`test_should_deserialize_closed_milestone`、`test_should_deserialize_milestone_list` 三条既有测试的 JSON fixture）
- Test: `crates/gitcode/src/label.rs`（新增一条测试，验证 `closed_issues`/`open_issues` 完全缺失时仍能靠 `#[serde(default)]` 正常解析）

**Interfaces:**
- Consumes: 无外部依赖
- Produces: `MilestoneApiResponse` 字段名与真实 `gitcode api milestones`/`gitcode milestone list` 响应（snake_case）匹配

- [ ] **Step 1: 把既有测试的 JSON fixture 改成真实 snake_case 形状**

修改 `crates/gitcode/src/label.rs` 中的 `test_should_deserialize_milestone_api_response`：

```rust
    #[test]
    fn test_should_deserialize_milestone_api_response() {
        let json = br#"{
            "number": 1,
            "title": "v1.0 Release",
            "description": "First stable release",
            "state": "open",
            "due_on": "2026-06-01T00:00:00Z",
            "closed_issues": 10,
            "open_issues": 5
        }"#;

        let api: MilestoneApiResponse =
            serde_json::from_slice(json).expect("valid MilestoneApiResponse");
        let data: MilestoneData = api.into();

        assert_eq!(data.number, 1);
        assert_eq!(data.title, "v1.0 Release");
        assert_eq!(data.description, Some("First stable release".into()));
        assert_eq!(data.state, State::Open);
        assert!(data.due_on.is_some());
        assert_eq!(data.closed_issues, 10);
        assert_eq!(data.open_issues, 5);
    }
```

修改 `test_should_deserialize_closed_milestone`：

```rust
    #[test]
    fn test_should_deserialize_closed_milestone() {
        let json = br#"{
            "number": 2,
            "title": "v0.9 Beta",
            "description": null,
            "state": "closed",
            "due_on": null,
            "closed_issues": 20,
            "open_issues": 0
        }"#;

        let api: MilestoneApiResponse =
            serde_json::from_slice(json).expect("valid MilestoneApiResponse");
        let data: MilestoneData = api.into();

        assert_eq!(data.state, State::Closed);
        assert!(data.description.is_none());
        assert!(data.due_on.is_none());
```

（该测试函数余下的断言行不变。）

修改 `test_should_deserialize_milestone_list`（只改 JSON）：

```rust
    #[test]
    fn test_should_deserialize_milestone_list() {
        let json = br#"[
            {"number": 1, "title": "v1.0", "description": null, "state": "open", "due_on": null, "closed_issues": 0, "open_issues": 3},
            {"number": 2, "title": "v0.9", "description": "Beta", "state": "closed", "due_on": "2026-01-01T00:00:00Z", "closed_issues": 15, "open_issues": 0}
        ]"#;
```

- [ ] **Step 2: 新增一条测试，覆盖真实 `milestone list` 端点会缺失 `closed_issues`/`open_issues` 键的场景（不是新任务、是本任务的一部分，验证 `#[serde(default)]` 仍然有效）**

紧邻 `test_should_deserialize_milestone_list` 之后新增（JSON 取自 2026-09-19 对 `gitcode milestone list --repo openharmony/docs` 的真实实测响应）：

```rust
    #[test]
    fn test_should_default_issue_counts_when_absent_from_real_list_response() {
        // 真实响应形状：closed_issues/open_issues 键完全不存在，不是"值为 0"。
        let json = br#"{
            "id": null,
            "number": 733070,
            "title": "IT26_OpenHarmony 7.0(Release)",
            "description": "",
            "state": "active",
            "due_on": "2026-08-31"
        }"#;

        let api: MilestoneApiResponse =
            serde_json::from_slice(json).expect("valid MilestoneApiResponse");
        let data: MilestoneData = api.into();

        assert_eq!(data.number, 733070);
        assert!(data.due_on.is_some(), "due_on 存在时必须被正确解析");
        assert_eq!(
            data.closed_issues, 0,
            "键缺失时应靠 #[serde(default)] 落到 0，而不是反序列化失败"
        );
        assert_eq!(data.open_issues, 0);
    }
```

已核实 `impl From<MilestoneApiResponse> for MilestoneData`（`crates/gitcode/src/label.rs:324-341`）：`state` 转换是 `if api.state == "closed" { State::Closed } else { State::Open }`，真实响应里的 `"active"` 会落到 `State::Open`，无需改动这段转换逻辑。

- [ ] **Step 3: 运行测试确认失败**

Run: `cargo test -p gitflow-gitcode test_should_deserialize_milestone_api_response test_should_deserialize_closed_milestone test_should_default_issue_counts_when_absent_from_real_list_response -- --nocapture`
Expected: FAIL —— 前两条因 fixture 已是真实形状而代码仍期待 camelCase 而失败（同 Task 1 Step 2）；新增的第三条应能通过（因为它本来就没有 camelCase/snake_case 冲突字段——`due_on` 在 camelCase 规则下会变成 `dueOn`，与 JSON 里的 `due_on` 不匹配，所以 `data.due_on.is_some()` 这条断言此时也应失败）

- [ ] **Step 4: 移除错误的 `rename_all` 属性**

修改 `crates/gitcode/src/label.rs` 第 308 行附近：

```rust
/// `gc api milestones` 返回的 JSON 结构。
#[derive(Debug, Clone, Deserialize)]
struct MilestoneApiResponse {
    number: u64,
    title: String,
    #[serde(default)]
    description: Option<String>,
    state: String,
    #[serde(default)]
    due_on: Option<String>,
    #[serde(default)]
    closed_issues: u64,
    #[serde(default)]
    open_issues: u64,
}
```

- [ ] **Step 5: 运行测试确认通过**

Run: `cargo test -p gitflow-gitcode label:: -- --nocapture`
Expected: PASS（改过的 3 条 + 新增 1 条 + 该文件其余全部 label/milestone 测试）

- [ ] **Step 6: 提交**

```bash
git add crates/gitcode/src/label.rs
git commit -m "fix(gitcode): MilestoneApiResponse match upstream snake_case, stop silently zeroing due_on/issue counts (audit finding alongside #364)"
```

---

## Final Verification

- [ ] **Step 1: 全量测试**

Run: `cargo test -p gitflow-github -p gitflow-gitcode`
Expected: PASS

- [ ] **Step 2: Lint**

Run: `make lint`
Expected: 0 warnings

- [ ] **Step 3: Pedantic clippy（本次改动的两个 crate）**

Run: `cargo clippy -p gitflow-github -p gitflow-gitcode --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: 0 warnings
