//! Optional semantic observations. Contract gates remain deterministic.

#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "bounded local workflow contract I/O"
)]

use std::{collections::BTreeMap, io::Read, path::Path};

use chrono::Utc;
use gitflow_core::{
    decision::DecisionResponse,
    workflow_semantic::{CheckInput, Finding, Phase, Rule, RuleSet},
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;

use super::workflow::{WorkflowContract, validate_workflow_id, workflow_dir};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditRecord {
    pub checked_at: String,
    pub policy_version: String,
    pub phase: Phase,
    pub rule_id: String,
    /// Deterministic path evidence, separate from provider inference.
    pub matched_paths: Vec<String>,
    pub finding: Finding,
}

const MAX_CONFIG_BYTES: u64 = 16_384;
const MAX_INPUT_BYTES: u64 = 16_384;
const MAX_RESPONSE_BYTES: u64 = 262_144;
const MAX_CONTRACT_BYTES: u64 = 1_048_576;

fn read_json<T: DeserializeOwned>(path: &Path, limit: u64) -> miette::Result<T> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .map_err(|_| miette::miette!("semantic check file cannot be opened"))?
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| miette::miette!("semantic check file cannot be read"))?;
    if bytes.len() as u64 > limit {
        return Err(miette::miette!("semantic check file is too large"));
    }
    serde_json::from_slice(&bytes).map_err(|_| miette::miette!("semantic check JSON is invalid"))
}

/// Run prefilter, optional provider, then append bounded audit results.
pub async fn handle(
    workflow_id: String,
    input: String,
    rules: String,
    response: Option<String>,
    live: bool,
) -> miette::Result<()> {
    validate_workflow_id(&workflow_id)?;
    let path = workflow_dir().join(format!("{workflow_id}.json"));
    let contract: WorkflowContract = read_json(&path, MAX_CONTRACT_BYTES)?;
    let input: CheckInput = read_json(Path::new(&input), MAX_INPUT_BYTES)?;
    let rules: RuleSet = read_json(Path::new(&rules), MAX_CONFIG_BYTES)?;
    let matches = rules.matching(&input).map_err(|e| miette::miette!("{e}"))?;
    let expected_phase = match input.phase {
        Phase::Plan => 2,
        Phase::Execute => 3,
        Phase::Deliver => 4,
    };
    if contract.current_phase != expected_phase {
        return Err(miette::miette!(
            "semantic check phase differs from active contract phase"
        ));
    }
    let saved: BTreeMap<String, DecisionResponse> = match response {
        Some(file) => read_json(Path::new(&file), MAX_RESPONSE_BYTES)?,
        None => BTreeMap::new(),
    };
    let mut records = Vec::new();
    for (rule, matched_paths) in matches {
        let request = rule
            .request(&matched_paths, &input.summary)
            .map_err(|e| miette::miette!("{e}"))?;
        let result = if let Some(saved) = saved.get(&rule.id) {
            rule.finding(saved, &request)
        } else if live {
            live_finding(rule, &request).await
        } else {
            Finding::unavailable(&rule.id)
        };
        records.push(AuditRecord {
            checked_at: Utc::now().to_rfc3339(),
            policy_version: rules.policy_version.clone(),
            phase: input.phase,
            rule_id: rule.id.clone(),
            matched_paths,
            finding: result,
        });
    }
    let no_match = records.is_empty();
    if no_match {
        records.push(AuditRecord {
            checked_at: Utc::now().to_rfc3339(),
            policy_version: rules.policy_version.clone(),
            phase: input.phase,
            rule_id: "__none__".into(),
            matched_paths: Vec::new(),
            finding: Finding::not_applicable(),
        });
    }
    // Reload just before writing so existing evidence fields survive a round trip.
    let mut raw: Value = read_json(&path, MAX_CONTRACT_BYTES)?;
    if raw.get("current_phase").and_then(Value::as_u64) != Some(u64::from(expected_phase))
        || raw.get("workflow_id").and_then(Value::as_str) != Some(&workflow_id)
    {
        return Err(miette::miette!(
            "workflow contract changed during semantic check"
        ));
    }
    {
        let root = raw
            .as_object_mut()
            .ok_or_else(|| miette::miette!("invalid workflow contract"))?;
        let array = root
            .entry("semantic_checks")
            .or_insert_with(|| Value::Array(Vec::new()))
            .as_array_mut()
            .ok_or_else(|| miette::miette!("invalid semantic audit history"))?;
        if array.len() + records.len() > 128 {
            return Err(miette::miette!("semantic audit history is full"));
        }
        for record in &records {
            array.push(
                serde_json::to_value(record)
                    .map_err(|_| miette::miette!("audit serialization failed"))?,
            );
        }
        root.insert("updated_at".into(), Value::String(Utc::now().to_rfc3339()));
        let parent = path
            .parent()
            .ok_or_else(|| miette::miette!("invalid contract path"))?;
        let mut temp = tempfile::NamedTempFile::new_in(parent)
            .map_err(|_| miette::miette!("contract audit write failed"))?;
        serde_json::to_writer_pretty(&mut temp, &raw)
            .map_err(|_| miette::miette!("contract audit write failed"))?;
        temp.persist(&path)
            .map_err(|_| miette::miette!("contract audit write failed"))?;
    }
    super::output::print_output(
        &serde_json::json!({"policyVersion": rules.policy_version, "phase": input.phase, "status": if no_match { "not_applicable" } else { "checked" }, "checks": records}),
        &crate::OutputFormat::Json,
    )
}

#[cfg(feature = "gitflow-jev")]
async fn live_finding(rule: &Rule, request: &gitflow_core::decision::DecisionRequest) -> Finding {
    if std::env::var("GF_DECISION_PROVIDER").as_deref() != Ok("jev") {
        return Finding::unavailable(&rule.id);
    }
    let Ok(engine) = gitflow_jev::JevEngine::from_env() else {
        return Finding::unavailable(&rule.id);
    };
    gitflow_core::workflow_semantic::evaluate_with_engine(rule, request, &engine).await
}

#[cfg(not(feature = "gitflow-jev"))]
async fn live_finding(rule: &Rule, _request: &gitflow_core::decision::DecisionRequest) -> Finding {
    Finding::unavailable(&rule.id)
}
