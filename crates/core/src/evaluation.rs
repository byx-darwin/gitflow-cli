//! Versioned, provider-independent offline decision evaluation.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::decision::{
    DecisionRequest, DecisionResponse, Question, TypedAnswer, state_contains_obvious_credential,
};

/// Current fixture, saved-response, and report schema version.
pub const EVALUATION_SCHEMA_VERSION: u32 = 1;
/// Maximum number of cases in one evaluation suite.
pub const MAX_EVALUATION_CASES: usize = 500;
const MIN_CALIBRATION_CASES: u64 = 30;

/// Content-free validation errors for evaluation artifacts.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum EvaluationError {
    /// An artifact violates a schema, bound, or privacy rule.
    #[error("invalid evaluation artifact: {0}")]
    Invalid(&'static str),
    /// The requested question or group is absent.
    #[error("evaluation question or group not found")]
    NotFound,
}

/// A versioned set of labeled requests. It contains no provider credentials.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FixtureSuite {
    /// Version of this fixture schema.
    pub schema_version: u32,
    /// Stable, caller-owned dataset identifier.
    pub dataset_id: String,
    /// Labeled cases in the dataset.
    pub cases: Vec<FixtureCase>,
}

/// A labeled request, with language, risk, and provenance for grouped analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FixtureCase {
    /// Stable identifier, unique within the suite.
    pub id: String,
    /// `english`, `chinese`, `mixed`, or `other`.
    pub language: String,
    /// `low`, `medium`, `high`, or `critical`.
    pub risk: String,
    /// `synthetic`, `public_issue`, or `reviewed_report`.
    pub source: String,
    /// Exact request whose question schema is evaluated.
    pub request: DecisionRequest,
    /// Reference answers keyed by question ID.
    pub expected: BTreeMap<String, ExpectedAnswer>,
}

/// Independently labeled answer for one fixture question.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase", deny_unknown_fields)]
pub enum ExpectedAnswer {
    /// Binary ground truth.
    Noul {
        /// Whether the proposition is true.
        yes: bool,
    },
    /// Closed-set ground truth.
    Choice {
        /// Expected option key.
        choice: String,
    },
    /// Ordered-scale ground truth, zero based.
    Score {
        /// Expected level index.
        level: u8,
    },
}

/// Versioned saved provider results for fully offline evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SavedResponses {
    /// Version of the response schema.
    pub schema_version: u32,
    /// Dataset identifier that these results belong to.
    pub dataset_id: String,
    /// Provider identifier, independent of the model version.
    pub provider: String,
    /// Input token price; omitted when unknown.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_price_per_million_usd: Option<f64>,
    /// Output token price; omitted when unknown.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_price_per_million_usd: Option<f64>,
    /// One outcome per observed case. Missing cases are allowed and reported.
    pub results: Vec<SavedResult>,
}

/// One saved case result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SavedResult {
    /// Fixture case identifier.
    pub case_id: String,
    /// Valid response or content-free provider failure.
    pub outcome: SavedOutcome,
}

/// Explicit success or failure, without raw provider errors or request content.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "lowercase", deny_unknown_fields)]
pub enum SavedOutcome {
    /// Valid provider response and observed end-to-end latency.
    Ok {
        /// Typed response.
        response: DecisionResponse,
        /// Elapsed milliseconds.
        #[serde(rename = "latencyMs")]
        latency_ms: u64,
    },
    /// Unavailable or failed request; excluded from accuracy denominators.
    Error {
        /// Safe failure class.
        kind: FailureKind,
    },
}

/// Safe failure categories permitted in saved results.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureKind {
    /// Provider not configured or not available.
    Unavailable,
    /// Request timed out.
    Timeout,
    /// Transport or HTTP status failure.
    Transport,
    /// Response schema violation.
    InvalidResponse,
}

impl FixtureSuite {
    /// Validate schema, labels, bounds, and obvious sensitive content.
    ///
    /// # Errors
    ///
    /// Returns [`EvaluationError::Invalid`] for a malformed or unsafe suite.
    pub fn validate(&self) -> Result<(), EvaluationError> {
        if self.schema_version != EVALUATION_SCHEMA_VERSION
            || !valid_id(&self.dataset_id)
            || self.cases.is_empty()
            || self.cases.len() > MAX_EVALUATION_CASES
        {
            return Err(EvaluationError::Invalid("fixture header is invalid"));
        }
        let mut ids = BTreeSet::new();
        for case in &self.cases {
            if !valid_id(&case.id) || !ids.insert(case.id.as_str()) {
                return Err(EvaluationError::Invalid(
                    "case identifier is invalid or repeated",
                ));
            }
            if !["english", "chinese", "mixed", "other"].contains(&case.language.as_str())
                || !["low", "medium", "high", "critical"].contains(&case.risk.as_str())
                || !["synthetic", "public_issue", "reviewed_report"].contains(&case.source.as_str())
            {
                return Err(EvaluationError::Invalid("case metadata is invalid"));
            }
            case.request
                .validate()
                .map_err(|_| EvaluationError::Invalid("case request is invalid"))?;
            let serialized = serde_json::to_string(&case.request)
                .map_err(|_| EvaluationError::Invalid("case request is invalid"))?;
            let value = serde_json::to_value(&case.request)
                .map_err(|_| EvaluationError::Invalid("case request is invalid"))?;
            if state_contains_obvious_credential(&value) {
                return Err(EvaluationError::Invalid(
                    "fixture contains a credential-like value",
                ));
            }
            let lower = serialized.to_ascii_lowercase();
            if lower.contains("http://") || lower.contains("https://") {
                return Err(EvaluationError::Invalid("fixture contains a URL"));
            }
            if case.expected.keys().ne(case.request.questions.keys()) {
                return Err(EvaluationError::Invalid("expected answer set is invalid"));
            }
            for (id, question) in &case.request.questions {
                let expected = case
                    .expected
                    .get(id)
                    .ok_or(EvaluationError::Invalid("expected answer missing"))?;
                match (question, expected) {
                    (Question::Noul { .. }, ExpectedAnswer::Noul { .. }) => {}
                    (Question::Choice { criteria, .. }, ExpectedAnswer::Choice { choice })
                        if criteria.contains_key(choice) => {}
                    (Question::Score { criteria, .. }, ExpectedAnswer::Score { level })
                        if usize::from(*level) < criteria.len() => {}
                    _ => return Err(EvaluationError::Invalid("expected answer is incompatible")),
                }
            }
        }
        Ok(())
    }
}

