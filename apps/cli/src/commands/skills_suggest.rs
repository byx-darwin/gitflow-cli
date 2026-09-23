//! Read-only task-to-skill routing over the bundled, local gf skill catalog.

#![allow(
    clippy::disallowed_types,
    reason = "Bounded stdin and file reads run inside spawn_blocking"
)]

use std::io::Read;

use gitflow_core::{
    decision::DecisionResponse,
    skill_suggestion::{SkillCatalogEntry, SkillRouter},
};

use super::skills::{SKILLS, SuggestArgs};
use crate::OutputFormat;

const MAX_QUERY_BYTES: u64 = 1_024;
const MAX_RESPONSE_BYTES: u64 = 131_072;

/// Route a bounded task description to local gf skill IDs only.
///
/// # Errors
///
/// Returns an error for invalid task text, catalog data, saved response, or output I/O.
pub async fn handle(args: &SuggestArgs, output: OutputFormat) -> miette::Result<()> {
    let query = match (&args.query, args.stdin) {
        (Some(query), false) => query.clone(),
        (None, true) => tokio::task::spawn_blocking(read_stdin)
            .await
            .map_err(|_| miette::miette!("skill task input read failed"))??,
        _ => return Err(miette::miette!("specify exactly one of --query or --stdin")),
    };
    let router =
        SkillRouter::new(bundled_catalog()?).map_err(|error| miette::miette!("{error}"))?;
    let request = router
        .decision_request(&query)
        .map_err(|error| miette::miette!("{error}"))?;
    let response: Option<DecisionResponse> = if let Some(path) = &args.response {
        let path = path.clone();
        Some(
            tokio::task::spawn_blocking(move || read_saved_response(&path))
                .await
                .map_err(|_| miette::miette!("skill response read failed"))??,
        )
    } else if args.live {
        live_response(&request).await
    } else {
        None
    };
    let report = router
        .suggest(&query, response.as_ref())
        .map_err(|error| miette::miette!("{error}"))?;
    let format = if matches!(output, OutputFormat::Auto) {
        OutputFormat::Json
    } else {
        output
    };
    super::output::print_output(&report, &format)
}

fn bundled_catalog() -> miette::Result<Vec<SkillCatalogEntry>> {
    let mut entries = Vec::new();
    for (path, bytes) in SKILLS {
        let Some(id) = path.strip_suffix("/SKILL.md") else {
            continue;
        };
        if !id.starts_with("gf-") || id.contains('/') {
            continue;
        }
        let content = std::str::from_utf8(bytes)
            .map_err(|_| miette::miette!("bundled skill metadata is invalid"))?;
        let (name, description) = frontmatter_summary(content)?;
        if name != id {
            return Err(miette::miette!(
                "bundled skill metadata does not match its catalog ID"
            ));
        }
        entries.push(SkillCatalogEntry::new(id, &description));
    }
    Ok(entries)
}

fn frontmatter_summary(content: &str) -> miette::Result<(&str, String)> {
    let front = content
        .strip_prefix("---\n")
        .or_else(|| content.strip_prefix("---\r\n"))
        .ok_or_else(|| miette::miette!("bundled skill frontmatter is invalid"))?;
    let mut name = None;
    let mut description = Vec::new();
    let mut in_description = false;
    for line in front.lines() {
        if line == "---" {
            break;
        }
        if let Some(value) = line.strip_prefix("name: ") {
            name = Some(value.trim());
            in_description = false;
        } else if let Some(value) = line.strip_prefix("description: ") {
            in_description = true;
            if value != "|" && value != ">" {
                description.push(value.trim());
            }
        } else if in_description && line.starts_with("  ") {
            description.push(line.trim());
        } else {
            in_description = false;
        }
    }
    let name = name.ok_or_else(|| miette::miette!("bundled skill name is missing"))?;
    let description = description
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    if description.is_empty() {
        return Err(miette::miette!("bundled skill description is missing"));
    }
    Ok((name, description))
}

fn read_stdin() -> miette::Result<String> {
    let mut bytes = Vec::new();
    std::io::stdin()
        .take(MAX_QUERY_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| miette::miette!("skill task input cannot be read"))?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > MAX_QUERY_BYTES {
        return Err(miette::miette!("skill task input is too large"));
    }
    String::from_utf8(bytes).map_err(|_| miette::miette!("skill task input is not UTF-8"))
}

fn read_saved_response(path: &str) -> miette::Result<DecisionResponse> {
    let safe = gitflow_core::SafePath::new_allow_absolute(path)
        .map_err(|_| miette::miette!("saved skill response path is invalid"))?;
    let file = std::fs::File::open(safe.as_path())
        .map_err(|_| miette::miette!("saved skill response cannot be opened"))?;
    let mut bytes = Vec::new();
    file.take(MAX_RESPONSE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| miette::miette!("saved skill response cannot be read"))?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > MAX_RESPONSE_BYTES {
        return Err(miette::miette!("saved skill response is too large"));
    }
    serde_json::from_slice(&bytes)
        .map_err(|_| miette::miette!("saved skill response JSON is invalid"))
}

#[cfg(feature = "gitflow-jev")]
async fn live_response(
    request: &gitflow_core::decision::DecisionRequest,
) -> Option<DecisionResponse> {
    use gitflow_core::decision::DecisionEngine;

    if std::env::var("GF_DECISION_PROVIDER").as_deref() != Ok("jev") {
        return None;
    }
    let engine = gitflow_jev::JevEngine::from_env().ok()?;
    engine.decide(request).await.ok()
}

