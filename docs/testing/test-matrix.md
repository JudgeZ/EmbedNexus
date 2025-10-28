# Test Matrix for Planned Feature Work

This matrix catalogues the intentionally failing tests that must be in place before implementation begins on each subsystem. The objective is to drive test-first delivery so that every planned capability is protected by unit, integration, fuzz, and performance coverage ahead of landing code changes. Refer to the [Fixture and Golden Data Management Plan](./fixtures-plan.md) for guidance on creating and reviewing the shared assets referenced below.

## Subsystem Matrices

### Embedding Engine Core
- **Planned feature focus**: Adaptive chunking pipeline, multimodal embedding expansion.
- **Required failing tests**:
  - **Unit** – tokenizer boundary handling and embedding dimensionality negotiation helpers.
  - **Integration** – ingestion pipeline end-to-end execution with adaptive chunking enabled.
  - **Fuzz** – malformed document payloads targeting tokenizer and encoder surfaces.
  - **Performance** – throughput regression guard covering a 10k document batch window.

### Secure Storage & Retrieval
- **Planned feature focus**: Encrypted vector persistence and rotated key support.
- **Required failing tests**:
  - **Unit** – key rotation helpers and encryption envelope edge cases.
  - **Integration** – encrypted round-trip search across sharded stores.
  - **Fuzz** – randomized key schedules and ciphertext tampering inputs.
  - **Performance** – latency guard on rotated index rebuild cycles.
- **Coverage added (2025-10-28)** – Unit tests `capacity_eviction_removes_oldest_entry_fifo`, `purges_entries_exceeding_max_age`, and `requeue_preserves_original_age_for_expiration` in `crates/storage-ledger/src/lib.rs` validate offline replay buffer eviction and expiry semantics to support Phase 1 transport→ledger workflows.
- **Coverage added (2025-10-29)** – Phase 2 tests `requeue_after_partial_flush_maintains_sequence` and `drain_ready_concurrent_push_preserves_order` assert replay ordering when partial flushes requeue entries and new pushes occur concurrently.

### Client Tooling (CLI & SDK)
- **Planned feature focus**: Offline sync script and custom embedding hooks.
- **Required failing tests**:
  - **Unit** – CLI flag parsing for embedding hooks and SDK request builders.
  - **Integration** – offline sync against staging MCP endpoints using shared fixtures.
  - **Fuzz** – CLI argument permutations leveraging the fixture corpus.
  - **Performance** – sync duration guard leveraging the large fixture dataset.

### IDE Extensions
- **Planned feature focus**: Inline suggestion surfacing and MCP golden message replay.
- **Required failing tests**:
  - **Unit** – adapter transformers for IDE protocol messages.
  - **Integration** – replay of golden MCP conversations against the extension bridge.
  - **Fuzz** – randomized suggestion payload ordering via golden transcript mutators.
  - **Performance** – response time guard for multi-file suggestion batches.
