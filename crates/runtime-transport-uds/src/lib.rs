//! Unix domain socket transport adapter implementation.

use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use blake3::Hasher;
use runtime_router::{RouterCommand, RouterError, SessionContext, SharedRouter};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use uuid::Uuid;

/// UDS adapter configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UdsConfig {
    pub socket_path: String,
    pub allowed_principals: Vec<String>,
    pub allowed_uids: Vec<u32>,
    pub token_secret: String,
}

impl UdsConfig {
    pub fn validate(&self) -> Result<(), TransportError> {
        if !self.socket_path.starts_with('/') {
            return Err(TransportError::Configuration(
                "socket path must be absolute".into(),
            ));
        }
        if self.allowed_principals.is_empty() {
            return Err(TransportError::Configuration(
                "at least one principal must be allowed".into(),
            ));
        }
        if self.allowed_uids.is_empty() {
            return Err(TransportError::Configuration(
                "allowed UID list cannot be empty".into(),
            ));
        }
        Ok(())
    }
}

/// Captures peer credentials extracted from the UDS handshake.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerCredentials {
    pub uid: u32,
    pub pid: u32,
    pub process_name: String,
}

/// Issued session token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionToken {
    pub token: String,
    pub token_id: Uuid,
    pub expires_at: SystemTime,
}

/// Request envelope for UDS dispatch.
#[derive(Debug, Clone)]
pub struct UdsRequest {
    pub peer: PeerCredentials,
    pub token: String,
    pub payload: Value,
}

impl UdsRequest {
    #[must_use]
    pub const fn new(peer: PeerCredentials, token: String, payload: Value) -> Self {
        Self {
            peer,
            token,
            payload,
        }
    }
}

/// Telemetry event captured for auditing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TelemetryEvent {
    pub kind: String,
    pub message: String,
    pub principal: Option<String>,
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

/// Errors produced by the adapter.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum TransportError {
    #[error("configuration error: {0}")]
    Configuration(String),
    #[error("unauthorized peer: {0}")]
    Unauthorized(String),
    #[error("router error: {0}")]
    Router(RouterError),
}

impl TransportError {
    #[must_use]
    pub const fn router_status_code(&self) -> Option<u16> {
        match self {
            Self::Router(err) => Some(err.status_code()),
            _ => None,
        }
    }
}

/// UDS adapter bridging IPC requests into the router.
pub struct UdsAdapter {
    config: UdsConfig,
    router: SharedRouter,
    telemetry: Arc<TelemetrySink>,
    signer: Arc<TokenSigner>,
    negotiated_uids: Mutex<HashSet<u32>>,
}

impl UdsAdapter {
    pub fn bind(config: UdsConfig, router: SharedRouter) -> Result<Self, TransportError> {
        config.validate()?;
        Ok(Self {
            signer: Arc::new(TokenSigner::new(config.token_secret.clone())),
            config,
            router,
            telemetry: Arc::new(TelemetrySink::default()),
            negotiated_uids: Mutex::new(HashSet::new()),
        })
    }

