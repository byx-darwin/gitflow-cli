//! Optional semantic regression evaluation over captured deterministic results.

#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "bounded local fixture I/O"
)]

use std::{collections::BTreeMap, io::Read, path::Path};

use clap::{Args, Subcommand};
use gitflow_core::semantic_regression::{Observation, Suite};
use serde::de::DeserializeOwned;

/// Regression analysis commands.
#[derive(Debug, Subcommand)]
pub enum RegressionCommand {
    /// Evaluate captured cases after deterministic assertions have run.
    SemanticEval(SemanticEvalArgs),
}

/// Offline replay or explicit live semantic oracle.
#[derive(Debug, Args)]
pub struct SemanticEvalArgs {
    /// Versioned suite with captured deterministic outcomes.
    #[arg(long)]
    suite: String,
    /// Saved typed observations keyed by case ID.
    #[arg(long, conflicts_with = "live")]
    responses: Option<String>,
    /// Opt in to Jev requests; never used by ordinary CI.
    #[arg(long)]
    live: bool,
    /// Save live observations for replay under .cache/regression/.
    #[arg(long, requires = "live")]
    save_responses: Option<String>,
    /// Input token price per million, for an optional cost estimate.
    #[arg(long)]
    input_price_per_million_usd: Option<f64>,
    /// Output token price per million, for an optional cost estimate.
    #[arg(long)]
    output_price_per_million_usd: Option<f64>,
    /// Maximum simultaneous live requests (1–4).
    #[arg(long, default_value_t = 2, value_parser = clap::value_parser!(u8).range(1..=4))]
    parallel: u8,
}

const MAX_SUITE_BYTES: u64 = 262_144;
const MAX_RESPONSES_BYTES: u64 = 1_048_576;

fn read_json<T: DeserializeOwned>(path: &Path, max: u64) -> miette::Result<T> {
    let safe = gitflow_core::SafePath::new_allow_absolute(
        path.to_str()
            .ok_or_else(|| miette::miette!("invalid fixture path"))?,
    )
    .map_err(|_| miette::miette!("invalid fixture path"))?;
    let mut bytes = Vec::new();
    std::fs::File::open(safe.as_path())
        .map_err(|_| miette::miette!("regression fixture cannot be opened"))?
        .take(max + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| miette::miette!("regression fixture cannot be read"))?;
    if bytes.len() as u64 > max {
        return Err(miette::miette!("regression fixture is too large"));
    }
    serde_json::from_slice(&bytes)
        .map_err(|_| miette::miette!("regression fixture JSON is invalid"))
}

/// Evaluate a bounded suite and print an advisory JSON report.
///
/// # Errors
/// Returns a safe error for invalid fixtures, factual assertion failures, or output I/O.
pub async fn handle(command: RegressionCommand) -> miette::Result<()> {
    let RegressionCommand::SemanticEval(args) = command;
    let suite: Suite = read_json(Path::new(&args.suite), MAX_SUITE_BYTES)?;
    suite.validate().map_err(|e| miette::miette!("{e}"))?;
    if args.live && suite.cases.len() > 50 {
        return Err(miette::miette!("live semantic suite exceeds 50 cases"));
    }
    let responses: BTreeMap<String, Observation> = if let Some(file) = args.responses {
        read_json(Path::new(&file), MAX_RESPONSES_BYTES)?
    } else if args.live {
        let responses = live_responses(&suite, usize::from(args.parallel)).await;
        if let Some(path) = args.save_responses {
            save_responses(&path, &responses)?;
        }
        responses
    } else {
        BTreeMap::new()
    };
    let prices = match (
        args.input_price_per_million_usd,
        args.output_price_per_million_usd,
    ) {
        (None, None) => None,
        (Some(input), Some(output)) => Some((input, output)),
        _ => {
            return Err(miette::miette!(
                "both token prices are required for a cost estimate"
            ));
        }
    };
    let report = gitflow_core::semantic_regression::evaluate(&suite, &responses, prices)
        .map_err(|e| miette::miette!("{e}"))?;
    super::output::print_output(&report, &crate::OutputFormat::Json)?;
    if report.deterministic_failures > 0 {
        return Err(miette::miette!(
            "deterministic regression assertions failed"
        ));
    }
    Ok(())
}

fn save_responses(path: &str, responses: &BTreeMap<String, Observation>) -> miette::Result<()> {
    use std::io::Write;
    let safe =
        gitflow_core::SafePath::new(path).map_err(|_| miette::miette!("invalid save path"))?;
    let destination = safe.as_path();
    if !destination.starts_with(".cache/regression")
        || destination.extension().is_none_or(|ext| ext != "json")
    {
        return Err(miette::miette!(
            "saved responses must be JSON under .cache/regression"
        ));
    }
    let bytes = serde_json::to_vec_pretty(responses)
        .map_err(|_| miette::miette!("response serialization failed"))?;
    if bytes.len() as u64 > MAX_RESPONSES_BYTES {
        return Err(miette::miette!("saved responses are too large"));
    }
    std::fs::create_dir_all(".cache/regression")
        .map_err(|_| miette::miette!("response directory cannot be created"))?;
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(destination)
        .map_err(|_| miette::miette!("response file cannot be created"))?;
    file.write_all(&bytes)
        .map_err(|_| miette::miette!("response file cannot be written"))
}

#[cfg(feature = "gitflow-jev")]
async fn live_responses(suite: &Suite, parallel: usize) -> BTreeMap<String, Observation> {
    use std::{sync::Arc, time::Instant};

    use gitflow_core::decision::DecisionEngine;
    let mut responses = BTreeMap::new();
    if std::env::var("GF_DECISION_PROVIDER").as_deref() != Ok("jev") || suite.cases.len() > 50 {
        return responses;
    }
    let Ok(engine) = gitflow_jev::JevEngine::from_env() else {
        return responses;
    };
    let engine = Arc::new(engine);
    let mut tasks = tokio::task::JoinSet::new();
    for case in &suite.cases {
        if !case.deterministic.passed() {
            continue;
        }
        let Some(contract) = suite.contract(&case.contract_id) else {
            continue;
        };
        let Ok(request) = contract.request(case) else {
            continue;
        };
        while tasks.len() >= parallel {
            if let Some(Ok((id, Ok(response), elapsed))) = tasks.join_next().await {
                responses.insert(
                    id,
                    Observation {
                        response,
                        latency_ms: u64::try_from(elapsed).unwrap_or(u64::MAX),
                    },
                );
            }
        }
        let id = case.id.clone();
        let engine = Arc::clone(&engine);
        tasks.spawn(async move {
            let start = Instant::now();
            (
                id,
                engine.decide(&request).await,
                start.elapsed().as_millis(),
            )
        });
    }
    while let Some(Ok((id, Ok(response), elapsed))) = tasks.join_next().await {
        responses.insert(
            id,
            Observation {
                response,
                latency_ms: u64::try_from(elapsed).unwrap_or(u64::MAX),
            },
        );
    }
    responses
}

#[cfg(not(feature = "gitflow-jev"))]
async fn live_responses(_suite: &Suite, _parallel: usize) -> BTreeMap<String, Observation> {
    BTreeMap::new()
}
