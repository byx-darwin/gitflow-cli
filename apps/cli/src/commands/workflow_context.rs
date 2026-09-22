//! Read-only workflow evidence selection and recovery.

#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "bounded local evidence file I/O"
)]

use std::{
    collections::{BTreeMap, BTreeSet},
    io::Read,
    path::{Path, PathBuf},
    process::Command,
    time::Instant,
};

use chrono::Utc;
use clap::Subcommand;
use gitflow_core::{
    SafePath,
    decision::DecisionResponse,
    workflow_context::{
        ContextManifest, ContextSnapshot, EvidenceKind, EvidenceRef, SCHEMA_VERSION,
        build_manifest, sha256_hex,
    },
};
use serde::Deserialize;

use super::workflow::{validate_workflow_id, workflow_dir};

const MAX_CATALOG_BYTES: u64 = 32_768;
const MAX_CONTRACT_BYTES: u64 = 1_048_576;
const MAX_SOURCE_BYTES: u64 = 65_536;
const MAX_RESPONSE_BYTES: u64 = 262_144;
const MAX_MANIFEST_BYTES: u64 = 131_072;

/// Local context operations. Source files and workflow contracts remain unchanged.
#[derive(Debug, Subcommand)]
pub enum ContextCommand {
    /// Build a derived active-context manifest from a reviewed evidence catalog.
    Build {
        /// Active workflow ID.
        #[arg(long)]
        workflow_id: String,
        /// Repository-relative JSON catalog of source evidence.
        #[arg(long)]
        input: String,
        /// Saved typed decision responses keyed by evidence ID.
        #[arg(long, conflicts_with = "live")]
        response: Option<String>,
        /// Explicitly query the configured Jev provider for older candidates.
        #[arg(long)]
        live: bool,
        /// Optional human-reviewed required evidence IDs for offline false-drop metrics.
        #[arg(long)]
        labels: Option<String>,
    },
    /// Recover parked evidence by ID, source path, or summary keyword after checking freshness.
    Restore {
        /// Active workflow ID.
        #[arg(long)]
        workflow_id: String,
        /// Same catalog used to build the manifest.
        #[arg(long)]
        input: String,
        /// Exact evidence ID.
        #[arg(long)]
        id: Option<String>,
        /// Exact repository-relative source path.
        #[arg(long)]
        source: Option<String>,
        /// Case-insensitive summary keyword.
        #[arg(long)]
        keyword: Option<String>,
    },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct EvidenceCatalog {
    schema_version: u32,
    objective: String,
    evidence: Vec<CatalogEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CatalogEntry {
    id: String,
    kind: EvidenceKind,
    source: String,
    summary: String,
    recorded_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct EvaluationLabels {
    schema_version: u32,
    required_ids: Vec<String>,
}

fn read_labels(root: &Path, path: &str) -> miette::Result<BTreeSet<String>> {
    let safe = safe_relative_path(root, path)?;
    let labels: EvaluationLabels = serde_json::from_slice(&read_limited(&safe, MAX_CATALOG_BYTES)?)
        .map_err(|_| miette::miette!("context evaluation labels JSON is invalid"))?;
    let unique: BTreeSet<_> = labels.required_ids.iter().cloned().collect();
    if labels.schema_version != SCHEMA_VERSION
        || labels.required_ids.len() > 64
        || unique.len() != labels.required_ids.len()
    {
        return Err(miette::miette!("context evaluation labels are invalid"));
    }
    Ok(unique)
}

fn read_limited(path: &Path, limit: u64) -> miette::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .map_err(|_| miette::miette!("context source cannot be opened"))?
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| miette::miette!("context source cannot be read"))?;
    if bytes.len() as u64 > limit {
        return Err(miette::miette!("context source exceeds size limit"));
    }
    Ok(bytes)
}

fn safe_relative_path(root: &Path, path: &str) -> miette::Result<PathBuf> {
    let safe =
        SafePath::new(path).map_err(|_| miette::miette!("invalid relative evidence path"))?;
    let joined = root.join(safe.as_path());
    let canonical = joined
        .canonicalize()
        .map_err(|_| miette::miette!("evidence path cannot be resolved"))?;
    if !canonical.starts_with(root) || !canonical.is_file() {
        return Err(miette::miette!(
            "evidence source must be a file inside the repository"
        ));
    }
    Ok(canonical)
}

fn git_output(root: &Path, args: &[&str]) -> miette::Result<Vec<u8>> {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|_| miette::miette!("cannot inspect git working tree"))?;
    if !output.status.success() || output.stdout.len() > 131_072 {
        return Err(miette::miette!("cannot inspect git working tree"));
    }
    Ok(output.stdout)
}

