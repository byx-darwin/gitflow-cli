//! `gitflow workflow` 子命令实现。
//!
//! 管理工作流合同的创建、读取、归档和清理。
//! 合同存储在 `.cache/workflows/active/` 和 `.cache/workflows/archive/`。

// 本模块仅做本地文件 I/O（同步操作），无需 tokio::fs。
#![allow(
    clippy::disallowed_methods,
    reason = "Workflow contract I/O is synchronous local file access, not async network I/O"
)]

use std::{
    collections::BTreeMap,
    fmt,
    path::{Path, PathBuf},
};

use chrono::{DateTime, TimeDelta, Utc};
use clap::{Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};

/// 工作流模式：完整模式（四阶段）或快速模式（跳过阶段二）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum WorkflowMode {
    /// 完整四阶段流程。
    Full,
    /// 快速模式，跳过计划制定阶段。
    Fast,
}

impl fmt::Display for WorkflowMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Full => write!(f, "full"),
            Self::Fast => write!(f, "fast"),
        }
    }
}

/// 阶段状态。
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PhaseStatus {
    /// 待处理。
    #[default]
    Pending,
    /// 进行中。
    InProgress,
    /// 已完成。
    Complete,
    /// 已跳过（快速模式下适用）。
    Skipped,
}

/// 阶段证据，记录各阶段的关键产出物。
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PhaseEvidence {
    /// Issue 链接。
    pub issue_url: Option<String>,
    /// 评论 ID。
    pub comment_id: Option<String>,
    /// 规格文件路径。
    pub spec_path: Option<String>,
    /// 用户是否已批准。
    pub user_approved: Option<bool>,
    /// 关联分支。
    pub branch: Option<String>,
    /// PR 链接。
    pub pr_url: Option<String>,
    /// 测试是否通过。
    pub tests_passed: Option<bool>,
    /// 流水线是否 OK。
    pub pipeline_ok: Option<bool>,
    /// 审查报告路径。
    pub review_report_path: Option<String>,
}

/// 工作流阶段。
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Phase {
    /// 阶段名称。
    pub name: String,
    /// 阶段状态。
    pub status: PhaseStatus,
    /// 开始时间（RFC 3339）。
    pub started_at: Option<String>,
    /// 完成时间（RFC 3339）。
    pub completed_at: Option<String>,
    /// 执行者。
    pub executor: Option<String>,
    /// 阶段证据。
    pub evidence: PhaseEvidence,
}

/// 关卡检查结果。
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GateCheck {
    /// 通过。
    Pass,
    /// 缺少证据（证据名称）。
    MissingEvidence(String),
    /// 前一阶段未完成（阶段编号）。
    PhaseNotComplete(u8),
}

/// 工作流合同，记录从需求到交付的完整执行轨迹。
#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowContract {
    /// 合同版本。
    pub version: String,
    /// 工作流 ID（格式: `wf-YYYY-MM-DD-NNN`）。
    pub workflow_id: String,
    /// 标题。
    pub title: String,
    /// 模式。
    pub mode: WorkflowMode,
    /// 创建时间（RFC 3339）。
    pub created_at: String,
    /// 更新时间（RFC 3339）。
    pub updated_at: String,
    /// 当前阶段（1-4）。
    pub current_phase: u8,
    /// 阶段映射（键 `"1"`-`"4"` 对应四个阶段，与合同 schema 保持一致）。
    pub phases: BTreeMap<String, Phase>,
}

#[allow(
    dead_code,
    reason = "new() is used by tests and will be used by the workflow orchestrator"
)]
impl WorkflowContract {
    /// 创建一个新的工作流合同。
    ///
    /// 自动生成 `workflow_id`（格式 `wf-YYYY-MM-DD-NNN`），
    /// 初始化为阶段一进行中，其余阶段待处理。
    #[must_use]
    pub fn new(title: String, mode: WorkflowMode) -> Self {
        let now = Utc::now();
        let date = now.format("%Y-%m-%d").to_string();
        // 简化实现：固定后缀 001，实际应扫描目录自增
        let workflow_id = format!("wf-{date}-001");

        Self {
            version: "1.0".to_string(),
            workflow_id,
            title,
            mode,
            created_at: now.to_rfc3339(),
            updated_at: now.to_rfc3339(),
            current_phase: 1,
            phases: BTreeMap::from([
                (
                    "1".to_string(),
                    Phase {
                        name: "需求澄清".into(),
                        status: PhaseStatus::InProgress,
                        started_at: Some(now.to_rfc3339()),
                        ..Default::default()
                    },
                ),
                (
                    "2".to_string(),
                    Phase {
                        name: "计划制定".into(),
                        ..Default::default()
                    },
                ),
                (
                    "3".to_string(),
                    Phase {
                        name: "执行".into(),
                        ..Default::default()
                    },
                ),
                (
                    "4".to_string(),
                    Phase {
                        name: "交付".into(),
                        ..Default::default()
                    },
                ),
            ]),
        }
    }