impl SavedResponses {
    /// Validate saved outcomes against a fixture without requiring completeness.
    ///
    /// # Errors
    ///
    /// Returns [`EvaluationError::Invalid`] for unknown, repeated, or malformed results.
    pub fn validate_against(&self, fixture: &FixtureSuite) -> Result<(), EvaluationError> {
        if self.schema_version != EVALUATION_SCHEMA_VERSION
            || self.dataset_id != fixture.dataset_id
            || !valid_id(&self.provider)
            || self.results.len() > fixture.cases.len()
            || self
                .input_price_per_million_usd
                .is_some_and(|value| !value.is_finite() || !(0.0..=100_000.0).contains(&value))
            || self
                .output_price_per_million_usd
                .is_some_and(|value| !value.is_finite() || !(0.0..=100_000.0).contains(&value))
        {
            return Err(EvaluationError::Invalid("response header is invalid"));
        }
        let cases: BTreeMap<&str, &FixtureCase> = fixture
            .cases
            .iter()
            .map(|case| (case.id.as_str(), case))
            .collect();
        let mut ids = BTreeSet::new();
        for result in &self.results {
            if !ids.insert(result.case_id.as_str()) {
                return Err(EvaluationError::Invalid("response case is repeated"));
            }
            let case = cases
                .get(result.case_id.as_str())
                .ok_or(EvaluationError::Invalid("response case is unknown"))?;
            if let SavedOutcome::Ok {
                response,
                latency_ms,
            } = &result.outcome
            {
                if *latency_ms > 300_000 {
                    return Err(EvaluationError::Invalid("latency is out of bounds"));
                }
                response
                    .validate_against(&case.request)
                    .map_err(|_| EvaluationError::Invalid("saved response is invalid"))?;
            }
        }
        Ok(())
    }
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
}

/// Completeness counts; failures and missing records never count as incorrect predictions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Completeness {
    /// Number of labeled fixture cases.
    pub total_cases: u64,
    /// Cases with a valid response.
    pub completed_cases: u64,
    /// Cases with a saved provider failure.
    pub failed_cases: u64,
    /// Cases without any saved record.
    pub missing_cases: u64,
    /// Whether every case has a valid response.
    pub complete: bool,
}

/// Accuracy and coverage at one observed confidence cutoff.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AcceptancePoint {
    /// Inclusive minimum confidence.
    pub threshold: f64,
    /// Number of accepted predictions.
    pub accepted_count: u64,
    /// Correct accepted predictions.
    pub accepted_correct: u64,
    /// Correct share among accepted predictions.
    pub accepted_accuracy: f64,
    /// Accepted share of all labeled cases in the group, including failures.
    pub coverage: f64,
    /// Cases not automatically accepted at this cutoff, including failures.
    pub abstention_count: u64,
    /// Abstained share of all labeled cases in the group.
    pub abstention_rate: f64,
    /// Lower 95% Wilson bound for accepted accuracy.
    pub wilson_lower: f64,
    /// Upper 95% Wilson bound for accepted accuracy.
    pub wilson_upper: f64,
}

/// Metrics for one question and optional language/risk slice.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct QuestionMetrics {
    /// Caller-owned question identifier.
    pub question_id: String,
    /// Language filter, or `None` for all languages.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    /// Risk filter, or `None` for all risk levels.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub risk: Option<String>,
    /// Labeled cases containing this question.
    pub total: u64,
    /// Cases with a valid response.
    pub observed: u64,
    /// Exact matches with reference answers.
    pub correct: u64,
    /// Correct share among observed cases; absent when no case was observed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accuracy: Option<f64>,
    /// Reference label to selected label counts.
    pub confusion: BTreeMap<String, BTreeMap<String, u64>>,
    /// False positives by label.
    pub false_positives: BTreeMap<String, u64>,
    /// False negatives by label.
    pub false_negatives: BTreeMap<String, u64>,
    /// Coverage and accepted accuracy at each observed cutoff.
    pub acceptance_curve: Vec<AcceptancePoint>,
}