fn worktree_fingerprint(root: &Path) -> miette::Result<String> {
    let mut bytes = git_output(root, &["rev-parse", "HEAD"])?;
    bytes.extend(git_output(
        root,
        &["status", "--porcelain", "--untracked-files=normal"],
    )?);
    bytes.extend(git_output(root, &["diff", "--cached", "--raw", "HEAD"])?);
    let paths = git_output(root, &["ls-files", "-m", "-o", "--exclude-standard", "-z"])?;
    let mut count = 0usize;
    let mut total_bytes = 0u64;
    for name in paths
        .split(|byte| *byte == 0)
        .filter(|name| !name.is_empty())
    {
        count += 1;
        if count > 512 {
            return Err(miette::miette!("too many changed files to fingerprint"));
        }
        let relative =
            std::str::from_utf8(name).map_err(|_| miette::miette!("changed path is not UTF-8"))?;
        let safe = SafePath::new(relative).map_err(|_| miette::miette!("invalid changed path"))?;
        let path = root.join(safe.as_path());
        let metadata = std::fs::symlink_metadata(&path)
            .map_err(|_| miette::miette!("cannot inspect changed file"))?;
        let content = if metadata.file_type().is_symlink() {
            std::fs::read_link(&path)
                .map_err(|_| miette::miette!("cannot inspect changed link"))?
                .into_os_string()
                .into_encoded_bytes()
        } else if metadata.is_file() {
            read_limited(&path, 4_194_304)?
        } else {
            return Err(miette::miette!("unsupported changed file type"));
        };
        total_bytes = total_bytes.saturating_add(content.len() as u64);
        if total_bytes > 33_554_432 {
            return Err(miette::miette!("changed files exceed fingerprint limit"));
        }
        bytes.extend(name);
        bytes.push(0);
        bytes.extend(sha256_hex(&content).as_bytes());
    }
    Ok(sha256_hex(&bytes))
}

fn context_path(workflow_id: &str) -> miette::Result<PathBuf> {
    validate_workflow_id(workflow_id)?;
    let base = workflow_dir()
        .parent()
        .ok_or_else(|| miette::miette!("invalid workflow directory"))?
        .join("context");
    Ok(base.join(format!("{workflow_id}.json")))
}

