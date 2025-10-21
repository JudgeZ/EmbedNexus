//! HTTP transport adapter implementation surface.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use blake3::Hasher;
use runtime_router::{RouterCommand, SessionContext, SharedRouter};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use uuid::Uuid;

/// HTTP binding and security policy configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HttpConfig {
    /// Hostname or address to bind (loopback enforced by validation).
    pub host: String,
    /// Listening port.
    pub port: u16,
    /// Whether TLS must be negotiated before accepting payloads.
    pub tls_required: bool,
    /// Principals that may authenticate via issued session tokens.
    pub allowed_principals: Vec<String>,
    /// Shared secret used to sign session tokens.
    pub token_secret: String,
    /// Whether CSRF protection headers are required for state-changing requests.
    pub require_csrf: bool,
}

impl HttpConfig {
    /// Basic validation to ensure loopback binding.
    pub fn validate(&self) -> Result<(), TransportError> {
        if !self.host.starts_with("127.") && self.host != "::1" && self.host != "localhost" {
            return Err(TransportError::Configuration(
                "HTTP adapters must bind to loopback addresses".into(),
            ));
        }
        if self.port == 0 {
            return Err(TransportError::Configuration(
                "port must be greater than zero".into(),
            ));
        }
        if self.allowed_principals.is_empty() {
            return Err(TransportError::Configuration(
                "at least one principal must be allowed".into(),
            ));
        }
        Ok(())
    }
}

/// Envelope used to issue and validate HTTP session tokens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionToken {
    /// Opaque token string returned to the client.
    pub token: String,
    /// CSRF nonce that must accompany state-changing requests.
    pub csrf_nonce: String,
    /// Session expiration timestamp.
    pub expires_at: SystemTime,
    /// Token identifier used for audit correlation.
    pub token_id: Uuid,
}

/// Canonical HTTP request representation for adapter processing.
#[derive(Debug, Clone)]
pub struct HttpRequest {
    /// HTTP method (normalized to uppercase by clients).
    pub method: String,
    /// REST-like path of the command entry point.
    pub path: String,
    /// Headers forwarded by the client.
    pub headers: HashMap<String, String>,
    /// Parsed JSON body.
    pub body: Value,
}

impl HttpRequest {
    /// Construct a new HTTP request with JSON body.
    pub fn new(method: impl Into<String>, path: impl Into<String>, body: Value) -> Self {
        Self {
            method: method.into(),
            path: path.into(),
            headers: HashMap::new(),
            body,
        }
    }

    /// Attach a header value.
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }
}

/// Canonical HTTP response returned by the adapter.
#[derive(Debug, Clone, PartialEq)]
pub struct HttpResponse {
    /// HTTP status code.
    pub status: u16,
    /// Headers emitted by the adapter.
    pub headers: HashMap<String, String>,
    /// JSON payload.
    pub body: Value,
}

/// Telemetry event emitted by the adapter lifecycle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TelemetryEvent {
    /// Event type (e.g., `http.request`, `http.auth.failure`).
    pub kind: String,
    /// Principal associated with the event.
    pub principal: Option<String>,
    /// Additional message for debugging.
    pub message: String,
}

/// Sink capturing telemetry events for auditing and testing.
#[derive(Debug, Default)]
pub struct TelemetrySink {
    events: Mutex<Vec<TelemetryEvent>>,
}

impl TelemetrySink {
    /// Record a telemetry event synchronously.
    pub fn record(&self, event: TelemetryEvent) {
        self.events.lock().unwrap().push(event);
    }

    /// Retrieve recorded events.
    pub fn events(&self) -> Vec<TelemetryEvent> {
        self.events.lock().unwrap().clone()
    }
}

/// Errors surfaced by the HTTP adapter.
#[derive(Debug, Error, PartialEq)]
pub enum TransportError {
    /// Configuration validation failure.
    #[error("configuration error: {0}")]
    Configuration(String),
    /// Authentication or authorization failure.
    #[error("unauthorized: {0}")]
    Unauthorized(String),
    /// CSRF guard failure.
    #[error("csrf violation: {0}")]
    Csrf(String),
    /// Invalid request payload or method.
    #[error("invalid request: {0}")]
    InvalidRequest(String),
    /// Router surfaced an error.
    #[error("router error: {0}")]
    Router(String),
}

/// HTTP adapter bridging requests into the runtime router.
pub struct HttpAdapter {
    config: HttpConfig,
    router: SharedRouter,
    telemetry: Arc<TelemetrySink>,
    signer: TokenSigner,
}

