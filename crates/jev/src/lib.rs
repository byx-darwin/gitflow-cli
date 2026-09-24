//! Optional `TypeSafe` Jev adapter for the provider-neutral decision contract.

#![forbid(unsafe_code)]
#![cfg_attr(
    test,
    allow(clippy::unwrap_used, reason = "Test fixture setup may panic")
)]

#[cfg(any(target_os = "macos", test))]
use std::sync::Once;
use std::time::Duration;

use async_trait::async_trait;
use gitflow_core::decision::{DecisionEngine, DecisionError, DecisionRequest, DecisionResponse};
use secrecy::{ExposeSecret, SecretString};

const ENDPOINT: &str = "https://api.typesafe.ai/v1/systemone";
const MAX_RESPONSE_BYTES: usize = 131_072;
#[cfg(any(target_os = "macos", test))]
const MAX_KEY_BYTES: usize = 4_096;
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);
#[cfg(any(target_os = "macos", test))]
const PRIMARY_KEYCHAIN_SERVICE: &str = "ai.typesafe.api-key";
#[cfg(any(target_os = "macos", test))]
const LEGACY_KEYCHAIN_SERVICE: &str = "gitflow-cli-typesafe";
#[cfg(any(target_os = "macos", test))]
const MIGRATION_COMMAND: &str = r#"security add-generic-password -a "$USER" -s ai.typesafe.api-key -U -w "$(security find-generic-password -a "$USER" -s gitflow-cli-typesafe -w)""#;

#[cfg(any(target_os = "macos", test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KeychainSource {
    Primary,
    Legacy,
}

#[cfg(any(target_os = "macos", test))]
#[derive(Debug)]
struct KeychainKey {
    value: SecretString,
    source: KeychainSource,
}

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
    keychain: impl FnOnce() -> Option<SecretString>,
) -> Option<SecretString> {
    env_key
        .filter(|value| !value.trim().is_empty())
        .map(SecretString::from)
        .or_else(keychain)
}

#[cfg(any(target_os = "macos", test))]
fn select_keychain_key(
    mut read_service: impl FnMut(&str) -> Option<SecretString>,
) -> Option<KeychainKey> {
    read_service(PRIMARY_KEYCHAIN_SERVICE)
        .filter(|value| !value.expose_secret().trim().is_empty())
        .map(|value| KeychainKey {
            value,
            source: KeychainSource::Primary,
        })
        .or_else(|| {
            read_service(LEGACY_KEYCHAIN_SERVICE)
                .filter(|value| !value.expose_secret().trim().is_empty())
                .map(|value| KeychainKey {
                    value,
                    source: KeychainSource::Legacy,
                })
        })
}

#[cfg(any(target_os = "macos", test))]
fn notify_legacy_keychain_once(once: &Once, notify: impl FnOnce(&'static str)) {
    once.call_once(|| notify(MIGRATION_COMMAND));
}

#[cfg(target_os = "macos")]
#[allow(
    clippy::disallowed_types,
    reason = "Synchronous, bounded Keychain lookup during adapter configuration"
)]
fn keychain_key() -> Option<SecretString> {
    static LEGACY_KEYCHAIN_NOTICE: Once = Once::new();

    let user = std::env::var("USER").ok()?;
    if user.is_empty() {
        return None;
    }
    let selected = select_keychain_key(|service| read_keychain_service(&user, service))?;
    if selected.source == KeychainSource::Legacy {
        notify_legacy_keychain_once(&LEGACY_KEYCHAIN_NOTICE, |migration_command| {
            tracing::warn!(
                keychain_service = LEGACY_KEYCHAIN_SERVICE,
                migration_command,
                "legacy TypeSafe Keychain service found; migrate it to the shared service name"
            );
        });
    }
    Some(selected.value)
}

