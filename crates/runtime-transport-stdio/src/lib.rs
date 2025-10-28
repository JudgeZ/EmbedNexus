//! STDIO transport adapter framing and dispatch scaffolding.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use blake3::Hasher;
use runtime_router::{RouterCommand, RouterError, SessionContext, SharedRouter};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use thiserror::Error;
use uuid::Uuid;

/// STDIO adapter configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StdioConfig {
    /// Maximum frame size permitted by the adapter.
    pub max_frame_length: usize,
    /// Principals authorized to authenticate via STDIO tokens.
    pub allowed_principals: Vec<String>,
    /// Secret used to sign frame tokens.
    pub token_secret: String,
}

impl StdioConfig {
    pub fn validate(&self) -> Result<(), TransportError> {
        if self.max_frame_length < 64 {
            return Err(TransportError::Configuration(
                "max frame length must be >= 64 bytes".into(),
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

/// STDIO session token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionToken {
    pub token: String,
}

/// Serialized STDIO frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StdioFrame {
    pub payload: Vec<u8>,
}

/// Framing codec implementing checksum and length validation.
#[derive(Debug, Clone)]
pub struct FramingCodec {
    max_frame_length: usize,
    signer: Arc<TokenSigner>,
}

impl FramingCodec {
    pub(crate) fn new(max_frame_length: usize, signer: Arc<TokenSigner>) -> Self {
        Self {
            max_frame_length,
            signer,
        }
    }

    pub fn encode(&self, json: &Value, token: &SessionToken) -> Result<StdioFrame, TransportError> {
        self.signer.verify(&token.token)?;
        let payload_bytes = serde_json::to_vec(json)
            .map_err(|err| TransportError::Framing(format!("json serialization failed: {err}")))?;
        let token_bytes = token.token.as_bytes();

        let frame_len = 4 + 2 + payload_bytes.len() + token_bytes.len() + 16;
        if frame_len > self.max_frame_length {
            return Err(TransportError::Framing(
                "frame exceeds maximum length".into(),
            ));
        }

        let mut buffer = Vec::with_capacity(frame_len);
        buffer.extend_from_slice(&(payload_bytes.len() as u32).to_be_bytes());
        buffer.extend_from_slice(&(token_bytes.len() as u16).to_be_bytes());
        buffer.extend_from_slice(&payload_bytes);
        buffer.extend_from_slice(token_bytes);
        let checksum = self.checksum(&buffer);
        buffer.extend_from_slice(&checksum);

        Ok(StdioFrame { payload: buffer })
    }

    pub fn decode(&self, frame: &StdioFrame) -> Result<(Value, String), TransportError> {
        let (value, envelope) = self.decode_with_envelope(frame)?;
        Ok((value, envelope.raw_token))
    }

    fn decode_with_envelope(
        &self,
        frame: &StdioFrame,
    ) -> Result<(Value, TokenEnvelope), TransportError> {
        if frame.payload.len() < 4 + 2 + 16 {
            return Err(TransportError::Framing("frame too short".into()));
        }
        if frame.payload.len() > self.max_frame_length {
            return Err(TransportError::Framing(
                "frame exceeds maximum length".into(),
            ));
        }

        let mut cursor = &frame.payload[..];
        let payload_len = u32::from_be_bytes(cursor[0..4].try_into().unwrap()) as usize;
        cursor = &cursor[4..];
        let token_len = u16::from_be_bytes(cursor[0..2].try_into().unwrap()) as usize;
        cursor = &cursor[2..];

        if payload_len + token_len + 16 != cursor.len() {
            return Err(TransportError::Framing("frame length mismatch".into()));
        }

        let (payload_bytes, remainder) = cursor.split_at(payload_len);
        let (token_bytes, checksum) = remainder.split_at(token_len);
        let expected_checksum = self.checksum(&frame.payload[..frame.payload.len() - 16]);
        if checksum != expected_checksum {
            return Err(TransportError::Framing("checksum mismatch".into()));
        }

        let payload: Value = serde_json::from_slice(payload_bytes)
            .map_err(|err| TransportError::Framing(format!("invalid json: {err}")))?;
        let token = std::str::from_utf8(token_bytes)
            .map_err(|_| TransportError::Framing("token not utf8".into()))?;
        let envelope = self.signer.verify(token)?;
        Ok((payload, envelope))
    }

    fn checksum(&self, data: &[u8]) -> [u8; 16] {
        let digest = blake3::hash(data);
        let mut out = [0u8; 16];
        out.copy_from_slice(&digest.as_bytes()[..16]);
        out
    }
}

/// STDIO telemetry event for testing purposes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TelemetryEvent {
    pub kind: String,
    pub message: String,
}

#[derive(Debug, Default)]
pub struct TelemetrySink {
    events: Mutex<Vec<TelemetryEvent>>,
}

impl TelemetrySink {
    pub fn record(&self, event: TelemetryEvent) {
        self.events.lock().unwrap().push(event);
    }

    pub fn events(&self) -> Vec<TelemetryEvent> {
        self.events.lock().unwrap().clone()
    }
}

/// Errors exposed by the STDIO adapter.
#[derive(Debug, Error, PartialEq)]
pub enum TransportError {
    #[error("configuration error: {0}")]
    Configuration(String),
    #[error("authentication failure: {0}")]
    Unauthorized(String),
    #[error("framing error: {0}")]
    Framing(String),
    #[error("router error: {0}")]
    Router(RouterError),
}

impl TransportError {
    pub fn router_status_code(&self) -> Option<u16> {
        match self {
            TransportError::Router(err) => Some(err.status_code()),
            _ => None,
        }
    }
}

/// STDIO adapter entry point.
pub struct StdioAdapter {
    config: StdioConfig,
    router: SharedRouter,
    telemetry: Arc<TelemetrySink>,
    signer: Arc<TokenSigner>,
    codec: FramingCodec,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetryPayload {
    pub sequence: u64,
    pub command: String,
    pub payload: Value,
    pub token_id: String,
}

#[derive(Debug, Clone)]
pub struct RetryEntry {
    pub payload: RetryPayload,
    enqueued_at: SystemTime,
    attempts: u32,
}

impl RetryEntry {
    pub fn attempts(&self) -> u32 {
        self.attempts
    }

    pub fn enqueued_at(&self) -> SystemTime {
        self.enqueued_at
    }

    pub fn into_payload(self) -> RetryPayload {
        self.payload
    }

    pub fn payload(&self) -> &RetryPayload {
        &self.payload
    }
}

#[derive(Debug)]
pub struct RetryBuffer {
    max_entries: usize,
    max_age: Duration,
    inner: Mutex<VecDeque<RetryEntry>>,
    max_sequence_seen: Mutex<Option<u64>>,
}

impl RetryBuffer {
    pub fn new(max_entries: usize, max_age: Duration) -> Self {
        Self {
            max_entries,
            max_age,
            inner: Mutex::new(VecDeque::new()),
            max_sequence_seen: Mutex::new(None),
        }
    }

    pub fn enqueue(&self, payload: RetryPayload) -> Result<(), RetryError> {
        self.push_entry(RetryEntry {
            payload,
            enqueued_at: SystemTime::now(),
            attempts: 0,
        })
    }

    pub fn enqueue_at(
        &self,
        payload: RetryPayload,
        enqueued_at: SystemTime,
    ) -> Result<(), RetryError> {
        self.push_entry(RetryEntry {
            payload,
            enqueued_at,
            attempts: 0,
        })
    }

    pub fn requeue(&self, entry: RetryEntry) -> Result<(), RetryError> {
        self.push_entry(entry)
    }

    pub fn drain_ready(&self) -> Vec<RetryEntry> {
        let mut guard = self.inner.lock().expect("retry buffer mutex poisoned");
        let now = SystemTime::now();
        guard.retain(|entry| match now.duration_since(entry.enqueued_at) {
            Ok(age) => age <= self.max_age,
            Err(_) => true,
        });
        guard.drain(..).collect()
    }

    pub fn max_sequence(&self) -> Option<u64> {
        *self.max_sequence_seen.lock().unwrap()
    }

    fn push_entry(&self, mut entry: RetryEntry) -> Result<(), RetryError> {
        if self.max_entries == 0 {
            return Err(RetryError::Misconfigured(
                "max_entries cannot be zero".into(),
            ));
        }
        // preserve original enqueue time on requeue
        entry.attempts = entry.attempts.saturating_add(1);
        let mut guard = self.inner.lock().expect("retry buffer mutex poisoned");
        while guard.len() >= self.max_entries {
            guard.pop_front();
        }
        {
            let mut max_seen = self.max_sequence_seen.lock().unwrap();
            match *max_seen {
                Some(existing) if existing >= entry.payload.sequence => {}
                _ => *max_seen = Some(entry.payload.sequence),
            }
        }
        guard.push_back(entry);
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RetryError {
    #[error("retry buffer misconfigured: {0}")]
    Misconfigured(String),
}

impl StdioAdapter {
    pub fn bind(config: StdioConfig, router: SharedRouter) -> Result<Self, TransportError> {
        config.validate()?;
        let signer = Arc::new(TokenSigner::new(config.token_secret.clone()));
        let codec = FramingCodec::new(config.max_frame_length, signer.clone());
        Ok(Self {
            config,
            router,
            telemetry: Arc::new(TelemetrySink::default()),
            signer,
            codec,
        })
    }

    pub fn issue_session_token(&self, principal: &str) -> Result<SessionToken, TransportError> {
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
        let IssuedToken { token, token_id } =
            self.signer
                .issue(principal, &["stdio".into()], Duration::from_secs(3600));
        self.telemetry.record(TelemetryEvent {
            kind: "stdio.session.issued".into(),
            message: token_id.to_string(),
        });
        Ok(SessionToken { token })
    }

    pub async fn dispatch_frame(&self, frame: StdioFrame) -> Result<StdioFrame, TransportError> {
        let (payload, envelope) = self.codec.decode_with_envelope(&frame)?;
        let command = payload
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TransportError::Framing("command missing".into()))?;
        let body = payload.get("payload").cloned().unwrap_or(Value::Null);

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

        let context = SessionContext {
            principal: envelope.principal.clone(),
            capabilities: envelope.capabilities.clone(),
            trace_id: Uuid::new_v4(),
            token_id: Some(envelope.token_id),
            peer: Some("stdio".into()),
        };

        self.telemetry.record(TelemetryEvent {
            kind: "stdio.request".into(),
            message: command.to_string(),
        });

        let response = self
            .router
            .dispatch(context, RouterCommand::new(command, body))
            .await
            .map_err(|err| {
                self.telemetry.record(TelemetryEvent {
                    kind: "stdio.router.error".into(),
                    message: err.to_string(),
                });
                TransportError::Router(err)
            })?;

        let status = if (200..=299).contains(&response.status_code) {
            "ok"
        } else {
            "error"
        };
        let response_body = json!({
            "status": status,
            "payload": response.payload,
        });
        self.telemetry.record(TelemetryEvent {
            kind: "stdio.response".into(),
            message: response.status_code.to_string(),
        });
        self.codec.encode(
            &response_body,
            &SessionToken {
                token: envelope.raw_token,
            },
        )
    }

    pub fn codec(&self) -> &FramingCodec {
        &self.codec
    }

    pub fn telemetry(&self) -> Arc<TelemetrySink> {
        Arc::clone(&self.telemetry)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TokenEnvelope {
    raw_token: String,
    token_id: Uuid,
    principal: String,
    capabilities: Vec<String>,
    expires_at: u64,
}

impl TokenEnvelope {
    fn canonical(&self) -> String {
        format!(
            "{}|{}|{}|{}",
            self.token_id,
            self.principal,
            self.capabilities.join(","),
            self.expires_at
        )
    }
}

#[derive(Debug)]
struct TokenSigner {
    secret: String,
}

impl TokenSigner {
    fn new(secret: String) -> Self {
        Self { secret }
    }

    fn issue(&self, principal: &str, capabilities: &[String], ttl: Duration) -> IssuedToken {
        let expires_at = SystemTime::now() + ttl;
        let expires_unix = expires_at
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let envelope = TokenEnvelope {
            raw_token: String::new(),
            token_id: Uuid::new_v4(),
            principal: principal.into(),
            capabilities: capabilities.to_vec(),
            expires_at: expires_unix,
        };
        let token_id = envelope.token_id;
        let signature = self.sign(&envelope.canonical());
        let signed = SignedToken {
            envelope,
            signature,
        };
        let token = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&signed).unwrap());
        IssuedToken { token, token_id }
    }

    fn verify(&self, token: &str) -> Result<TokenEnvelope, TransportError> {
        let bytes = URL_SAFE_NO_PAD
            .decode(token)
            .map_err(|_| TransportError::Unauthorized("invalid token encoding".into()))?;
        let signed: SignedToken = serde_json::from_slice(&bytes)
            .map_err(|_| TransportError::Unauthorized("invalid token payload".into()))?;
        let expected = self.sign(&signed.envelope.canonical());
        if signed.signature != expected {
            return Err(TransportError::Unauthorized(
                "token signature mismatch".into(),
            ));
        }
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        if signed.envelope.expires_at <= now {
            return Err(TransportError::Unauthorized("token expired".into()));
        }
        Ok(TokenEnvelope {
            raw_token: token.into(),
            ..signed.envelope
        })
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

#[derive(Debug, Serialize, Deserialize)]
struct SignedToken {
    envelope: TokenEnvelope,
    signature: String,
}

struct IssuedToken {
    token: String,
    token_id: Uuid,
}

#[cfg(test)]
mod tests {
    use super::*;
    use runtime_router::{RecordingRouter, RouterResponse};
    use serde_json::json;
    use std::sync::Arc;

    fn config() -> StdioConfig {
        StdioConfig {
            max_frame_length: 2048,
            allowed_principals: vec!["alice".into()],
            token_secret: "stdio-secret".into(),
        }
    }

    #[tokio::test]
    async fn round_trips_frame() {
        let router = Arc::new(RecordingRouter::default());
        router
            .script_response(Ok(RouterResponse::ok(json!({ "ok": true }))))
            .await;

        let adapter = StdioAdapter::bind(config(), router.clone() as SharedRouter).unwrap();
        let token = adapter
            .issue_session_token("alice")
            .expect("token issuance should succeed");
        let frame = adapter
            .codec()
            .encode(&json!({ "command": "status" }), &token)
            .expect("encode should work");

        let response = adapter
            .dispatch_frame(frame)
            .await
            .expect("dispatch should succeed");
        let (body, _) = adapter.codec().decode(&response).expect("decode response");
        assert_eq!(body["status"], json!("ok"));
    }

    #[tokio::test]
    async fn rejects_bad_checksum() {
        let router = Arc::new(RecordingRouter::default());
        let adapter = StdioAdapter::bind(config(), router as SharedRouter).unwrap();
        let frame = StdioFrame {
            payload: vec![0, 1, 2, 3],
        };
        let err = adapter
            .dispatch_frame(frame)
            .await
            .expect_err("checksum must be validated");
        assert!(matches!(err, TransportError::Framing(_)));
    }

    #[tokio::test]
    async fn rejects_unknown_principal() {
        let router = Arc::new(RecordingRouter::default());
        let adapter = StdioAdapter::bind(config(), router as SharedRouter).unwrap();
        let forged = SessionToken {
            token: "forged".into(),
        };
        let frame = adapter
            .codec()
            .encode(&json!({ "command": "status" }), &forged)
            .expect_err("forged tokens should fail");
        assert!(matches!(frame, TransportError::Unauthorized(_)));
    }

    #[tokio::test]
    async fn rejects_revoked_principal_on_dispatch() {
        let router = Arc::new(RecordingRouter::default());
        let mut adapter = StdioAdapter::bind(config(), router.clone() as SharedRouter).unwrap();
        let token = adapter
            .issue_session_token("alice")
            .expect("token issuance should succeed");

        adapter.config.allowed_principals.clear();

        let frame = adapter
            .codec()
            .encode(&json!({ "command": "status" }), &token)
            .expect("encode should succeed");

        let err = adapter
            .dispatch_frame(frame)
            .await
            .expect_err("revoked principal should be rejected");
        assert!(matches!(err, TransportError::Unauthorized(msg) if msg.contains("alice")));
    }

    #[tokio::test]
    async fn treats_non_200_success_status_as_ok() {
        let router = Arc::new(RecordingRouter::default());
        router
            .script_response(Ok(RouterResponse {
                status_code: 204,
                payload: json!({ "ok": true }),
                diagnostics: Vec::new(),
            }))
            .await;

        let adapter = StdioAdapter::bind(config(), router.clone() as SharedRouter).unwrap();
        let token = adapter
            .issue_session_token("alice")
            .expect("token issuance should succeed");
        let frame = adapter
            .codec()
            .encode(&json!({ "command": "status" }), &token)
            .expect("encode should work");

        let response = adapter
            .dispatch_frame(frame)
            .await
            .expect("dispatch should succeed");
        let (body, _) = adapter.codec().decode(&response).expect("decode response");
        assert_eq!(body["status"], json!("ok"));
    }

    #[test]
    fn retry_buffer_enforces_capacity_fifo() {
        let buffer = RetryBuffer::new(3, Duration::from_secs(60));
        for seq in 1..=4 {
            buffer
                .enqueue(RetryPayload {
                    sequence: seq,
                    command: "ingest".into(),
                    payload: json!({ "id": seq }),
                    token_id: format!("tok-{seq}"),
                })
                .unwrap();
        }

        assert_eq!(buffer.max_sequence(), Some(4));
        let drained = buffer.drain_ready();
        let sequences: Vec<u64> = drained.iter().map(|entry| entry.payload.sequence).collect();
        assert_eq!(sequences, vec![2, 3, 4]);
    }

    #[test]
    fn retry_buffer_requeue_retains_order() {
        let buffer = RetryBuffer::new(5, Duration::from_secs(60));
        for seq in 10..=12 {
            buffer
                .enqueue(RetryPayload {
                    sequence: seq,
                    command: "ingest".into(),
                    payload: json!({ "id": seq }),
                    token_id: format!("tok-{seq}"),
                })
                .unwrap();
        }

        let mut drained = buffer.drain_ready();
        assert_eq!(drained.len(), 3);
        let retry_entry = drained.remove(1);
        buffer.requeue(retry_entry).unwrap();

        buffer
            .enqueue(RetryPayload {
                sequence: 13,
                command: "search".into(),
                payload: json!({ "q": "docs" }),
                token_id: "tok-13".into(),
            })
            .unwrap();

        let drained_again = buffer.drain_ready();
        let sequences: Vec<u64> = drained_again
            .iter()
            .map(|entry| entry.payload.sequence)
            .collect();
        assert_eq!(sequences, vec![11, 13]);
    }
}
