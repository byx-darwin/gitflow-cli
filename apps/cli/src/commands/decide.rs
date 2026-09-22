//! Explicit, bounded entry point for optional typed decisions.

#![allow(
    clippy::disallowed_types,
    reason = "Bounded local input is read inside spawn_blocking"
)]

use std::io::Read;

use clap::{Args, Subcommand};
#[cfg(feature = "gitflow-jev")]
use gitflow_core::decision::DecisionEngine;
use gitflow_core::decision::{DecisionRequest, Question};

use crate::OutputFormat;

const MAX_INPUT_BYTES: u64 = 131_072;

/// Typed decision commands. All variants accept a JSON `DecisionRequest`.
#[derive(Debug, Subcommand)]
pub enum DecideCommand {
    /// Evaluate one or more yes/no questions.
    Noul(DecideArgs),
    /// Evaluate one or more closed-set questions.
    Choice(DecideArgs),
    /// Evaluate one or more ordered-scale questions.
    Score(DecideArgs),
    /// Evaluate mixed questions in one provider request.
    Batch(DecideArgs),
    /// Evaluate versioned fixtures from saved responses, or explicitly run live.
    Eval(EvalArgs),
    /// Suggest a question-specific confidence threshold from a saved report.
    Calibrate(CalibrateArgs),
    /// Compare two saved evaluation reports.
    Compare(CompareArgs),
}

/// Offline response evaluation or explicit live evaluation.
#[derive(Debug, Args)]
pub struct EvalArgs {
    /// Versioned fixture JSON file.
    #[arg(long)]
    pub fixtures: String,
    /// Versioned saved-response JSON file for offline evaluation.
    #[arg(long, conflicts_with = "live")]
    pub responses: Option<String>,
    /// Explicitly call the configured provider; never enabled by default.
    #[arg(long)]
    pub live: bool,
    /// Save live typed responses under the ignored `.cache/decision/` directory.
    #[arg(long, requires = "live")]
    pub save_responses: Option<String>,
    /// Optional input token price for estimated cost.
    #[arg(long, requires = "live")]
    pub input_price_per_million_usd: Option<f64>,
    /// Optional output token price for estimated cost.
    #[arg(long, requires = "live")]
    pub output_price_per_million_usd: Option<f64>,
}

/// Select a conservative acceptance threshold from a saved report.
#[derive(Debug, Args)]
pub struct CalibrateArgs {
    /// Evaluation report JSON file.
    #[arg(long)]
    pub report: String,
    /// Question identifier to calibrate.
    #[arg(long)]
    pub question: String,
    /// Target for the lower 95% accepted-accuracy confidence bound.
    #[arg(long)]
    pub target_accuracy: f64,
    /// Optional language slice.
    #[arg(long)]
    pub language: Option<String>,
    /// Optional risk slice.
    #[arg(long)]
    pub risk: Option<String>,
}

/// Compare two evaluation reports.
#[derive(Debug, Args)]
pub struct CompareArgs {
    /// Baseline report JSON file.
    #[arg(long)]
    pub baseline: String,
    /// Candidate report JSON file.
    #[arg(long)]
    pub candidate: String,
}

/// Input location. Omit `--input` to read standard input.
#[derive(Debug, Args)]
pub struct DecideArgs {
    /// JSON request file. Paths are checked by `SafePath`.
    #[arg(long)]
    input: Option<String>,
}

/// Execute a typed decision without changing repository or ticket state.
///
/// # Errors
///
/// Returns a content-free error for unavailable providers, invalid input,
/// transport failures, and malformed responses.
pub async fn handle(command: DecideCommand, output: OutputFormat) -> miette::Result<()> {
    match command {
        DecideCommand::Eval(args) => super::decide_eval::evaluate(args, output).await,
        DecideCommand::Calibrate(args) => super::decide_eval::calibrate(args, output).await,
        DecideCommand::Compare(args) => super::decide_eval::compare(args, output).await,
        other => handle_inference(other, output).await,
    }
}

async fn handle_inference(command: DecideCommand, output: OutputFormat) -> miette::Result<()> {
    let (kind, args) = match command {
        DecideCommand::Noul(args) => ("noul", args),
        DecideCommand::Choice(args) => ("choice", args),
        DecideCommand::Score(args) => ("score", args),
        DecideCommand::Batch(args) => ("batch", args),
        _ => return Err(miette::miette!("invalid decision command")),
    };
    let bytes = tokio::task::spawn_blocking(move || read_bounded_input(args.input))
        .await
        .map_err(|_| miette::miette!("decision input read failed"))??;
    let request: DecisionRequest = serde_json::from_slice(&bytes)
        .map_err(|_| miette::miette!("decision input JSON is invalid"))?;
    request.validate().map_err(|e| miette::miette!("{e}"))?;
    if !request
        .questions
        .values()
        .all(|question| matches_kind(question, kind))
    {
        return Err(miette::miette!("question type does not match subcommand"));
    }
    #[cfg(feature = "gitflow-jev")]
    {
        if std::env::var("GF_DECISION_PROVIDER").as_deref() != Ok("jev") {
            return Err(miette::miette!("decision provider unavailable"));
        }
        let engine = gitflow_jev::JevEngine::from_env().map_err(|e| miette::miette!("{e}"))?;
        let response = engine
            .decide(&request)
            .await
            .map_err(|e| miette::miette!("{e}"))?;
        super::output::print_output(&response, &decision_output_format(output))
    }
    #[cfg(not(feature = "gitflow-jev"))]
    {
        let _ = output;
        Err(miette::miette!(
            "decision provider unavailable: build with feature gitflow-jev"
        ))
    }
}

pub(super) fn decision_output_format(format: OutputFormat) -> OutputFormat {
    if matches!(format, OutputFormat::Auto) {
        OutputFormat::Json
    } else {
        format
    }
}

fn matches_kind(question: &Question, kind: &str) -> bool {
    match kind {
        "noul" => matches!(question, Question::Noul { .. }),
        "choice" => matches!(question, Question::Choice { .. }),
        "score" => matches!(question, Question::Score { .. }),
        "batch" => true,
        _ => false,
    }
}

fn read_bounded_input(path: Option<String>) -> miette::Result<Vec<u8>> {
    let input: Box<dyn Read> = if let Some(path) = path {
        let safe = gitflow_core::SafePath::new_allow_absolute(&path)
            .map_err(|_| miette::miette!("decision input path is invalid"))?;
        let file = std::fs::File::open(safe.as_path())
            .map_err(|_| miette::miette!("decision input file cannot be opened"))?;
        Box::new(file)
    } else {
        Box::new(std::io::stdin())
    };
    let mut bytes = Vec::new();
    input
        .take(MAX_INPUT_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| miette::miette!("decision input cannot be read"))?;
    if bytes.len() as u64 > MAX_INPUT_BYTES {
        return Err(miette::miette!("decision input is too large"));
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use gitflow_core::decision::Question;

    use super::matches_kind;

    #[test]
    fn test_should_reject_question_type_mismatch() {
        let question = Question::Noul {
            instructions: "Is it blocked?".into(),
        };
        assert!(!matches_kind(&question, "choice"));
        assert!(matches_kind(&question, "noul"));
        assert!(matches_kind(&question, "batch"));
    }

    #[test]
    fn test_should_use_json_for_default_decision_output() {
        assert!(matches!(
            super::decision_output_format(crate::OutputFormat::Auto),
            crate::OutputFormat::Json
        ));
    }
}
