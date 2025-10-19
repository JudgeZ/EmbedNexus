# Transport Adapter Specification

This module expands on the adapter responsibilities outlined in the [architecture overview](./overview.md), detailing how local clients bind to the runtime over HTTP, STDIO, and Unix domain socket transports.

## Module Responsibilities
- Provide authenticated, validated entry points for CLI, IDE, and automation clients.
- Normalize requests into the runtime command router contract while preserving per-transport telemetry.
- Enforce loopback-only exposure and enforceable session lifetimes to uphold offline-first guarantees.
- Surface structured response envelopes with deterministic error codes so downstream automation remains resilient.

## Public Interfaces

| Interface | Description | Inputs | Outputs |
|-----------|-------------|--------|---------|
| `TransportAdapter::bind(config)` | Initialize a transport endpoint based on configuration | Transport configuration (port/path, auth policy, timeouts) | Running listener handle, lifecycle hooks |
| `TransportAdapter::dispatch(request)` | Validate, authenticate, and route a client request | Raw protocol payload | Normalized runtime command + context |
| `TransportAdapter::shutdown()` | Gracefully stop listeners and flush audit logs | Shutdown reason | Confirmation of teardown + persisted audit pointers |
| `SessionToken::issue(principal, scope)` | Issue scoped session tokens for HTTP/UDS clients | Principal identity, requested capabilities | Signed session token |
| `FramingCodec::encode/::decode` | Frame STDIO payloads with checksum + length headers | Raw bytes | Structured payload (request or response) |

## Data Models
- **`TransportConfig`**: YAML/JSON schema referencing adapter type, bind target, allowed principals, retry budget, and telemetry sinks.
- **`SessionContext`**: Captures principal, capabilities, CSRF nonce (HTTP), or peer credentials (UDS), and tracing identifiers.
- **`RequestEnvelope`**: `{ transport_id, session, payload, received_at, retry_count }` forwarded to the command router.
- **`ResponseEnvelope`**: `{ transport_id, status_code, payload, emitted_at, diagnostics[] }` delivered back to clients.

## Sequencing

```mermaid
sequenceDiagram
    participant Client
    participant Adapter
    participant Auth as AuthN/Z
    participant Router as Command Router
    participant Audit

    Note over Client,Adapter: Preconditions: adapter bound, auth policy loaded, audit sink available
    Client->>Adapter: Initiate connection/request
    Adapter->>Auth: Validate session token / negotiate credentials
    Auth-->>Adapter: Auth result (scoped capabilities)
    Adapter->>Adapter: Enforce rate limits and payload size quotas
    Adapter->>Router: Dispatch normalized command with SessionContext
    Router-->>Adapter: Response payload or structured error
    Adapter->>Audit: Record request/response metadata
    Adapter-->>Client: Respond with ResponseEnvelope
    Note over Client,Adapter: Postconditions: telemetry flushed, audit entry persisted, connection state updated
```

## Preconditions & Postconditions
- **Preconditions**
  - Adapter listener is bound with loopback-only ACLs and validated configuration.
  - Authentication providers (token signer, peer credential verifier) are reachable.
  - Audit sink storage is writable and has sufficient quota.
- **Postconditions**
  - All accepted requests are logged with immutable identifiers and latency metrics.
  - Failed authentications produce structured error responses without leaking policy internals.
  - Session lifetimes are updated or revoked according to policy outcomes.

## Cross-Cutting Concerns
- **Error Handling**: Normalize transport-specific errors into `TransportError` codes; surface remediation hints and ensure retries respect idempotency.
- **Concurrency**: Use async executors per transport, isolating HTTP worker pools from STDIO single-flight handlers to prevent starvation.
- **Resource Limits**: Enforce per-transport connection caps, request body limits, and back-pressure thresholds aligning with the offline-first resource profile.
- **Security Alignment**: Enforce threat model mitigations by logging validation failures and mapping them to the [Input Validation Checklist](../security/threat-model.md#input-validation-checklist).

## Required Failing Tests
The following tests must be authored (and initially fail) as part of the transport subsystem enablement, referencing the [test matrix](../testing/test-matrix.md#client-tooling-cli--sdk):
- Unit tests covering HTTP session negotiation, STDIO framing edge cases, and UDS credential propagation.
- Integration tests exercising CLI, IDE, and automation clients against loopback-only transports with audit verification.
- Fuzz tests mutating request envelopes and session tokens to ensure adapters reject malformed inputs.
- Performance tests measuring sustained throughput under connection churn while maintaining latency SLOs.