- **Security traceability** – Aligns with the [Input Validation Checklist](../security/threat-model.md#input-validation-checklist) for protocol payload vetting.

### Filesystem Watch Service
- **Planned feature focus**: Workspace filesystem watching latency with debounce and ignore rules.
- **Measurement template**: [Filesystem Watcher Latency Metrics](./templates/watcher-latency-metrics.md) for capturing watcher timing evidence.
- **Required failing tests**:
  - **Unit** – latency window heuristics for debounce helpers using the future fixture `tests/fixtures/filesystem/latency-window.yaml`.
  - **Integration** – propagation latency from watch events into ingestion queues replayed via `tests/fixtures/filesystem/workspace-replay/` with added clock skew controls.
  - **Fuzz** – burst latency permutations sourced from the golden transcript `tests/golden/filesystem/watch-latency-burst.log` to validate ignore glob safety under stress.
  - **Performance** – sustained burst benchmark asserting <200 ms median watch-to-queue latency using the upcoming dataset `tests/golden/filesystem/watch-latency-burst.log`.
- **Traceability** – Aligns with the [Ingestion Pipeline Specification](../design/ingestion.md) for watcher orchestration and the [Sandboxing](../security/threat-model.md#sandboxing-checklist) and [Input Validation](../security/threat-model.md#input-validation-checklist) checklists governing event sources.

### Archive Extraction Quotas
- **Planned feature focus**: Controlled archive ingestion with byte quotas, type filters, and extraction latency limits.
- **Measurement template**: [Archive Extraction Usage](./templates/archive-extraction-usage.md) for documenting quota and latency metrics.
- **Required failing tests**:
  - **Unit** – quota calculators and MIME gatekeepers validated against `tests/fixtures/archives/quota-latency.toml`.
  - **Integration** – staged overflow extraction from `tests/fixtures/archives/overflow-latency.tar.zst` asserting quota violations and latency logging.
  - **Fuzz** – randomized archive metadata and clock skew injections driven by `tests/golden/archives/quota-throughput.jsonl` to stress parser hardening.
  - **Performance** – throughput guard ensuring quota enforcement overhead stays within 2% using the bulk corpus `tests/fixtures/archives/bulk-sample/` and latency checkpoints emitted to `tests/golden/archives/quota-throughput.jsonl`.
- **Traceability** – Anchored to the [Ingestion Pipeline Specification](../design/ingestion.md#cross-cutting-concerns) for resource controls and the [Input Validation](../security/threat-model.md#input-validation-checklist) and [Sandboxing](../security/threat-model.md#sandboxing-checklist) checklists.

### Encryption & TLS Controls
- **Planned feature focus**: Toggleable encryption-at-rest policies and deterministic TLS transport negotiation.
- **Measurement template**: [Encryption & TLS Validation Logging](./templates/encryption-tls-validation.md) for recording encryption and handshake evidence.
- **Required failing tests**:
  - **Encryption-at-rest unit** – key rotation toggle helpers verified with the fixture `tests/fixtures/security/encryption-latency.json`.
  - **Encryption-at-rest integration** – encrypted store rebuild with toggle permutations replaying `tests/golden/security/encryption-toggle.trace`.
  - **Encryption-at-rest fuzz/performance** – randomized toggle sequences with rebuild timing captured in `tests/golden/security/encryption-toggle.trace` to ensure <5% latency regression.
  - **TLS unit** – cipher-suite negotiation validators referencing `tests/fixtures/security/tls-config-matrix.yaml` for coverage of mandatory/optional suites.
  - **TLS integration** – end-to-end handshake negotiation using the golden transcript `tests/golden/security/tls-negotiation.trace` across downgraded clients.
  - **TLS fuzz/performance** – fuzzed handshake transcripts and throughput guards sourced from `tests/golden/security/tls-performance.jsonl` ensuring downgrade protection and handshake latency targets.
  - **WSL transport handshake regression** – failing integration coverage replaying `tests/golden/transport/wsl-handshake-negotiation.trace` across the Windows loopback proxy to confirm telemetry parity with native Linux/macOS adapters and to validate DPAPI-backed key recovery requirements before WSL session reuse.
  - **Encrypted storage DPAPI recovery** – failing unit and integration coverage leveraging `tests/fixtures/security/dpapi-recovery/` and the golden event log `tests/golden/security/dpapi-recovery-audit.jsonl` to assert that encrypted shards restored inside WSL honor the Windows DPAPI recovery policy prior to re-keying.
- **Traceability** – References the [Encryption Design](../design/encryption.md) for storage toggles, the [Transport Adapter Design](../design/transport.md) for negotiation sequencing, and the [Encryption](../security/threat-model.md#encryption-checklist) plus [Input Validation](../security/threat-model.md#input-validation-checklist) checklists.

### Multi-Repository Routing
- **Planned feature focus**: Routing embeddings and retrieval across federated repository workspaces.
- **Required failing tests**:
  - **Unit** – routing table merge helpers referencing the planned fixture `tests/fixtures/routing/latency-matrix.json`.
  - **Integration** – cross-repo retrieval latency validated by `tests/golden/routing/multi-repo-latency.transcript` to confirm tenant isolation.
  - **Fuzz** – randomized repository affinity hints derived from `tests/golden/routing/fuzz-affinity.jsonl` with jitter injection.
  - **Performance** – routing throughput guard validating scheduler latency across the scenario pack `tests/golden/routing/fanout-throughput.jsonl` and fixture directory `tests/fixtures/routing/high-fanout/`.
- **Traceability** – Tied to the [Architecture Overview](../design/overview.md#local-ingestion-pipeline) for multi-repo orchestration and the [Sandboxing](../security/threat-model.md#sandboxing-checklist) and [Encryption](../security/threat-model.md#encryption-checklist) checklists governing cross-tenant routing.
- **Coverage added (2025-10-29)** – Routing matrix loader tests `routing_matrix_merges_latency_fixture` and `routing_matrix_aligns_with_latency_transcript` confirm adjacency/shortest path calculations for the multi-repo fixtures/transcripts.

### WSL Multi-Repository Regressions
- **Planned feature focus**: Windows-to-WSL indexing parity, encrypted persistence, and ignore rule reconciliation across NTFS/ext4 mounts.
- **Required failing tests**:
  - **Unit** – WSL path normalization and ignore precedence helpers validated against the upcoming fixture `tests/fixtures/wsl/multi-repo-ignore.yaml`.
  - **Integration** – dual-repository ingestion replay driven by the golden event log `tests/golden/wsl/multi-repo-ingestion.log`, asserting deterministic manifest stitching and encrypted store segregation.
  - **Fuzz** – randomized mount remapping and clock skew payloads sourced from `tests/golden/wsl/wsl-remap-permutations.jsonl`.
  - **Performance** – throughput guard referencing `tests/golden/wsl/multi-repo-throughput.jsonl` that enforces budgeted latency for cross-filesystem diffing.
- **Traceability** – Aligns with the [Runtime Packaging & Distribution Plan](../implementation/runtime-packaging-plan.md) for consuming packaged artifacts during WSL validation, the [Encryption](../security/threat-model.md#encryption-checklist) checklist for DPAPI parity, and the [Sandboxing](../security/threat-model.md#sandboxing-checklist) checklist for bridge execution.
- **Automation mandate** – All fixtures and goldens listed above are regenerated exclusively by the `Regenerate Fixture Corpus` and `Regenerate Golden Artifacts` GitHub Actions workflows; developers must not hand-edit the assets locally.

### Offline Resilience & Replay
- **Planned feature focus**: Transport retry buffering during network isolation and deterministic ingestion manifest replays after storage recovery.
- **Required failing tests**:
- **Transport retry buffer integration** – Exercise adapter-level retry queue persistence with air-gapped transport harnesses referenced in [Transport Adapter Specification](../design/transport.md#offline-backpressure).
- **Retry buffer fuzz** – Burst enqueue/dequeue permutations sourced from `tests/fixtures/transport/offline-queue/` and `tests/golden/transport/offline-buffer-replay.transcript` to validate bounded growth and telemetry integrity.
- **Manifest replay integration** – Simulate delayed storage availability by applying the ingestion replay harness in [Ingestion Pipeline Specification](../design/ingestion.md#offline-replay-hooks) with datasets `tests/fixtures/ingestion/delayed-ledger/` and transcripts `tests/golden/ingestion/manifest-replay.log`.
- **Performance** – Measure replay catch-up latency across the delayed-storage fixtures while asserting audit log ordering guarantees.
- **Security traceability** – Aligns with the [Sandboxing](../security/threat-model.md#sandboxing-checklist), [Access Control](../security/threat-model.md#access-control-checklist), and [Encryption](../security/threat-model.md#encryption-checklist) checklists to ensure buffered data remains protected during offline windows.
- **Coverage added (2025-10-29)** – Ingestion manifest tests `flush_offline_handles_queue_failure_mid_flush` and `emit_during_active_flush_queues_follow_up` exercise emitter backpressure/resume behaviour while relying on existing `TestQueue` failover logic.
- **Coverage added (2025-10-30)** – STDIO retry buffer unit tests (`retry_buffer_enforces_capacity_fifo`, `retry_buffer_requeue_retains_order`) and integration coverage (`tests/runtime_transport/tests/offline_queue.rs`) replay queued frames from `tests/fixtures/transport/offline-queue/snapshot.jsonl`.
- **Coverage added (2025-10-28)** – HTTP adapter tests `issue_session_token_records_telemetry_and_claims`, `issue_session_token_rejects_disallowed_principal`, `dispatch_rejects_expired_token`, and `dispatch_propagates_capabilities_to_router` in `crates/runtime-transport-http/src/lib.rs` assert token issuance/expiry handling, auth-failure telemetry, and capability propagation for the Phase 1 transport spine.
- **Coverage added (2025-10-29)** – Ingestion manifest tests `flush_offline_handles_queue_failure_mid_flush` and `emit_during_active_flush_queues_follow_up` exercise emitter backpressure/resume behaviour while relying on existing `TestQueue` failover logic.
- **Coverage added (2025-10-29)** – STDIO adapter tests `issue_session_token_records_telemetry` and `dispatch_rejects_expired_token` bring parity with HTTP auth lifecycle checks and guard against stale tokens.

## Shared Testing Assets

- **Fixture Library**: Reuse the shared fixture bundle under `tests/fixtures/shared/` for cross-subsystem consistency. Extend fixtures instead of duplicating data and document each addition inside the fixture README files.
- **Golden MCP Messages**: Client scripts and IDE validation must rely on the curated golden message transcripts in `tests/golden/mcp/`. Update transcripts when protocol changes occur and ensure replay harnesses remain deterministic.

## Automation & Regeneration Workflows

The GitHub Actions workflows under `.github/workflows/` codify the regeneration
sequences captured in the fixture plan. Reference their outputs when updating the
matrix entries above, and follow the [automation runbook](./fixtures-plan.md#automation-runbook)
for evidence collection and documentation.

| Workflow | Toolchain Baseline | Coverage | Notes |
| --- | --- | --- | --- |
| `Regenerate Fixture Corpus` | Ubuntu runner, Python 3.11 (`watchdog`, `pyyaml`, `typer`, `rich`, `click`, `cryptography`, `networkx`), Rust stable, `openssl`, `tshark`, `jq`, `shellcheck`, `shfmt`, `zstd` | Filesystem, archives, routing, transport, ingestion, and shared fixture directories. Produces `fixture-regeneration-output` artifact with refreshed `tests/fixtures/` content plus adjacent goldens. | Includes a Windows job that runs `scripts/collect_dpapi.ps1` and `scripts/trace_capture.sh` to stage DPAPI recovery fixtures, TLS latency samples, WSL bridge metadata, and the multi-repo datasets (`wsl-multi-repo-ignore.yaml`, etc.) before the Linux job downloads the `windows-security-fixtures` artifact and completes the remaining regeneration steps. |
| `Regenerate Golden Artifacts` | Same as above | Regenerates transcripts and logs under `tests/golden/`, uploading the `golden-regeneration-output` artifact. | Windows automation executes `collect_dpapi.ps1`, `trace_capture.sh`, and `wsl_transport_proxy.ps1` to refresh TLS handshakes, negotiation traces, DPAPI audits, WSL bridge captures, and the new multi-repo ingestion/throughput transcripts. The Linux job consumes the `windows-security-goldens` artifact and then rebuilds the remaining goldens plus checksum manifests. |

**Verification tracking**

- Download the workflow artifacts and run `scripts/checksums.sh --verify` locally
  once the helper is implemented, recording the verification transcript referenced
  in the runbook above.
- Matrix updates must capture, in both subsystem README updates and the associated
  pull request description/template, the workflow run URL, every published artifact
  name, and the checksum verification output. Attach this metadata bundle as part
  of the documentation changes so reviewers can trace automation evidence without
  leaving the change set.

## Execution Notes

1. Add each failing test to the appropriate suite before feature implementation begins. Reference the planned feature label in the test description to maintain traceability.
2. Link new test coverage to existing CI job definitions so that failures surface immediately while the features remain pending.
3. Document deviations or temporary skips inside the subsystem-specific testing READMEs with rationale and target removal milestones.
4. Capture and archive the failing-then-passing test evidence (local logs or CI links) so reviewers can validate the TDD red/green cycle when the implementation lands.
