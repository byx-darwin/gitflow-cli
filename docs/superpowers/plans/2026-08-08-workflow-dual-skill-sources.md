# gf-workflow 双 skills 来源兼容 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 让 gf-workflow 在 superpowers 与 mattpocock/skills 两种技能来源下均可运行：启动时检测来源写入契约，Phase 1-3 按来源分支（含 user-invoked 暂停语义与 Phase 3 GO 闸门执行模式选择），安装时硬阻断兜底。

**Architecture:** 编排骨架（契约/四阶段/三闸门/gf-* 步骤）恒定；SKILL.md 只承载检测算法与角色别名，分支细节单点维护于 references.md；Rust 侧扩展契约结构（`skill_source`/`ticket_refs`）并在 `gf skills install` 增加 Step 0 文件系统探测硬阻断。

**Tech Stack:** Rust 2024（serde/serde_json/clap/miette/dirs）、JSON Schema、Markdown skill 文档、gf CLI、jq。

**Source of truth:** 设计文档 `docs/superpowers/specs/2026-08-08-workflow-dual-skill-sources-design.md`（Issue #141，用户已批准）。验收基准 = 设计文档 §10（20 条 AC）。

## Global Constraints

- **TDD 强制**：每个 Rust 任务 RED → GREEN → REFACTOR，每步跑 `cargo test -p gitflow-cli <filter>` 验证
- **生产代码禁止 `unwrap()`/`expect()`**；错误用 `miette::miette!`，测试代码可 `expect`
- **公共项必须有文档注释**（中文，与现有代码一致）；新类型 derive `Debug`
- **提交纪律**：conventional commit（`feat(workflow):` / `docs:`）；所有提交只发生在 Phase 3 feature 分支上（Gate 2→3 批准授权），不碰 `main`；不提交未经评审的内容
- **禁改配置**：`deny.toml` 策略、`.pre-commit-config.yaml`、`rust-toolchain.toml` 一律不动
- **语言约定**：skill 文档叙述正文英文，触发词表保留中文列；本 plan 中的中文文档内容（gates/references/docs 为中文文档）保持中文
- **模板占位符**：保留 `gf` 等模板占位符，不实例化
- **E2E 任务（Task 11）不在 Phase 3 执行**，由 Phase 4 dogfooding 消费

---

## File Structure

| 文件 | 操作 | 责任 |
|---|---|---|
| `skills/gf-workflow/contract.schema.json` | Modify | 契约 schema：`skill_source`（顶层必填）+ `ticket_refs`（phase evidence 可选） |
| `apps/cli/src/commands/workflow.rs` | Modify | Rust 契约结构对齐 schema + 测试 |
| `apps/cli/src/commands/skills.rs` | Modify | 安装时来源检测（`detect_skill_sources`）+ Step 0 硬阻断 + 测试 |
| `skills/gf-workflow/SKILL.md` | Modify | Skill Source Resolution 节、角色别名、GO 闸门、Red Flags |
| `skills/gf-workflow/references.md` | Modify | 双来源映射表 + 分支语义 + 哨兵权威清单（单点维护） |
| `skills/gf-workflow/gates.md` | Modify | Gate 2→3 GO 子步骤 + 证据来源无关说明 |
| `docs/gf-workflow-guide.md` | Modify | 「技能来源适配」节 |
| `docs/integration-guide.md` | Modify | mattpocock/skills 集成章 |
| `docs/index.md` | Modify | 登记设计文档 |
| `docs/superpowers/specs/2026-08-08-workflow-dual-skill-sources-design.md` | Commit | 设计文档（已存在于工作树，首个 commit 入库） |
| `.claude/skills/gf-workflow/*` | Sync | git-ignored 安装副本，最后同步 |

依赖顺序：Task 1 → 2（schema 先行，Rust 对齐）；Task 3 → 4（探测函数先行，Step 0 复用）；Task 5-7 依赖 Task 1 的字段语义；Task 8 依赖 5-7；Task 9 依赖 1-8；Task 10 收尾。

---

### Task 1: contract.schema.json — 新增 `skill_source` 与 `ticket_refs`

**Files:**
- Modify: `skills/gf-workflow/contract.schema.json`
- Commit（首个提交，含设计文档与本 plan）: `docs/superpowers/specs/2026-08-08-workflow-dual-skill-sources-design.md`, `docs/superpowers/plans/2026-08-08-workflow-dual-skill-sources.md`, `skills/gf-workflow/contract.schema.json`

**Interfaces:**
- Produces: schema 字段定义——`skill_source`（顶层，enum `superpowers|mattpocock|inline`，必填）；`ticket_refs`（`$defs.phase.evidence`，string 数组，可选）。Task 2 的 Rust 结构、Task 5-7 的文档语义以此为准。

- [ ] **Step 1: 修改 schema——顶层 `skill_source`**

在 `contract.schema.json` 顶层 `required` 数组追加 `"skill_source"`，并在 `properties` 中（`mode` 之后）插入：

```json
    "skill_source": {
      "type": "string",
      "enum": ["superpowers", "mattpocock", "inline"],
      "description": "启动时检测的技能来源；新合同必填，跨会话恢复沿用（恢复时重验在场性）"
    },
```

注意：`required` 变为 `["version", "workflow_id", "title", "mode", "skill_source", "created_at", "updated_at", "current_phase", "phases"]`。

- [ ] **Step 2: 修改 schema——phase evidence `ticket_refs`**

在 `$defs.phase.evidence.properties` 中（`spec_path` 之后）插入：

```json
            "ticket_refs": {
              "type": "array",
              "items": { "type": "string" },
              "description": "to-tickets 产出的票据清单（文件路径或 URL），mattpocock 路径 Phase 2 记录"
            },
```

- [ ] **Step 3: 验证 JSON 合法且关键字段就位**

Run:
```bash
jq empty skills/gf-workflow/contract.schema.json && \
jq -e '.required | index("skill_source")' skills/gf-workflow/contract.schema.json && \
jq -e '.properties.skill_source.enum' skills/gf-workflow/contract.schema.json && \
jq -e '.["$defs"].phase.evidence.properties.ticket_refs.type' skills/gf-workflow/contract.schema.json
```
Expected: 全部输出非空、退出码 0；enum 为 `["superpowers","mattpocock","inline"]`。

- [ ] **Step 4: Commit**

```bash
git add skills/gf-workflow/contract.schema.json \
        docs/superpowers/specs/2026-08-08-workflow-dual-skill-sources-design.md \
        docs/superpowers/plans/2026-08-08-workflow-dual-skill-sources.md
git commit -m "feat(workflow): contract schema adds skill_source and ticket_refs (#141)"
```

---

### Task 2: workflow.rs — `WorkflowContract.skill_source` + `PhaseEvidence.ticket_refs`（TDD）

**Files:**
- Modify: `apps/cli/src/commands/workflow.rs`（`PhaseEvidence` 结构、`WorkflowContract` 结构、`new_contract`、`mod tests`）
- Test: 同文件 `#[cfg(test)] mod tests`

**Interfaces:**
- Consumes: Task 1 schema 字段语义（字段名/类型/可选性完全一致）
- Produces: `WorkflowContract.skill_source: Option<String>`（`skip_serializing_if = Option::is_none`）、`PhaseEvidence.ticket_refs: Option<Vec<String>>`。后续合同 jq 写入依赖 serde 忽略未知字段的现状不受影响；`gf workflow status` 输出将包含新字段。

