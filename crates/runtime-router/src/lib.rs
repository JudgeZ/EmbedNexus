//! Runtime command router contract and lightweight testing utilities.

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    #[must_use]
    pub const fn ok(payload: Value) -> Self {
        Self {
            status_code: 200,
            payload,
            diagnostics: Vec::new(),
        }
    }
}

/// Router errors mapped back to transport adapters.
#[derive(Debug, Clone, Error, PartialEq, Eq, Serialize, Deserialize)]
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
    #[must_use]
    pub const fn status_code(&self) -> u16 {
        match self {
            Self::Unauthorized { .. } => 401,
            Self::InvalidRequest { .. } => 400,
            Self::NotFound { .. } => 404,
            Self::Internal { .. } => 500,
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

/// Routing matrix describing cross-repository adjacency and weights.
#[derive(Debug, Clone)]
pub struct RoutingMatrix {
    adjacency: HashMap<String, HashMap<String, u64>>,
}

impl RoutingMatrix {
    pub fn from_reader<R: Read>(reader: R) -> Result<Self, RoutingMatrixError> {
        let parsed: SerializedMatrix = serde_json::from_reader(reader)?;
        Ok(Self {
            adjacency: parsed.adjacency,
        })
    }

    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, RoutingMatrixError> {
        let file = File::open(path)?;
        Self::from_reader(file)
    }

    #[must_use]
    pub fn weight(&self, from: &str, to: &str) -> Option<u64> {
        self.adjacency
            .get(from)
            .and_then(|edges| edges.get(to).copied())
    }

    #[must_use]
    pub fn nodes(&self) -> HashSet<String> {
        let mut nodes: HashSet<String> = self.adjacency.keys().cloned().collect();
        for edges in self.adjacency.values() {
            nodes.extend(edges.keys().cloned());
        }
        nodes
    }

    #[must_use]
    pub fn shortest_path(&self, start: &str, end: &str) -> Option<Vec<String>> {
        if start == end {
            return Some(vec![start.to_string()]);
        }

        let mut dist: HashMap<&str, u64> = HashMap::new();
        let mut prev: HashMap<&str, &str> = HashMap::new();
        let mut heap: BinaryHeap<(Reverse<u64>, &str)> = BinaryHeap::new();

        dist.insert(start, 0);
        heap.push((Reverse(0), start));

        while let Some((Reverse(cost), node)) = heap.pop() {
            if node == end {
                break;
            }
            if Some(&cost) > dist.get(node) {
                continue;
            }
            if let Some(neighbors) = self.adjacency.get(node) {
                for (next, weight) in neighbors {
                    let next_cost = cost + *weight;
                    let entry = dist.entry(next).or_insert(u64::MAX);
                    if next_cost < *entry {
                        *entry = next_cost;
                        prev.insert(next, node);
                        heap.push((Reverse(next_cost), next));
                    }
                }
            }
        }

        if !dist.contains_key(end) {
            return None;
        }

        let mut path = Vec::new();
        let mut current = end;
        path.push(current.to_string());
        while let Some(&p) = prev.get(current) {
            current = p;
            path.push(current.to_string());
        }
        if path.last().map(std::string::String::as_str) != Some(start) {
            return None;
        }
        path.reverse();
        Some(path)
    }

    #[must_use]
    pub fn edge_count(&self) -> usize {
        self.adjacency
            .values()
            .map(std::collections::HashMap::len)
            .sum()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RoutingMatrixError {
    #[error("failed to read matrix: {0}")]
    Io(#[from] io::Error),
    #[error("failed to parse matrix json: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Deserialize)]
struct SerializedMatrix {
    adjacency: HashMap<String, HashMap<String, u64>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;
    use std::fs;
    use std::path::PathBuf;

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
                .is_some_and(|req| req.iter().all(|cap| caps.contains(cap)))
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

    #[test]
    fn routing_matrix_merges_latency_fixture() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("tests")
            .join("fixtures")
            .join("routing")
            .join("multi-repo-matrix.json");
        let matrix = RoutingMatrix::from_file(&path).expect("matrix loads");
        assert_eq!(matrix.edge_count(), 5);
        assert!(matrix.weight("ingest-api", "routing-control").is_some());
        assert!(matrix.weight("routing-control", "artifact-store").is_some());
        assert_eq!(matrix.nodes().len(), 5);
    }

    #[test]
    fn routing_matrix_aligns_with_latency_transcript() {
        let base_tests = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("tests");
        let matrix_path = base_tests
            .join("fixtures")
            .join("routing")
            .join("multi-repo-matrix.json");
        let transcript_path = base_tests
            .join("golden")
            .join("routing")
            .join("multi-repo-latency.transcript");

        let matrix = RoutingMatrix::from_file(&matrix_path).expect("matrix loads");
        let transcript = fs::read_to_string(transcript_path).expect("transcript loads");

        for line in transcript.lines() {
            if let Some((from, rest)) = line.split_once(" -> ") {
                let mut parts = rest.split_whitespace();
                if let Some(to) = parts.next() {
                    assert!(
                        matrix.weight(from, to).is_some(),
                        "expected edge {from} -> {to} in matrix"
                    );
                }
            }
        }

        let path = matrix
            .shortest_path("ingest-api", "audit-log")
            .expect("path exists");
        assert_eq!(
            path,
            vec![
                "ingest-api",
                "routing-control",
                "artifact-store",
                "audit-log"
            ]
        );
    }
}