#[cfg(target_os = "macos")]
#[allow(
    clippy::disallowed_types,
    reason = "Synchronous, bounded Keychain lookup during adapter configuration"
)]
fn read_keychain_service(user: &str, service: &str) -> Option<SecretString> {
    let output = std::process::Command::new("/usr/bin/security")
        .args(["find-generic-password", "-a", user, "-s", service, "-w"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    parse_keychain_output(output.stdout)
}

#[cfg(any(target_os = "macos", test))]
fn parse_keychain_output(output: Vec<u8>) -> Option<SecretString> {
    if output.len() > MAX_KEY_BYTES {
        return None;
    }
    let mut key = String::from_utf8(output).ok()?;
    while key.ends_with('\n') || key.ends_with('\r') {
        key.pop();
    }
    (!key.trim().is_empty()).then(|| SecretString::from(key))
}

#[cfg(not(target_os = "macos"))]
fn keychain_key() -> Option<SecretString> {
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
    use std::{
        cell::{Cell, RefCell},
        collections::BTreeMap,
        sync::Once,
        time::Duration,
    };

    use gitflow_core::decision::{DecisionEngine, DecisionError, DecisionRequest, Question};
    use secrecy::{ExposeSecret, SecretString};
    use serde_json::json;
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
    };

    use super::{
        JevEngine, KeychainSource, LEGACY_KEYCHAIN_SERVICE, MAX_KEY_BYTES, MIGRATION_COMMAND,
        PRIMARY_KEYCHAIN_SERVICE, notify_legacy_keychain_once, parse_keychain_output, select_key,
        select_keychain_key,
    };

    fn secret(value: &str) -> SecretString {
        SecretString::from(value.to_string())
    }

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
        let key = select_key(Some(" ".into()), || Some(secret("keychain-test-key"))).unwrap();
        assert_eq!(key.expose_secret(), "keychain-test-key");
    }

    #[test]
    fn test_should_return_none_when_no_key_source_is_available() {
        assert!(select_key(None, || None).is_none());
    }

    #[test]
    fn test_should_read_key_from_primary_keychain_service() {
        let selected = select_keychain_key(|service| {
            (service == PRIMARY_KEYCHAIN_SERVICE).then(|| secret("primary-test-key"))
        })
        .unwrap();

        assert_eq!(selected.value.expose_secret(), "primary-test-key");
        assert_eq!(selected.source, KeychainSource::Primary);
    }

    #[test]
    fn test_should_fall_back_to_legacy_keychain_service() {
        let services = RefCell::new(Vec::new());
        let selected = select_keychain_key(|service| {
            services.borrow_mut().push(service.to_string());
            (service == LEGACY_KEYCHAIN_SERVICE).then(|| secret("legacy-test-key"))
        })
        .unwrap();

        assert_eq!(selected.value.expose_secret(), "legacy-test-key");
        assert_eq!(selected.source, KeychainSource::Legacy);
        assert_eq!(
            services.into_inner(),
            [PRIMARY_KEYCHAIN_SERVICE, LEGACY_KEYCHAIN_SERVICE]
        );
    }

    #[test]
    fn test_should_prefer_primary_keychain_service_without_reading_legacy() {
        let legacy_read = Cell::new(false);
        let selected = select_keychain_key(|service| match service {
            PRIMARY_KEYCHAIN_SERVICE => Some(secret("primary-test-key")),
            LEGACY_KEYCHAIN_SERVICE => {
                legacy_read.set(true);
                Some(secret("legacy-test-key"))
            }
            _ => None,
        })
        .unwrap();

        assert_eq!(selected.value.expose_secret(), "primary-test-key");
        assert_eq!(selected.source, KeychainSource::Primary);
        assert!(!legacy_read.get());
    }

    #[test]
    fn test_should_return_none_when_keychain_services_are_empty_or_missing() {
        let selected = select_keychain_key(|service| {
            (service == PRIMARY_KEYCHAIN_SERVICE).then(|| secret(" "))
        });

        assert!(selected.is_none());
    }

    #[test]
    fn test_should_redact_keychain_key_debug_output() {
        let selected = select_keychain_key(|_| Some(secret("sensitive-test-key"))).unwrap();
        let debug = format!("{selected:?}");

        assert!(!debug.contains("sensitive-test-key"));
        assert!(debug.contains("[REDACTED]"));
    }

    #[test]
    fn test_should_parse_keychain_output_and_trim_line_endings() {
        let key = parse_keychain_output(b"keychain-test-key\r\n".to_vec()).unwrap();

        assert_eq!(key.expose_secret(), "keychain-test-key");
    }

    #[test]
    fn test_should_reject_invalid_or_empty_keychain_output() {
        assert!(parse_keychain_output(vec![0xff]).is_none());
        assert!(parse_keychain_output(b" \r\n".to_vec()).is_none());
        assert!(parse_keychain_output(vec![b'x'; MAX_KEY_BYTES + 1]).is_none());
    }

    #[test]
    fn test_should_emit_legacy_migration_notice_only_once() {
        let once = Once::new();
        let notice_count = Cell::new(0);
        let notice = Cell::new("");

        notify_legacy_keychain_once(&once, |command| {
            notice_count.set(notice_count.get() + 1);
            notice.set(command);
        });
        notify_legacy_keychain_once(&once, |_| {
            notice_count.set(notice_count.get() + 1);
        });

        assert_eq!(notice_count.get(), 1);
        assert_eq!(notice.get(), MIGRATION_COMMAND);
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