- [ ] **Step 1: RED——写失败测试**

在 `mod tests` 末尾追加三个测试：

```rust
    /// Issue #141：skill_source（顶层）与 ticket_refs（phase evidence）往返保真。
    #[test]
    fn test_should_roundtrip_skill_source_and_ticket_refs() {
        let mut contract = base_contract("full");
        contract.skill_source = Some("mattpocock".to_string());
        contract
            .phases
            .get_mut("2")
            .expect("phase 2")
            .evidence
            .ticket_refs = Some(vec![
            ".scratch/demo/issues/01-setup.md".to_string(),
            "https://github.com/org/repo/issues/10".to_string(),
        ]);
        let json = serde_json::to_string(&contract).expect("serialize");
        assert!(
            json.contains(r#""skill_source":"mattpocock""#),
            "skill_source must serialize: {json}"
        );
        assert!(
            json.contains(r#""ticket_refs":["#),
            "ticket_refs must serialize: {json}"
        );
        let back: WorkflowContract = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.skill_source.as_deref(), Some("mattpocock"));
        let refs = back
            .phases
            .get("2")
            .expect("phase 2")
            .evidence
            .ticket_refs
            .as_ref()
            .expect("ticket_refs");
        assert_eq!(refs.len(), 2);
    }

    /// Issue #141：未设置的新字段不得序列化为 null（与既有 evidence 约定一致）。
    #[test]
    fn test_should_omit_absent_skill_source_and_ticket_refs() {
        let contract = base_contract("full");
        let json = serde_json::to_string(&contract).expect("serialize");
        assert!(
            !json.contains("skill_source"),
            "absent skill_source must be omitted: {json}"
        );
        assert!(
            !json.contains("ticket_refs"),
            "absent ticket_refs must be omitted: {json}"
        );
    }

    /// Issue #141：旧合同（无 skill_source）仍可反序列化——向后兼容。
    #[test]
    fn test_should_deserialize_legacy_contract_without_skill_source() {
        let contract: WorkflowContract =
            serde_json::from_str(SCHEMA_EXAMPLE_CONTRACT).expect("legacy contract");
        assert!(contract.skill_source.is_none());
    }
```

- [ ] **Step 2: 确认测试失败**

Run: `cargo test -p gitflow-cli workflow -- --skip 2>/dev/null; cargo test -p gitflow-cli test_should_roundtrip_skill_source`
Expected: 编译失败——`WorkflowContract` 无 `skill_source` 字段。

- [ ] **Step 3: GREEN——最小实现**

`PhaseEvidence` 结构体（`spec_path` 字段之后）追加：

```rust
    /// 票据清单（文件路径或 URL），mattpocock 路径阶段二 `to-tickets` 产出（Issue #141）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ticket_refs: Option<Vec<String>>,
```

`WorkflowContract` 结构体（`mode` 字段之后）追加：

```rust
    /// 技能来源（`superpowers` / `mattpocock` / `inline`），启动时检测写入（Issue #141）。
    ///
    /// 旧合同可能缺失此字段，故为 `Option`；新合同由编排器在 Bootstrap 后必填。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_source: Option<String>,
```

`new_contract` 的 `WorkflowContract { ... }` 字面量（`mode,` 之后）追加：

```rust
        skill_source: None,
```

- [ ] **Step 4: 跑全部 workflow 测试**

Run: `cargo test -p gitflow-cli commands::workflow`
Expected: 全部 PASS（含三个新测试与既有 21 个测试）。

- [ ] **Step 5: REFACTOR——确认无 pedantic 告警**

Run: `cargo clippy -p gitflow-cli --all-targets -- -D warnings -W clippy::pedantic`
Expected: 0 warnings（结构体字段文档注释齐备，无缺失）。

- [ ] **Step 6: Commit**

```bash
git add apps/cli/src/commands/workflow.rs
git commit -m "feat(workflow): contract struct aligns with skill_source/ticket_refs schema (#141)"
```

---

### Task 3: skills.rs — `detect_skill_sources` 探测函数（TDD）

**Files:**
- Modify: `apps/cli/src/commands/skills.rs`（新增常量 + 函数 + `mod tests` 测试）
- Test: 同文件 `#[cfg(test)] mod tests`

**Interfaces:**
- Consumes: 无（独立探测模块）
- Produces: `pub enum SkillSourceKind { Superpowers, Mattpocock }`（含 `Display`）与 `fn detect_skill_sources(home: &Path) -> Vec<SkillSourceKind>`——Task 4 的 Step 0 直接调用。哨兵常量是安装时/运行时**共享权威定义**的 Rust 侧载体（与 Task 6 references.md 哨兵表逐字一致）。

- [ ] **Step 1: RED——写失败测试**

在 `skills.rs` 的 `mod tests` 末尾追加（沿用既有 `tempfile::tempdir()` 模式；探测函数接受 `home` 参数故无需 `temp_env`）：

```rust
    /// 在临时 HOME 写入 plugin 形态注册表。
    fn seed_plugin_registry(home: &std::path::Path, plugin_keys: &[&str]) {
        let dir = home.join(".claude/plugins");
        std::fs::create_dir_all(&dir).expect("create plugins dir");
        let mut plugins = serde_json::Map::new();
        for key in plugin_keys {
            plugins.insert((*key).to_string(), serde_json::json!([]));
        }
        let content = serde_json::json!({ "version": 2, "plugins": plugins });
        std::fs::write(
            dir.join("installed_plugins.json"),
            serde_json::to_string(&content).expect("serialize"),
        )
        .expect("write registry");
    }

    /// 在临时 HOME 写入裸名 skill 目录。
    fn seed_bare_skills(home: &std::path::Path, names: &[&str]) {
        for name in names {
            let dir = home.join(".claude/skills").join(name);
            std::fs::create_dir_all(&dir).expect("create skill dir");
            std::fs::write(dir.join("SKILL.md"), "---\nname: x\n---\n").expect("write SKILL.md");
        }
    }

    #[test]
    fn test_detect_plugin_superpowers() {
        let tmp = tempfile::tempdir().expect("temp dir");
        seed_plugin_registry(tmp.path(), &["superpowers@claude-plugins-official"]);
        let found = detect_skill_sources(tmp.path());
        assert_eq!(found, vec![SkillSourceKind::Superpowers]);
    }

    #[test]
    fn test_detect_plugin_mattpocock() {
        let tmp = tempfile::tempdir().expect("temp dir");
        seed_plugin_registry(tmp.path(), &["mattpocock-skills@mattpocock"]);
        let found = detect_skill_sources(tmp.path());
        assert_eq!(found, vec![SkillSourceKind::Mattpocock]);
    }

    #[test]
    fn test_detect_bare_mattpocock_requires_double_sentinel() {
        let tmp = tempfile::tempdir().expect("temp dir");
        seed_bare_skills(tmp.path(), &["to-spec", "grilling"]);
        let found = detect_skill_sources(tmp.path());
        assert_eq!(found, vec![SkillSourceKind::Mattpocock]);
    }

    #[test]
    fn test_detect_bare_partial_sentinel_is_not_detected() {
        // 只有 to-spec 缺 grilling → 部分命中视同缺失（防同名碰撞误判）
        let tmp = tempfile::tempdir().expect("temp dir");
        seed_bare_skills(tmp.path(), &["to-spec"]);
        assert!(
            detect_skill_sources(tmp.path()).is_empty(),
            "partial sentinel hit must not be detected"
        );
    }

    #[test]
    fn test_detect_bare_superpowers_requires_double_sentinel() {
        let tmp = tempfile::tempdir().expect("temp dir");
        seed_bare_skills(tmp.path(), &["brainstorming", "writing-plans"]);
        let found = detect_skill_sources(tmp.path());
        assert_eq!(found, vec![SkillSourceKind::Superpowers]);
    }

    #[test]
    fn test_detect_empty_home_finds_nothing() {
        let tmp = tempfile::tempdir().expect("temp dir");
        assert!(detect_skill_sources(tmp.path()).is_empty());
    }

    #[test]
    fn test_detect_both_sources_when_both_installed() {
        let tmp = tempfile::tempdir().expect("temp dir");
        seed_plugin_registry(
            tmp.path(),
            &["superpowers@claude-plugins-official", "mattpocock-skills@mattpocock"],
        );
        let found = detect_skill_sources(tmp.path());
        assert_eq!(found.len(), 2, "both sources must be reported: {found:?}");
    }

    #[test]
    fn test_detect_malformed_registry_falls_back_to_bare() {
        // 注册表损坏不应 panic，降级到裸名探测
        let tmp = tempfile::tempdir().expect("temp dir");
        let dir = tmp.path().join(".claude/plugins");
        std::fs::create_dir_all(&dir).expect("create dir");
        std::fs::write(dir.join("installed_plugins.json"), "not json").expect("write");
        seed_bare_skills(tmp.path(), &["to-spec", "grilling"]);
        let found = detect_skill_sources(tmp.path());
        assert_eq!(found, vec![SkillSourceKind::Mattpocock]);
    }
```