fn snapshot(root: &Path, workflow_id: &str, input: &str) -> miette::Result<ContextSnapshot> {
    validate_workflow_id(workflow_id)?;
    let contract_path = workflow_dir().join(format!("{workflow_id}.json"));
    let contract_bytes = read_limited(&contract_path, MAX_CONTRACT_BYTES)?;
    let contract: serde_json::Value = serde_json::from_slice(&contract_bytes)
        .map_err(|_| miette::miette!("workflow contract JSON is invalid"))?;
    if contract
        .get("workflow_id")
        .and_then(serde_json::Value::as_str)
        != Some(workflow_id)
    {
        return Err(miette::miette!("workflow contract ID mismatch"));
    }
    let phase: u8 = contract
        .get("current_phase")
        .and_then(serde_json::Value::as_u64)
        .and_then(|value| value.try_into().ok())
        .ok_or_else(|| miette::miette!("workflow phase is invalid"))?;
    let input_path = safe_relative_path(root, input)?;
    let catalog: EvidenceCatalog =
        serde_json::from_slice(&read_limited(&input_path, MAX_CATALOG_BYTES)?)
            .map_err(|_| miette::miette!("context catalog JSON is invalid"))?;
    if catalog.schema_version != SCHEMA_VERSION || catalog.evidence.len() > 63 {
        return Err(miette::miette!(
            "context catalog version or size is invalid"
        ));
    }
    let mut evidence = Vec::with_capacity(catalog.evidence.len() + 1);
    let contract_source = contract_path
        .strip_prefix(root)
        .map_err(|_| miette::miette!("workflow contract is outside repository"))?
        .to_string_lossy()
        .into_owned();
    evidence.push(EvidenceRef {
        id: "workflow_contract".into(),
        kind: EvidenceKind::PermissionDecision,
        source: contract_source,
        source_hash: sha256_hex(&contract_bytes),
        bytes: contract_bytes.len() as u64,
        summary: "Workflow contract and phase evidence".into(),
        recorded_at: contract
            .get("updated_at")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| miette::miette!("workflow update time is missing"))?
            .into(),
    });
    for item in catalog.evidence {
        let path = safe_relative_path(root, &item.source)?;
        let bytes = read_limited(&path, MAX_SOURCE_BYTES)?;
        evidence.push(EvidenceRef {
            id: item.id,
            kind: item.kind,
            source: item.source,
            source_hash: sha256_hex(&bytes),
            bytes: bytes.len() as u64,
            summary: item.summary,
            recorded_at: item.recorded_at,
        });
    }
    let snapshot = ContextSnapshot {
        schema_version: SCHEMA_VERSION,
        workflow_id: workflow_id.into(),
        phase,
        objective: catalog.objective,
        contract_hash: sha256_hex(&contract_bytes),
        worktree_fingerprint: worktree_fingerprint(root)?,
        evidence,
    };
    snapshot.validate().map_err(|e| miette::miette!("{e}"))?;
    Ok(snapshot)
}

fn read_response(root: &Path, path: &str) -> miette::Result<BTreeMap<String, DecisionResponse>> {
    let safe = safe_relative_path(root, path)?;
    serde_json::from_slice(&read_limited(&safe, MAX_RESPONSE_BYTES)?)
        .map_err(|_| miette::miette!("context response JSON is invalid"))
}

fn save_manifest(path: &Path, manifest: &ContextManifest) -> miette::Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| miette::miette!("invalid context path"))?;
    std::fs::create_dir_all(parent)
        .map_err(|_| miette::miette!("cannot create context directory"))?;
    let mut temp = tempfile::NamedTempFile::new_in(parent)
        .map_err(|_| miette::miette!("cannot create context manifest"))?;
    serde_json::to_writer_pretty(&mut temp, manifest)
        .map_err(|_| miette::miette!("cannot serialize context manifest"))?;
    temp.persist(path)
        .map_err(|_| miette::miette!("cannot save context manifest"))?;
    Ok(())
}

fn evaluate_manifest(
    root: &Path,
    manifest: &ContextManifest,
    labels: Option<&str>,
    started: Instant,
) -> miette::Result<gitflow_core::workflow_context::ContextEvaluation> {
    let latency = Some(started.elapsed().as_millis().try_into().unwrap_or(u64::MAX));
    if let Some(path) = labels {
        let required_ids = read_labels(root, path)?;
        manifest
            .evaluation_with_labels(&required_ids, latency, None)
            .map_err(|e| miette::miette!("{e}"))
    } else {
        Ok(manifest.evaluation(latency, None))
    }
}

