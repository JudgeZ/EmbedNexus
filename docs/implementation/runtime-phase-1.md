# Runtime Implementation — Phase 1 Milestone

This milestone translates the Level 4 code view into an executable Rust workspace skeleton. It focuses on creating the multi-crate workspace, scaffolding the module crates aligned with the runtime and ingestion design narratives, and capturing prerequisite configuration that would otherwise block contributors.

## Milestone Scope

1. **Workspace bootstrapping**
   - Create a top-level `Cargo.toml` declaring a virtual workspace and shared `[workspace.dependencies]`.
   - Define workspace members for each Phase 1 crate.
   - Establish common lint and formatting settings (`rustfmt.toml`, `clippy.toml`) as shared configuration.
2. **Crate scaffolding**
   - Generate library crates for the runtime transports, ingestion core, storage layer, and governance utilities identified in the [Level 4 — Code View](../design/c4/level-4-code/README.md#key-modules-and-responsibilities).
   - Provide crate-level `lib.rs` stubs exposing module namespaces that mirror the design names (e.g., `runtime::transport::http` → `crates/runtime-transport-http`).
   - Add placeholder integration test harnesses under each crate’s `tests/` directory to receive the failing tests mandated by the [Test Matrix](../testing/test-matrix.md).
3. **Blocking configuration**
   - Pin the Rust toolchain in `rust-toolchain.toml` to the project-approved channel (Rust 1.76+) for deterministic builds.
   - Document feature flags that must exist in `Cargo.toml` (e.g., `transport-http`, `transport-stdio`, `storage-encrypted`, `governance-audit`) to support conditional compilation called out in the [Transport Adapter Specification](../design/transport.md) and [Vector Store Specification](../design/vector-store.md).
   - Capture security-critical default settings (enforced TLS cipher matrix, DPAPI-backed key recovery toggles, sandboxing guards) so they remain visible before implementation, referencing the [Security Review Checklists](../security/threat-model.md#security-review-checklists).

## Ordered Work Items and Dependencies

### 1. Scaffold Transport Layer Crates
- **Deliverables**: `runtime-transport-http`, `runtime-transport-stdio`, and `runtime-transport-uds` crates with `lib.rs`, feature flags, and placeholder adapters (`HttpAdapter`, `FramingCodec`, `UdsAdapter`).
- **Design references**: [Transport Adapter Specification](../design/transport.md#module-responsibilities) and [Level 4 — Code View](../design/c4/level-4-code/README.md#key-modules-and-responsibilities).
- **Security checklists**: [Input Validation](../security/threat-model.md#input-validation-checklist), [Encryption](../security/threat-model.md#encryption-checklist) for TLS requirements, and [Sandboxing](../security/threat-model.md#sandboxing-checklist) for IPC boundaries.
- **Dependencies**: Requires workspace manifest in place and `rust-toolchain.toml` defined. No upstream crate dependencies beyond shared types crate stub.

### 2. Scaffold Ingestion Core Crates
- **Deliverables**: `ingestion-workspace`, `ingestion-planning`, `ingestion-sanitization`, `ingestion-embedding`, and `ingestion-manifest` crates with module stubs mirroring `WorkspaceEnumerator`, `ChunkPlanner`, `Sanitizer`, `EmbeddingGenerator`, and `ManifestEmitter`.
- **Design references**: [Ingestion Pipeline Specification](../design/ingestion.md#module-responsibilities) and [Level 4 — Code View](../design/c4/level-4-code/README.md#key-modules-and-responsibilities).
- **Security checklists**: [Input Validation](../security/threat-model.md#input-validation-checklist), [Sandboxing](../security/threat-model.md#sandboxing-checklist), and [File Handling](../security/threat-model.md#file-handling-checklist) for workspace enumeration and archive processing.
- **Dependencies**: Builds on the workspace manifest and shared ingestion types defined during transport scaffolding; depends on transport crate feature flags for CLI/stdin ingestion triggers.

### 3. Scaffold Storage Layer Crates
- **Deliverables**: `storage-vector` and `storage-ledger` crates exposing `VectorStore` and `LedgerWriter` with encryption-aware configuration stubs and feature toggles for `storage-encrypted` and `storage-ledger-audit`.
- **Design references**: [Vector Store Specification](../design/vector-store.md#storage-layout) and [Encryption Design](../design/encryption.md#key-management-overview).
- **Security checklists**: [Encryption](../security/threat-model.md#encryption-checklist), [Key Management](../security/threat-model.md#key-management-checklist), and [Access Control](../security/threat-model.md#access-control-checklist).
- **Dependencies**: Requires ingestion crate manifests (for data model structs) and the shared security configuration recorded during transport and ingestion scaffolding.

### 4. Scaffold Governance & Routing Crates
- **Deliverables**: `runtime-router`, `runtime-policy`, `governance-audit`, and `governance-traceability` crates with stubbed `CommandRouter`, `PolicyEngine`, `AuditWriter`, and `TraceabilitySync` modules, plus integration test harnesses.
- **Design references**: [Architecture Overview](../design/overview.md#finalized-architecture-overview), [Traceability Index](../design/traceability.md#traceability-map), and [Level 4 — Code View](../design/c4/level-4-code/README.md#key-modules-and-responsibilities).
- **Security checklists**: [Authentication](../security/threat-model.md#authentication-checklist), [Access Control](../security/threat-model.md#access-control-checklist), and [Audit Logging](../security/threat-model.md#file-handling-checklist) to capture telemetry retention controls.
- **Dependencies**: Builds atop transport, ingestion, and storage crate manifests so routing traits can reference their interfaces. Requires governance feature flags and shared policy configuration emitted earlier.

### Dependency Flow Summary
1 → 2 → 3 → 4 ensures transports exist before ingestion hooks reference them; ingestion crates precede storage scaffolding to supply data models; storage scaffolding precedes governance so router traits can import storage contracts.

## TDD Entry Criteria

Each work item must begin by introducing the failing tests described below. Contributors should commit the failing tests with a minimal stub implementation that ensures the workspace builds. All test evidence must be captured in the [PR Release Checklist](../process/pr-release-checklist.md) before merge.

| Work Item | Required Failing Tests | Source Guidance | Notes |
| --- | --- | --- | --- |
| Transport Layer Crates | - Unit: TLS negotiation validators referencing `tests/fixtures/security/tls-config-matrix.yaml`.<br>- Integration: Offline retry buffer harness from `tests/golden/transport/offline-buffer-replay.transcript`.<br>- Fuzz/Perf: WSL handshake trace in `tests/golden/transport/wsl-handshake-negotiation.trace`. | [Test Matrix — Encryption & TLS Controls](../testing/test-matrix.md#encryption--tls-controls) and [Offline Resilience & Replay](../testing/test-matrix.md#offline-resilience--replay). | Ensure CI captures failing transport tests before adapter implementations land. |
| Ingestion Core Crates | - Unit: Workspace enumeration fixtures in `tests/fixtures/filesystem/` (latency window YAML).<br>- Integration: Manifest replay harness `tests/golden/ingestion/manifest-replay.log`.<br>- Fuzz/Perf: Archive quota datasets `tests/golden/archives/quota-throughput.jsonl`. | [Test Matrix — Filesystem Watch Service](../testing/test-matrix.md#filesystem-watch-service) and [Archive Extraction Quotas](../testing/test-matrix.md#archive-extraction-quotas). | Tests must assert deterministic chunk planning and quota enforcement prior to code implementation. |
| Storage Layer Crates | - Unit: Encryption toggle helpers using `tests/fixtures/security/encryption-latency.json`.<br>- Integration: Encrypted round-trip trace `tests/golden/security/encryption-toggle.trace`.<br>- Fuzz/Perf: DPAPI recovery assets `tests/fixtures/security/dpapi-recovery/`. | [Test Matrix — Secure Storage & Retrieval](../testing/test-matrix.md#secure-storage--retrieval). | Include Windows/WSL DPAPI recovery expectations in test descriptions. |
| Governance & Routing Crates | - Unit: Policy evaluation tables `tests/fixtures/routing/latency-matrix.json` (planned).<br>- Integration: Multi-repo latency transcript `tests/golden/routing/multi-repo-latency.transcript`.<br>- Fuzz/Perf: Transport replay telemetry `tests/golden/transport/offline-buffer-replay.transcript` for audit ordering. | [Test Matrix — Multi-Repository Routing](../testing/test-matrix.md#multi-repository-routing) and [Offline Resilience & Replay](../testing/test-matrix.md#offline-resilience--replay). | Ensure governance tests assert audit log persistence before command router logic is implemented. |

## Contributor Expectations Before Coding

- **Planning alignment**: Validate this plan with the documentation review workflow in [doc-review.md](../process/doc-review.md) to secure sign-off prior to implementation.
- **PR checklist**: Every PR must attach the completed [PR Release Checklist](../process/pr-release-checklist.md), including evidence of failing tests (logs or CI links), security checklist review outcomes, and toolchain verification.
- **Security documentation**: Map each crate’s placeholder modules back to the relevant security checklist items in [threat-model.md](../security/threat-model.md) when opening the implementation PRs.
- **Governance logging**: Record the planning approval in the governance log per [governance-log.md](../process/governance-log.md) once reviewers sign off on Phase 1 scaffolding.

Following this milestone provides an executable plan that upholds the repository’s plan-before-code and TDD mandates while keeping design and security documentation synchronized.

### Security Defaults

To keep transport, storage, and governance scaffolding aligned with the threat model from the outset, the workspace enforces the following defaults:

- **TLS 1.3 cipher matrix** — HTTP and UDS transports must negotiate strictly within `TLS_AES_256_GCM_SHA384`, `TLS_CHACHA20_POLY1305_SHA256`, or `TLS_AES_128_GCM_SHA256`, matching the hardening controls captured in the [encryption checklist](../security/threat-model.md#encryption-checklist).
- **DPAPI recovery toggle** — Storage crates default to DPAPI-backed key recovery being disabled until explicitly approved, ensuring escrow flows satisfy the [key management checklist](../security/threat-model.md#key-management-checklist).
- **Sandbox guard rails** — Transports, ingestion sanitizers, and embedding adapters ship with process sandboxing enabled by default, in line with the [sandboxing checklist](../security/threat-model.md#sandboxing-checklist).