- [ ] **Step 2: 确认测试失败**

Run: `cargo test -p gitflow-cli detect_skill_sources`
Expected: 编译失败——`detect_skill_sources` 未定义。

- [ ] **Step 3: GREEN——实现探测**

在 `skills.rs` 的 `Command handlers` 分隔注释之前（`skills_source_dir` 之后）插入：

```rust
// ---------------------------------------------------------------------------
// Skill source detection (Issue #141)
// ---------------------------------------------------------------------------

/// 技能来源类型。
///
/// 与运行时检测（`skills/gf-workflow/references.md` 哨兵表）共享同一份权威定义；
/// 修改哨兵规则时两处必须同步（测试 `test_sentinel_rules_match_references` 守护）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillSourceKind {
    /// Superpowers（plugin `superpowers` 或裸名安装）。
    Superpowers,
    /// mattpocock/skills（plugin `mattpocock-skills` 或裸名安装）。
    Mattpocock,
}

impl fmt::Display for SkillSourceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Superpowers => write!(f, "superpowers"),
            Self::Mattpocock => write!(f, "mattpocock"),
        }
    }
}

/// plugin 形态的注册表键前缀（`installed_plugins.json` 键形如 `<plugin>@<marketplace>`）。
const SUPERPOWERS_PLUGIN_PREFIX: &str = "superpowers@";
const MATTPOCOCK_PLUGIN_PREFIX: &str = "mattpocock-skills@";

/// 裸名形态哨兵（双哨兵同时命中才判定；裸名脆弱从严）。
const SUPERPOWERS_BARE_SENTINELS: &[&str] = &["brainstorming", "writing-plans"];
const MATTPOCOCK_BARE_SENTINELS: &[&str] = &["to-spec", "grilling"];

/// plugin 形态探测：解析 `~/.claude/plugins/installed_plugins.json` 键前缀。
///
/// 注册表缺失或损坏返回 `false`（降级到裸名探测，不 panic）。
fn plugin_source_present(home: &Path, prefix: &str) -> bool {
    let path = home.join(".claude/plugins/installed_plugins.json");
    let Ok(content) = std::fs::read_to_string(&path) else {
        return false;
    };
    let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&content) else {
        return false;
    };
    parsed
        .get("plugins")
        .and_then(serde_json::Value::as_object)
        .is_some_and(|plugins| plugins.keys().any(|key| key.starts_with(prefix)))
}

/// 裸名形态探测：`~/.claude/skills/` 下哨兵目录双命中。
fn bare_sentinels_present(home: &Path, sentinels: &[&str]) -> bool {
    sentinels
        .iter()
        .all(|name| home.join(".claude/skills").join(name).is_dir())
}

/// 检测已安装的技能来源（安装时 Step 0，Issue #141）。
///
/// 依次探测 plugin 形态与裸名形态，返回所有在场的来源（0/1/2 个）。
/// 两者共存时全部返回，由调用方决定提示方式。
#[must_use]
pub fn detect_skill_sources(home: &Path) -> Vec<SkillSourceKind> {
    let mut found = Vec::new();
    if plugin_source_present(home, SUPERPOWERS_PLUGIN_PREFIX)
        || bare_sentinels_present(home, SUPERPOWERS_BARE_SENTINELS)
    {
        found.push(SkillSourceKind::Superpowers);
    }
    if plugin_source_present(home, MATTPOCOCK_PLUGIN_PREFIX)
        || bare_sentinels_present(home, MATTPOCOCK_BARE_SENTINELS)
    {
        found.push(SkillSourceKind::Mattpocock);
    }
    found
}
```

注意：确认文件顶部 `use std::fmt;` 存在（若无需补上——`WorkflowMode` 在本文件没有 Display，可能需要新增 import：`use std::fmt;`）。

- [ ] **Step 4: 跑探测测试**

Run: `cargo test -p gitflow-cli detect`
Expected: 8 个新测试全部 PASS。

- [ ] **Step 5: REFACTOR——pedantic 检查**

Run: `cargo clippy -p gitflow-cli --all-targets -- -D warnings -W clippy::pedantic`
Expected: 0 warnings。

- [ ] **Step 6: Commit**

```bash
git add apps/cli/src/commands/skills.rs
git commit -m "feat(skills): add install-time skill source detection (#141)"
```

---

### Task 4: skills.rs — `install_skills` Step 0 硬阻断（TDD）

**Files:**
- Modify: `apps/cli/src/commands/skills.rs`（新增 `check_skill_source`，`install_skills` 首行接入）
- Test: 同文件 `mod tests`

**Interfaces:**
- Consumes: Task 3 `detect_skill_sources` / `SkillSourceKind`
- Produces: `gf skills install` 行为变更——Claude 平台两来源皆无 → 三条安装引导 + `Err`（非 0 退出码）；任一在场 → 打印 `✓ 检测到技能来源: <list>` 继续；非 Claude 平台 → 提示跳过检查。

- [ ] **Step 1: RED——写失败测试**

