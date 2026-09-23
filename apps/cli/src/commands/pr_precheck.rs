//! Bounded, read-only PR review precheck entry point.

use gitflow_core::{
    decision::DecisionResponse,
    pr_precheck::{PrPrecheckInput, PrPrecheckReport, Visibility, precheck, report_from_response},
};

const MAX_INPUT_BYTES: u64 = 131_072;
const MAX_RESPONSE_BYTES: u64 = 131_072;

/// Run deterministic collection and optional typed semantic advice.
///
/// # Errors
///
/// Returns an error for invalid input or disallowed private provider access.
pub async fn handle(
    input: String,
    response: Option<String>,
    live: bool,
    allow_private: bool,
) -> miette::Result<PrPrecheckReport> {
    let input: PrPrecheckInput = super::issue_precheck::read_json(input, MAX_INPUT_BYTES).await?;
    input
        .decision_request()
        .map_err(|e| miette::miette!("{e}"))?;
    if let Some(path) = response {
        let saved: DecisionResponse =
            super::issue_precheck::read_json(path, MAX_RESPONSE_BYTES).await?;
        return report_from_response(&input, &saved).map_err(|e| miette::miette!("{e}"));
    }
    if live && input.visibility != Visibility::Public && !allow_private {
        return Err(miette::miette!(
            "PR visibility is private or unknown; use --allow-private for live provider access"
        ));
    }
    if live {
        return live_report(&input).await;
    }
    precheck(&input, None)
        .await
        .map_err(|e| miette::miette!("{e}"))
}

#[cfg(feature = "gitflow-jev")]
async fn live_report(input: &PrPrecheckInput) -> miette::Result<PrPrecheckReport> {
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
async fn live_report(input: &PrPrecheckInput) -> miette::Result<PrPrecheckReport> {
    precheck(input, None)
        .await
        .map_err(|e| miette::miette!("{e}"))
}