impl QuestionMetrics {
    fn empty(question_id: &str, language: Option<&str>, risk: Option<&str>) -> Self {
        Self {
            question_id: question_id.to_string(),
            language: language.map(str::to_string),
            risk: risk.map(str::to_string),
            total: 0,
            observed: 0,
            correct: 0,
            accuracy: None,
            confusion: BTreeMap::new(),
            false_positives: BTreeMap::new(),
            false_negatives: BTreeMap::new(),
            acceptance_curve: Vec::new(),
        }
    }
}

/// Deterministic offline evaluation report.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EvaluationReport {
    /// Report schema version.
    pub schema_version: u32,
    /// Fixture schema version.
    pub fixture_schema_version: u32,
    /// Saved response schema version.
    pub response_schema_version: u32,
    /// Dataset identifier.
    pub dataset_id: String,
    /// Provider identifier from saved responses.
    pub provider: String,
    /// Actual model versions observed, sorted and deduplicated.
    pub models: Vec<String>,
    /// Stable FNV-1a hash of case IDs and typed question schemas.
    pub question_schema_hash: String,
    /// Stable FNV-1a hash of all labeled fixture content.
    pub fixture_content_hash: String,
    /// Valid, failed, and missing case counts.
    pub completeness: Completeness,
    /// Per-question and grouped statistics.
    pub metrics: Vec<QuestionMetrics>,
    /// Total input tokens from valid responses.
    pub total_input_tokens: u64,
    /// Total output tokens from valid responses.
    pub total_output_tokens: u64,
    /// Estimated input-token charge, if its price is known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_input_cost_usd: Option<f64>,
    /// Estimated output-token charge, if its price is known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_output_cost_usd: Option<f64>,
    /// Estimated combined charge, only when both prices are known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_cost_usd: Option<f64>,
    /// Median end-to-end latency of valid responses.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_p50_ms: Option<f64>,
    /// Nearest-rank 95th percentile latency of valid responses.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_p95_ms: Option<u64>,
}

impl EvaluationReport {
    /// Find metrics for a question and a specific language/risk slice.
    #[must_use]
    pub fn metric_for(
        &self,
        question: &str,
        language: Option<&str>,
        risk: Option<&str>,
    ) -> Option<&QuestionMetrics> {
        self.metrics.iter().find(|metric| {
            metric.question_id == question
                && metric.language.as_deref() == language
                && metric.risk.as_deref() == risk
        })
    }

    /// Validate a report loaded from a file before calibration or comparison.
    ///
    /// # Errors
    ///
    /// Returns [`EvaluationError::Invalid`] for malformed metadata or statistics.
    pub fn validate(&self) -> Result<(), EvaluationError> {
        if self.schema_version != EVALUATION_SCHEMA_VERSION
            || self.fixture_schema_version != EVALUATION_SCHEMA_VERSION
            || self.response_schema_version != EVALUATION_SCHEMA_VERSION
            || !valid_id(&self.dataset_id)
            || !valid_id(&self.provider)
            || !self.question_schema_hash.starts_with("fnv1a64:")
            || self.question_schema_hash.len() != 24
            || !self.fixture_content_hash.starts_with("fnv1a64:")
            || self.fixture_content_hash.len() != 24
        {
            return Err(EvaluationError::Invalid("report header is invalid"));
        }
        if self
            .completeness
            .completed_cases
            .saturating_add(self.completeness.failed_cases)
            .saturating_add(self.completeness.missing_cases)
            != self.completeness.total_cases
            || self.completeness.complete
                != (self.completeness.completed_cases == self.completeness.total_cases)
        {
            return Err(EvaluationError::Invalid("report completeness is invalid"));
        }
        for metric in &self.metrics {
            if !valid_id(&metric.question_id)
                || metric.total == 0
                || metric.observed > metric.total
                || metric.correct > metric.observed
                || metric.accuracy.is_some_and(|value| !is_probability(value))
                || metric.acceptance_curve.iter().any(|point| {
                    !is_probability(point.threshold)
                        || !is_probability(point.accepted_accuracy)
                        || !is_probability(point.coverage)
                        || !is_probability(point.abstention_rate)
                        || !is_probability(point.wilson_lower)
                        || !is_probability(point.wilson_upper)
                        || point.accepted_count > metric.observed
                        || point.accepted_correct > point.accepted_count
                        || point.abstention_count
                            != metric.total.saturating_sub(point.accepted_count)
                })
            {
                return Err(EvaluationError::Invalid("report metric is invalid"));
            }
        }
        Ok(())
    }
}

/// Whether calibration may recommend a threshold from the available evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CalibrationStatus {
    /// A threshold satisfies the target at the lower confidence bound.
    Recommended,
    /// Too few examples to make an automatic-acceptance recommendation.
    InsufficientEvidence,
    /// Enough examples, but no threshold satisfies the target.
    TargetUnmet,
    /// Missing or failed case results preclude calibration.
    Incomplete,
}