```rust
    /// Test is Unix-only: uses `dirs::home_dir()` which on Windows ignores HOME env var.
    #[cfg(unix)]
    #[test]
    fn test_install_check_blocks_when_no_skill_source() {
        let tmp = tempfile::tempdir().expect("temp dir");
        temp_env::with_var("HOME", Some(tmp.path()), || {
            let result = check_skill_source(AgentPlatform::Claude);
            let err = result.expect_err("must block when no source installed");
            assert!(
                err.to_string().contains("技能来源缺失"),
                "error must state missing source: {err}"
            );
        });
    }

    /// Test is Unix-only: uses `dirs::home_dir()` which on Windows ignores HOME env var.
    #[cfg(unix)]
    #[test]
    fn test_install_check_passes_when_source_detected() {
        let tmp = tempfile::tempdir().expect("temp dir");
        seed_plugin_registry(tmp.path(), &["superpowers@claude-plugins-official"]);
        temp_env::with_var("HOME", Some(tmp.path()), || {
            check_skill_source(AgentPlatform::Claude).expect("must pass with source present");
        });
    }

    #[test]
    fn test_install_check_skips_non_claude_platform() {
        // 非 Claude 平台不做来源检查（技能来源是 Claude Code 生态概念）
        check_skill_source(AgentPlatform::Codex).expect("non-claude must skip check");
    }
```

- [ ] **Step 2: 确认测试失败**

Run: `cargo test -p gitflow-cli install_check`
Expected: 编译失败——`check_skill_source` 未定义。

- [ ] **Step 3: GREEN——实现 Step 0**

在 `detect_skill_sources` 之后追加：

```rust
/// 安装时技能来源前置检查（`install_skills` Step 0，Issue #141）。
///
/// 仅 Claude 平台执行；其他平台提示跳过。两来源皆无时返回错误（非 0 退出码）
/// 并打印三条安装引导——硬阻断，保证「装了 gf-workflow 就必然能跑」。
///
/// # Errors
///
/// Claude 平台且两来源皆未安装，或无法确定 HOME 目录时返回错误。
fn check_skill_source(platform: AgentPlatform) -> miette::Result<()> {
    if !matches!(platform, AgentPlatform::Claude) {
        println!("ℹ 非 Claude 平台，跳过技能来源检测");
        return Ok(());
    }
    let home = dirs::home_dir().ok_or_else(|| miette::miette!("无法确定 HOME 目录"))?;
    let sources = detect_skill_sources(&home);
    if sources.is_empty() {
        eprintln!("⛔ 未检测到任何技能来源，gf-workflow 无法运行。请先安装其一：");
        eprintln!("  · claude plugins install superpowers");
        eprintln!("  · claude plugins install mattpocock-skills");
        eprintln!("  · npx skills@latest add mattpocock/skills");
        return Err(miette::miette!("技能来源缺失，安装中止"));
    }
    let names: Vec<String> = sources.iter().map(ToString::to_string).collect();
    println!("✓ 检测到技能来源: {}", names.join(" + "));
    Ok(())
}
```

在 `install_skills` 函数体首部（`let platform = ...` 之后、`let target = ...` 之前）插入：

```rust
    // Step 0（Issue #141）：技能来源前置检查；两来源皆无硬阻断。
    check_skill_source(platform)?;
```

- [ ] **Step 4: 跑测试**

Run: `cargo test -p gitflow-cli install_check`
Expected: 3 个新测试 PASS。再跑 `cargo test -p gitflow-cli commands::skills` 确认无回归。

- [ ] **Step 5: 手工验证真实环境行为**

Run: `cargo build -p gitflow-cli && ./target/debug/gf skills install --agent claude --global=false --custom-path /tmp/gf-skills-e2e 2>&1 | head -5`
Expected: 本机装有 superpowers → 首行输出 `✓ 检测到技能来源: superpowers`，随后正常安装到临时目录。验证后 `rm -rf /tmp/gf-skills-e2e`。

（硬阻断路径无法在装有 superpowers 的本机直接验证，由单测覆盖；不伪造 HOME 跑真实命令。）

- [ ] **Step 6: REFACTOR + pedantic**

Run: `cargo clippy -p gitflow-cli --all-targets -- -D warnings -W clippy::pedantic`
Expected: 0 warnings。

- [ ] **Step 7: Commit**

```bash
git add apps/cli/src/commands/skills.rs
git commit -m "feat(skills): hard-block install when no skill source present (#141)"
```

---

### Task 5: SKILL.md — Skill Source Resolution + 角色别名 + GO 闸门

**Files:**
- Modify: `skills/gf-workflow/SKILL.md`

**Interfaces:**
- Consumes: Task 1 字段语义；Task 6 的 references.md 节名（「双来源映射表」「来源分支语义」「Phase 3 执行模式」）——本任务先按这些名称引用，Task 6 落地
- Produces: 编排器运行时指令——检测时机/结果矩阵/暂停语义/GO 闸门；后续所有文档以此为准

**验证方式**：Edit 序列完成后跑断言 grep（Step 3）。

- [ ] **Step 1: 依次应用以下 Edit（old → new 精确替换）**

**Edit 5.1 — Contract First 列表加入检测步骤：**

old:
```
2. Run mode auto-detection (full / standard / fast)
3. Create the contract file at `.cache/workflows/active/<workflow_id>.json` (schema: `contract.schema.json`)
4. Announce the workflow start with: workflow_id, mode, title
```
new:
```
2. Run mode auto-detection (full / standard / fast)
3. **Detect skill source** — see `## Skill Source Resolution`. Runs BEFORE the contract exists; if both sources are absent the user chooses inline-continue or abort (abort → no contract)
4. Create the contract file at `.cache/workflows/active/<workflow_id>.json` (schema: `contract.schema.json`), then record `skill_source` via jq immediately after creation
5. Announce the workflow start with: workflow_id, mode, title, `skill_source`
```

**Edit 5.2 — 在 `**If no contract exists, no sub-skill may be invoked.**` 段落之后、`### Cross-Session Resume` 之前插入新节：**

```markdown
## Skill Source Resolution

gf-workflow runs on top of ONE external skill source: `superpowers` or `mattpocock/skills`.
Phase steps below use **role aliases** only; actual skill names resolve via
`references.md` → **Dual-Source Mapping Table** (single point of maintenance).

### Detection (at Bootstrap, BEFORE contract creation)

1. Introspect the session's available-skills list (primary signal — "invocable as detected").
   Filesystem probing is diagnostics-only for error messages, never a decision source.
2. Sentinels (each matches namespaced or bare form; bare form requires double hits):
   - superpowers: `superpowers:brainstorming` (or bare `brainstorming` + `writing-plans`)
   - mattpocock: `to-spec` + `grilling` double hit (namespaced `mattpocock-skills:*` or bare)
3. Result matrix:

| Detection | Action | `skill_source` |
|---|---|---|
| Only superpowers | adopt | `superpowers` |
| Only mattpocock | adopt | `mattpocock` |
| Both present | **ASK user** which source this workflow uses — no default priority | user's choice |
| Neither | **ASK**: continue inline / abort (never degrade silently; abort → no contract) | `inline` if continuing |

4. Record: `jq '.skill_source = "<value>"'` on the contract right after creation.
5. Resume: reuse the contract's `skill_source`, then re-verify its sentinels are still
   present; if vanished, re-run the neither-present prompt.

### Pause Semantics (mattpocock path)

mattpocock's `to-spec` / `to-tickets` / `implement` are `disable-model-invocation: true` —
the orchestrator MUST NOT attempt to invoke them. At each such step: ✋ **PAUSE**, print the
exact slash command with its constraint instructions from `references.md` → Source Branch
Semantics, and wait for the user to run it.
```