/// Build or restore a workflow context manifest.
///
/// # Errors
/// Rejects unsafe paths, stale manifests, invalid evidence, or I/O failures.
pub async fn handle(command: ContextCommand) -> miette::Result<()> {
    let root = std::env::current_dir()
        .map_err(|_| miette::miette!("cannot find repository directory"))?
        .canonicalize()
        .map_err(|_| miette::miette!("cannot resolve repository directory"))?;
    match command {
        ContextCommand::Build {
            workflow_id,
            input,
            response,
            live,
            labels,
        } => {
            let started = Instant::now();
            let current = snapshot(&root, &workflow_id, &input)?;
            let mut responses = if let Some(path) = response {
                read_response(&root, &path)?
            } else {
                BTreeMap::new()
            };
            if live {
                responses.extend(live_responses(&current).await);
            }
            let manifest = build_manifest(&current, &responses, &Utc::now().to_rfc3339())
                .map_err(|e| miette::miette!("{e}"))?;
            // Recheck just before writing: a changed contract, source, or tree makes the result
            // stale.
            if !manifest.is_current(&snapshot(&root, &workflow_id, &input)?) {
                return Err(miette::miette!("workflow context changed while building"));
            }
            let evaluation = evaluate_manifest(&root, &manifest, labels.as_deref(), started)?;
            save_manifest(&context_path(&workflow_id)?, &manifest)?;
            super::output::print_output(
                &serde_json::json!({"manifest": manifest, "evaluation": evaluation}),
                &crate::OutputFormat::Json,
            )
        }
        ContextCommand::Restore {
            workflow_id,
            input,
            id,
            source,
            keyword,
        } => {
            if usize::from(id.is_some())
                + usize::from(source.is_some())
                + usize::from(keyword.is_some())
                != 1
            {
                return Err(miette::miette!(
                    "select exactly one of --id, --source, or --keyword"
                ));
            }
            let bytes = read_limited(&context_path(&workflow_id)?, MAX_MANIFEST_BYTES)?;
            let manifest: ContextManifest = serde_json::from_slice(&bytes)
                .map_err(|_| miette::miette!("context manifest JSON is invalid"))?;
            let current = snapshot(&root, &workflow_id, &input)?;
            if !manifest.is_current(&current) {
                return Err(miette::miette!(
                    "context manifest is stale; rebuild for the current contract and worktree"
                ));
            }
            let matches: Vec<_> = manifest
                .entries
                .iter()
                .filter(|entry| {
                    !entry.retained
                        && (id.as_ref().is_some_and(|value| entry.id == *value)
                            || source.as_ref().is_some_and(|value| entry.source == *value)
                            || keyword.as_ref().is_some_and(|value| {
                                !value.trim().is_empty()
                                    && value.len() <= 100
                                    && entry.summary.to_lowercase().contains(&value.to_lowercase())
                            }))
                })
                .collect();
            if matches.is_empty() {
                return Err(miette::miette!("no parked evidence matches"));
            }
            let mut recovered = Vec::with_capacity(matches.len());
            for entry in matches {
                let path = safe_relative_path(&root, &entry.source)?;
                let bytes = read_limited(&path, MAX_SOURCE_BYTES)?;
                if sha256_hex(&bytes) != entry.source_hash {
                    return Err(miette::miette!("source evidence changed; rebuild context"));
                }
                let content = String::from_utf8(bytes)
                    .map_err(|_| miette::miette!("source evidence is not UTF-8"))?;
                recovered.push(
                    serde_json::json!({"id": entry.id, "source": entry.source, "content": content}),
                );
            }
            super::output::print_output(
                &serde_json::json!({"workflowId": workflow_id, "recovered": recovered}),
                &crate::OutputFormat::Json,
            )
        }
    }
}

#[cfg(feature = "gitflow-jev")]
async fn live_responses(snapshot: &ContextSnapshot) -> BTreeMap<String, DecisionResponse> {
    use gitflow_core::decision::DecisionEngine;
    if std::env::var("GF_DECISION_PROVIDER").as_deref() != Ok("jev") {
        return BTreeMap::new();
    }
    let Ok(engine) = gitflow_jev::JevEngine::from_env() else {
        return BTreeMap::new();
    };
    let Ok(candidates) = snapshot.candidates() else {
        return BTreeMap::new();
    };
    let mut responses = BTreeMap::new();
    for item in candidates {
        if let Ok(request) = gitflow_core::workflow_context::decision_request(snapshot, item)
            && let Ok(response) = engine.decide(&request).await
        {
            responses.insert(item.id.clone(), response);
        }
    }
    responses
}

#[cfg(not(feature = "gitflow-jev"))]
async fn live_responses(_snapshot: &ContextSnapshot) -> BTreeMap<String, DecisionResponse> {
    BTreeMap::new()
}