impl HttpAdapter {
    /// Bind the adapter using the provided configuration and router handle.
    pub fn bind(config: HttpConfig, router: SharedRouter) -> Result<Self, TransportError> {
        config.validate()?;
        let telemetry = Arc::new(TelemetrySink::default());
        let signer = TokenSigner::new(config.token_secret.clone());
        Ok(Self {
            config,
            router,
            telemetry,
            signer,
        })
    }

    /// Issue a session token for the provided principal and capabilities.
    pub fn issue_session_token(
        &self,
        principal: &str,
        capabilities: &[String],
    ) -> Result<SessionToken, TransportError> {
        if !self
            .config
            .allowed_principals
            .iter()
            .any(|p| p == principal)
        {
            return Err(TransportError::Unauthorized(format!(
                "principal {principal} is not permitted",
            )));
        }
        let token = self
            .signer
            .issue(principal, capabilities, Duration::from_secs(3600));
        self.telemetry.record(TelemetryEvent {
            kind: "http.session.issued".into(),
            principal: Some(principal.into()),
            message: token.token_id.to_string(),
        });
        Ok(token)
    }

    /// Dispatch a normalized request to the router.
    pub async fn dispatch(&self, request: HttpRequest) -> Result<HttpResponse, TransportError> {
        let token_str = self
            .header(&request, "authorization")
            .ok_or_else(|| TransportError::Unauthorized("missing authorization header".into()))?
            .strip_prefix("Bearer ")
            .ok_or_else(|| TransportError::Unauthorized("expected bearer token".into()))?
            .to_string();
        let envelope = self.signer.verify(&token_str)?;

        if !self
            .config
            .allowed_principals
            .iter()
            .any(|p| p == &envelope.principal)
        {
            return Err(TransportError::Unauthorized(format!(
                "principal {} is not permitted",
                envelope.principal
            )));
        }

        if self.config.require_csrf {
            let csrf = self
                .header(&request, "x-csrf-token")
                .ok_or_else(|| TransportError::Csrf("missing csrf token".into()))?;
            if csrf != &envelope.csrf_nonce {
                return Err(TransportError::Csrf("csrf token mismatch".into()));
            }
        }

        let command_name = request
            .body
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TransportError::InvalidRequest("command field missing".into()))?;
        let payload = request.body.get("payload").cloned().unwrap_or(Value::Null);

        let context = SessionContext {
            principal: envelope.principal.clone(),
            capabilities: envelope.capabilities.clone(),
            trace_id: Uuid::new_v4(),
            token_id: Some(envelope.token_id),
            peer: Some(format!(
                "http://{}:{}{}",
                self.config.host, self.config.port, request.path
            )),
        };

        self.telemetry.record(TelemetryEvent {
            kind: "http.request".into(),
            principal: Some(envelope.principal.clone()),
            message: command_name.to_string(),
        });

        let response = self
            .router
            .dispatch(context, RouterCommand::new(command_name, payload))
            .await
            .map_err(|err| {
                self.telemetry.record(TelemetryEvent {
                    kind: "http.router.error".into(),
                    principal: Some(envelope.principal.clone()),
                    message: err.to_string(),
                });
                TransportError::Router(err.to_string())
            })?;

        self.telemetry.record(TelemetryEvent {
            kind: "http.response".into(),
            principal: Some(envelope.principal.clone()),
            message: response.status_code.to_string(),
        });

        let mut headers = HashMap::new();
        headers.insert("content-type".into(), "application/json".into());

        Ok(HttpResponse {
            status: response.status_code,
            headers,
            body: response.payload,
        })
    }

    /// Access the telemetry sink for testing.
    pub fn telemetry(&self) -> Arc<TelemetrySink> {
        Arc::clone(&self.telemetry)
    }

    fn header<'a>(&self, request: &'a HttpRequest, key: &str) -> Option<&'a String> {
        request
            .headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(key))
            .map(|(_, v)| v)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TokenEnvelope {
    token_id: Uuid,
    principal: String,
    capabilities: Vec<String>,
    expires_at: u64,
    csrf_nonce: String,
    signature: String,
}

impl TokenEnvelope {
    fn canonical(&self) -> String {
        format!(
            "{}|{}|{}|{}|{}",
            self.token_id,
            self.principal,
            self.capabilities.join(","),
            self.expires_at,
            self.csrf_nonce
        )
    }
}

#[derive(Debug, Clone)]
struct TokenSigner {
    secret: String,
}

impl TokenSigner {
    fn new(secret: String) -> Self {
        Self { secret }
    }