**Edit 5.3 — Cross-Session Resume 表后补一行：**

old:
```
Full recovery procedure: see `references.md` → Cross-Session Recovery.
```
new:
```
Full recovery procedure: see `references.md` → Cross-Session Recovery.
`skill_source` is always loaded from the contract (never re-detected silently) and re-verified per `## Skill Source Resolution`.
```

**Edit 5.4 — Phase 1 Step 3 分支化：**

old:
```
3. **[CALL] `superpowers:brainstorming`**
   - Pass: Issue description or user requirements
   - **⚠️ RETURN RULE:** Terminal state = **RETURN TO ORCHESTRATOR** (not `writing-plans`)
   - Brainstorming will: explore context → ask questions → propose approaches → present design → write spec → **return control**
   - Output: `design_doc_path`
```
new:
```
3. **[CALL] Clarification skill** (per `skill_source`; names in `references.md` → Dual-Source Mapping Table)
   - superpowers: `superpowers:brainstorming` (model-invoked)
   - mattpocock: `grilling` (model-invoked), then ✋ PAUSE → user runs `/to-spec` (local-only constraint — see references.md; issue creation stays with `gf-issue-create`)
   - inline: orchestrator self-interviews, then writes the design doc itself
   - Pass: Issue description or user requirements
   - **⚠️ RETURN RULE:** Terminal state = **RETURN TO ORCHESTRATOR** (not `writing-plans`)
   - Output: `design_doc_path`
```

**Edit 5.5 — Phase 1 Step 4 补注：**

old:
```
4. **[AUTO] `gf-issue-create`** — **MANDATORY**
   - Create Issue (or use existing), reference design doc in body
   - Output: `issue_url`
```
new:
```
4. **[AUTO] `gf-issue-create`** — **MANDATORY**
   - Create Issue (or use existing), reference design doc in body
   - mattpocock path: issue creation authority is UNIFIED here — `/to-spec` never publishes to the tracker
   - Output: `issue_url`
```

**Edit 5.6 — Phase 2 Step 1 分支化：**

old:
```
| 1 | **[CALL]** `superpowers:writing-plans` (input: `design_doc_path`) — **⚠️ RETURN to orchestrator**. Create a full plan covering architecture, data flow, API design, component tree, and route design. The plan must create a full plan document with all design decisions. | `spec_path` |
```
new:
```
| 1 | **[CALL] Planning skill** (per `skill_source`) — **⚠️ RETURN to orchestrator**. superpowers: `superpowers:writing-plans` (input: `design_doc_path`) → full plan document (architecture, data flow, API design, component tree, route design). mattpocock: ✋ PAUSE → user runs `/to-tickets` on the Phase 1 spec; orchestrator records `ticket_refs` and sets `spec_path` = the spec file `to-tickets` consumed; the gate presents the ticket list + blocking edges. | `spec_path` (+ `ticket_refs` on mattpocock) |
```

**Edit 5.7 — Phase 2 Step 4 GO 闸门：**

old:
```
| 4 | **[PAUSE]** Gate 2→3 + user approval: "approved" → Phase 3 · "changes" → revise · "rejected" → terminate | `user_approved` |
```
new:
```
| 4 | **[PAUSE]** Gate 2→3 + user approval: "approved" → **execution-mode choice (GO gate)**: ① background agent (default, superpowers only) ② manual new window ③ same-session (explicit request only); mattpocock menu is trimmed — see `references.md` → Phase 3 Execution Modes · "changes" → revise · "rejected" → terminate | `user_approved` |
```

**Edit 5.8 — Phase 3 Steps 1-2 分支化：**

old:
```
| 1 | **[AUTO]** Record `base_branch` via `git rev-parse --abbrev-ref HEAD`, then create worktree: `feat/<issue-number>-<short-description>` | `branch`, `base_branch`, `worktree_path` |
| 2 | **[AUTO]** `superpowers:subagent-driven-development` (TDD: RED → GREEN → REFACTOR) | implementation |
```
new:
```
| 1 | **[AUTO]** Record `base_branch` via `git rev-parse --abbrev-ref HEAD`. Worktree `feat/<issue-number>-<short-description>`: created here for same-session mode; created by the executor (background agent / new window) otherwise — see `references.md` → Phase 3 Execution Modes | `branch`, `base_branch`, `worktree_path` |
| 2 | **[AUTO] Execution engine** (per `skill_source` + chosen execution mode): superpowers → `superpowers:subagent-driven-development` (same-session) or `superpowers:executing-plans` (new window / background agent); mattpocock → ✋ PAUSE per ticket → user runs `/implement` in dependency order (internal `/tdd` mandatory). All paths: TDD RED → GREEN → REFACTOR | implementation |
```

**Edit 5.9 — Fast Mode checklist（三行替换）：**

old:
```
**Phase 1:** `gf-issue-create` (required), `superpowers:brainstorming` (optional)

**Phase 2:** `superpowers:writing-plans` (optional, skippable)

**Phase 3:** `superpowers:subagent-driven-development` with TDD + Code Review (required)
```
new:
```
**Phase 1:** `gf-issue-create` (required), Clarification skill per `skill_source` (optional)

**Phase 2:** Planning skill per `skill_source` (optional, skippable)

**Phase 3:** Execution engine per `skill_source` with TDD + Code Review (required)
```

**Edit 5.10 — Standard Mode checklist（三行替换）：**

old:
```
**Phase 1:** `superpowers:brainstorming` (required), `gf-issue-create` (required), `gf-issue-review` (required)

**Phase 2:** `superpowers:writing-plans` (required) + `gf-quality` gate (required)

**Phase 3:** `superpowers:subagent-driven-development` with TDD + Code Review (required)
```
new:
```
**Phase 1:** Clarification skill per `skill_source` (required), `gf-issue-create` (required), `gf-issue-review` (required)

**Phase 2:** Planning skill per `skill_source` (required) + `gf-quality` gate (required)

