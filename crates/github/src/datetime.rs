//! Shared parsing for timestamps returned by GitHub APIs and `gh`.

/// Parse an RFC 3339 timestamp, warning and using the Unix epoch if malformed.
pub(crate) fn parse_api_datetime(value: &str) -> chrono::DateTime<chrono::Utc> {
    value.parse().unwrap_or_else(|_| {
        tracing::warn!(value, "Failed to parse GitHub API timestamp, using epoch");
        chrono::DateTime::UNIX_EPOCH
    })
}
