//! Runtime command router contract and lightweight testing utilities.

use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use tokio::sync::Mutex;
use uuid::Uuid;

/// Contextual information derived from a validated session token.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionContext {
    /// Unique identifier for the authenticated principal.
    pub principal: String,
    /// Capabilities granted to the session (e.g., `ingest`, `search`).
    pub capabilities: Vec<String>,
    /// Identifier that lets telemetry sinks correlate adapter and router spans.
    pub trace_id: Uuid,
    /// Optional session token identifier.
    pub token_id: Option<Uuid>,
    /// Optional peer identity as reported by the transport layer.
    pub peer: Option<String>,
}

impl SessionContext {
    /// Helper for constructing a context with no peer information.
    pub fn new(principal: impl Into<String>, capabilities: Vec<String>) -> Self {
        Self {
            principal: principal.into(),
            capabilities,
            trace_id: Uuid::new_v4(),
            token_id: None,
            peer: None,
        }
    }
}

/// Normalized command forwarded from a transport adapter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouterCommand {
    /// Logical command name (e.g., `status`, `ingest.batch`).
    pub name: String,
    /// Raw payload forwarded to the runtime policy engine.
    pub payload: Value,
}

impl RouterCommand {
    /// Construct a new router command.
    pub fn new(name: impl Into<String>, payload: Value) -> Self {
        Self {
            name: name.into(),
            payload,
        }
    }
}

/// Successful response emitted by the router.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RouterResponse {
    /// Status code aligned with transport-level status semantics.
    pub status_code: u16,
    /// Payload returned to the client.
    pub payload: Value,
    /// Optional diagnostics for telemetry correlation.
    pub diagnostics: Vec<String>,
}

impl RouterResponse {
    /// Convenience constructor for OK responses.
    pub fn ok(payload: Value) -> Self {
        Self {
            status_code: 200,
            payload,
            diagnostics: Vec::new(),
        }
    }
}

/// Router errors mapped back to transport adapters.
#[derive(Debug, Clone, Error, PartialEq, Serialize, Deserialize)]
pub enum RouterError {
    /// The principal is not permitted to execute the requested command.
    #[error("unauthorized: {detail}")]
    Unauthorized { detail: String },
    /// The request payload failed validation.
    #[error("invalid request: {detail}")]
    InvalidRequest { detail: String },
    /// The target command is not registered.
    #[error("not found: {detail}")]
    NotFound { detail: String },
    /// Any other unexpected failure.
    #[error("internal error: {detail}")]
    Internal { detail: String },
}

impl RouterError {
    /// Map the error into an HTTP-like status code for adapter usage.
    pub fn status_code(&self) -> u16 {
        match self {
            RouterError::Unauthorized { .. } => 401,
            RouterError::InvalidRequest { .. } => 400,
            RouterError::NotFound { .. } => 404,
            RouterError::Internal { .. } => 500,
        }
    }
}

/// Command router abstraction used by all transport adapters.
#[async_trait]
pub trait CommandRouter: Send + Sync {
    /// Dispatch a normalized command.
    async fn dispatch(
        &self,
        ctx: SessionContext,
        command: RouterCommand,
    ) -> Result<RouterResponse, RouterError>;
}

/// Recorded invocation captured by [`RecordingRouter`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RouterCall {
    /// Session context forwarded by the adapter.
    pub context: SessionContext,
    /// Command issued by the adapter.
    pub command: RouterCommand,
}

/// Simple in-memory router for integration testing.
#[derive(Debug, Default)]
pub struct RecordingRouter {
    calls: Mutex<Vec<RouterCall>>,
    scripted_responses: Mutex<Vec<Result<RouterResponse, RouterError>>>,
}

impl RecordingRouter {
    /// Queue the response that should be returned for the next dispatch call.
    pub async fn script_response(&self, response: Result<RouterResponse, RouterError>) {
        self.scripted_responses.lock().await.push(response);
    }

