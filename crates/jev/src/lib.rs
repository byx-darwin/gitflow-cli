//! Optional `TypeSafe` Jev adapter for the provider-neutral decision contract.

#![forbid(unsafe_code)]
#![cfg_attr(
    test,
    allow(clippy::unwrap_used, reason = "Test fixture setup may panic")
)]

use std::time::Duration;

use async_trait::async_trait;
use gitflow_core::decision::{DecisionEngine, DecisionError, DecisionRequest, DecisionResponse};
use secrecy::{ExposeSecret, SecretString};

const ENDPOINT: &str = "https://api.typesafe.ai/v1/systemone";
const MAX_RESPONSE_BYTES: usize = 131_072;
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

/// Bounded configuration for a Jev request.
pub struct JevEngine {
    client: reqwest::Client,
    key: Option<SecretString>,
    model: String,
    endpoint: String,
}

impl std::fmt::Debug for JevEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JevEngine")
            .field("key", &self.key.as_ref().map(|_| "[REDACTED]"))
            .field("model", &self.model)
            .finish_non_exhaustive()
    }
}

impl JevEngine {
    /// Create an engine using `TYPESAFE_API_KEY` (or the macOS Keychain)
    /// and `GF_JEV_MODEL`.
    ///
    /// # Errors
    ///
    /// Returns a content-free error if timeout configuration or the HTTP
    /// client cannot be initialized.
    pub fn from_env() -> Result<Self, DecisionError> {
        let key = select_key(std::env::var("TYPESAFE_API_KEY").ok(), keychain_key);
        let model = std::env::var("GF_JEV_MODEL").unwrap_or_else(|_| "jev-latest".to_string());
        if model.is_empty()
            || model.len() > 64
            || !model
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
        {
            return Err(DecisionError::InvalidInput("model identifier is invalid"));
        }
        let timeout_ms = std::env::var("GF_JEV_TIMEOUT_MS")
            .ok()
            .map_or(Ok(DEFAULT_TIMEOUT.as_secs() * 1_000), |value| {
                value.parse::<u64>()
            })
            .map_err(|_| DecisionError::InvalidInput("timeout is invalid"))?;
        if !(100..=30_000).contains(&timeout_ms) {
            return Err(DecisionError::InvalidInput("timeout is out of bounds"));
        }
        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(timeout_ms))
            .connect_timeout(Duration::from_secs(3))
            .no_proxy()
            .build()
            .map_err(|_| DecisionError::Transport)?;
        Ok(Self {
            client,
            key,
            model,
            endpoint: ENDPOINT.to_string(),
        })
    }

    #[cfg(test)]
    fn with_key_for_test(key: Option<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            key: key.map(SecretString::from),
            model: "jev-latest".into(),
            endpoint: ENDPOINT.to_string(),
        }
    }
}

fn select_key(
    env_key: Option<String>,
    keychain: impl FnOnce() -> Option<String>,
) -> Option<SecretString> {
    env_key
        .filter(|value| !value.trim().is_empty())
        .or_else(keychain)
        .filter(|value| !value.trim().is_empty())
        .map(SecretString::from)
}

