//! Bounded, read-only workflow recommendation input and optional provider call.

#![allow(
    clippy::disallowed_types,
    reason = "Bounded local reads run inside spawn_blocking"
)]

use std::io::Read;

use gitflow_core::{decision::DecisionResponse, workflow_recommendation::RecommendationInput};
use serde::de::DeserializeOwned;

const MAX_INPUT_BYTES: u64 = 8_192;
const MAX_RESPONSE_BYTES: u64 = 131_072;

/// Evaluate local workflow rules and optional typed Jev advice.
///
/// # Errors
///
/// Returns a safe error for invalid input, response, or output I/O.
pub async fn handle(input: String, response: Option<String>, live: bool) -> miette::Result<()> {
    let input: RecommendationInput = read_json(input, MAX_INPUT_BYTES).await?;
    input
        .validate()
        .map_err(|error| miette::miette!("{error}"))?;
    let response: Option<DecisionResponse> = if let Some(path) = response {
        Some(read_json(path, MAX_RESPONSE_BYTES).await?)
    } else if live {
        live_response(&input).await
    } else {
        None
    };
    let recommendation =
        gitflow_core::workflow_recommendation::recommend(&input, response.as_ref())
            .map_err(|error| miette::miette!("{error}"))?;
    super::output::print_output(&recommendation, &crate::OutputFormat::Json)
}

async fn read_json<T: DeserializeOwned>(path: String, limit: u64) -> miette::Result<T> {
    let bytes = tokio::task::spawn_blocking(move || read_bounded_file(&path, limit))
        .await
        .map_err(|_| miette::miette!("workflow recommendation file read failed"))??;
    serde_json::from_slice(&bytes)
        .map_err(|_| miette::miette!("workflow recommendation JSON is invalid"))
}

fn read_bounded_file(path: &str, limit: u64) -> miette::Result<Vec<u8>> {
    let safe = gitflow_core::SafePath::new_allow_absolute(path)
        .map_err(|_| miette::miette!("workflow recommendation file path is invalid"))?;
    let file = std::fs::File::open(safe.as_path())
        .map_err(|_| miette::miette!("workflow recommendation file cannot be opened"))?;
    let mut bytes = Vec::new();
    file.take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| miette::miette!("workflow recommendation file cannot be read"))?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > limit {
        return Err(miette::miette!("workflow recommendation file is too large"));
    }
    Ok(bytes)
}

#[cfg(feature = "gitflow-jev")]
async fn live_response(input: &RecommendationInput) -> Option<DecisionResponse> {
    use gitflow_core::decision::DecisionEngine;

    if std::env::var("GF_DECISION_PROVIDER").as_deref() != Ok("jev") {
        return None;
    }
    let engine = gitflow_jev::JevEngine::from_env().ok()?;
    let request = input.decision_request().ok()?;
    engine.decide(&request).await.ok()
}

#[cfg(not(feature = "gitflow-jev"))]
async fn live_response(_input: &RecommendationInput) -> Option<DecisionResponse> {
    None
}