/// Per-question calibration suggestion, never applied to production configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CalibrationResult {
    /// Status explaining whether a recommendation is available.
    pub status: CalibrationStatus,
    /// Question being calibrated.
    pub question_id: String,
    /// Requested lower-bound accuracy target.
    pub target_accuracy: f64,
    /// Labeled cases in the selected group.
    pub sample_size: u64,
    /// Suggested inclusive cutoff, if supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold: Option<f64>,
    /// Accepted cases under the suggested cutoff.
    pub accepted_count: u64,
    /// Coverage among all labeled cases.
    pub coverage: f64,
    /// Correct share among accepted cases.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accepted_accuracy: Option<f64>,
    /// Lower Wilson 95% bound, if a threshold was selected.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wilson_lower: Option<f64>,
    /// Upper Wilson 95% bound, if a threshold was selected.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wilson_upper: Option<f64>,
}

/// One changed metric in a comparable evaluation pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MetricRegression {
    /// Question with a regression.
    pub question_id: String,
    /// Optional language slice.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    /// Optional risk slice.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub risk: Option<String>,
    /// Metric name, such as `accuracy` or `false_negatives:3`.
    pub metric: String,
    /// Baseline value.
    pub baseline: f64,
    /// Candidate value.
    pub candidate: f64,
}

/// Read-only comparison result; regressions do not change the process exit code.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ComparisonReport {
    /// Whether dataset and question schema are compatible.
    pub comparable: bool,
    /// Metadata changes and incompatibility reasons.
    pub changes: Vec<String>,
    /// Regressed metrics on comparable reports.
    pub regressions: Vec<MetricRegression>,
}

fn is_probability(value: f64) -> bool {
    value.is_finite() && (0.0..=1.0).contains(&value)
}

type MetricKey = (String, Option<String>, Option<String>);

#[derive(Debug)]
struct MetricAccumulator {
    metric: QuestionMetrics,
    observations: Vec<(f64, bool)>,
}

/// Evaluate saved responses without network access or provider credentials.
///
/// # Errors
///
/// Returns [`EvaluationError::Invalid`] if either artifact is invalid.
#[allow(
    clippy::too_many_lines,
    reason = "Single pass keeps case and question accounting together"
)]
pub fn evaluate(
    fixture: &FixtureSuite,
    saved: &SavedResponses,
) -> Result<EvaluationReport, EvaluationError> {
    fixture.validate()?;
    saved.validate_against(fixture)?;
    let by_case: BTreeMap<&str, &SavedOutcome> = saved
        .results
        .iter()
        .map(|result| (result.case_id.as_str(), &result.outcome))
        .collect();
    let mut metrics: BTreeMap<MetricKey, MetricAccumulator> = BTreeMap::new();
    let mut models = BTreeSet::new();
    let mut latencies = Vec::new();
    let mut completed = 0_u64;
    let mut failed = 0_u64;
    let mut missing = 0_u64;
    let mut input_tokens = 0_u64;
    let mut output_tokens = 0_u64;

    for case in &fixture.cases {
        let outcome = by_case.get(case.id.as_str()).copied();
        let response = match outcome {
            Some(SavedOutcome::Ok {
                response,
                latency_ms,
            }) => {
                completed = completed.saturating_add(1);
                models.insert(response.model.clone());
                latencies.push(*latency_ms);
                input_tokens = input_tokens.saturating_add(response.usage.input_tokens);
                output_tokens = output_tokens.saturating_add(response.usage.output_tokens);
                Some(response)
            }
            Some(SavedOutcome::Error { .. }) => {
                failed = failed.saturating_add(1);
                None
            }
            None => {
                missing = missing.saturating_add(1);
                None
            }
        };
        for (id, expected) in &case.expected {
            let observed = response.and_then(|value| value.answers.get(id));
            let question = case
                .request
                .questions
                .get(id)
                .ok_or(EvaluationError::Invalid("question missing"))?;
            let truth = expected_label(expected);
            let prediction = observed
                .map(|answer| predicted_label(question, answer))
                .transpose()?;
            for (language, risk) in [
                (None, None),
                (Some(case.language.as_str()), None),
                (None, Some(case.risk.as_str())),
                (Some(case.language.as_str()), Some(case.risk.as_str())),
            ] {
                let key = (
                    id.clone(),
                    language.map(str::to_string),
                    risk.map(str::to_string),
                );
                let entry = metrics.entry(key).or_insert_with(|| MetricAccumulator {
                    metric: QuestionMetrics::empty(id, language, risk),
                    observations: Vec::new(),
                });
                entry.metric.total = entry.metric.total.saturating_add(1);
                if let Some((label, confidence)) = &prediction {
                    entry.metric.observed = entry.metric.observed.saturating_add(1);
                    let correct = label == &truth;
                    if correct {
                        entry.metric.correct = entry.metric.correct.saturating_add(1);
                    }
                    *entry
                        .metric
                        .confusion
                        .entry(truth.clone())
                        .or_default()
                        .entry(label.clone())
                        .or_default() += 1;
                    entry.observations.push((*confidence, correct));
                }
            }
        }
    }

    let metrics = metrics.into_values().map(finalize_metric).collect();
    latencies.sort_unstable();
    let latency_p50_ms = if latencies.is_empty() {
        None
    } else {
        let middle = latencies.len() / 2;
        if latencies.len() % 2 == 0 {
            let left = latencies
                .get(middle.saturating_sub(1))
                .copied()
                .unwrap_or(0);
            let right = latencies.get(middle).copied().unwrap_or(0);
            Some(f64::midpoint(to_f64(left), to_f64(right)))
        } else {
            latencies.get(middle).copied().map(to_f64)
        }
    };
    let latency_p95_ms = if latencies.is_empty() {
        None
    } else {
        let rank = 95_usize
            .saturating_mul(latencies.len())
            .div_ceil(100)
            .saturating_sub(1);
        latencies.get(rank).copied()
    };
    let total_cases = u64::try_from(fixture.cases.len()).unwrap_or(u64::MAX);
    let report = EvaluationReport {
        schema_version: EVALUATION_SCHEMA_VERSION,
        fixture_schema_version: fixture.schema_version,
        response_schema_version: saved.schema_version,
        dataset_id: fixture.dataset_id.clone(),
        provider: saved.provider.clone(),
        models: models.into_iter().collect(),
        question_schema_hash: question_schema_hash(fixture)?,
        fixture_content_hash: content_hash(fixture)?,
        completeness: Completeness {
            total_cases,
            completed_cases: completed,
            failed_cases: failed,
            missing_cases: missing,
            complete: completed == total_cases,
        },
        metrics,
        total_input_tokens: input_tokens,
        total_output_tokens: output_tokens,
        total_input_cost_usd: saved
            .input_price_per_million_usd
            .map(|price| price * to_f64(input_tokens) / 1_000_000.0),
        total_output_cost_usd: saved
            .output_price_per_million_usd
            .map(|price| price * to_f64(output_tokens) / 1_000_000.0),
        total_cost_usd: saved
            .input_price_per_million_usd
            .zip(saved.output_price_per_million_usd)
            .map(|(input_price, output_price)| {
                (input_price * to_f64(input_tokens) + output_price * to_f64(output_tokens))
                    / 1_000_000.0
            }),
        latency_p50_ms,
        latency_p95_ms,
    };
    report.validate()?;
    Ok(report)
}

