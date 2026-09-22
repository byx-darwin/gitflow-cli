//! Provider-independent typed decision requests and validated answers.

use std::collections::BTreeMap;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

/// Maximum serialized state size sent to a decision provider.
pub const MAX_STATE_BYTES: usize = 32_768;
/// Maximum number of independent questions in one request.
pub const MAX_QUESTIONS: usize = 16;
/// Maximum number of Choice options (lower than the provider's 255 limit).
pub const MAX_CHOICE_OPTIONS: usize = 32;
/// Maximum number of ordered Score levels.
pub const MAX_SCORE_LEVELS: usize = 10;
/// Maximum serialized size of the complete request before provider wrapping.
pub const MAX_REQUEST_BYTES: usize = 131_072;

/// A bounded, provider-neutral decision request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionRequest {
    /// Minimal state for the decision; callers must remove unrelated private fields.
    pub state: Value,
    /// Questions keyed by stable caller-owned identifiers.
    pub questions: BTreeMap<String, Question>,
}

/// A typed question, independent of any provider wire protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase", deny_unknown_fields)]
pub enum Question {
    /// A calibrated yes/no question.
    Noul {
        /// The single yes/no question to evaluate.
        instructions: String,
    },
    /// Selection from a closed set of options.
    Choice {
        /// The choice question.
        instructions: String,
        /// Option identifier to distinct description.
        criteria: BTreeMap<String, String>,
    },
    /// Rating on an ordered low-to-high scale.
    Score {
        /// The rating question.
        instructions: String,
        /// Descriptive levels in ascending order.
        criteria: Vec<String>,
    },
}

/// Validated answer returned by a decision provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase", deny_unknown_fields)]
pub enum TypedAnswer {
    /// Probability that the answer to a yes/no question is yes.
    Noul {
        /// Probability in the inclusive range 0–1.
        noul: f64,
    },
    /// Closed-set selection and distribution.
    Choice {
        /// Selected option identifier.
        choice: String,
        /// Provider confidence in the selection.
        confidence: f64,
        /// Per-option probability distribution.
        probabilities: BTreeMap<String, f64>,
    },
    /// Rating and distribution on an ordered scale.
    Score {
        /// Probability-weighted position on the scale.
        score: f64,
        /// Provider confidence in the rating.
        confidence: f64,
        /// Numeric level identifier to description.
        legend: BTreeMap<String, String>,
        /// Per-level probability distribution.
        probabilities: BTreeMap<String, f64>,
    },
}

/// Token usage reported by a provider, without request or response content.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionUsage {
    /// Number of input tokens billed or counted.
    pub input_tokens: u64,
    /// Number of output tokens billed or counted.
    pub output_tokens: u64,
}

/// Complete decision response with one answer per requested question.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionResponse {
    /// Actual model identifier returned by the provider.
    pub model: String,
    /// Answers keyed by request question identifier.
    pub answers: BTreeMap<String, TypedAnswer>,
    /// Provider-reported token usage.
    pub usage: DecisionUsage,
}

