//! Offline evaluation, explicit live capture, calibration, and report comparison.

#![allow(
    clippy::disallowed_types,
    reason = "Bounded file I/O runs inside spawn_blocking"
)]

use std::io::Read;
#[cfg(feature = "gitflow-jev")]
use std::time::Instant;

use gitflow_core::evaluation::{
    EvaluationReport, FixtureSuite, SavedOutcome, SavedResponses, SavedResult,
};
use serde::de::DeserializeOwned;

use super::decide::{CalibrateArgs, CompareArgs, EvalArgs, decision_output_format};
use crate::OutputFormat;

const MAX_ARTIFACT_BYTES: u64 = 4_194_304;
const MAX_LIVE_CASES: usize = 50;

/// Evaluate saved responses, or explicitly call the live decision provider.
///
/// # Errors
///
/// Returns a content-free error for invalid paths, artifacts, or output I/O.
pub async fn evaluate(args: EvalArgs, output: OutputFormat) -> miette::Result<()> {
    let fixture: FixtureSuite = read_json(args.fixtures).await?;
    fixture
        .validate()
        .map_err(|error| miette::miette!("{error}"))?;
    let saved: SavedResponses = if args.live {
        if fixture.cases.len() > MAX_LIVE_CASES {
            return Err(miette::miette!("live evaluation case count exceeds 50"));
        }
        let saved = live_responses(
            &fixture,
            args.input_price_per_million_usd,
            args.output_price_per_million_usd,
        )
        .await;
        if let Some(path) = args.save_responses {
            write_saved_responses(path, &saved).await?;
        }
        saved
    } else {
        let path = args
            .responses
            .ok_or_else(|| miette::miette!("--responses is required unless --live is set"))?;
        read_json(path).await?
    };
    let report = gitflow_core::evaluation::evaluate(&fixture, &saved)
        .map_err(|error| miette::miette!("{error}"))?;
    super::output::print_output(&report, &decision_output_format(output))
}

/// Suggest a threshold from an offline evaluation report.
///
/// # Errors
///
/// Returns an error for invalid input, missing metric groups, or output I/O.
pub async fn calibrate(args: CalibrateArgs, output: OutputFormat) -> miette::Result<()> {
    let report: EvaluationReport = read_json(args.report).await?;
    let result = gitflow_core::evaluation::calibrate(
        &report,
        &args.question,
        args.language.as_deref(),
        args.risk.as_deref(),
        args.target_accuracy,
    )
    .map_err(|error| miette::miette!("{error}"))?;
    super::output::print_output(&result, &decision_output_format(output))
}

/// Compare two offline evaluation reports.
///
/// # Errors
///
/// Returns an error for invalid input or output I/O.
pub async fn compare(args: CompareArgs, output: OutputFormat) -> miette::Result<()> {
    let baseline: EvaluationReport = read_json(args.baseline).await?;
    let candidate: EvaluationReport = read_json(args.candidate).await?;
    let result = gitflow_core::evaluation::compare(&baseline, &candidate)
        .map_err(|error| miette::miette!("{error}"))?;
    super::output::print_output(&result, &decision_output_format(output))
}

async fn read_json<T: DeserializeOwned>(path: String) -> miette::Result<T> {
    let bytes = tokio::task::spawn_blocking(move || read_bounded_file(&path))
        .await
        .map_err(|_| miette::miette!("evaluation file read failed"))??;
    serde_json::from_slice(&bytes).map_err(|_| miette::miette!("evaluation JSON is invalid"))
}

fn read_bounded_file(path: &str) -> miette::Result<Vec<u8>> {
    let safe = gitflow_core::SafePath::new_allow_absolute(path)
        .map_err(|_| miette::miette!("evaluation file path is invalid"))?;
    let file = std::fs::File::open(safe.as_path())
        .map_err(|_| miette::miette!("evaluation file cannot be opened"))?;
    let mut bytes = Vec::new();
    file.take(MAX_ARTIFACT_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| miette::miette!("evaluation file cannot be read"))?;
    if bytes.len() as u64 > MAX_ARTIFACT_BYTES {
        return Err(miette::miette!("evaluation file is too large"));
    }
    Ok(bytes)
}

