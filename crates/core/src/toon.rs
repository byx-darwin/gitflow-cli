//! TOON (Token-Oriented Object Notation) output support.
//!
//! Provides structure analysis, strategy selection, and TOON encoding
//! for token-efficient output. Uses the `toon-format` crate for encoding.

use serde_json::Value;

/// The top-level type of a JSON value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TopLevel {
    /// A JSON object (`{...}`).
    Object,
    /// A JSON array (`[...]`).
    Array,
    /// A JSON scalar (string, number, bool, null).
    Scalar,
}

/// Structural characteristics of a JSON value.
///
/// Produced by [`analyze`] and consumed by [`select_strategy`] to decide
/// whether TOON or JSON encoding is more token-efficient.
#[derive(Debug, Clone)]
pub struct JsonShape {
    /// The top-level JSON type.
    pub top_level: TopLevel,
    /// Number of items if top-level is an array, 0 otherwise.
    pub item_count: usize,
    /// `true` if top-level is an array and all items are objects with identical key sets.
    pub is_uniform_array: bool,
    /// Maximum nesting depth (0 for flat values).
    pub max_depth: usize,
    /// Total character count of the pretty-printed JSON representation.
    pub char_count: usize,
}

/// Strategy selected by [`select_strategy`] for encoding output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToonStrategy {
    /// Encode as TOON (uniform arrays or deeply nested structures).
    Toon,
    /// Keep as JSON (small data or structures not suited for TOON).
    Json,
}

/// Error type for TOON encoding failures.
#[derive(Debug, Clone)]
pub struct ToonError {
    /// Human-readable error message.
    pub message: String,
}

impl std::fmt::Display for ToonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TOON encoding error: {}", self.message)
    }
}

impl std::error::Error for ToonError {}

/// Analyze the structure of a JSON value.
///
/// Performs a single-pass O(n) traversal to collect structural metrics
/// used by [`select_strategy`].
#[must_use]
pub fn analyze(value: &Value) -> JsonShape {
    let char_count = serde_json::to_string_pretty(value).map_or(0, |s| s.len());

    let top_level = match value {
        Value::Object(_) => TopLevel::Object,
        Value::Array(_) => TopLevel::Array,
        _ => TopLevel::Scalar,
    };

    let (item_count, is_uniform_array) = match value {
        Value::Array(arr) => {
            let count = arr.len();
            let uniform = count > 0
                && arr.iter().all(|item| {
                    item.as_object().is_some_and(|obj| {
                        let keys: Vec<&String> = {
                            let mut ks: Vec<_> = obj.keys().collect();
                            ks.sort_unstable();
                            ks
                        };
                        arr.first().is_some_and(|first| {
                            let first_keys: Vec<_> = first
                                .as_object()
                                .map(|o| {
                                    let mut ks: Vec<_> = o.keys().collect();
                                    ks.sort_unstable();
                                    ks
                                })
                                .unwrap_or_default();
                            keys == first_keys
                        })
                    })
                });
            (count, uniform)
        }
        _ => (0, false),
    };

    let max_depth = compute_depth(value, 0);

    JsonShape {
        top_level,
        item_count,
        is_uniform_array,
        max_depth,
        char_count,
    }
}

/// Recursively compute the maximum nesting depth.
fn compute_depth(value: &Value, current: usize) -> usize {
    match value {
        Value::Object(obj) => obj
            .values()
            .map(|v| compute_depth(v, current + 1))
            .max()
            .unwrap_or(current),
        Value::Array(arr) => arr
            .iter()
            .map(|v| compute_depth(v, current + 1))
            .max()
            .unwrap_or(current),
        _ => current,
    }
}

/// Select the optimal encoding strategy based on structural analysis.
///
/// Rules (applied in order):
/// 1. `char_count < 200` → [`ToonStrategy::Json`] (not worth converting)
/// 2. Uniform array with `item_count >= 5` → [`ToonStrategy::Toon`]
/// 3. `max_depth > 3` → [`ToonStrategy::Toon`]
/// 4. Otherwise → [`ToonStrategy::Json`]
#[must_use]
pub fn select_strategy(shape: &JsonShape) -> ToonStrategy {
    if shape.char_count < 200 {
        return ToonStrategy::Json;
    }
    if shape.top_level == TopLevel::Array && shape.is_uniform_array && shape.item_count >= 5 {
        return ToonStrategy::Toon;
    }
    if shape.max_depth > 3 {
        return ToonStrategy::Toon;
    }
    ToonStrategy::Json
}