    /// Retrieve the calls recorded so far.
    pub async fn calls(&self) -> Vec<RouterCall> {
        self.calls.lock().await.clone()
    }

    /// Clear recorded calls.
    pub async fn clear(&self) {
        self.calls.lock().await.clear();
        self.scripted_responses.lock().await.clear();
    }
}

#[async_trait]
impl CommandRouter for RecordingRouter {
    async fn dispatch(
        &self,
        ctx: SessionContext,
        command: RouterCommand,
    ) -> Result<RouterResponse, RouterError> {
        self.calls.lock().await.push(RouterCall {
            context: ctx.clone(),
            command: command.clone(),
        });

        let mut scripted = self.scripted_responses.lock().await;
        if scripted.is_empty() {
            return Ok(RouterResponse {
                status_code: 200,
                payload: Value::Null,
                diagnostics: vec!["default-script".into()],
            });
        }

        scripted.remove(0)
    }
}

/// Shared pointer helper for adapters.
pub type SharedRouter = Arc<dyn CommandRouter>;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;

    struct CapabilityRouter {
        required: HashMap<String, Vec<String>>,
    }

    impl CapabilityRouter {
        fn new() -> Self {
            let mut required = HashMap::new();
            required.insert("search".into(), vec!["search".into()]);
            required.insert("ingest".into(), vec!["ingest".into()]);
            required.insert("admin.reset".into(), vec!["admin".into(), "write".into()]);
            Self { required }
        }

        fn has_required(&self, command: &str, caps: &[String]) -> bool {
            self.required
                .get(command)
                .map(|req| req.iter().all(|cap| caps.contains(cap)))
                .unwrap_or(false)
        }
    }

    #[async_trait]
    impl CommandRouter for CapabilityRouter {
        async fn dispatch(
            &self,
            ctx: SessionContext,
            command: RouterCommand,
        ) -> Result<RouterResponse, RouterError> {
            if !self.has_required(&command.name, &ctx.capabilities) {
                return Err(RouterError::Unauthorized {
                    detail: format!(
                        "command '{}' requires capabilities not granted to principal '{}'",
                        command.name, ctx.principal
                    ),
                });
            }
            Ok(RouterResponse::ok(json!({ "executed": command.name })))
        }
    }

    #[tokio::test]
    async fn dispatch_enforces_capability_requirements() {
        let router = CapabilityRouter::new();

        let ctx_ok = SessionContext::new("alice", vec!["search".into()]);
        let cmd_ok = RouterCommand::new("search", json!({ "term": "docs" }));
        let response_ok = router
            .dispatch(ctx_ok.clone(), cmd_ok.clone())
            .await
            .expect("dispatch succeeds with required capability");
        assert_eq!(response_ok.status_code, 200);
        assert_eq!(response_ok.payload["executed"], json!("search"));

        let ctx_missing = SessionContext::new("bob", vec!["read".into()]);
        let err_missing = router
            .dispatch(ctx_missing, cmd_ok.clone())
            .await
            .expect_err("dispatch should fail without capability");
        assert!(matches!(err_missing, RouterError::Unauthorized { .. }));
        assert_eq!(err_missing.status_code(), 401);

        let ctx_partial = SessionContext::new("charlie", vec!["admin".into()]);
        let cmd_admin = RouterCommand::new("admin.reset", json!({}));
        let err_partial = router
            .dispatch(ctx_partial, cmd_admin.clone())
            .await
            .expect_err("missing secondary capability should fail");
        assert!(matches!(err_partial, RouterError::Unauthorized { .. }));

        let ctx_full = SessionContext::new("admin", vec!["admin".into(), "write".into()]);
        let response_full = router
            .dispatch(ctx_full, cmd_admin)
            .await
            .expect("dispatch succeeds with all capabilities");
        assert_eq!(response_full.status_code, 200);
        assert_eq!(response_full.payload["executed"], json!("admin.reset"));
    }
}