    fn issue(&self, principal: &str, capabilities: &[String], ttl: Duration) -> SessionToken {
        let expires_at = SystemTime::now() + ttl;
        let expires_unix = expires_at
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let mut envelope = TokenEnvelope {
            token_id: Uuid::new_v4(),
            principal: principal.into(),
            capabilities: capabilities.to_vec(),
            expires_at: expires_unix,
            csrf_nonce: Uuid::new_v4().to_string(),
            signature: String::new(),
        };
        envelope.signature = self.sign(&envelope.canonical());
        let token = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&envelope).unwrap());
        SessionToken {
            token,
            csrf_nonce: envelope.csrf_nonce,
            expires_at,
            token_id: envelope.token_id,
        }
    }

    fn verify(&self, token: &str) -> Result<TokenEnvelope, TransportError> {
        let bytes = URL_SAFE_NO_PAD
            .decode(token)
            .map_err(|_| TransportError::Unauthorized("invalid token encoding".into()))?;
        let envelope: TokenEnvelope = serde_json::from_slice(&bytes)
            .map_err(|_| TransportError::Unauthorized("invalid token payload".into()))?;
        let expected = self.sign(&envelope.canonical());
        if envelope.signature != expected {
            return Err(TransportError::Unauthorized(
                "token signature mismatch".into(),
            ));
        }
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        if envelope.expires_at <= now {
            return Err(TransportError::Unauthorized("token expired".into()));
        }
        Ok(envelope)
    }

    fn sign(&self, canonical: &str) -> String {
        let mut hasher = Hasher::new_keyed(&self.key());
        hasher.update(canonical.as_bytes());
        URL_SAFE_NO_PAD.encode(hasher.finalize().as_bytes())
    }

    fn key(&self) -> [u8; 32] {
        let mut key = [0u8; 32];
        let digest = blake3::hash(self.secret.as_bytes());
        key.copy_from_slice(digest.as_bytes());
        key
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use runtime_router::{RecordingRouter, RouterResponse};
    use serde_json::json;

    fn config() -> HttpConfig {
        HttpConfig {
            host: "127.0.0.1".into(),
            port: 8443,
            tls_required: true,
            allowed_principals: vec!["alice".into()],
            token_secret: "super-secret".into(),
            require_csrf: true,
        }
    }

    #[tokio::test]
    async fn dispatches_authorized_request() {
        let router = Arc::new(RecordingRouter::default());
        router
            .script_response(Ok(RouterResponse::ok(json!({ "ok": true }))))
            .await;

        let adapter = HttpAdapter::bind(config(), router.clone() as SharedRouter).unwrap();
        let token = adapter
            .issue_session_token("alice", &["ingest".into()])
            .expect("token issuance should work");

        let request = HttpRequest::new(
            "POST",
            "/commands/ingest",
            json!({ "command": "ingest", "payload": {"doc": 1} }),
        )
        .with_header("Authorization", format!("Bearer {}", token.token))
        .with_header("X-Csrf-Token", token.csrf_nonce.clone());

        let response = adapter
            .dispatch(request)
            .await
            .expect("dispatch should succeed");
        assert_eq!(response.status, 200);

        let calls = router.calls().await;
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].context.principal, "alice");
        assert_eq!(calls[0].command.name, "ingest");
    }

    #[tokio::test]
    async fn rejects_missing_csrf() {
        let router = Arc::new(RecordingRouter::default());
        let adapter = HttpAdapter::bind(config(), router as SharedRouter).unwrap();
        let token = adapter
            .issue_session_token("alice", &["ingest".into()])
            .expect("token issuance should work");

        let request = HttpRequest::new("POST", "/commands/ingest", json!({ "command": "ingest" }))
            .with_header("Authorization", format!("Bearer {}", token.token));

        let err = adapter.dispatch(request).await.expect_err("csrf required");
        assert!(matches!(err, TransportError::Csrf(_)));
    }

    #[tokio::test]
    async fn rejects_tampered_token() {
        let router = Arc::new(RecordingRouter::default());
        let adapter = HttpAdapter::bind(config(), router as SharedRouter).unwrap();
        let token = adapter
            .issue_session_token("alice", &["ingest".into()])
            .expect("token issuance should work");

        let request = HttpRequest::new("POST", "/commands/ingest", json!({ "command": "ingest" }))
            .with_header("Authorization", format!("Bearer {}tampered", token.token))
            .with_header("X-Csrf-Token", token.csrf_nonce);

        let err = adapter
            .dispatch(request)
            .await
            .expect_err("tampered token must fail");
        assert!(matches!(err, TransportError::Unauthorized(_)));
    }
}