fn expected_label(answer: &ExpectedAnswer) -> String {
    match answer {
        ExpectedAnswer::Noul { yes } => if *yes { "yes" } else { "no" }.to_string(),
        ExpectedAnswer::Choice { choice } => choice.clone(),
        ExpectedAnswer::Score { level } => level.to_string(),
    }
}

fn predicted_label(
    question: &Question,
    answer: &TypedAnswer,
) -> Result<(String, f64), EvaluationError> {
    match (question, answer) {
        (Question::Noul { .. }, TypedAnswer::Noul { noul }) => Ok((
            if *noul >= 0.5 { "yes" } else { "no" }.to_string(),
            noul.max(1.0 - noul),
        )),
        (
            Question::Choice { .. },
            TypedAnswer::Choice {
                choice, confidence, ..
            },
        ) => Ok((choice.clone(), *confidence)),
        (
            Question::Score { criteria, .. },
            TypedAnswer::Score {
                score, confidence, ..
            },
        ) => {
            let index = (0..criteria.len())
                .find(|level| *score < f64::from(u32::try_from(*level).unwrap_or(u32::MAX)) + 0.5)
                .unwrap_or_else(|| criteria.len().saturating_sub(1));
            Ok((index.to_string(), *confidence))
        }
        _ => Err(EvaluationError::Invalid("answer type is incompatible")),
    }
}

fn finalize_metric(mut accumulator: MetricAccumulator) -> QuestionMetrics {
    let metric = &mut accumulator.metric;
    if metric.observed > 0 {
        metric.accuracy = Some(to_f64(metric.correct) / to_f64(metric.observed));
    }
    let labels: BTreeSet<String> = metric
        .confusion
        .iter()
        .flat_map(|(truth, predictions)| {
            std::iter::once(truth.clone()).chain(predictions.keys().cloned())
        })
        .collect();
    for label in labels {
        let true_count = metric
            .confusion
            .get(&label)
            .map_or(0, |row| row.values().sum::<u64>());
        let predicted_count = metric
            .confusion
            .values()
            .map(|row| row.get(&label).copied().unwrap_or(0))
            .sum::<u64>();
        let true_positive = metric
            .confusion
            .get(&label)
            .and_then(|row| row.get(&label))
            .copied()
            .unwrap_or(0);
        metric
            .false_negatives
            .insert(label.clone(), true_count.saturating_sub(true_positive));
        metric
            .false_positives
            .insert(label, predicted_count.saturating_sub(true_positive));
    }
    accumulator
        .observations
        .sort_by(|left, right| right.0.total_cmp(&left.0));
    let mut accepted = 0_u64;
    let mut correct = 0_u64;
    for (index, (confidence, is_correct)) in accumulator.observations.iter().enumerate() {
        accepted = accepted.saturating_add(1);
        if *is_correct {
            correct = correct.saturating_add(1);
        }
        let next_confidence = accumulator.observations.get(index + 1).map(|next| next.0);
        if next_confidence.is_none_or(|next| next.total_cmp(confidence).is_ne()) {
            let (lower, upper) = wilson_interval(correct, accepted);
            metric.acceptance_curve.push(AcceptancePoint {
                threshold: *confidence,
                accepted_count: accepted,
                accepted_correct: correct,
                accepted_accuracy: to_f64(correct) / to_f64(accepted),
                coverage: to_f64(accepted) / to_f64(metric.total),
                abstention_count: metric.total.saturating_sub(accepted),
                abstention_rate: to_f64(metric.total.saturating_sub(accepted))
                    / to_f64(metric.total),
                wilson_lower: lower,
                wilson_upper: upper,
            });
        }
    }
    accumulator.metric
}

