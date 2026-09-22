//! Read-only, replayable workflow progress reports.

#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "bounded local workflow report I/O"
)]

use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
};

use chrono::Utc;
use clap::Subcommand;
use gitflow_core::{
    SafePath,
    decision::DecisionResponse,
    workflow_progress::{Assessment, TraceInput, assess},
};
use serde::{Deserialize, Serialize};

use super::workflow::{WorkflowContract, validate_workflow_id, workflow_dir};

const MAX_INPUT: u64 = 65_536;
const MAX_CONTRACT: u64 = 1_048_576;
const MAX_RESPONSE: u64 = 131_072;
const MAX_HISTORY: u64 = 1_048_576;

/// Assess a structured trace or inspect saved progress history.
#[derive(Debug, Subcommand)]
pub enum ProgressCommand {
    /// Assess one event window and save an advisory report.
    Assess {
        /// Active workflow ID.
        #[arg(long)]
        workflow_id: String,
        /// Repository-relative typed trace JSON.
        #[arg(long)]
        input: String,
        /// Saved typed provider response for offline replay.
        #[arg(long, conflicts_with = "live")]
        response: Option<String>,
        /// Explicitly query the configured Jev provider.
        #[arg(long)]
        live: bool,
        /// RFC 3339 time override for deterministic replay.
        #[arg(long)]
        now: Option<String>,
    },
    /// Replay labeled windows and report precision, recall, false positives, and delay.
    Evaluate {
        /// Repository-relative JSON array of labeled evaluation cases.
        #[arg(long)]
        input: String,
    },
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ReportHistory {
    schema_version: u32,
    workflow_id: String,
    assessments: Vec<Assessment>,
}

fn read_limited(path: &Path, limit: u64) -> miette::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .map_err(|_| miette::miette!("progress input cannot be opened"))?
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| miette::miette!("progress input cannot be read"))?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > limit {
        return Err(miette::miette!("progress input exceeds size limit"));
    }
    Ok(bytes)
}

fn safe_input(root: &Path, input: &str) -> miette::Result<PathBuf> {
    let safe = SafePath::new(input).map_err(|_| miette::miette!("invalid progress input path"))?;
    let path = root
        .join(safe.as_path())
        .canonicalize()
        .map_err(|_| miette::miette!("progress input cannot be resolved"))?;
    if !path.starts_with(root) || !path.is_file() {
        return Err(miette::miette!("progress input must be a repository file"));
    }
    Ok(path)
}

fn history_path(root: &Path, workflow_id: &str) -> PathBuf {
    root.join(".cache/workflows/progress")
        .join(format!("{workflow_id}.json"))
}

fn load_history(path: &Path, workflow_id: &str) -> miette::Result<ReportHistory> {
    if !path.exists() {
        return Ok(ReportHistory {
            schema_version: 1,
            workflow_id: workflow_id.into(),
            assessments: Vec::new(),
        });
    }
    let history: ReportHistory = serde_json::from_slice(&read_limited(path, MAX_HISTORY)?)
        .map_err(|_| miette::miette!("progress history JSON is invalid"))?;
    if history.schema_version != 1
        || history.workflow_id != workflow_id
        || history.assessments.len() > 64
    {
        return Err(miette::miette!(
            "progress history identity or size is invalid"
        ));
    }
    Ok(history)
}

fn save_history(path: &Path, history: &ReportHistory) -> miette::Result<()> {
    let bytes = serde_json::to_vec_pretty(history)
        .map_err(|_| miette::miette!("progress report serialization failed"))?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > MAX_HISTORY {
        return Err(miette::miette!("progress report exceeds size limit"));
    }
    let parent = path
        .parent()
        .ok_or_else(|| miette::miette!("invalid report path"))?;
    std::fs::create_dir_all(parent)
        .map_err(|_| miette::miette!("progress directory creation failed"))?;
    let mut temp = tempfile::NamedTempFile::new_in(parent)
        .map_err(|_| miette::miette!("progress report write failed"))?;
    temp.write_all(&bytes)
        .map_err(|_| miette::miette!("progress report write failed"))?;
    temp.persist(path)
        .map_err(|_| miette::miette!("progress report write failed"))?;
    Ok(())
}