async fn write_saved_responses(path: String, saved: &SavedResponses) -> miette::Result<()> {
    let bytes = serde_json::to_vec_pretty(saved)
        .map_err(|_| miette::miette!("saved responses cannot be serialized"))?;
    if bytes.len() as u64 > MAX_ARTIFACT_BYTES {
        return Err(miette::miette!("saved responses are too large"));
    }
    tokio::task::spawn_blocking(move || write_private_file(&path, &bytes))
        .await
        .map_err(|_| miette::miette!("saved responses write failed"))?
}

#[allow(
    clippy::disallowed_methods,
    reason = "Directory creation is bounded to the ignored decision cache"
)]
fn write_private_file(path: &str, bytes: &[u8]) -> miette::Result<()> {
    use std::io::Write;

    let safe = gitflow_core::SafePath::new(path)
        .map_err(|_| miette::miette!("saved response path is invalid"))?;
    let destination = safe.as_path();
    if !destination.starts_with(".cache/decision")
        || destination
            .extension()
            .is_none_or(|extension| extension != "json")
    {
        return Err(miette::miette!(
            "saved responses must be a JSON file under .cache/decision"
        ));
    }
    std::fs::create_dir_all(".cache/decision")
        .map_err(|_| miette::miette!("saved response directory cannot be created"))?;
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(destination)
        .map_err(|_| miette::miette!("saved response file cannot be created"))?;
    file.write_all(bytes)
        .map_err(|_| miette::miette!("saved response file cannot be written"))?;
    Ok(())
}

#[cfg(feature = "gitflow-jev")]
async fn live_responses(
    fixture: &FixtureSuite,
    input_price: Option<f64>,
    output_price: Option<f64>,
) -> SavedResponses {
    use gitflow_core::{
        decision::{DecisionEngine, DecisionError},
        evaluation::FailureKind,
    };

    let enabled = std::env::var("GF_DECISION_PROVIDER").as_deref() == Ok("jev");
    let engine = if enabled {
        gitflow_jev::JevEngine::from_env().ok()
    } else {
        None
    };
    let mut results = Vec::with_capacity(fixture.cases.len());
    for case in &fixture.cases {
        let outcome = if let Some(engine) = &engine {
            let start = Instant::now();
            match engine.decide(&case.request).await {
                Ok(response) => SavedOutcome::Ok {
                    response,
                    latency_ms: u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX),
                },
                Err(error) => SavedOutcome::Error {
                    kind: match error {
                        DecisionError::Unavailable => FailureKind::Unavailable,
                        DecisionError::Timeout => FailureKind::Timeout,
                        DecisionError::InvalidResponse(_) => FailureKind::InvalidResponse,
                        _ => FailureKind::Transport,
                    },
                },
            }
        } else {
            SavedOutcome::Error {
                kind: FailureKind::Unavailable,
            }
        };
        results.push(SavedResult {
            case_id: case.id.clone(),
            outcome,
        });
    }
    SavedResponses {
        schema_version: gitflow_core::evaluation::EVALUATION_SCHEMA_VERSION,
        dataset_id: fixture.dataset_id.clone(),
        provider: "jev".to_string(),
        input_price_per_million_usd: input_price,
        output_price_per_million_usd: output_price,
        results,
    }
}

#[cfg(not(feature = "gitflow-jev"))]
async fn live_responses(
    fixture: &FixtureSuite,
    input_price: Option<f64>,
    output_price: Option<f64>,
) -> SavedResponses {
    use gitflow_core::evaluation::FailureKind;

    SavedResponses {
        schema_version: gitflow_core::evaluation::EVALUATION_SCHEMA_VERSION,
        dataset_id: fixture.dataset_id.clone(),
        provider: "jev".to_string(),
        input_price_per_million_usd: input_price,
        output_price_per_million_usd: output_price,
        results: fixture
            .cases
            .iter()
            .map(|case| SavedResult {
                case_id: case.id.clone(),
                outcome: SavedOutcome::Error {
                    kind: FailureKind::Unavailable,
                },
            })
            .collect(),
    }
}