    /// 检查是否可以进入目标阶段。
    ///
    /// 根据当前各阶段状态和证据检查关卡条件：
    /// - 进入阶段二：阶段一必须完成且有 `issue_url`
    /// - 进入阶段三：快速模式跳过阶段二；完整模式需阶段二完成且有 `spec_path` 和 `user_approved`
    /// - 进入阶段四：阶段三必须完成且有 `pr_url` 和 `tests_passed`
    #[must_use]
    pub fn can_enter_phase(&self, target: u8) -> GateCheck {
        match target {
            2 => {
                let Some(phase1) = self.phases.get("1") else {
                    return GateCheck::PhaseNotComplete(1);
                };
                if phase1.status != PhaseStatus::Complete {
                    return GateCheck::PhaseNotComplete(1);
                }
                if phase1.evidence.issue_url.is_none() {
                    return GateCheck::MissingEvidence("issue_url".into());
                }
                if self.mode == WorkflowMode::Full && phase1.evidence.comment_id.is_none() {
                    return GateCheck::MissingEvidence("comment_id".into());
                }
                GateCheck::Pass
            }
            3 => {
                // 快速模式跳过阶段二
                if self.mode == WorkflowMode::Fast {
                    return GateCheck::Pass;
                }
                let Some(phase2) = self.phases.get("2") else {
                    return GateCheck::PhaseNotComplete(2);
                };
                if phase2.status != PhaseStatus::Complete {
                    return GateCheck::PhaseNotComplete(2);
                }
                if phase2.evidence.spec_path.is_none() {
                    return GateCheck::MissingEvidence("spec_path".into());
                }
                if phase2.evidence.user_approved != Some(true) {
                    return GateCheck::MissingEvidence("user_approved".into());
                }
                GateCheck::Pass
            }
            4 => {
                let Some(phase3) = self.phases.get("3") else {
                    return GateCheck::PhaseNotComplete(3);
                };
                if phase3.status != PhaseStatus::Complete {
                    return GateCheck::PhaseNotComplete(3);
                }
                if phase3.evidence.pr_url.is_none() {
                    return GateCheck::MissingEvidence("pr_url".into());
                }
                if phase3.evidence.tests_passed != Some(true) {
                    return GateCheck::MissingEvidence("tests_passed".into());
                }
                GateCheck::Pass
            }
            _ => GateCheck::MissingEvidence("invalid phase".into()),
        }
    }
}

/// CLI 子命令枚举。
#[derive(Debug, Subcommand)]
pub enum WorkflowCommand {
    /// 列出当前 active workflows。
    List,
    /// 查看 workflow 合同详情（JSON 格式）。
    Status {
        /// 工作流 ID。
        workflow_id: String,
    },
    /// 归档已完成的 workflow（仅阶段四完成才可归档）。
    Archive {
        /// 工作流 ID。
        workflow_id: String,
    },
    /// 清理过期归档。
    Cleanup {
        /// 超过多少天的归档会被清理（默认 90）。
        #[arg(long, default_value = "90")]
        older_than: i64,
    },
}

/// 处理 `gitflow workflow` 子命令。
///
/// # Errors
///
/// 返回错误当：
/// - 指定的 workflow 不存在。
/// - 文件读写失败。
/// - workflow 未完成时尝试归档。
pub fn handle(command: WorkflowCommand) -> miette::Result<()> {
    match command {
        WorkflowCommand::List => list_workflows(),
        WorkflowCommand::Status { workflow_id } => show_status(&workflow_id),
        WorkflowCommand::Archive { workflow_id } => archive_workflow(&workflow_id),
        WorkflowCommand::Cleanup { older_than } => cleanup_archives(older_than),
    }
}