/// A safe, content-free decision error.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum DecisionError {
    /// Provider is disabled or missing required configuration.
    #[error("decision provider unavailable")]
    Unavailable,
    /// Caller supplied a request that violates limits or schema.
    #[error("invalid decision request: {0}")]
    InvalidInput(&'static str),
    /// Provider response is incomplete or violates the requested schema.
    #[error("invalid decision response: {0}")]
    InvalidResponse(&'static str),
    /// The request timed out.
    #[error("decision provider timed out")]
    Timeout,
    /// The provider failed without exposing any sensitive body or header.
    #[error("decision provider request failed")]
    Transport,
}

/// Provider-independent asynchronous decision engine.
#[async_trait]
pub trait DecisionEngine: std::fmt::Debug + Send + Sync {
    /// Evaluate a bounded request.
    ///
    /// # Errors
    ///
    /// Returns a content-free [`DecisionError`] for invalid input, provider
    /// failures, or an invalid response.
    async fn decide(&self, request: &DecisionRequest) -> Result<DecisionResponse, DecisionError>;
}

impl DecisionRequest {
    /// Check state, question, and option limits before external transmission.
    ///
    /// # Errors
    ///
    /// Returns [`DecisionError::InvalidInput`] when a bound is exceeded or
    /// a question is malformed.
    pub fn validate(&self) -> Result<(), DecisionError> {
        let state_len = serde_json::to_vec(&self.state)
            .map_err(|_| DecisionError::InvalidInput("state is not serializable"))?
            .len();
        if !matches!(&self.state, Value::String(value) if !value.trim().is_empty())
            && !matches!(&self.state, Value::Array(values) if !values.is_empty())
            && !matches!(&self.state, Value::Object(values) if !values.is_empty())
        {
            return Err(DecisionError::InvalidInput("state is empty or invalid"));
        }
        if state_len > MAX_STATE_BYTES {
            return Err(DecisionError::InvalidInput("state size is out of bounds"));
        }
        if state_contains_obvious_credential(&self.state) {
            return Err(DecisionError::InvalidInput(
                "state contains a credential-like value",
            ));
        }
        if self.questions.is_empty() || self.questions.len() > MAX_QUESTIONS {
            return Err(DecisionError::InvalidInput(
                "question count is out of bounds",
            ));
        }
        for (id, question) in &self.questions {
            if id.is_empty()
                || id.len() > 64
                || !id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
            {
                return Err(DecisionError::InvalidInput("question id is invalid"));
            }
            let instructions = match question {
                Question::Noul { instructions }
                | Question::Choice { instructions, .. }
                | Question::Score { instructions, .. } => instructions,
            };
            if instructions.trim().is_empty() || instructions.len() > 1_024 {
                return Err(DecisionError::InvalidInput(
                    "instructions are out of bounds",
                ));
            }
            match question {
                Question::Noul { .. } => {}
                Question::Choice { criteria, .. } => {
                    if criteria.len() < 2 || criteria.len() > MAX_CHOICE_OPTIONS {
                        return Err(DecisionError::InvalidInput(
                            "choice option count is out of bounds",
                        ));
                    }
                    if criteria.iter().any(|(key, value)| {
                        key.is_empty()
                            || key.len() > 64
                            || value.trim().is_empty()
                            || value.len() > 512
                    }) {
                        return Err(DecisionError::InvalidInput("choice option is invalid"));
                    }
                }
                Question::Score { criteria, .. } => {
                    if criteria.len() < 2 || criteria.len() > MAX_SCORE_LEVELS {
                        return Err(DecisionError::InvalidInput(
                            "score level count is out of bounds",
                        ));
                    }
                    if criteria
                        .iter()
                        .any(|value| value.trim().is_empty() || value.len() > 512)
                    {
                        return Err(DecisionError::InvalidInput("score level is invalid"));
                    }
                }
            }
        }
        let request_len = serde_json::to_vec(self)
            .map_err(|_| DecisionError::InvalidInput("request is not serializable"))?
            .len();
        if request_len > MAX_REQUEST_BYTES {
            return Err(DecisionError::InvalidInput("request is too large"));
        }
        Ok(())
    }
}

impl DecisionResponse {
    /// Validate provider answers against the exact caller-owned question set.
    ///
    /// # Errors
    ///
    /// Returns [`DecisionError::InvalidResponse`] when any answer is missing,
    /// extra, mistyped, nonfinite, or inconsistent with its criteria.
    pub fn validate_against(&self, request: &DecisionRequest) -> Result<(), DecisionError> {
        if self.model.trim().is_empty()
            || self.model.len() > 128
            || self.answers.keys().ne(request.questions.keys())
        {
            return Err(DecisionError::InvalidResponse(
                "model or answer set is invalid",
            ));
        }
        for (id, question) in &request.questions {
            let answer = self
                .answers
                .get(id)
                .ok_or(DecisionError::InvalidResponse("answer missing"))?;
            match (question, answer) {
                (Question::Noul { .. }, TypedAnswer::Noul { noul }) => {
                    if !is_probability(*noul) {
                        return Err(DecisionError::InvalidResponse(
                            "noul probability is invalid",
                        ));
                    }
                }
                (
                    Question::Choice { criteria, .. },
                    TypedAnswer::Choice {
                        choice,
                        confidence,
                        probabilities,
                    },
                ) => {
                    if !is_probability(*confidence)
                        || !distribution_matches(probabilities, criteria.keys())
                        || !criteria.contains_key(choice)
                        || probabilities.get(choice).is_none_or(|selected| {
                            probabilities.values().any(|value| value > selected)
                        })
                    {
                        return Err(DecisionError::InvalidResponse(
                            "choice distribution is invalid",
                        ));
                    }
                }
                (
                    Question::Score { criteria, .. },
                    TypedAnswer::Score {
                        score,
                        confidence,
                        legend,
                        probabilities,
                    },
                ) => {
                    if !is_probability(*confidence)
                        || !score.is_finite()
                        || *score < 0.0
                        || *score
                            > f64::from(
                                u32::try_from(criteria.len().saturating_sub(1)).unwrap_or(u32::MAX),
                            )
                    {
                        return Err(DecisionError::InvalidResponse("score is invalid"));
                    }
                    let expected: BTreeMap<String, String> = criteria
                        .iter()
                        .enumerate()
                        .map(|(index, label)| (index.to_string(), label.clone()))
                        .collect();
                    let weighted = probabilities
                        .iter()
                        .filter_map(|(level, probability)| {
                            level
                                .parse::<u32>()
                                .ok()
                                .map(|level| f64::from(level) * probability)
                        })
                        .sum::<f64>();
                    if legend != &expected
                        || !distribution_matches(probabilities, expected.keys())
                        || (weighted - score).abs() > 0.02
                    {
                        return Err(DecisionError::InvalidResponse(
                            "score distribution is invalid",
                        ));
                    }
                }
                _ => {
                    return Err(DecisionError::InvalidResponse(
                        "answer type does not match question",
                    ));
                }
            }
        }
        Ok(())
    }
}

fn is_probability(value: f64) -> bool {
    value.is_finite() && (0.0..=1.0).contains(&value)
}

fn state_contains_obvious_credential(value: &Value) -> bool {
    match value {
        Value::String(text) => {
            let lower = text.to_ascii_lowercase();
            [
                "apikey_",
                "github_pat_",
                "ghp_",
                "gho_",
                "ghu_",
                "ghs_",
                "ghr_",
                "glpat-",
                "sk-",
            ]
            .iter()
            .any(|prefix| {
                lower.match_indices(prefix).any(|(index, _)| {
                    let rest = &lower[index + prefix.len()..];
                    rest.bytes()
                        .take_while(|byte| {
                            byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-')
                        })
                        .count()
                        >= 20
                })
            })
        }
        Value::Array(values) => values.iter().any(state_contains_obvious_credential),
        Value::Object(values) => values.iter().any(|(key, value)| {
            [
                "token",
                "api_key",
                "access_token",
                "password",
                "secret",
                "authorization",
                "private_key",
            ]
            .iter()
            .any(|sensitive| key.eq_ignore_ascii_case(sensitive))
                || state_contains_obvious_credential(value)
        }),
        _ => false,
    }
}

fn distribution_matches<'a>(
    actual: &BTreeMap<String, f64>,
    expected: impl Iterator<Item = &'a String>,
) -> bool {
    let expected: Vec<&String> = expected.collect();
    actual.len() == expected.len()
        && expected.iter().all(|key| actual.contains_key(*key))
        && actual.values().all(|value| is_probability(*value))
        && (actual.values().sum::<f64>() - 1.0).abs() <= 0.01
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::json;

    use super::{DecisionRequest, DecisionResponse, Question};

    #[test]
    fn test_should_reject_oversized_state() {
        let request = DecisionRequest {
            state: json!({ "body": "x".repeat(33_000) }),
            questions: BTreeMap::from([(
                "blocked".to_string(),
                Question::Noul {
                    instructions: "Is this blocked?".to_string(),
                },
            )]),
        };
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_should_reject_oversized_total_request() {
        let criteria: BTreeMap<String, String> = (0..32)
            .map(|index| (format!("option_{index}"), "x".repeat(512)))
            .collect();
        let questions = (0..16)
            .map(|index| {
                (
                    format!("q_{index}"),
                    Question::Choice {
                        instructions: "Choose one".into(),
                        criteria: criteria.clone(),
                    },
                )
            })
            .collect();
        let request = DecisionRequest {
            state: json!("small"),
            questions,
        };
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_should_reject_empty_state() {
        let mut request = DecisionRequest {
            state: json!("   "),
            questions: BTreeMap::from([(
                "blocked".into(),
                Question::Noul {
                    instructions: "Is this blocked?".into(),
                },
            )]),
        };
        assert!(request.validate().is_err());
        request.state = json!({});
        assert!(request.validate().is_err());
        request.state = json!([]);
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_should_reject_obvious_credential_in_state() {
        let mut request = DecisionRequest {
            state: json!({"title": format!("credential {}{}", "apikey_", "x".repeat(40))}),
            questions: BTreeMap::from([(
                "blocked".into(),
                Question::Noul {
                    instructions: "Is this blocked?".into(),
                },
            )]),
        };
        assert!(request.validate().is_err());
        request.state = json!({"token": "something-private"});
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_should_reject_answer_with_unknown_choice() {
        let request = DecisionRequest {
            state: json!({ "title": "Fix crash" }),
            questions: BTreeMap::from([(
                "type".to_string(),
                Question::Choice {
                    instructions: "Classify the issue".to_string(),
                    criteria: BTreeMap::from([
                        ("bug".to_string(), "Broken behavior".to_string()),
                        ("feature".to_string(), "New behavior".to_string()),
                    ]),
                },
            )]),
        };
        let response: DecisionResponse = serde_json::from_value(json!({
            "model": "jev-1.13.0",
            "answers": {"type": {
                "type": "choice", "choice": "other", "confidence": 0.9,
                "probabilities": {"bug": 0.1, "feature": 0.9}
            }},
            "usage": {"input_tokens": 10, "output_tokens": 3}
        }))
        .unwrap();
        assert!(response.validate_against(&request).is_err());
    }

    #[test]
    fn test_should_accept_valid_noul_answer() {
        let request = DecisionRequest {
            state: json!({ "title": "CI fails" }),
            questions: BTreeMap::from([(
                "blocked".to_string(),
                Question::Noul {
                    instructions: "Is work blocked?".to_string(),
                },
            )]),
        };
        let response: DecisionResponse = serde_json::from_value(json!({
            "model": "jev-1.13.0",
            "answers": {"blocked": {"type": "noul", "noul": 0.8}},
            "usage": {"input_tokens": 10, "output_tokens": 2}
        }))
        .unwrap();
        assert!(request.validate().is_ok());
        assert!(response.validate_against(&request).is_ok());
    }
}
