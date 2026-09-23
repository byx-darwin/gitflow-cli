//! Progress reports stay separate from workflow contracts.

#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "isolated CLI fixture uses local files and explicit assertions"
)]

use std::{fs, process::Command};

use serde_json::json;

#[test]
fn test_should_save_unavailable_telemetry_without_changing_contract() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join(".cache/workflows/active")).unwrap();
    let contract = json!({
        "version":"1.0", "workflow_id":"wf-2026-09-22-001", "title":"Fixture",
        "mode":"full", "skill_source":null, "created_at":"2026-09-22T10:00:00Z",
        "updated_at":"2026-09-22T10:00:00Z", "current_phase":1,
        "phases":{"1":{"name":"clarify","status":"in_progress","started_at":"2026-09-22T10:00:00Z",
            "completed_at":null,"executor":null,"evidence":{}}}
    })
    .to_string();
    let contract_path = root.join(".cache/workflows/active/wf-2026-09-22-001.json");
    fs::write(&contract_path, &contract).unwrap();
    fs::write(
        root.join(".cache/trace.json"),
        json!({
            "schemaVersion":1,"workflowId":"wf-2026-09-22-001","phase":1,
            "phaseStartedAt":"2026-09-22T10:00:00Z",
            "events":[{"id":"ev-1","at":"2026-09-22T10:10:00Z",
                "kind":{"type":"completion_claim","verified":false,"proof_hash":null}}]
        })
        .to_string(),
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_gf"))
        .args([
            "workflow",
            "progress",
            "assess",
            "--workflow-id",
            "wf-2026-09-22-001",
            "--input",
            ".cache/trace.json",
            "--now",
            "2026-09-22T12:00:00Z",
        ])
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["status"], "unavailable");
    assert_eq!(result["blocker"], "verification_gap");
    assert_eq!(result["evidenceRefs"], json!(["ev-1"]));
    assert_eq!(fs::read_to_string(&contract_path).unwrap(), contract);
    let saved: serde_json::Value = serde_json::from_slice(
        &fs::read(root.join(".cache/workflows/progress/wf-2026-09-22-001.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(saved["assessments"].as_array().unwrap().len(), 1);
    fs::write(root.join(".cache/malformed-response.json"), "{invalid").unwrap();
    let fallback = Command::new(env!("CARGO_BIN_EXE_gf"))
        .args([
            "workflow",
            "progress",
            "assess",
            "--workflow-id",
            "wf-2026-09-22-001",
            "--input",
            ".cache/trace.json",
            "--response",
            ".cache/malformed-response.json",
            "--now",
            "2026-09-22T12:11:00Z",
        ])
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        fallback.status.success(),
        "{}",
        String::from_utf8_lossy(&fallback.stderr)
    );
    let result: serde_json::Value = serde_json::from_slice(&fallback.stdout).unwrap();
    assert_eq!(result["status"], "unavailable");
    assert_eq!(result["evidenceRefs"], json!(["ev-1"]));
}