#[cfg(feature = "gitflow-jev")]
async fn live_response(input: &TraceInput, now: &str) -> Option<DecisionResponse> {
    use gitflow_core::decision::DecisionEngine;
    if std::env::var("GF_DECISION_PROVIDER").as_deref() != Ok("jev") {
        return None;
    }
    let engine = gitflow_jev::JevEngine::from_env().ok()?;
    input.validate(now).ok()?;
    // The provider sees only the bounded synopsis and deterministic signal metadata.
    let baseline = assess(input, None, None, now).ok()?;
    let request = gitflow_core::workflow_progress::decision_request(&baseline.summary).ok()?;
    engine.decide(&request).await.ok()
}

#[cfg(not(feature = "gitflow-jev"))]
async fn live_response(_input: &TraceInput, _now: &str) -> Option<DecisionResponse> {
    None
}

/// Run a progress operation.
///
/// # Errors
/// Returns an error for invalid paths, contract mismatch, or invalid structured input.
pub async fn handle(command: ProgressCommand) -> miette::Result<()> {
    match command {
        ProgressCommand::Assess {
            workflow_id,
            input,
            response,
            live,
            now,
        } => handle_assess(workflow_id, input, response, live, now).await,
        ProgressCommand::Evaluate { input } => handle_evaluate(&input),
    }
}

fn handle_evaluate(input: &str) -> miette::Result<()> {
    let root = std::env::current_dir()
        .map_err(|_| miette::miette!("cannot resolve repository"))?
        .canonicalize()
        .map_err(|_| miette::miette!("cannot resolve repository"))?;
    let path = safe_input(&root, input)?;
    let cases: Vec<gitflow_core::workflow_progress::EvaluationCase> =
        serde_json::from_slice(&read_limited(&path, MAX_HISTORY)?)
            .map_err(|_| miette::miette!("progress evaluation JSON is invalid"))?;
    let report = gitflow_core::workflow_progress::evaluate(&cases)
        .map_err(|error| miette::miette!("{error}"))?;
    super::output::print_output(&report, &crate::OutputFormat::Json)
}

async fn handle_assess(
    workflow_id: String,
    input: String,
    response: Option<String>,
    live: bool,
    now: Option<String>,
) -> miette::Result<()> {
    validate_workflow_id(&workflow_id)?;
    let root = std::env::current_dir()
        .map_err(|_| miette::miette!("cannot resolve repository"))?
        .canonicalize()
        .map_err(|_| miette::miette!("cannot resolve repository"))?;
    let contract_path = workflow_dir().join(format!("{workflow_id}.json"));
    let contract: WorkflowContract =
        serde_json::from_slice(&read_limited(&contract_path, MAX_CONTRACT)?)
            .map_err(|_| miette::miette!("workflow contract JSON is invalid"))?;
    if contract.workflow_id != workflow_id {
        return Err(miette::miette!("workflow contract ID mismatch"));
    }
    let input_path = safe_input(&root, &input)?;
    let trace: TraceInput = serde_json::from_slice(&read_limited(&input_path, MAX_INPUT)?)
        .map_err(|_| miette::miette!("progress trace JSON is invalid"))?;
    let phase = contract
        .phases
        .get(&contract.current_phase.to_string())
        .ok_or_else(|| miette::miette!("active contract phase is missing"))?;
    if trace.workflow_id != workflow_id
        || trace.phase != contract.current_phase
        || phase.started_at.as_deref() != Some(&trace.phase_started_at)
    {
        return Err(miette::miette!(
            "progress trace does not match active workflow phase"
        ));
    }
    let now = now.unwrap_or_else(|| Utc::now().to_rfc3339());
    trace
        .validate(&now)
        .map_err(|error| miette::miette!("{error}"))?;
    let response = if let Some(path) = response {
        let path = safe_input(&root, &path)?;
        serde_json::from_slice::<DecisionResponse>(&read_limited(&path, MAX_RESPONSE)?).ok()
    } else if live {
        live_response(&trace, &now).await
    } else {
        None
    };
    let path = history_path(&root, &workflow_id);
    let mut history = load_history(&path, &workflow_id)?;
    let previous = history.assessments.last();
    let assessment = assess(&trace, previous, response.as_ref(), &now)
        .map_err(|error| miette::miette!("{error}"))?;
    if history
        .assessments
        .last()
        .is_some_and(|last| last.window_hash == assessment.window_hash)
    {
        history.assessments.pop();
    }
    history.assessments.push(assessment.clone());
    if history.assessments.len() > 64 {
        history.assessments.remove(0);
    }
    save_history(&path, &history)?;
    super::output::print_output(&assessment, &crate::OutputFormat::Json)
}
