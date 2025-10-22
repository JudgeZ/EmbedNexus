use runtime_router::{RecordingRouter, RouterError, RouterResponse};
use runtime_transport_http::{HttpAdapter, HttpConfig, HttpRequest, TransportError as HttpError};
use runtime_transport_stdio::{StdioAdapter, StdioConfig, TransportError as StdioError};
use runtime_transport_uds::{
    PeerCredentials, TransportError as UdsError, UdsAdapter, UdsConfig, UdsRequest,
};
use serde_json::json;
use std::sync::Arc;

fn http_config() -> HttpConfig {
    HttpConfig {
        host: "127.0.0.1".into(),
        port: 9443,
        tls_required: true,
        allowed_principals: vec!["alice".into()],
        token_secret: "integration-http".into(),
        require_csrf: true,
    }
}

fn stdio_config() -> StdioConfig {
    StdioConfig {
        max_frame_length: 4096,
        allowed_principals: vec!["alice".into()],
        token_secret: "integration-stdio".into(),
    }
}

fn uds_config() -> UdsConfig {
    UdsConfig {
        socket_path: "/tmp/runtime-integration.sock".into(),
        allowed_principals: vec!["alice".into()],
        allowed_uids: vec![1000],
        token_secret: "integration-uds".into(),
    }
}

fn peer() -> PeerCredentials {
    PeerCredentials {
        uid: 1000,
        pid: 123,
        process_name: "integration-test".into(),
    }
}

#[tokio::test]
async fn http_end_to_end_and_csrf_failure() {
    let router = Arc::new(RecordingRouter::default());
    router
        .script_response(Ok(RouterResponse::ok(json!({ "ok": true }))))
        .await;

    let adapter = HttpAdapter::bind(http_config(), router.clone() as _).unwrap();
    let token = adapter
        .issue_session_token("alice", &["ingest".into()])
        .expect("token issuance works");

    let request = HttpRequest::new(
        "POST",
        "/commands/ingest",
        json!({ "command": "ingest", "payload": {"id": 1} }),
    )
    .with_header("Authorization", format!("Bearer {}", token.token))
    .with_header("X-Csrf-Token", token.csrf_nonce.clone());

    let response = adapter.dispatch(request).await.expect("dispatch ok");
    assert_eq!(response.status, 200);

    let calls = router.calls().await;
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].command.name, "ingest");

    let bad_request = HttpRequest::new("POST", "/commands/ingest", json!({ "command": "ingest" }))
        .with_header("Authorization", format!("Bearer {}", token.token));

    let err = adapter
        .dispatch(bad_request)
        .await
        .expect_err("missing csrf");
    assert!(matches!(err, HttpError::Csrf(_)));
}

#[tokio::test]
async fn stdio_and_uds_authentication_failures_surface() {
    let router = Arc::new(RecordingRouter::default());
    router
        .script_response(Err(RouterError::Unauthorized {
            detail: "denied".into(),
        }))
        .await;
    router
        .script_response(Err(RouterError::Unauthorized {
            detail: "denied".into(),
        }))
        .await;

    let stdio = StdioAdapter::bind(stdio_config(), router.clone() as _).unwrap();
    let uds = UdsAdapter::bind(uds_config(), router.clone() as _).unwrap();

    let stdio_token = stdio
        .issue_session_token("alice")
        .expect("token issuance works");
    let frame = stdio
        .codec()
        .encode(&json!({ "command": "ingest", "payload": {} }), &stdio_token)
        .expect("frame encode");

    let err = stdio
        .dispatch_frame(frame)
        .await
        .expect_err("router unauthorized should bubble");
    assert!(matches!(err, StdioError::Router(_)));
    assert_eq!(err.router_status_code(), Some(401));

    uds.negotiate_peer(&peer())
        .expect("peer negotiation should succeed");
    let uds_token = uds
        .issue_session_token("alice", &["search".into()])
        .expect("uds token");
    let request = UdsRequest::new(
        peer(),
        uds_token.token.clone(),
        json!({
            "command": "search",
            "payload": {}
        }),
    );
    let err = uds
        .dispatch(request)
        .await
        .expect_err("router error surfaces");
    assert!(matches!(err, UdsError::Router(_)));
    assert_eq!(err.router_status_code(), Some(401));

    let mut bad_peer = peer();
    bad_peer.uid = 77;
    let auth_err = uds.negotiate_peer(&bad_peer).expect_err("bad uid rejected");
    assert!(matches!(auth_err, UdsError::Unauthorized(_)));
}