**Phase 3:** Execution engine per `skill_source` with TDD + Code Review (required)
```

**Edit 5.11 — Red Flags 表追加三行（表格末尾）：**

```markdown
| About to invoke a source sub-skill without reading `references.md` mapping table | **STOP** — read the mapping table first; resolve names from the session skills list |
| About to auto-invoke a user-invoked skill (`/to-spec`, `/to-tickets`, `/implement`) | **STOP** — these are `disable-model-invocation`; ✋ PAUSE and prompt the user |
| About to run Phase 3 same-session without an explicit user request | **STOP** — Gate 2→3 includes execution-mode choice; same-session is explicit-only |
| About to let `/to-spec` publish to the tracker | **STOP** — local-only constraint; issue creation belongs to `gf-issue-create` |
```

**Edit 5.12 — Rationalization Table 追加三行（表格末尾）：**

```markdown
| "to-spec can publish the issue itself" | No — to-spec is constrained local-only; issue creation is unified under `gf-issue-create`. |
| "The background agent can run /implement" | No — /implement is user-invoked; mattpocock's mode menu is trimmed to new-window / same-session. |
| "Both sources installed — pick the better one" | No priority — ask the user which source this workflow uses. |
```

**Edit 5.13 — When NOT to Use 行：**

old:
```
**When NOT to Use:** quick fix → `gf-commit` · PR review → `gf-pr-review` · architecture discussion → `superpowers:brainstorming` directly · user says "don't create an Issue" → do NOT invoke.
```
new:
```
**When NOT to Use:** quick fix → `gf-commit` · PR review → `gf-pr-review` · architecture discussion → the installed source's clarification skill directly (per `references.md` mapping) · user says "don't create an Issue" → do NOT invoke.
```

- [ ] **Step 2: 验证断言**

Run:
```bash
# 无残留的无条件 superpowers [CALL] 指令
! grep -nE '\[CALL\]\*?\*? `superpowers:' skills/gf-workflow/SKILL.md
# 新节就位
grep -c "Skill Source Resolution" skills/gf-workflow/SKILL.md   # ≥ 2
grep -c "disable-model-invocation" skills/gf-workflow/SKILL.md # ≥ 2
grep -c "execution-mode" skills/gf-workflow/SKILL.md           # ≥ 1
```
Expected: 首条无匹配（退出码 1 被 `!` 反转）；计数达标。

- [ ] **Step 3: 校对渲染**——通读修改后的 SKILL.md，确认表格未断行、Phase 编号连贯（Contract First 从 4 步变 5 步后无残留引用）。

- [ ] **Step 4: Commit**

```bash
git add skills/gf-workflow/SKILL.md
git commit -m "feat(workflow): skill source resolution, role aliases, GO gate in SKILL.md (#141)"
```

---

### Task 6: references.md — 双来源映射表 + 分支语义（单点维护）

**Files:**
- Modify: `skills/gf-workflow/references.md`（文末追加两大节）

**Interfaces:**
- Consumes: 设计文档 §4-§7；Task 3 哨兵常量（逐字一致）
- Produces: SKILL.md 引用的三个节名——`## Dual-Source Skill Resolution`（含「Sentinels」「Detection & Recording」「Dual-Source Mapping Table」「Source Branch Semantics」「Phase 3 Execution Modes」小节）。哨兵表是运行时侧权威清单，与 `apps/cli/src/commands/skills.rs` 常量互为镜像。

- [ ] **Step 1: 在 references.md 文末追加以下内容（整段复制）**

````markdown
## Dual-Source Skill Resolution (Issue #141)

gf-workflow runs on ONE external skill source: `superpowers` or `mattpocock/skills`.
SKILL.md phase steps use role aliases; actual names resolve from the tables below.
**This section is the single point of maintenance** — upstream renames are fixed here only.

### Sentinels (shared authoritative definition)

Mirrored by the Rust constants in `apps/cli/src/commands/skills.rs`
(`SUPERPOWERS_PLUGIN_PREFIX` / `MATTPOCOCK_PLUGIN_PREFIX` /
`SUPERPOWERS_BARE_SENTINELS` / `MATTPOCOCK_BARE_SENTINELS`) used by install-time
Step 0. Change both sides together.

| Source | Namespaced form | Bare form (double hit required) |
|---|---|---|
| superpowers | `superpowers:brainstorming` | `brainstorming` + `writing-plans` |
| mattpocock | `mattpocock-skills:to-spec` + `mattpocock-skills:grilling` | `to-spec` + `grilling` |

Bare forms cover skills.sh / symlink installs. A partial hit (e.g. only `to-spec`)
counts as absent; report which sentinel is missing.

### Detection & Recording

- Mechanism: introspect the session available-skills list at Bootstrap, BEFORE the
  contract exists. Filesystem probing is diagnostics-only.
- Both present → ask the user which source this workflow uses (no default priority).
- Neither present → ask: continue inline (`skill_source: "inline"`) or abort (no contract).
- Record after contract creation:

```bash
jq --arg src "<superpowers|mattpocock|inline>" \
   '.skill_source = $src | .updated_at = (now | todate)' \
   ".cache/workflows/active/${WORKFLOW_ID}.json" > tmp && mv tmp ".cache/workflows/active/${WORKFLOW_ID}.json"
```

- Resume: reuse `skill_source` from the contract; re-verify sentinels are still present;
  if vanished, re-run the neither-present prompt.

### Dual-Source Mapping Table

| Role alias | superpowers | mattpocock | Invocation form |
|---|---|---|---|
| Clarification | `brainstorming` | `grilling` | model-invoked / model-invoked |
| Spec | (merged into brainstorming design doc) | ✋ `/to-spec` (local-only) | — / user-invoked |
| Issue creation | `gf-issue-create` | `gf-issue-create` (unchanged; authority unified) | gf CLI |
| Issue review | `gf-issue-review` | `gf-issue-review` (unchanged) | gf CLI |
| Planning | `writing-plans` | ✋ `/to-tickets` | model-invoked / user-invoked |
| Quality gate | `gf-quality` | `gf-quality` (unchanged) | gf CLI |
| Execution engine | `subagent-driven-development` (same-session) / `executing-plans` (new window) / background agent — per GO gate | ✋ `/implement` per ticket (internal `/tdd` mandatory) | per mode / user-invoked |
| Execution review | SDD built-in two-stage review | `code-review` (driven inside `/implement`) | — |
| Delivery review | `gf-review` | `gf-review` (unchanged; no extra code-review pass) | gf skill |
| Triage (full mode) | `gf-issue-triage` | `gf-issue-triage` (unchanged; mattpocock `triage` NOT adopted) | gf skill |
| Pipeline analysis | `gf-pipeline-analyzer` | (unchanged) | gf skill |

### Source Branch Semantics

**Invariants (both sources):** four-phase state machine, three gates, contract evidence
semantics, all `gf-*` steps, mandatory TDD + code review, mode matrix (full/standard/fast).

**mattpocock path:**

- **Prerequisite:** `docs/agents/issue-tracker.md` exists (`setup-mat-pocock-skills`
  output). Missing → ask the user: run `setup-mat-pocock-skills` now (one-time) or abort.
- **Phase 1:** `grilling` (auto) → ✋ PAUSE prompting:

  > 请运行 `/to-spec`：综合当前对话撰写 spec，写入本地文件
  > `docs/specs/<workflow-id>-spec.md`。**只写本地，不发布 tracker、不打标签。**

  Verify the local spec exists afterwards. **Fallback** (constraint failed / skill refused):
  the orchestrator writes the design doc itself from the grilling record, bypassing `to-spec`.
  Then `gf-issue-create` creates the Issue (authority unified — no duplicate) and
  `gf-issue-review` reviews it. Evidence: `issue_url`, `comment_id`, `design_doc_path`.
- **Phase 2:** ✋ PAUSE prompting `/to-tickets` with the Phase 1 spec reference.
  `to-tickets` publishes tickets per the configured tracker (local `.scratch/<feature>/issues/`
  files or real tracker issues) and includes its own breakdown quiz. Its rule "do NOT close
  or modify any parent issue" is compatible with gf-workflow. Orchestrator records
  `ticket_refs` (paths/URLs) and sets `spec_path` = the Phase 1 spec file. Gate 2→3
  presents the ticket list + blocking edges.
- **Phase 3:** worktree per chosen execution mode; per ticket in dependency order
  (frontier): ✋ PAUSE → user runs `/implement` (internal `/tdd` + `/code-review` + commit);
  suggest `/clear` between tickets (context recovery via contract + ticket files).
  `gf-pr-create` with `Closes #<issue>`, then `make test`.
