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
    let (kind, args) = match command {
        DecideCommand::Noul(args) => ("noul", args),
        DecideCommand::Choice(args) => ("choice", args),
        DecideCommand::Score(args) => ("score", args),
        DecideCommand::Batch(args) => ("batch", args),
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
        super::output::print_output(&response, &output)
    }
    #[cfg(not(feature = "gitflow-jev"))]
    {
        let _ = output;
        Err(miette::miette!(
            "decision provider unavailable: build with feature gitflow-jev"
        ))
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
}