/// Encode a JSON value as TOON format.
///
/// # Errors
///
/// Returns [`ToonError`] if the `toon-format` crate fails to encode.
pub fn encode(value: &Value) -> Result<String, ToonError> {
    toon_format::encode_default(value).map_err(|e| ToonError {
        message: e.to_string(),
    })
}

#[cfg(test)]
#[allow(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "Tests legitimately unwrap known-good fixture data"
)]
mod tests {
    use serde_json::json;

    use super::*;

    // ── analyze tests ──────────────────────────────────────────────

    #[test]
    fn test_analyze_uniform_array() {
        let value = json!([
            {"id": 1, "name": "a"},
            {"id": 2, "name": "b"},
            {"id": 3, "name": "c"},
            {"id": 4, "name": "d"},
            {"id": 5, "name": "e"}
        ]);
        let shape = analyze(&value);
        assert_eq!(shape.top_level, TopLevel::Array);
        assert_eq!(shape.item_count, 5);
        assert!(shape.is_uniform_array);
    }

    #[test]
    fn test_analyze_mixed_array() {
        let value = json!([
            {"a": 1, "b": 2},
            {"a": 3, "c": 4}
        ]);
        let shape = analyze(&value);
        assert_eq!(shape.top_level, TopLevel::Array);
        assert_eq!(shape.item_count, 2);
        assert!(!shape.is_uniform_array);
    }

    #[test]
    fn test_analyze_deep_nesting() {
        let value = json!({"a": {"b": {"c": {"d": {"e": 1}}}}});
        let shape = analyze(&value);
        assert_eq!(shape.top_level, TopLevel::Object);
        assert_eq!(shape.max_depth, 5);
    }

    #[test]
    fn test_analyze_small_data() {
        let value = json!({"x": 1});
        let shape = analyze(&value);
        assert!(shape.char_count < 200);
    }

    #[test]
    fn test_analyze_scalar() {
        let value = json!(42);
        let shape = analyze(&value);
        assert_eq!(shape.top_level, TopLevel::Scalar);
        assert_eq!(shape.item_count, 0);
        assert!(!shape.is_uniform_array);
    }

    // ── select_strategy tests ──────────────────────────────────────

    #[test]
    fn test_strategy_small_data_returns_json() {
        let shape = JsonShape {
            top_level: TopLevel::Array,
            item_count: 10,
            is_uniform_array: true,
            max_depth: 1,
            char_count: 100,
        };
        assert_eq!(select_strategy(&shape), ToonStrategy::Json);
    }

    #[test]
    fn test_strategy_uniform_array_returns_toon() {
        let shape = JsonShape {
            top_level: TopLevel::Array,
            item_count: 10,
            is_uniform_array: true,
            max_depth: 1,
            char_count: 500,
        };
        assert_eq!(select_strategy(&shape), ToonStrategy::Toon);
    }

    #[test]
    fn test_strategy_deep_nesting_returns_toon() {
        let shape = JsonShape {
            top_level: TopLevel::Object,
            item_count: 0,
            is_uniform_array: false,
            max_depth: 5,
            char_count: 500,
        };
        assert_eq!(select_strategy(&shape), ToonStrategy::Toon);
    }

    #[test]
    fn test_strategy_default_returns_json() {
        let shape = JsonShape {
            top_level: TopLevel::Object,
            item_count: 0,
            is_uniform_array: false,
            max_depth: 2,
            char_count: 500,
        };
        assert_eq!(select_strategy(&shape), ToonStrategy::Json);
    }

    // ── encode tests ───────────────────────────────────────────────

    #[test]
    fn test_toon_encode_simple_object() {
        let value = json!({"name": "Alice", "age": 30});
        let result = encode(&value);
        assert!(result.is_ok());
        let encoded = result.unwrap();
        assert!(encoded.contains("name"));
        assert!(encoded.contains("Alice"));
    }

    #[test]
    fn test_toon_encode_array() {
        let value = json!([
            {"id": 1, "name": "a"},
            {"id": 2, "name": "b"},
            {"id": 3, "name": "c"},
            {"id": 4, "name": "d"},
            {"id": 5, "name": "e"}
        ]);
        let result = encode(&value);
        assert!(result.is_ok());
    }
}