    pub fn negotiate_peer(&self, peer: &PeerCredentials) -> Result<(), TransportError> {
        if !self.config.allowed_uids.contains(&peer.uid) {
            return Err(TransportError::Unauthorized(format!(
                "uid {} not permitted",
                peer.uid
            )));
        }
        self.telemetry.record(TelemetryEvent {
            kind: "uds.peer.accepted".into(),
            message: format!("{}:{}", peer.uid, peer.process_name),
            principal: None,
        });
        self.negotiated_uids.lock().unwrap().insert(peer.uid);
        Ok(())
    }

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
        let issued = self
            .signer
            .issue(principal, capabilities, Duration::from_secs(3600));
        self.telemetry.record(TelemetryEvent {
            kind: "uds.session.issued".into(),
            message: issued.token_id.to_string(),
            principal: Some(principal.into()),
        });
        Ok(SessionToken {
            token: issued.token,
            token_id: issued.token_id,
            expires_at: issued.expires_at,
        })
    }

    pub async fn dispatch(&self, request: UdsRequest) -> Result<Value, TransportError> {
        if !self
            .negotiated_uids
            .lock()
            .unwrap()
            .contains(&request.peer.uid)
        {
            return Err(TransportError::Unauthorized(format!(
                "uid {} not negotiated",
                request.peer.uid
            )));
        }
        let envelope = self.signer.verify(&request.token)?;
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

        let command = request
            .payload
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TransportError::Unauthorized("command missing".into()))?;
        let body = request
            .payload
            .get("payload")
            .cloned()
            .unwrap_or(Value::Null);

        let context = SessionContext {
            principal: envelope.principal.clone(),
            capabilities: envelope.capabilities.clone(),
            trace_id: Uuid::new_v4(),
            token_id: Some(envelope.token_id),
            peer: Some(format!("uds://{}", request.peer.process_name)),
        };

        self.telemetry.record(TelemetryEvent {
            kind: "uds.request".into(),
            message: command.to_string(),
            principal: Some(envelope.principal.clone()),
        });

        let response = self
            .router
            .dispatch(context, RouterCommand::new(command, body))
            .await
            .map_err(|err| {
                self.telemetry.record(TelemetryEvent {
                    kind: "uds.router.error".into(),
                    message: err.to_string(),
                    principal: Some(envelope.principal.clone()),
                });
                TransportError::Router(err)
            })?;

        self.telemetry.record(TelemetryEvent {
            kind: "uds.response".into(),
            message: response.status_code.to_string(),
            principal: Some(envelope.principal.clone()),
        });

        Ok(response.payload)
    }

    pub fn telemetry(&self) -> Arc<TelemetrySink> {
        Arc::clone(&self.telemetry)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TokenEnvelope {
    token_id: Uuid,
    principal: String,
    capabilities: Vec<String>,
    expires_at: u64,
    signature: String,
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
    const fn new(secret: String) -> Self {
        Self { secret }
    }

    fn issue(&self, principal: &str, capabilities: &[String], ttl: Duration) -> IssuedToken {
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
            signature: String::new(),
        };
        envelope.signature = self.sign(&envelope.canonical());
        let token = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&envelope).unwrap());
        IssuedToken {
            token,
            token_id: envelope.token_id,
            expires_at,
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

struct IssuedToken {
    token: String,
    token_id: Uuid,
    expires_at: SystemTime,
}

#[cfg(test)]
mod tests {
    use super::*;
    use runtime_router::{RecordingRouter, RouterError, RouterResponse};
    use serde_json::json;
    use std::sync::Arc;

    fn config() -> UdsConfig {
        UdsConfig {
            socket_path: "/tmp/runtime.sock".into(),
            allowed_principals: vec!["alice".into()],
            allowed_uids: vec![1000],
            token_secret: "uds-secret".into(),
        }
    }

    fn peer() -> PeerCredentials {
        PeerCredentials {
            uid: 1000,
            pid: 42,
            process_name: "editor".into(),
        }
    }

    #[tokio::test]
    async fn dispatches_after_negotiation() {
        let router = Arc::new(RecordingRouter::default());
        router
            .script_response(Ok(RouterResponse::ok(json!({ "status": "ok" }))))
            .await;

        let adapter = UdsAdapter::bind(config(), router.clone() as SharedRouter).unwrap();
        adapter
            .negotiate_peer(&peer())
            .expect("peer negotiation succeeds");
        let token = adapter
            .issue_session_token("alice", &["search".into()])
            .expect("token issuance works");
        let request = UdsRequest::new(
            peer(),
            token.token.clone(),
            json!({
                "command": "search",
                "payload": {"term": "docs"}
            }),
        );
        let response = adapter.dispatch(request).await.expect("dispatch succeeds");
        assert_eq!(response["status"], json!("ok"));
        let calls = router.calls().await;
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].context.principal, "alice");
    }

    #[tokio::test]
    async fn rejects_unapproved_uid() {
        let router = Arc::new(RecordingRouter::default());
        let adapter = UdsAdapter::bind(config(), router as SharedRouter).unwrap();
        let mut creds = peer();
        creds.uid = 99;
        let err = adapter
            .negotiate_peer(&creds)
            .expect_err("uid must be validated");
        assert!(matches!(err, TransportError::Unauthorized(_)));
    }

    #[tokio::test]
    async fn surfaces_router_errors() {
        let router = Arc::new(RecordingRouter::default());
        router
            .script_response(Err(RouterError::InvalidRequest {
                detail: "bad payload".into(),
            }))
            .await;

        let adapter = UdsAdapter::bind(config(), router as SharedRouter).unwrap();
        adapter
            .negotiate_peer(&peer())
            .expect("peer negotiation succeeds");
        let token = adapter
            .issue_session_token("alice", &["search".into()])
            .expect("token issuance works");
        let request = UdsRequest::new(
            peer(),
            token.token.clone(),
            json!({
                "command": "search",
                "payload": {}
            }),
        );
        let err = adapter
            .dispatch(request)
            .await
            .expect_err("router error surfaces");
        assert!(matches!(err, TransportError::Router(_)));
    }

    #[tokio::test]
    async fn negotiate_peer_concurrent_connections_same_uid() {
        let router = Arc::new(RecordingRouter::default());
        let adapter = UdsAdapter::bind(config(), router as SharedRouter).unwrap();

        let peer_one = peer();
        adapter
            .negotiate_peer(&peer_one)
            .expect("first negotiation succeeds");

        let peer_two = PeerCredentials {
            uid: 1000,
            pid: 4242,
            process_name: "another-client".into(),
        };
        adapter
            .negotiate_peer(&peer_two)
            .expect("second negotiation for same uid succeeds");

        let negotiated = adapter.negotiated_uids.lock().unwrap();
        assert!(negotiated.contains(&1000), "uid 1000 should be tracked");
    }

    #[tokio::test]
    async fn dispatch_rejects_unnegotiated_peer() {
        let router = Arc::new(RecordingRouter::default());
        router
            .script_response(Ok(RouterResponse::ok(json!({ "status": "ok" }))))
            .await;

        let adapter = UdsAdapter::bind(config(), router as SharedRouter).unwrap();

        let token = adapter
            .issue_session_token("alice", &["search".into()])
            .expect("token issuance succeeds");

        let unnegotiated_peer = peer();
        let request = UdsRequest::new(
            unnegotiated_peer.clone(),
            token.token.clone(),
            json!({
                "command": "search",
                "payload": {"term": "docs"}
            }),
        );

        let err = adapter
            .dispatch(request)
            .await
            .expect_err("unnegotiated peer must be rejected");
        assert!(matches!(err, TransportError::Unauthorized(_)));
        assert!(
            err.to_string().contains(&unnegotiated_peer.uid.to_string()),
            "error should mention uid"
        );
    }
}