/// 获取 active workflow 目录。
fn workflow_dir() -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    cwd.join(".cache/workflows/active")
}

/// 获取归档目录。
fn archive_dir() -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    cwd.join(".cache/workflows/archive")
}

/// 列出所有 active workflows。
fn list_workflows() -> miette::Result<()> {
    let dir = workflow_dir();
    if !dir.exists() {
        println!("(无 active workflows)");
        return Ok(());
    }
    let mut found = 0;
    for entry in std::fs::read_dir(&dir).map_err(|e| miette::miette!("读取目录失败: {e}"))? {
        let entry = entry.map_err(|e| miette::miette!("读取条目失败: {e}"))?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            let content =
                std::fs::read_to_string(&path).map_err(|e| miette::miette!("读取合同失败: {e}"))?;
            let contract: WorkflowContract =
                serde_json::from_str(&content).map_err(|e| miette::miette!("解析合同失败: {e}"))?;
            println!(
                "  {} | {} | Phase {} | {}",
                contract.workflow_id, contract.title, contract.current_phase, contract.mode
            );
            found += 1;
        }
    }
    if found == 0 {
        println!("(无 active workflows)");
    } else {
        println!("\n共 {found} 个 active workflows");
    }
    Ok(())
}

/// 查看指定 workflow 的合同详情（JSON 格式）。
fn show_status(workflow_id: &str) -> miette::Result<()> {
    let path = workflow_dir().join(format!("{workflow_id}.json"));
    if !path.exists() {
        return Err(miette::miette!("workflow {workflow_id} 不存在"));
    }
    let content =
        std::fs::read_to_string(&path).map_err(|e| miette::miette!("读取合同失败: {e}"))?;
    let contract: WorkflowContract =
        serde_json::from_str(&content).map_err(|e| miette::miette!("解析合同失败: {e}"))?;
    println!(
        "{}",
        serde_json::to_string_pretty(&contract).map_err(|e| miette::miette!("{e}"))?
    );
    print_gate_status(&contract);
    Ok(())
}

/// 打印合同各目标阶段的门控检查结果。
///
/// 遍历目标阶段 2-4，展示当前是否满足进入条件（复用 `can_enter_phase`）。
fn print_gate_status(contract: &WorkflowContract) {
    println!("\n门控检查（当前可进入的 Phase）:");
    for target in 2u8..=4 {
        match contract.can_enter_phase(target) {
            GateCheck::Pass => println!("  Phase {target}: 可进入"),
            GateCheck::MissingEvidence(ev) => {
                println!("  Phase {target}: 阻止（缺少证据 {ev}）");
            }
            GateCheck::PhaseNotComplete(p) => {
                println!("  Phase {target}: 阻止（阶段 {p} 未完成）");
            }
        }
    }
}

/// 归档已完成的 workflow。
///
/// 仅当当前阶段为四且阶段四状态为 `Complete` 时才可归档。
/// 归档按月分目录存放（`archive/YYYY-MM/`）。
fn archive_workflow(workflow_id: &str) -> miette::Result<()> {
    archive_workflow_at(&workflow_dir(), &archive_dir(), workflow_id)
}

/// 在指定的 active/archive 目录下归档已完成的 workflow。
///
/// 抽出目录参数便于单元测试注入临时目录，逻辑与 [`archive_workflow`] 一致。
fn archive_workflow_at(
    active_dir: &Path,
    archive_root: &Path,
    workflow_id: &str,
) -> miette::Result<()> {
    let src = active_dir.join(format!("{workflow_id}.json"));
    if !src.exists() {
        return Err(miette::miette!("workflow {workflow_id} 不存在"));
    }
    let content =
        std::fs::read_to_string(&src).map_err(|e| miette::miette!("读取合同失败: {e}"))?;
    let contract: WorkflowContract =
        serde_json::from_str(&content).map_err(|e| miette::miette!("解析合同失败: {e}"))?;
    let phase4_status = contract.phases.get("4").map(|p| p.status);
    if contract.current_phase != 4 || phase4_status != Some(PhaseStatus::Complete) {
        return Err(miette::miette!(
            "workflow 未完成（current_phase={}, phase_4_status={:?}）",
            contract.current_phase,
            phase4_status
        ));
    }
    // 按月归档
    let now = Utc::now();
    let month_dir = archive_root.join(now.format("%Y-%m").to_string());
    std::fs::create_dir_all(&month_dir).map_err(|e| miette::miette!("创建归档目录失败: {e}"))?;
    let dst = month_dir.join(format!("{workflow_id}.json"));
    std::fs::copy(&src, &dst).map_err(|e| miette::miette!("复制到归档失败: {e}"))?;
    std::fs::remove_file(&src).map_err(|e| miette::miette!("删除 active 合同失败: {e}"))?;
    println!("workflow {workflow_id} 已归档到 {}", month_dir.display());
    Ok(())
}