- **Phase 4:** identical to superpowers (pipeline → triage[full] → gf-review →
  dogfooding[full] → Branch Finish → archive). `code-review` already ran inside `/implement`.
- **Evidence mapping:** `design_doc_path` ← local spec file; `spec_path` ← same spec file
  (what `to-tickets` consumed); `ticket_refs` ← ticket paths/URLs.

**inline source (both absent, user chose to continue):** orchestrator performs each phase
inline — self-interview, self-written design doc/plan, TDD loop with a review subagent.
Evidence fields follow the superpowers shape. Degraded but explicit.

### Phase 3 Execution Modes (GO gate)

Gate 2→3 = plan approval + execution-mode choice. Same-session execution left the default
menu (SDD objection: approving a plan ≠ authorizing hours of autonomous subagent fan-out;
same-session SDD hijacks the conversation once started).

| Mode | Description | Availability |
|---|---|---|
| ① Background agent ⭐default | Dispatch with `isolation: worktree` + `run_in_background`; handoff = contract path + plan doc + engine instructions; `task-notification` returns to the original window; executor writes evidence back to the contract | superpowers only (`/implement` is user-invoked → unusable on mattpocock) |
| ② Manual new window | Print opening guidance: worktree path (or creation command) + contract recovery command (`gf workflow status <id>` + plan doc path); new window creates branch/worktree itself and runs `executing-plans` (superpowers) or per-ticket `/implement` (mattpocock); user reports back, orchestrator verifies evidence | both sources |
| ③ Same-session | Current behavior: orchestrator creates worktree and drives the engine inline | explicit request only |

Quality compensation: `executing-plans` (light path) lacks per-task review → gates
compensate (`make test` before PR + Phase 4 `gf-review`). SDD carries per-task review built in.
````

- [ ] **Step 2: 验证哨兵表与 Rust 常量一致**

Run:
```bash
grep -A2 'SUPERPOWERS_BARE_SENTINELS\|MATTPOCOCK_BARE_SENTINELS' apps/cli/src/commands/skills.rs | head -6
grep -n 'brainstorming.*writing-plans\|to-spec.*grilling' skills/gf-workflow/references.md
```
Expected: 两侧哨兵名单逐字一致（brainstorming+writing-plans / to-spec+grilling）。

- [ ] **Step 3: Commit**

```bash
git add skills/gf-workflow/references.md
git commit -m "feat(workflow): dual-source mapping table and branch semantics in references.md (#141)"
```

---

### Task 7: gates.md — Gate 2→3 GO 子步骤 + 证据来源无关说明

**Files:**
- Modify: `skills/gf-workflow/gates.md`

**Interfaces:**
- Consumes: Task 6 的「Phase 3 Execution Modes」节名
- Produces: 闸门定义与 GO 闸门一致（Gate 2→3 = 唯一暂停点 = 批准 + 模式选择）

- [ ] **Step 1: 应用两处 Edit**

**Edit 7.1 —** 在 `### Gate 2→3: 计划制定 → 执行` 的「**暂停行为:**」段落之后追加：

```markdown
**GO 闸门——执行模式选择（Issue #141）:** 用户批准后、进入 Phase 3 前，编排器必须提供执行模式选择：
① 后台代理（默认推荐，仅 superpowers 来源可用）② 手动新窗口 ③ 同会话执行（仅显式要求）。
mattpocock 来源下菜单自动裁剪为 ②③（`/implement` 为 user-invoked，后台代理无法调用）。
模式语义详见 `references.md` → Phase 3 Execution Modes。
```

**Edit 7.2 —** 在 `## 门控定义` 标题之后、`### Gate 1→2` 之前插入：

```markdown
> **证据字段来源无关（Issue #141）:** 三个闸门的证据条件不因技能来源变化——
> `design_doc_path` / `spec_path` 等字段由两来源各自的产物填充
> （映射见 `references.md` → Source Branch Semantics），闸门只校验字段存在性与取值。
> mattpocock 路径额外产出 `ticket_refs`（可选字段，不参与闸门判定，Gate 2→3 展示用）。
```

- [ ] **Step 2: 验证**

Run: `grep -c "GO 闸门\|来源无关" skills/gf-workflow/gates.md`
Expected: ≥ 2。

- [ ] **Step 3: Commit**

```bash
git add skills/gf-workflow/gates.md
git commit -m "docs(workflow): gates.md GO-gate sub-step and source-agnostic evidence note (#141)"
```

---

### Task 8: docs 同步（gf-workflow-guide + integration-guide + index）

**Files:**
- Modify: `docs/gf-workflow-guide.md`
- Modify: `docs/integration-guide.md`
- Modify: `docs/index.md`

**Interfaces:**
- Consumes: Task 5-7 落地的 skill 文档语义
- Produces: AC「同步更新两份 docs」+ 设计文档登记

- [ ] **Step 1: gf-workflow-guide.md——在「三种模式」表格之后插入新节**

```markdown
## 技能来源适配（Issue #141）

gf-workflow 支持两种外部技能来源：**superpowers** 与 **mattpocock/skills**。启动时自动检测
（技能清单探测 + 哨兵规则），结果写入契约 `skill_source`，跨会话沿用。

| | superpowers（全自动流水线） | mattpocock（人工驾驶流水线） |
|---|---|---|
| Phase 1 澄清 | `brainstorming` | `grilling` → ✋ `/to-spec`（只写本地） |
| Phase 1 Issue | `gf-issue-create` | `gf-issue-create`（创建权统一，不重复建） |
| Phase 2 计划 | `writing-plans` | ✋ `/to-tickets`（票据图 + blocking edges） |
| Phase 3 执行 | SDD / executing-plans / 后台代理（GO 闸门选择） | ✋ `/implement` 逐票据（内部 `/tdd`） |
| Phase 4 交付 | gf-* 骨架 | 完全相同 |
| 人的触点 | 2 个（Gate 2→3、Branch Finish） | 5 个（✋×3 + 审批 + 确认） |

**安装前置**：`gf skills install` 会检测来源，两者皆无时硬阻断并给出安装引导
（`claude plugins install superpowers` / `claude plugins install mattpocock-skills` /
`npx skills@latest add mattpocock/skills`）。

**Phase 3 GO 闸门**：Gate 2→3 批准后选择执行模式——后台代理（默认，仅 superpowers）/
手动新窗口 / 同会话（仅显式要求）。mattpocock 来源菜单自动裁剪。

检测规则、映射表与分支细节：`.claude/skills/gf-workflow/references.md`（单点维护）。
```

- [ ] **Step 2: integration-guide.md——在 Superpowers 集成章节之后追加 mattpocock 章**

定位现有 Superpowers 章节末尾（`## ` 级标题之间），追加：