#[cfg(target_os = "macos")]
#[allow(
    clippy::disallowed_types,
    reason = "Synchronous, bounded Keychain lookup during adapter configuration"
)]
fn keychain_key() -> Option<String> {
    let user = std::env::var("USER").ok()?;
    if user.is_empty() {
        return None;
    }
    let output = std::process::Command::new("/usr/bin/security")
        .args([
            "find-generic-password",
            "-a",
            &user,
            "-s",
            "gitflow-cli-typesafe",
            "-w",
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let mut key = String::from_utf8(output.stdout).ok()?;
    while key.ends_with('\n') || key.ends_with('\r') {
        key.pop();
    }
    (!key.trim().is_empty()).then_some(key)
}

#[cfg(not(target_os = "macos"))]
fn keychain_key() -> Option<String> {
    None
}

#[async_trait]
impl DecisionEngine for JevEngine {
    async fn decide(&self, request: &DecisionRequest) -> Result<DecisionResponse, DecisionError> {
        request.validate()?;
        let key = self.key.as_ref().ok_or(DecisionError::Unavailable)?;
        let body = serde_json::json!({
            "state": request.state,
            "model": self.model,
            "questions": request.questions,
        });
        let mut response = self
            .client
            .post(&self.endpoint)
            .bearer_auth(key.expose_secret())
            .json(&body)
            .send()
            .await
            .map_err(|error| map_transport_error(&error))?;
        if !response.status().is_success() {
            return Err(DecisionError::Transport);
        }
        if response
            .content_length()
            .is_some_and(|len| len > MAX_RESPONSE_BYTES as u64)
        {
            return Err(DecisionError::InvalidResponse("response is too large"));
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|error| map_transport_error(&error))?
        {
            if bytes.len().saturating_add(chunk.len()) > MAX_RESPONSE_BYTES {
                return Err(DecisionError::InvalidResponse("response is too large"));
            }
            bytes.extend_from_slice(&chunk);
        }
        let parsed: DecisionResponse = serde_json::from_slice(&bytes)
            .map_err(|_| DecisionError::InvalidResponse("response JSON is malformed"))?;
        parsed.validate_against(request)?;
        Ok(parsed)
    }
}

fn map_transport_error(error: &reqwest::Error) -> DecisionError {
    if error.is_timeout() {
        DecisionError::Timeout
    } else {
        DecisionError::Transport
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, time::Duration};

    use gitflow_core::decision::{DecisionEngine, DecisionError, DecisionRequest, Question};
    use secrecy::ExposeSecret;
    use serde_json::json;
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
    };

    use super::{JevEngine, select_key};

    #[test]
    fn test_should_prefer_environment_key_without_reading_keychain() {
        let called = std::cell::Cell::new(false);
        let key = select_key(Some("environment-test-key".into()), || {
            called.set(true);
            None
        })
        .unwrap();
        assert_eq!(key.expose_secret(), "environment-test-key");
        assert!(!called.get());
    }

    #[test]
    fn test_should_use_keychain_when_environment_key_is_empty() {
        let key = select_key(Some(" ".into()), || Some("keychain-test-key".into())).unwrap();
        assert_eq!(key.expose_secret(), "keychain-test-key");
    }

    fn request() -> DecisionRequest {
        DecisionRequest {
            state: json!({"title": "Issue"}),
            questions: BTreeMap::from([(
                "blocked".into(),
                Question::Noul {
                    instructions: "Is work blocked?".into(),
                },
            )]),
        }
    }

    #[tokio::test]
    async fn test_should_reject_missing_key_without_network() {
        let engine = JevEngine::with_key_for_test(None);
        assert!(matches!(
            engine.decide(&request()).await,
            Err(DecisionError::Unavailable)
        ));
    }

    #[test]
    fn test_should_redact_key_in_debug_output() {
        let engine = JevEngine::with_key_for_test(Some("test-sensitive-key".into()));
        assert!(!format!("{engine:?}").contains("test-sensitive-key"));
    }

    async fn serve_once(body: &'static str, delay: Duration) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut request_bytes = [0_u8; 4096];
            let _ = stream.read(&mut request_bytes).await;
            tokio::time::sleep(delay).await;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: \
                 {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = stream.write_all(response.as_bytes()).await;
        });
        format!("http://{address}/v1/systemone")
    }

    #[tokio::test]
    async fn test_should_accept_valid_provider_response() {
        let endpoint = serve_once(
            r#"{"model":"jev-1.13.0","answers":{"blocked":{"type":"noul","noul":0.8}},"usage":{"input_tokens":10,"output_tokens":2}}"#,
            Duration::ZERO,
        ).await;
        let mut engine = JevEngine::with_key_for_test(Some("test-key".into()));
        engine.endpoint = endpoint;
        let response = engine.decide(&request()).await.unwrap();
        assert_eq!(response.answers.len(), 1);
    }

    #[tokio::test]
    async fn test_should_reject_malformed_provider_response() {
        let endpoint = serve_once("not-json", Duration::ZERO).await;
        let mut engine = JevEngine::with_key_for_test(Some("test-key".into()));
        engine.endpoint = endpoint;
        assert!(matches!(
            engine.decide(&request()).await,
            Err(DecisionError::InvalidResponse(_))
        ));
    }

    #[tokio::test]
    async fn test_should_stop_at_timeout() {
        let endpoint = serve_once("{}", Duration::from_millis(200)).await;
        let mut engine = JevEngine::with_key_for_test(Some("test-key".into()));
        engine.endpoint = endpoint;
        engine.client = reqwest::Client::builder()
            .timeout(Duration::from_millis(40))
            .build()
            .unwrap();
        assert!(matches!(
            engine.decide(&request()).await,
            Err(DecisionError::Timeout)
        ));
    }
}