#[cfg(not(feature = "gitflow-jev"))]
async fn live_response(
    _request: &gitflow_core::decision::DecisionRequest,
) -> Option<DecisionResponse> {
    None
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use gitflow_core::{
        decision::{DecisionResponse, DecisionUsage, TypedAnswer},
        skill_suggestion::{SkillRouter, SuggestionStatus},
    };
    use serde::Deserialize;

    use super::{bundled_catalog, frontmatter_summary};

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct EvaluationSuite {
        schema_version: u32,
        input_price_per_million_usd: f64,
        output_price_per_million_usd: f64,
        cases: Vec<EvaluationCase>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct EvaluationCase {
        id: String,
        language: String,
        kind: String,
        query: String,
        expected: Vec<Option<String>>,
        selected: Vec<String>,
        secondary: Vec<String>,
        confidence: Option<f64>,
        latency_ms: Option<u64>,
    }

    #[test]
    fn test_should_load_only_known_bundled_skill_ids() {
        let entries = bundled_catalog().unwrap();
        assert_eq!(entries.len(), 30);
        assert!(
            entries
                .iter()
                .all(|entry| entry.skill_id.starts_with("gf-") && !entry.description.is_empty())
        );
    }

    #[test]
    fn test_should_reject_missing_skill_frontmatter() {
        assert!(frontmatter_summary("# no frontmatter").is_err());
    }

    #[test]
    fn test_should_accept_crlf_skill_frontmatter() {
        let (name, description) = frontmatter_summary(
            "---\r\nname: gf-example\r\ndescription: Windows checkout\r\n---\r\n",
        )
        .unwrap();
        assert_eq!(name, "gf-example");
        assert_eq!(description, "Windows checkout");
    }

    #[test]
    fn test_should_replay_synthetic_skill_routing_metrics() {
        let suite: EvaluationSuite = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures/skill-suggestion/evaluation-v1.json"
        )))
        .unwrap();
        assert_eq!(suite.schema_version, 1);
        let router = SkillRouter::new(bundled_catalog().unwrap()).unwrap();
        let catalog: Vec<String> = bundled_catalog()
            .unwrap()
            .into_iter()
            .map(|entry| entry.skill_id)
            .collect();
        let mut top1_correct = 0;
        let mut top3_recalled = 0;
        let mut answerable = 0;
        let mut abstentions = 0;
        let mut accepted = 0;
        let mut wrong_high_confidence = 0;
        let mut latencies = Vec::new();
        let mut input_tokens = 0_u64;
        let mut output_tokens = 0_u64;
        for case in &suite.cases {
            assert!(!case.id.is_empty() && !case.language.is_empty() && !case.kind.is_empty());
            let request = router.decision_request(&case.query).unwrap();
            assert_eq!(request.questions.len(), case.expected.len());
            let response = if let Some(confidence) = case.confidence {
                assert_eq!(case.selected.len(), case.expected.len());
                assert_eq!(case.secondary.len(), case.expected.len());
                let mut answers = BTreeMap::new();
                for index in 0..case.selected.len() {
                    let mut probabilities: BTreeMap<String, f64> =
                        catalog.iter().cloned().map(|id| (id, 0.0)).collect();
                    probabilities.insert("none".to_string(), 0.0);
                    probabilities.insert(case.selected[index].clone(), confidence);
                    probabilities.insert(case.secondary[index].clone(), 1.0 - confidence);
                    answers.insert(
                        format!("intent_{index}"),
                        TypedAnswer::Choice {
                            choice: case.selected[index].clone(),
                            confidence,
                            probabilities,
                        },
                    );
                }
                latencies.push(case.latency_ms.unwrap());
                input_tokens += 100;
                output_tokens += 20;
                Some(DecisionResponse {
                    model: "fake-eval-1".to_string(),
                    answers,
                    usage: DecisionUsage {
                        input_tokens: 100,
                        output_tokens: 20,
                    },
                })
            } else {
                assert!(case.selected.is_empty() && case.latency_ms.is_none());
                None
            };
            let report = router.suggest(&case.query, response.as_ref()).unwrap();
            if report.status != SuggestionStatus::Accepted {
                abstentions += 1;
            }
            if report.status == SuggestionStatus::Accepted {
                accepted += 1;
            }
            for (suggestion, expected) in report.suggestions.iter().zip(&case.expected) {
                if let Some(expected) = expected
                    && response.is_some()
                {
                    answerable += 1;
                    if suggestion.skill_id.as_deref() == Some(expected) {
                        top1_correct += 1;
                    }
                    if suggestion.skill_id.as_deref() == Some(expected)
                        || suggestion
                            .alternatives
                            .iter()
                            .any(|alternative| &alternative.skill_id == expected)
                    {
                        top3_recalled += 1;
                    }
                    if report.status == SuggestionStatus::Accepted
                        && suggestion.skill_id.as_deref() != Some(expected)
                    {
                        wrong_high_confidence += 1;
                    }
                }
            }
        }
        latencies.sort_unstable();
        let cost = (suite.input_price_per_million_usd
            * f64::from(u32::try_from(input_tokens).unwrap())
            + suite.output_price_per_million_usd
                * f64::from(u32::try_from(output_tokens).unwrap()))
            / 1_000_000.0;
        assert_eq!((top1_correct, top3_recalled, answerable), (5, 6, 6));
        assert_eq!((abstentions, accepted, wrong_high_confidence), (4, 3, 1));
        assert_eq!(latencies, vec![90, 110, 120, 130, 160, 180]);
        assert!((cost - 0.000_061_2).abs() < 1e-12);
    }
}