/// 清理超过指定天数的过期归档。
///
/// 逐月扫描归档目录，删除创建时间早于阈值的合同文件，
/// 并清理因此变为空的月份目录。
fn cleanup_archives(older_than_days: i64) -> miette::Result<()> {
    cleanup_archives_at(&archive_dir(), older_than_days)
}

/// 在指定归档目录下清理过期合同。
///
/// 抽出目录参数便于单元测试注入临时目录，逻辑与 [`cleanup_archives`] 一致。
fn cleanup_archives_at(dir: &Path, older_than_days: i64) -> miette::Result<()> {
    if !dir.exists() {
        println!("(无归档)");
        return Ok(());
    }
    let duration = TimeDelta::try_days(older_than_days)
        .ok_or_else(|| miette::miette!("无效的天数: {older_than_days}"))?;
    let threshold = Utc::now() - duration;
    let mut cleaned = 0;
    for month_entry in
        std::fs::read_dir(dir).map_err(|e| miette::miette!("读取归档目录失败: {e}"))?
    {
        let month_entry = month_entry.map_err(|e| miette::miette!("读取月份目录失败: {e}"))?;
        let month_dir = month_entry.path();
        if !month_dir.is_dir() {
            continue;
        }
        for entry in
            std::fs::read_dir(&month_dir).map_err(|e| miette::miette!("读取归档条目失败: {e}"))?
        {
            let entry = entry.map_err(|e| miette::miette!("读取文件失败: {e}"))?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let content =
                std::fs::read_to_string(&path).map_err(|e| miette::miette!("读取合同失败: {e}"))?;
            let contract: WorkflowContract =
                serde_json::from_str(&content).map_err(|e| miette::miette!("解析合同失败: {e}"))?;
            if let Ok(created) = DateTime::parse_from_rfc3339(&contract.created_at)
                && created.with_timezone(&Utc) < threshold
            {
                std::fs::remove_file(&path)
                    .map_err(|e| miette::miette!("删除过期归档失败: {e}"))?;
                cleaned += 1;
            }
        }
        // 清理空月份目录
        if std::fs::read_dir(&month_dir).is_ok_and(|mut d| d.next().is_none()) {
            std::fs::remove_dir(&month_dir).ok();
        }
    }
    println!("已清理 {cleaned} 个过期归档");
    Ok(())
}

