//! Explicit, read-only pipeline failure analysis.

use gitflow_core::{
    decision::DecisionResponse,
    pipeline_failure::{
        PipelineFailureInput, PipelineFailureReport, analyze, report_from_response,
    },
};

const MAX_INPUT_BYTES: u64 = 200_000;
const MAX_RESPONSE_BYTES: u64 = 131_072;

/// Analyze reviewed excerpts with deterministic telemetry and optional Jev.
///
/// # Errors
///
/// Returns an error for invalid caller input or unreadable files.
pub async fn handle(
    input: String,
    response: Option<String>,
    live: bool,
) -> miette::Result<PipelineFailureReport> {
    let input: PipelineFailureInput =
        super::issue_precheck::read_json(input, MAX_INPUT_BYTES).await?;
    input
        .decision_request()
        .map_err(|e| miette::miette!("{e}"))?;
    if let Some(path) = response {
        let saved: DecisionResponse =
            super::issue_precheck::read_json(path, MAX_RESPONSE_BYTES).await?;
        return report_from_response(&input, &saved).map_err(|e| miette::miette!("{e}"));
    }
    if live {
        return live_report(&input).await;
    }
    analyze(&input, None)
        .await
        .map_err(|e| miette::miette!("{e}"))
}

#[cfg(feature = "gitflow-jev")]
async fn live_report(input: &PipelineFailureInput) -> miette::Result<PipelineFailureReport> {
    if std::env::var("GF_DECISION_PROVIDER").as_deref() != Ok("jev") {
        return analyze(input, None)
            .await
            .map_err(|e| miette::miette!("{e}"));
    }
    let Ok(engine) = gitflow_jev::JevEngine::from_env() else {
        return analyze(input, None)
            .await
            .map_err(|e| miette::miette!("{e}"));
    };
    analyze(input, Some(&engine))
        .await
        .map_err(|e| miette::miette!("{e}"))
}

#[cfg(not(feature = "gitflow-jev"))]
async fn live_report(input: &PipelineFailureInput) -> miette::Result<PipelineFailureReport> {
    analyze(input, None)
        .await
        .map_err(|e| miette::miette!("{e}"))
}
