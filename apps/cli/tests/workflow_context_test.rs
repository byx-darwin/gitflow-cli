//! Local context manifests retain critical evidence and recover unchanged parked sources.

#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::too_many_lines,
    reason = "isolated CLI fixture uses synchronous local files and explicit test assertions"
)]

use std::{fs, path::Path, process::Command};

use serde_json::json;

fn git(root: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn gf(root: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_gf"))
        .args(args)
        .current_dir(root)
        .output()
        .unwrap()
}

#[test]
fn test_should_build_restore_and_invalidate_context_without_editing_evidence() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fs::create_dir_all(root.join(".cache/workflows/active")).unwrap();
    fs::create_dir_all(root.join("docs")).unwrap();
    fs::write(root.join(".gitignore"), ".cache/\n").unwrap();
    let contract = json!({"workflow_id":"wf-2026-09-22-001","current_phase":2,"updated_at":"2026-09-22T10:00:00Z"}).to_string();
    fs::write(
        root.join(".cache/workflows/active/wf-2026-09-22-001.json"),
        &contract,
    )
    .unwrap();
    let mut catalog = Vec::new();
    for index in 0..6 {
        let id = format!("item_{index}");
        let source = format!("docs/{id}.md");
        fs::write(root.join(&source), format!("Original evidence {index}\n")).unwrap();
        catalog.push(json!({"id":id,"kind":if index == 0 {"user_constraint"} else {"other"},"source":source,"summary":format!("Archive item {index}"),"recordedAt":format!("2026-09-22T09:{index:02}:00Z")}));
    }
    fs::write(
        root.join(".cache/context-input.json"),
        json!({"schemaVersion":1,"objective":"Deliver the workflow","evidence":catalog})
            .to_string(),
    )
    .unwrap();
    fs::write(
        root.join(".cache/context-labels.json"),
        json!({"schemaVersion":1,"requiredIds":["item_0","item_1"]}).to_string(),
    )
    .unwrap();
    git(root, &["init", "-q"]);
    git(root, &["config", "user.name", "Context Test"]);
    git(root, &["config", "user.email", "context@example.invalid"]);
    git(root, &["add", "."]);
    git(root, &["commit", "-qm", "fixture"]);

    let built = gf(
        root,
        &[
            "workflow",
            "context",
            "build",
            "--workflow-id",
            "wf-2026-09-22-001",
            "--input",
            ".cache/context-input.json",
            "--labels",
            ".cache/context-labels.json",
        ],
    );
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    let output: serde_json::Value = serde_json::from_slice(&built.stdout).unwrap();
    let entries = output["manifest"]["entries"].as_array().unwrap();
    assert!(
        entries
            .iter()
            .any(|entry| entry["id"] == "workflow_contract" && entry["retained"] == true)
    );
    assert!(
        entries
            .iter()
            .any(|entry| entry["id"] == "item_0" && entry["retained"] == true)
    );
    assert!(
        entries
            .iter()
            .any(|entry| entry["id"] == "item_1" && entry["retained"] == false)
    );
    assert_eq!(output["evaluation"]["criticalRecall"], 1.0);
    assert_eq!(output["evaluation"]["requiredRecall"], 0.5);
    assert_eq!(output["evaluation"]["erroneousDropRate"], 0.5);
    assert!(
        output["evaluation"]["estimatedTokenReductionRate"]
            .as_f64()
            .unwrap()
            > 0.0
    );
    assert_eq!(output["manifest"]["providerStatus"], "unavailable");
    assert_eq!(
        fs::read_to_string(root.join(".cache/workflows/active/wf-2026-09-22-001.json")).unwrap(),
        contract
    );

    let restored = gf(
        root,
        &[
            "workflow",
            "context",
            "restore",
            "--workflow-id",
            "wf-2026-09-22-001",
            "--input",
            ".cache/context-input.json",
            "--id",
            "item_1",
        ],
    );
    assert!(
        restored.status.success(),
        "{}",
        String::from_utf8_lossy(&restored.stderr)
    );
    let restored: serde_json::Value = serde_json::from_slice(&restored.stdout).unwrap();
    assert_eq!(restored["recovered"][0]["content"], "Original evidence 1\n");
    for (flag, value) in [
        ("--source", "docs/item_1.md"),
        ("--keyword", "Archive item 1"),
    ] {
        let recovered = gf(
            root,
            &[
                "workflow",
                "context",
                "restore",
                "--workflow-id",
                "wf-2026-09-22-001",
                "--input",
                ".cache/context-input.json",
                flag,
                value,
            ],
        );
        assert!(
            recovered.status.success(),
            "{}",
            String::from_utf8_lossy(&recovered.stderr)
        );
        assert!(String::from_utf8_lossy(&recovered.stdout).contains("Original evidence 1"));
    }

    let response = json!({"item_1": {"model":"fixture-model","answers":{
        "relevance":{"type":"score","score":2.0,"confidence":0.99,"legend":{"0":"Unrelated","1":"Possibly useful","2":"Needed now"},"probabilities":{"0":0.0,"1":0.0,"2":1.0}},
        "retain":{"type":"noul","noul":0.99}},"usage":{"input_tokens":10,"output_tokens":2}}});
    fs::write(
        root.join(".cache/context-response.json"),
        response.to_string(),
    )
    .unwrap();
    let inferred = gf(
        root,
        &[
            "workflow",
            "context",
            "build",
            "--workflow-id",
            "wf-2026-09-22-001",
            "--input",
            ".cache/context-input.json",
            "--response",
            ".cache/context-response.json",
        ],
    );
    assert!(
        inferred.status.success(),
        "{}",
        String::from_utf8_lossy(&inferred.stderr)
    );
    let inferred: serde_json::Value = serde_json::from_slice(&inferred.stdout).unwrap();
    assert_eq!(inferred["manifest"]["providerStatus"], "resolved");
    assert!(
        inferred["manifest"]["entries"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["id"] == "item_1"
                && entry["retained"] == true
                && entry["model"] == "fixture-model")
    );
    let persisted =
        fs::read_to_string(root.join(".cache/workflows/context/wf-2026-09-22-001.json")).unwrap();
    assert!(!persisted.contains("probabilities"));
    assert!(!persisted.contains("answers"));

    fs::write(root.join("docs/item_1.md"), "Changed evidence 1\n").unwrap();
    let stale = gf(
        root,
        &[
            "workflow",
            "context",
            "restore",
            "--workflow-id",
            "wf-2026-09-22-001",
            "--input",
            ".cache/context-input.json",
            "--id",
            "item_1",
        ],
    );
    assert!(!stale.status.success());
    assert!(String::from_utf8_lossy(&stale.stderr).contains("stale"));
}