```markdown
## mattpocock/skills 集成（Issue #141）

gf-workflow 同样支持 [mattpocock/skills](https://github.com/mattpocock/skills)
（plugin 名 `mattpocock-skills`）作为技能来源，与 Superpowers 互斥检测、按来源分支。

### 与 Superpowers 的关键差异

| | Superpowers | mattpocock/skills |
|---|---|---|
| 触发模型 | 模型全自动触发 | user-invoked 硬约束（`disable-model-invocation`）→ 暂停语义 ✋ |
| 计划产物 | 单一 plan 文档 | 票据图 + blocking edges（`ticket_refs`） |
| 主线 token | ≈14k + subagent 扇出 | ≈4.8k，不触发零消耗 |

### 前置条件

运行 `setup-mattpocock-skills` 生成 `docs/agents/issue-tracker.md`（tracker 与 triage
标签词表配置）。缺失时 gf-workflow 会询问：先配置或中止。

### 集成要点

- `to-spec` 受约束**只写本地** spec 文件、不发布 tracker；Issue 创建权统一归 `gf-issue-create`
- Phase 3 逐票据 ✋ `/implement`（内部强制 `/tdd` + `/code-review`）；Gate 2→3 执行模式
  菜单裁剪为「手动新窗口 / 同会话」（后台代理无法调用 user-invoked 技能）
- Phase 4 骨架不变：`gf-pipeline-analyzer` / `gf-issue-triage` / `gf-review` 照常
- 检测哨兵：`to-spec` + `grilling` 双命中（plugin 形 `mattpocock-skills:*` 或裸名）

完整映射表见 `.claude/skills/gf-workflow/references.md` → Dual-Source Skill Resolution。
```

- [ ] **Step 3: docs/index.md——登记设计文档**

在设计文档列表区域（`superpowers/specs` 引用附近）追加一行：

```markdown
- [gf-workflow 双 skills 来源兼容设计](./superpowers/specs/2026-08-08-workflow-dual-skill-sources-design.md) — Issue #141：superpowers + mattpocock/skills 双来源检测、分支适配、GO 闸门与安装时硬阻断。
```

- [ ] **Step 4: 校对链接**——确认两个 docs 中引用的 `.claude/skills/gf-workflow/references.md` 路径与实际一致。

- [ ] **Step 5: Commit**

```bash
git add docs/gf-workflow-guide.md docs/integration-guide.md docs/index.md
git commit -m "docs: dual skill source adaptation sections (#141)"
```

---

### Task 9: 安装副本同步 + 全局一致性检查

**Files:**
- Sync: `skills/gf-workflow/*` → `.claude/skills/gf-workflow/`（git-ignored）

**Interfaces:**
- Consumes: Task 1, 5, 6, 7 的最终文件
- Produces: 当前会话与后续 dogfooding 可见的新版 skill；一致性断言通过

- [ ] **Step 1: 同步副本**

```bash
cp skills/gf-workflow/SKILL.md skills/gf-workflow/references.md \
   skills/gf-workflow/gates.md skills/gf-workflow/contract.schema.json \
   .claude/skills/gf-workflow/
diff -r skills/gf-workflow .claude/skills/gf-workflow && echo "SYNCED"
```
Expected: `SYNCED`（无差异）。

- [ ] **Step 2: 全局断言**

```bash
# SKILL.md 中不再有未分支限定的 superpowers [CALL] 硬编码
! grep -nE '\[CALL\]\*?\*? `superpowers:' .claude/skills/gf-workflow/SKILL.md
# schema 新字段就位
jq -e '.properties.skill_source and .["$defs"].phase.evidence.properties.ticket_refs' \
  .claude/skills/gf-workflow/contract.schema.json
# 哨兵单点：references.md 与 Rust 常量并存
grep -c "Dual-Source Skill Resolution" .claude/skills/gf-workflow/references.md
```
Expected: 全部通过。

- [ ] **Step 3: Commit（无文件变更则跳过——副本被 gitignore）**

```bash
git status --short  # 确认 .claude/skills 未进入暂存
```

---

### Task 10: 完整 Rust 门禁

**Files:** 无新变更（验证任务）

- [ ] **Step 1: make build**

Run: `make build`
Expected: 成功。

- [ ] **Step 2: make test**

Run: `make test`
Expected: 全部通过（含 workflow.rs 3 个新测试 + skills.rs 11 个新测试）。

- [ ] **Step 3: make fmt**

Run: `make fmt`
Expected: 无格式差异（有差异则 `cargo +nightly fmt` 修正后重跑）。

- [ ] **Step 4: make clippy（pedantic）**

Run: `cargo clippy --all-targets --all-features -- -D warnings -W clippy::pedantic`
Expected: 0 warnings。

- [ ] **Step 5: make check-agent-sync**

Run: `make check-agent-sync`
Expected: 通过。

- [ ] **Step 6: 若有修正则 commit**

```bash
git add -A && git commit -m "chore: fmt/clippy fixes (#141)" || echo "nothing to commit"
```

---

### Task 11: E2E 验证清单（Phase 4 消费，非 Phase 3 执行）

**superpowers 路径 E2E** = 本 workflow 实例 `wf-2026-08-08-001`：

- [ ] 本 workflow 的 Gate 2→3 已按新 GO 闸门语义执行模式选择（新行为的手工预演）
- [ ] Phase 4 dogfooding checklist 时核对：检测→契约→分支适配链路在本实例全程生效

**mattpocock 路径 E2E**（真实插件安装；需用户操作交互步骤）：

- [ ] 用户安装插件：`/plugin marketplace add mattpocock/skills` → `/plugin install mattpocock-skills@mattpocock`
- [ ] 新会话确认 `mattpocock-skills:*` 出现在 available-skills
- [ ] `gf skills install` 输出 `✓ 检测到技能来源: superpowers + mattpocock`（共存场景）
- [ ] 在独立验证仓库（真实 GitHub tracker）安装 feature 分支 gf skills，运行 `/gf-workflow` 小需求：
  - [ ] 检测报两来源共存 → 询问用户 → 选 mattpocock → 契约 `skill_source: "mattpocock"`
  - [ ] `grilling` 自动 → ✋ `/to-spec` 本地约束生效（无 tracker 发布、无重复 Issue）
  - [ ] ✋ `/to-tickets` → `ticket_refs` 记录 → Gate 2→3 展示票据清单 + 模式菜单裁剪（无后台代理项）
  - [ ] ✋ `/implement` 逐票据 → PR + make test → Phase 4 骨架完整
- [ ] 清理：测试仓库产物处理；插件去留由用户决定

---

## Self-Review 记录

1. **Spec coverage**：设计文档 §4（检测）→ Task 3/4 + Task 5/6；§5（映射表）→ Task 6；§6（分支语义）→ Task 5/6；§7（GO 闸门）→ Task 5（Edit 5.7/5.8）+ 6 + 7；§8（schema/Rust）→ Task 1/2；安装时硬阻断（§2.7/§4.3）→ Task 3/4；§9（docs）→ Task 8；§11（E2E）→ Task 11；验收 20 条均有任务承载。✅
2. **Placeholder scan**：无 TBD/TODO；所有代码块为完整可用代码；doc 任务给出整段待插入文本。✅
3. **Type consistency**：`detect_skill_sources(home: &Path) -> Vec<SkillSourceKind>`（Task 3）被 Task 4 `check_skill_source` 原样消费；`WorkflowContract.skill_source: Option<String>` / `PhaseEvidence.ticket_refs: Option<Vec<String>>`（Task 2）与 schema（Task 1）字段名逐字一致；references.md 哨兵表与 Rust 常量名在 Task 6 Step 2 显式核对。✅
