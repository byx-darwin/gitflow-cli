//! Bounded read-only Issue quality precheck input and optional Jev call.

#![allow(
    clippy::disallowed_types,
    reason = "Bounded local reads run inside spawn_blocking"
)]

use std::io::Read;

use gitflow_core::{
    decision::DecisionResponse,
    issue_quality::{IssueQualityInput, IssueQualityReport, precheck, report_from_response},
};
use serde::de::DeserializeOwned;

const MAX_INPUT_BYTES: u64 = 12_000;
const MAX_RESPONSE_BYTES: u64 = 131_072;

/// Run deterministic input validation and optional typed semantic advice.
///
/// # Errors
///
/// Returns a content-free error for invalid input or unreadable files.
pub async fn handle(
    input: String,
    response: Option<String>,
    live: bool,
) -> miette::Result<IssueQualityReport> {
    let input: IssueQualityInput = read_json(input, MAX_INPUT_BYTES).await?;
    let request = input
        .decision_request()
        .map_err(|e| miette::miette!("{e}"))?;
    if let Some(path) = response {
        let saved: DecisionResponse = read_json(path, MAX_RESPONSE_BYTES).await?;
        return Ok(report_from_response(&request, &saved));
    }
    if live {
        return live_report(&input).await;
    }
    precheck(&input, None)
        .await
        .map_err(|e| miette::miette!("{e}"))
}

pub(super) async fn read_json<T: DeserializeOwned>(path: String, limit: u64) -> miette::Result<T> {
    let bytes = tokio::task::spawn_blocking(move || read_bounded_file(&path, limit))
        .await
        .map_err(|_| miette::miette!("Issue precheck file read failed"))??;
    serde_json::from_slice(&bytes).map_err(|_| miette::miette!("Issue precheck JSON is invalid"))
}

fn read_bounded_file(path: &str, limit: u64) -> miette::Result<Vec<u8>> {
    let safe = gitflow_core::SafePath::new_allow_absolute(path)
        .map_err(|_| miette::miette!("Issue precheck file path is invalid"))?;
    let file = std::fs::File::open(safe.as_path())
        .map_err(|_| miette::miette!("Issue precheck file cannot be opened"))?;
    let mut bytes = Vec::new();
    file.take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| miette::miette!("Issue precheck file cannot be read"))?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > limit {
        return Err(miette::miette!("Issue precheck file is too large"));
    }
    Ok(bytes)
}

#[cfg(feature = "gitflow-jev")]
async fn live_report(input: &IssueQualityInput) -> miette::Result<IssueQualityReport> {
    if std::env::var("GF_DECISION_PROVIDER").as_deref() != Ok("jev") {
        return precheck(input, None)
            .await
            .map_err(|e| miette::miette!("{e}"));
    }
    let Ok(engine) = gitflow_jev::JevEngine::from_env() else {
        return precheck(input, None)
            .await
            .map_err(|e| miette::miette!("{e}"));
    };
    precheck(input, Some(&engine))
        .await
        .map_err(|e| miette::miette!("{e}"))
}

#[cfg(not(feature = "gitflow-jev"))]
async fn live_report(input: &IssueQualityInput) -> miette::Result<IssueQualityReport> {
    precheck(input, None)
        .await
        .map_err(|e| miette::miette!("{e}"))
}