fn wilson_interval(correct: u64, total: u64) -> (f64, f64) {
    if total == 0 {
        return (0.0, 1.0);
    }
    let n = to_f64(total);
    let proportion = to_f64(correct) / n;
    let z = 1.96_f64;
    let z_squared = z * z;
    let denominator = 1.0 + z_squared / n;
    let center = (proportion + z_squared / (2.0 * n)) / denominator;
    let margin =
        z * ((proportion * (1.0 - proportion) + z_squared / (4.0 * n)) / n).sqrt() / denominator;
    ((center - margin).max(0.0), (center + margin).min(1.0))
}

fn question_schema_hash(fixture: &FixtureSuite) -> Result<String, EvaluationError> {
    let mut schemas: Vec<(&str, &BTreeMap<String, Question>)> = fixture
        .cases
        .iter()
        .map(|case| (case.id.as_str(), &case.request.questions))
        .collect();
    schemas.sort_by_key(|(id, _)| *id);
    let bytes = serde_json::to_vec(&schemas)
        .map_err(|_| EvaluationError::Invalid("question schema cannot be serialized"))?;
    Ok(fnv1a64(&bytes))
}

fn content_hash(fixture: &FixtureSuite) -> Result<String, EvaluationError> {
    let mut cases = fixture.cases.clone();
    cases.sort_by(|left, right| left.id.cmp(&right.id));
    let bytes = serde_json::to_vec(&cases)
        .map_err(|_| EvaluationError::Invalid("fixture cannot be serialized"))?;
    Ok(fnv1a64(&bytes))
}

fn fnv1a64(bytes: &[u8]) -> String {
    let hash = bytes.iter().fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    });
    format!("fnv1a64:{hash:016x}")
}

#[allow(
    clippy::cast_precision_loss,
    reason = "Evaluation counts are bounded to 500 cases; token estimates tolerate rounding"
)]
fn to_f64(value: u64) -> f64 {
    value as f64
}

/// Select a question-specific threshold only when the 95% lower confidence
/// bound meets the requested accepted-accuracy target on enough examples.
///
/// This returns advice; it never edits runtime configuration.
///
/// # Errors
///
/// Returns [`EvaluationError`] for an invalid report, target, or group.
pub fn calibrate(
    report: &EvaluationReport,
    question: &str,
    language: Option<&str>,
    risk: Option<&str>,
    target_accuracy: f64,
) -> Result<CalibrationResult, EvaluationError> {
    report.validate()?;
    if !target_accuracy.is_finite() || !(0.0..=1.0).contains(&target_accuracy) {
        return Err(EvaluationError::Invalid("target accuracy is invalid"));
    }
    let metric = report
        .metric_for(question, language, risk)
        .ok_or(EvaluationError::NotFound)?;
    let mut result = CalibrationResult {
        status: CalibrationStatus::InsufficientEvidence,
        question_id: question.to_string(),
        target_accuracy,
        sample_size: metric.total,
        threshold: None,
        accepted_count: 0,
        coverage: 0.0,
        accepted_accuracy: None,
        wilson_lower: None,
        wilson_upper: None,
    };
    if !report.completeness.complete || metric.observed != metric.total {
        result.status = CalibrationStatus::Incomplete;
        return Ok(result);
    }
    if metric.total < MIN_CALIBRATION_CASES {
        return Ok(result);
    }
    let enough: Vec<&AcceptancePoint> = metric
        .acceptance_curve
        .iter()
        .filter(|point| point.accepted_count >= MIN_CALIBRATION_CASES)
        .collect();
    if enough.is_empty() {
        return Ok(result);
    }
    if let Some(point) = enough
        .into_iter()
        .filter(|point| point.wilson_lower >= target_accuracy)
        .max_by_key(|point| point.accepted_count)
    {
        result.status = CalibrationStatus::Recommended;
        result.threshold = Some(point.threshold);
        result.accepted_count = point.accepted_count;
        result.coverage = point.coverage;
        result.accepted_accuracy = Some(point.accepted_accuracy);
        result.wilson_lower = Some(point.wilson_lower);
        result.wilson_upper = Some(point.wilson_upper);
    } else {
        result.status = CalibrationStatus::TargetUnmet;
    }
    Ok(result)
}