#[cfg(test)]
#[allow(
    clippy::panic,
    reason = "Test code: panic is acceptable for assertion failures"
)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_contract_new_id_format() {
        let contract = WorkflowContract::new("feat: test".to_string(), WorkflowMode::Full);
        assert!(
            regex::Regex::new(r"^wf-\d{4}-\d{2}-\d{2}-\d{3}$")
                .expect("regex compile")
                .is_match(&contract.workflow_id),
            "workflow_id format mismatch: {}",
            contract.workflow_id
        );
    }

    #[test]
    fn test_workflow_contract_serialization_roundtrip() {
        let contract = WorkflowContract::new("fix: pr merge".to_string(), WorkflowMode::Fast);
        let json = serde_json::to_string_pretty(&contract).expect("serialize");
        let deserialized: WorkflowContract = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.workflow_id, contract.workflow_id);
        assert_eq!(deserialized.title, contract.title);
        assert!(matches!(deserialized.mode, WorkflowMode::Fast));
    }

    #[test]
    fn test_gate_1_to_2_blocks_without_issue_url() {
        let mut contract = WorkflowContract::new("feat: test".to_string(), WorkflowMode::Full);
        // 阶段一状态为 Complete 但缺少 issue_url
        contract.phases.get_mut("1").expect("phase 1").status = PhaseStatus::Complete;
        let result = contract.can_enter_phase(2);
        assert!(
            matches!(result, GateCheck::MissingEvidence(_)),
            "should block without issue_url"
        );
    }

    #[test]
    fn test_gate_1_to_2_passes_with_issue_url() {
        let mut contract = WorkflowContract::new("feat: test".to_string(), WorkflowMode::Full);
        let phase1 = contract.phases.get_mut("1").expect("phase 1");
        phase1.status = PhaseStatus::Complete;
        phase1.evidence.issue_url = Some("https://github.com/org/repo/issues/1".to_string());
        phase1.evidence.comment_id = Some("12345".to_string());
        let result = contract.can_enter_phase(2);
        assert!(matches!(result, GateCheck::Pass));
    }

    #[test]
    fn test_gate_2_to_3_fast_mode_always_passes() {
        let contract = WorkflowContract::new("fix: typo".to_string(), WorkflowMode::Fast);
        // 快速模式下 Gate 2→3 自动通过（阶段二被跳过）
        let result = contract.can_enter_phase(3);
        assert!(
            matches!(result, GateCheck::Pass),
            "fast mode should skip phase 2 gate"
        );
    }

    #[test]
    fn test_gate_3_to_4_never_fast_exempt() {
        let mut contract = WorkflowContract::new("fix: test".to_string(), WorkflowMode::Fast);
        // 先标记阶段三为完成（绕过 PhaseNotComplete 检查）
        contract.phases.get_mut("3").expect("phase 3").status = PhaseStatus::Complete;
        // Gate 3→4 没有快速豁免——即使 Fast 模式也需要证据
        let result = contract.can_enter_phase(4);
        assert!(
            matches!(result, GateCheck::MissingEvidence(_)),
            "Gate 3→4 must never be fast-exempt"
        );
    }

    /// 合同 schema / SKILL.md 示例：mode 为小写、phases 为对象。
    /// 该样例必须能被反序列化，防止 Rust 类型与合同 schema 偏离。
    const SCHEMA_EXAMPLE_CONTRACT: &str = r#"{
  "version": "1.0",
  "workflow_id": "wf-2026-07-09-001",
  "title": "feat: TOON 输出格式",
  "mode": "full",
  "created_at": "2026-07-09T02:59:32Z",
  "updated_at": "2026-07-09T03:30:00Z",
  "current_phase": 3,
  "phases": {
    "1": {
      "name": "需求澄清",
      "status": "complete",
      "started_at": "2026-07-09T02:59:32Z",
      "completed_at": "2026-07-09T03:10:00Z",
      "executor": "claude-code-3.7",
      "evidence": {
        "issue_url": "https://github.com/org/repo/issues/74",
        "comment_id": "4921173903"
      }
    },
    "2": {
      "name": "计划制定",
      "status": "complete",
      "started_at": "2026-07-09T03:10:00Z",
      "completed_at": "2026-07-09T03:20:00Z",
      "executor": "claude-code-3.7",
      "evidence": {
        "spec_path": "docs/superpowers/specs/2026-07-09-toon-design.md",
        "user_approved": true
      }
    },
    "3": {
      "name": "执行",
      "status": "in_progress",
      "started_at": "2026-07-09T03:20:00Z",
      "completed_at": null,
      "executor": "subagent-task-3",
      "evidence": {}
    },
    "4": {
      "name": "交付",
      "status": "pending",
      "started_at": null,
      "completed_at": null,
      "executor": null,
      "evidence": {}
    }
  }
}"#;

    #[test]
    fn test_should_deserialize_schema_example_contract() {
        let contract: WorkflowContract =
            serde_json::from_str(SCHEMA_EXAMPLE_CONTRACT).expect("deserialize schema example");
        assert_eq!(contract.version, "1.0");
        assert!(matches!(contract.mode, WorkflowMode::Full));
        assert_eq!(contract.current_phase, 3);
        assert_eq!(contract.phases.len(), 4);
        let phase1 = contract.phases.get("1").expect("phase 1");
        assert_eq!(phase1.name, "需求澄清");
        assert_eq!(phase1.status, PhaseStatus::Complete);
        assert_eq!(
            phase1.evidence.issue_url.as_deref(),
            Some("https://github.com/org/repo/issues/74")
        );
        let phase3 = contract.phases.get("3").expect("phase 3");
        assert_eq!(phase3.status, PhaseStatus::InProgress);
        let phase4 = contract.phases.get("4").expect("phase 4");
        assert_eq!(phase4.status, PhaseStatus::Pending);
    }

    #[test]
    fn test_should_serialize_mode_lowercase_and_phases_object() {
        let contract = WorkflowContract::new("feat: x".to_string(), WorkflowMode::Full);
        let json = serde_json::to_string(&contract).expect("serialize");
        // mode 必须是小写字符串，phases 必须是带字符串键的对象
        assert!(
            json.contains(r#""mode":"full""#),
            "mode must be lowercase: {json}"
        );
        assert!(
            json.contains(r#""phases":{"#),
            "phases must be an object: {json}"
        );
        assert!(
            json.contains(r#""1":"#),
            "phases must be keyed by string: {json}"
        );
    }

    #[test]
    fn test_should_archive_completed_workflow() {
        let tmp = tempfile::TempDir::new().expect("temp dir");
        let active = tmp.path().join("active");
        let archive = tmp.path().join("archive");
        std::fs::create_dir_all(&active).expect("create active");

        let mut contract = WorkflowContract::new("feat: done".to_string(), WorkflowMode::Full);
        contract.current_phase = 4;
        contract.phases.get_mut("4").expect("phase 4").status = PhaseStatus::Complete;
        let id = contract.workflow_id.clone();
        std::fs::write(
            active.join(format!("{id}.json")),
            serde_json::to_string_pretty(&contract).expect("serialize"),
        )
        .expect("write contract");

        archive_workflow_at(&active, &archive, &id).expect("archive should succeed");

        assert!(
            !active.join(format!("{id}.json")).exists(),
            "active contract should be removed after archive"
        );
        let month = Utc::now().format("%Y-%m").to_string();
        assert!(
            archive.join(&month).join(format!("{id}.json")).exists(),
            "contract should be archived under YYYY-MM"
        );
    }

    #[test]
    fn test_should_reject_archive_of_incomplete_workflow() {
        let tmp = tempfile::TempDir::new().expect("temp dir");
        let active = tmp.path().join("active");
        let archive = tmp.path().join("archive");
        std::fs::create_dir_all(&active).expect("create active");

        // Phase 4 仍为 pending，不可归档
        let contract = WorkflowContract::new("feat: wip".to_string(), WorkflowMode::Full);
        let id = contract.workflow_id.clone();
        std::fs::write(
            active.join(format!("{id}.json")),
            serde_json::to_string_pretty(&contract).expect("serialize"),
        )
        .expect("write contract");

        let result = archive_workflow_at(&active, &archive, &id);
        assert!(result.is_err(), "incomplete workflow must not be archived");
        assert!(
            active.join(format!("{id}.json")).exists(),
            "active contract must remain when archive is rejected"
        );
    }

    #[test]
    fn test_should_cleanup_only_expired_archives() {
        let tmp = tempfile::TempDir::new().expect("temp dir");
        let archive = tmp.path().join("archive");
        let old_month = archive.join("2020-01");
        let fresh_month = archive.join(Utc::now().format("%Y-%m").to_string());
        std::fs::create_dir_all(&old_month).expect("create old month");
        std::fs::create_dir_all(&fresh_month).expect("create fresh month");

        // 过期合同（创建时间远早于阈值）
        let mut old = WorkflowContract::new("feat: old".to_string(), WorkflowMode::Full);
        old.created_at = "2020-01-01T00:00:00Z".to_string();
        std::fs::write(
            old_month.join("wf-2020-01-01-001.json"),
            serde_json::to_string(&old).expect("serialize"),
        )
        .expect("write old contract");

        // 新合同（刚创建，应保留）
        let fresh = WorkflowContract::new("feat: fresh".to_string(), WorkflowMode::Full);
        std::fs::write(
            fresh_month.join("wf-fresh-001.json"),
            serde_json::to_string(&fresh).expect("serialize"),
        )
        .expect("write fresh contract");

        cleanup_archives_at(&archive, 90).expect("cleanup should succeed");

        assert!(
            !old_month.join("wf-2020-01-01-001.json").exists(),
            "expired contract should be deleted"
        );
        assert!(
            !old_month.exists(),
            "empty month directory should be removed"
        );
        assert!(
            fresh_month.join("wf-fresh-001.json").exists(),
            "fresh contract should be retained"
        );
    }
}