/// Compare two saved reports without turning a regression into a CI gate.
///
/// Dataset or question-schema changes make reports incomparable. Model and
/// provider changes are recorded while comparable metric regressions remain
/// visible to a human caller.
///
/// # Errors
///
/// Returns [`EvaluationError::Invalid`] for malformed reports.
pub fn compare(
    baseline: &EvaluationReport,
    candidate: &EvaluationReport,
) -> Result<ComparisonReport, EvaluationError> {
    baseline.validate()?;
    candidate.validate()?;
    let mut changes = Vec::new();
    let mut comparable = true;
    if baseline.dataset_id != candidate.dataset_id {
        changes.push("dataset changed".to_string());
        comparable = false;
    }
    if baseline.question_schema_hash != candidate.question_schema_hash {
        changes.push("question schema changed".to_string());
        comparable = false;
    }
    if baseline.fixture_content_hash != candidate.fixture_content_hash {
        changes.push("fixture content changed".to_string());
        comparable = false;
    }
    if baseline.provider != candidate.provider {
        changes.push("provider changed".to_string());
    }
    if baseline.models != candidate.models {
        changes.push("model version changed".to_string());
    }
    if !baseline.completeness.complete || !candidate.completeness.complete {
        changes.push("one or both reports are incomplete".to_string());
        comparable = false;
    }
    let mut regressions = Vec::new();
    if comparable {
        for old in &baseline.metrics {
            let Some(new) = candidate.metric_for(
                &old.question_id,
                old.language.as_deref(),
                old.risk.as_deref(),
            ) else {
                changes.push("metric group changed".to_string());
                comparable = false;
                break;
            };
            if old.total != new.total {
                changes.push("metric sample count changed".to_string());
                comparable = false;
                break;
            }
            if let (Some(old_accuracy), Some(new_accuracy)) = (old.accuracy, new.accuracy)
                && new_accuracy < old_accuracy
            {
                regressions.push(MetricRegression {
                    question_id: old.question_id.clone(),
                    language: old.language.clone(),
                    risk: old.risk.clone(),
                    metric: "accuracy".to_string(),
                    baseline: old_accuracy,
                    candidate: new_accuracy,
                });
            }
            for point in &old.acceptance_curve {
                let candidate_accepted = new
                    .acceptance_curve
                    .iter()
                    .filter(|candidate_point| candidate_point.threshold >= point.threshold)
                    .map(|candidate_point| candidate_point.accepted_count)
                    .max()
                    .unwrap_or(0);
                let candidate_coverage = to_f64(candidate_accepted) / to_f64(new.total);
                if candidate_coverage < point.coverage {
                    regressions.push(MetricRegression {
                        question_id: old.question_id.clone(),
                        language: old.language.clone(),
                        risk: old.risk.clone(),
                        metric: format!("coverage_at_threshold:{}", point.threshold),
                        baseline: point.coverage,
                        candidate: candidate_coverage,
                    });
                }
            }
            for (label, old_count) in &old.false_negatives {
                let new_count = new.false_negatives.get(label).copied().unwrap_or(0);
                if new_count > *old_count {
                    regressions.push(MetricRegression {
                        question_id: old.question_id.clone(),
                        language: old.language.clone(),
                        risk: old.risk.clone(),
                        metric: format!("false_negatives:{label}"),
                        baseline: to_f64(*old_count),
                        candidate: to_f64(new_count),
                    });
                }
            }
        }
    }
    if !comparable {
        regressions.clear();
    }
    Ok(ComparisonReport {
        comparable,
        changes,
        regressions,
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{CalibrationStatus, FixtureSuite, SavedResponses, calibrate, compare, evaluate};

    fn fixture() -> FixtureSuite {
        serde_json::from_value(json!({
            "schemaVersion": 1,
            "datasetId": "synthetic_mixed_v1",
            "cases": [{
                "id": "case_1", "language": "english", "risk": "high", "source": "synthetic",
                "request": {
                    "state": {"title": "Existing command fails"},
                    "questions": {
                        "broken": {"type": "noul", "instructions": "Is the existing behavior broken?"},
                        "kind": {"type": "choice", "instructions": "Classify the issue", "criteria": {"bug": "Broken behavior", "feature": "New behavior"}},
                        "severity": {"type": "score", "instructions": "Rate impact", "criteria": ["Low impact", "Medium impact", "High impact"]}
                    }
                },
                "expected": {
                    "broken": {"type": "noul", "yes": true},
                    "kind": {"type": "choice", "choice": "bug"},
                    "severity": {"type": "score", "level": 2}
                }
            }]
        })).unwrap()
    }

    fn responses() -> SavedResponses {
        serde_json::from_value(json!({
            "schemaVersion": 1, "datasetId": "synthetic_mixed_v1",
            "provider": "saved_fixture", "inputPricePerMillionUsd": 0.042, "outputPricePerMillionUsd": 0.3,
            "results": [{"caseId": "case_1", "outcome": {
                "status": "ok", "latencyMs": 120,
                "response": {
                    "model": "fixture-model-1",
                    "answers": {
                        "broken": {"type": "noul", "noul": 0.9},
                        "kind": {"type": "choice", "choice": "bug", "confidence": 0.9, "probabilities": {"bug": 0.9, "feature": 0.1}},
                        "severity": {"type": "score", "score": 1.8, "confidence": 0.8, "legend": {"0": "Low impact", "1": "Medium impact", "2": "High impact"}, "probabilities": {"0": 0.0, "1": 0.2, "2": 0.8}}
                    },
                    "usage": {"input_tokens": 100, "output_tokens": 20}
                }
            }}]
        })).unwrap()
    }

    #[test]
    fn test_should_evaluate_mixed_questions_offline() {
        let report = evaluate(&fixture(), &responses()).unwrap();
        assert_eq!(report.completeness.completed_cases, 1);
        assert_eq!(report.models, vec!["fixture-model-1"]);
        assert_eq!(report.total_input_tokens, 100);
        assert!((report.total_cost_usd.unwrap() - 0.000_010_2).abs() < 1e-12);
        for question in ["broken", "kind", "severity"] {
            let metric = report.metric_for(question, None, None).unwrap();
            assert_eq!(metric.correct, 1);
            assert_eq!(metric.observed, 1);
        }
        assert!(
            report
                .metric_for("kind", Some("english"), Some("high"))
                .is_some()
        );
    }

    #[test]
    fn test_should_reject_sensitive_fixture() {
        let mut suite = fixture();
        suite.cases[0].request.state =
            json!({"title": "See https://internal.example.local/private"});
        assert!(suite.validate().is_err());
        suite.cases[0].request.state = json!({"title": format!("{}{}", "apikey_", "x".repeat(40))});
        assert!(suite.validate().is_err());
        suite = fixture();
        if let crate::decision::Question::Noul { instructions } =
            suite.cases[0].request.questions.get_mut("broken").unwrap()
        {
            *instructions = format!("Check {}{}", "apikey_", "x".repeat(40));
        }
        assert!(suite.validate().is_err());
    }

    #[test]
    fn test_should_mark_missing_response_incomplete() {
        let mut saved = responses();
        saved.results.clear();
        let report = evaluate(&fixture(), &saved).unwrap();
        assert_eq!(report.completeness.missing_cases, 1);
        assert!(!report.completeness.complete);
        assert_eq!(report.metric_for("kind", None, None).unwrap().observed, 0);
    }

    #[test]
    fn test_should_refuse_calibration_on_small_sample() {
        let report = evaluate(&fixture(), &responses()).unwrap();
        let result = calibrate(&report, "kind", None, None, 0.8).unwrap();
        assert_eq!(result.status, CalibrationStatus::InsufficientEvidence);
        assert!(result.threshold.is_none());
    }

    #[test]
    fn test_should_recommend_only_when_wilson_bound_meets_target() {
        let mut suite = fixture();
        let mut saved = responses();
        for index in 2..=40 {
            let id = format!("case_{index}");
            let mut case = suite.cases[0].clone();
            case.id.clone_from(&id);
            suite.cases.push(case);
            let mut result = saved.results[0].clone();
            result.case_id = id;
            saved.results.push(result);
        }
        let report = evaluate(&suite, &saved).unwrap();
        let recommended = calibrate(&report, "kind", None, None, 0.90).unwrap();
        assert_eq!(recommended.status, CalibrationStatus::Recommended);
        assert_eq!(recommended.sample_size, 40);
        assert_eq!(recommended.accepted_count, 40);
        assert!((recommended.coverage - 1.0).abs() < f64::EPSILON);
        assert!(recommended.wilson_lower.unwrap() >= 0.90);
        let unmet = calibrate(&report, "kind", None, None, 0.99).unwrap();
        assert_eq!(unmet.status, CalibrationStatus::TargetUnmet);
    }

    #[test]
    fn test_should_detect_schema_change_and_metric_regression() {
        let baseline = evaluate(&fixture(), &responses()).unwrap();
        let mut candidate = baseline.clone();
        candidate.question_schema_hash = "fnv1a64:0000000000000000".into();
        assert!(!compare(&baseline, &candidate).unwrap().comparable);
        candidate.question_schema_hash = baseline.question_schema_hash.clone();
        candidate.metrics[0].correct = 0;
        candidate.metrics[0].accuracy = Some(0.0);
        assert!(
            !compare(&baseline, &candidate)
                .unwrap()
                .regressions
                .is_empty()
        );
    }

    #[test]
    fn test_should_detect_fixture_content_change_with_same_dataset_id() {
        let baseline = evaluate(&fixture(), &responses()).unwrap();
        let mut changed = fixture();
        changed.cases[0].request.state = json!({"title": "A different public issue"});
        let candidate = evaluate(&changed, &responses()).unwrap();
        let comparison = compare(&baseline, &candidate).unwrap();
        assert!(!comparison.comparable);
        assert!(
            comparison
                .changes
                .iter()
                .any(|change| change == "fixture content changed")
        );
    }

    #[test]
    fn test_should_detect_coverage_regression_at_baseline_threshold() {
        let baseline = evaluate(&fixture(), &responses()).unwrap();
        let mut changed = responses();
        if let super::SavedOutcome::Ok { response, .. } = &mut changed.results[0].outcome
            && let crate::decision::TypedAnswer::Choice {
                confidence,
                probabilities,
                ..
            } = response.answers.get_mut("kind").unwrap()
        {
            *confidence = 0.6;
            probabilities.insert("bug".into(), 0.6);
            probabilities.insert("feature".into(), 0.4);
        }
        let candidate = evaluate(&fixture(), &changed).unwrap();
        let comparison = compare(&baseline, &candidate).unwrap();
        assert!(comparison.comparable);
        assert!(
            comparison
                .regressions
                .iter()
                .any(|regression| regression.question_id == "kind"
                    && regression.metric.starts_with("coverage_at_threshold:"))
        );
    }
}
