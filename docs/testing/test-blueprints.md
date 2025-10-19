# Test Blueprint to Matrix Mapping

This blueprint catalog extends the [test matrix](./test-matrix.md) by mapping every
planned entry to concrete `tests/<suite>/` locations, proposed test names, harness
types, fixtures, and CI expectations. Use it to bootstrap the failing coverage
required before feature implementation begins and to align security verification
with the [threat model checklists](../security/threat-model.md#security-review-checklists).

Each table below enumerates the blueprint rows for a subsystem. Unless otherwise
noted, harnesses should be authored as Rust integration tests under
`cargo test --test <suite>` with supporting helpers in the repository root.

## Embedding Engine Core

| Matrix focus | Test module | Test name | Harness | Fixtures & goldens | Initial failing assertion | CI job impact | Security checklist linkage |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Tokenizer boundary handling (unit) | `tests/unit/embedding/test_tokenizer_boundaries.rs` | `tokenizes_multilingual_boundaries` | `cargo test --test unit_embedding` | `tests/fixtures/shared/tokenizer-boundaries.yaml` (planned) | `assert_eq!(observed_windows, expected_windows)` – observed segments omit combining characters | `ci-unit` (`cargo test --test unit_embedding`) | Input Validation (payload size/sanitization)
| Embedding dimensionality negotiation (unit) | `tests/unit/embedding/test_dimension_negotiation.rs` | `rejects_mismatched_dimensions` | `cargo test --test unit_embedding` | `tests/fixtures/shared/dimension-contracts.json` | `assert!(handshake.supports(dim))` currently passes even when fixture declares incompatibility | `ci-unit` | Input Validation; Sandboxing (ensures negotiation guards parsing hooks)
| Adaptive chunking pipeline (integration) | `tests/integration/embedding/test_adaptive_chunking.rs` | `ingests_adaptive_chunks_with_backpressure` | `cargo test --test integration_embedding` | `tests/fixtures/shared/adaptive-chunking/`, `tests/golden/embedding/adaptive-chunking.log` | `assert_matches!(metrics.chunk_size_histogram, expected)` fails because instrumentation not wired | `ci-integration` (`cargo test --test integration_embedding`) | Sandboxing (ingestion worker isolation)
| Malformed document fuzzing (fuzz) | `tests/fuzz/embedding/fuzz_tokenizer_payloads.rs` | `fuzz_tokenizer_payloads` | `cargo fuzz run embedding_tokenizer_payloads` | `tests/fixtures/shared/tokenizer-malformed/` | Harness should panic with `ValidationError::RejectedPayload`; currently returns `Ok` | `ci-fuzz` (nightly) | Input Validation
| Throughput regression guard (performance) | `tests/perf/embedding/test_ingestion_throughput.rs` | `ensures_adaptive_pipeline_within_budget` | `cargo bench --bench embedding_throughput` | `tests/golden/embedding/perf-baseline.json` | `assert!(observed_qps >= baseline_qps)` fails because baseline not met | `ci-performance` (scheduled) | Sandboxing (resource isolation), Input Validation (guarding backpressure telemetry)

**Fixture prerequisites**: Populate adaptive chunking fixtures per the
[Fixture Plan](./fixtures-plan.md#shared-fixture-library). Link any generated logs
in `tests/golden/embedding/` with accompanying checksum manifests.

**CI wiring**: The `ci-unit`, `ci-integration`, `ci-fuzz`, and `ci-performance`
jobs must include the embedding suites in their matrices so each planned failure
surfaces before feature work begins.

## Secure Storage & Retrieval

| Matrix focus | Test module | Test name | Harness | Fixtures & goldens | Initial failing assertion | CI job impact | Security checklist linkage |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Key rotation helpers (unit) | `tests/unit/storage/test_key_rotation.rs` | `rotates_primary_key_material` | `cargo test --test unit_storage` | `tests/fixtures/security/encryption-toggles.json` | `assert_ne!(primary_key, rotated_key)` still equality due to placeholder implementation | `ci-unit` | Encryption; Key Management
| Encryption envelope edge cases (unit) | `tests/unit/storage/test_envelope.rs` | `rejects_truncated_ciphertext` | `cargo test --test unit_storage` | `tests/fixtures/security/encryption-latency.json` (planned) | `matches!(encrypt(payload), Err(Error::TruncatedCiphertext))` currently returns `Ok` | `ci-unit` | Encryption; Input Validation
| Encrypted round-trip search (integration) | `tests/integration/storage/test_encrypted_search.rs` | `search_requires_active_key_rotation` | `cargo test --test integration_storage` | `tests/golden/security/encryption-toggle.trace`, `tests/fixtures/security/dpapi-recovery/` | `assert_eq!(results.audit_log.rotation_id, fixture.expected)` fails because rotation IDs not persisted | `ci-integration` | Encryption; Access Control
| Key schedule fuzz (fuzz) | `tests/fuzz/storage/fuzz_key_rotation.rs` | `fuzz_key_rotation_schedules` | `cargo fuzz run storage_key_rotation` | `tests/golden/security/encryption-toggle.trace` | Expectation: `panic!("rotation drift detected")` when drift occurs; currently completes without detection | `ci-fuzz` | Encryption; Key Management
| Rotation rebuild latency (performance) | `tests/perf/storage/test_rotation_latency.rs` | `stays_within_rebuild_latency_budget` | `cargo bench --bench storage_rotation_latency` | `tests/golden/security/encryption-toggle.trace`, `tests/fixtures/security/perf-window/` | `assert!(observed_latency <= budget_ms)` fails | `ci-performance` | Encryption; Key Management

**Fixture prerequisites**: Ensure DPAPI recovery fixtures and encryption toggles
exist (see [Fixture Plan](./fixtures-plan.md#security-traces-encryption--tls)).
Link the golden traces inside `tests/golden/security/` before enabling tests.

**CI wiring**: Storage suites run in the same jobs as embedding but tagged with
`STORAGE_ONLY=true` for selective reruns when isolating failures in CI dashboards.

## Client Tooling (CLI & SDK)

| Matrix focus | Test module | Test name | Harness | Fixtures & goldens | Initial failing assertion | CI job impact | Security checklist linkage |
| --- | --- | --- | --- | --- | --- | --- | --- |
| CLI flag parsing (unit) | `tests/unit/client/test_cli_flags.rs` | `rejects_conflicting_embedding_hooks` | `cargo test --test unit_client` | `tests/fixtures/shared/cli-flags.toml` | `assert!(matches!(parse(..), Err(Error::ConflictingHooks)))` currently `Ok` | `ci-unit` | Input Validation; Access Control (scoping flags)
| SDK request builders (unit) | `tests/unit/client/test_sdk_builders.rs` | `requires_signed_transport_tokens` | `cargo test --test unit_client` | `tests/fixtures/transport/offline-queue/air-gapped-session.yaml` | `assert!(builder.requires_token())` false due to missing enforcement | `ci-unit` | Authentication; Access Control
| Offline sync integration | `tests/integration/client/test_offline_sync.rs` | `syncs_with_staging_manifest_snapshot` | `cargo test --test integration_client` | `tests/golden/transport/offline-buffer-replay.transcript`, `tests/fixtures/transport/offline-queue/` | `assert_eq!(manifest.digest, fixture.expected_digest)` mismatch | `ci-integration` | Sandboxing (offline isolation); Authentication
| CLI fuzz permutations | `tests/fuzz/client/fuzz_cli_args.rs` | `fuzz_cli_flag_permutations` | `cargo fuzz run client_cli_flags` | `tests/fixtures/shared/cli-flag-corpus/` | Expectation: `Err(ValidationError)` on invalid combos; currently `Ok` | `ci-fuzz` | Input Validation
| Sync duration guard (performance) | `tests/perf/client/test_sync_duration.rs` | `meets_offline_sync_sla` | `cargo bench --bench client_sync_duration` | `tests/golden/transport/offline-buffer-replay.transcript` | `assert!(observed_duration <= target_ms)` fails | `ci-performance` | Sandboxing; Authentication (token refresh timing)

**Fixture prerequisites**: Offline queue fixtures and transcripts must be present
(see [Fixture Plan](./fixtures-plan.md#offline-queue--replay-harnesses)).

**CI wiring**: Extend the future `clients` workflow described in
[CI Coverage Plan](./ci-coverage.md) to invoke the unit and integration suites after
replay diffing. Failures should block the matrix jobs before transcript upload.

## IDE Extensions

| Matrix focus | Test module | Test name | Harness | Fixtures & goldens | Initial failing assertion | CI job impact | Security checklist linkage |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Adapter transformers (unit) | `tests/unit/ide/test_adapter_transformers.rs` | `normalizes_multimodal_payloads` | `cargo test --test unit_ide` | `tests/golden/mcp/ide/adapter-normalization.json` | `assert_eq!(normalized, expected)` mismatch | `ci-unit` | Input Validation
| Golden replay bridge (integration) | `tests/integration/ide/test_mcp_replay.rs` | `replays_cursor_conversation_deteministically` | `cargo test --test integration_ide` | `tests/golden/mcp/ide/cursor-session.transcript` | `assert_eq!(replay.events, golden.events)` diverges due to TODO code | `ci-integration` | Sandboxing (bridge isolation); Authentication (session tokens)
| Suggestion payload fuzzing | `tests/fuzz/ide/fuzz_suggestion_payloads.rs` | `fuzz_suggestion_payload_ordering` | `cargo fuzz run ide_suggestion_payloads` | `tests/golden/mcp/ide/suggestion-fuzz.jsonl` | Expectation: rejects out-of-order payloads; currently accepts | `ci-fuzz` | Input Validation; Sandboxing
| Response time performance | `tests/perf/ide/test_response_time.rs` | `meets_multifile_response_slo` | `cargo bench --bench ide_response_time` | `tests/golden/mcp/ide/perf-baseline.json` | `assert!(latency_p95 <= baseline_p95)` fails | `ci-performance` | Sandboxing; Authentication (session refresh)

**Fixture prerequisites**: Generate IDE golden transcripts and normalization
fixtures via the integration transcripts workflow outlined in
[`docs/integration/transcripts/ide/README.md`](../integration/transcripts/ide/README.md).

**CI wiring**: IDE suites run post transcript replay in the IDE client workflows
so golden parity failures halt the job before artifact upload.

## Filesystem Watch Service

| Matrix focus | Test module | Test name | Harness | Fixtures & goldens | Initial failing assertion | CI job impact | Security checklist linkage |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Debounce heuristics (unit) | `tests/unit/filesystem/test_debounce.rs` | `applies_latency_window_debounce` | `cargo test --test unit_filesystem` | `tests/fixtures/filesystem/latency-window.yaml` | `assert!(window.contains(event_latency))` fails due to placeholder window | `ci-unit` | File Handling; Sandboxing
| Watch-to-queue propagation (integration) | `tests/integration/filesystem/test_watch_propagation.rs` | `propagates_replay_events_within_budget` | `cargo test --test integration_filesystem` | `tests/fixtures/filesystem/workspace-replay/`, `tests/golden/filesystem/watch-latency-burst.log` | `assert!(ingestion_latency <= 200_ms)` fails | `ci-integration` | File Handling; Sandboxing
| Burst latency fuzzing | `tests/fuzz/filesystem/fuzz_watch_latency.rs` | `fuzz_watch_latency_bursts` | `cargo fuzz run filesystem_watch_latency` | `tests/golden/filesystem/watch-latency-burst.log` | Expect panic on over-threshold latency; currently silent | `ci-fuzz` | File Handling
| Sustained burst benchmark | `tests/perf/filesystem/test_burst_latency.rs` | `guards_sustained_watch_latency` | `cargo bench --bench filesystem_watch_latency` | `tests/golden/filesystem/watch-latency-burst.log` | `assert!(median_latency <= 200_ms)` fails | `ci-performance` | File Handling; Sandboxing

**Fixture prerequisites**: Follow the filesystem corpus steps in the
[Fixture Plan](./fixtures-plan.md#filesystem-events-corpus). Include README updates
for any new scenario directories.

**CI wiring**: Hook filesystem suites into the ingestion CI stage so failures block
artifact generation for downstream ingestion jobs.

## Archive Extraction Quotas

| Matrix focus | Test module | Test name | Harness | Fixtures & goldens | Initial failing assertion | CI job impact | Security checklist linkage |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Quota calculator unit tests | `tests/unit/archives/test_quota_calculator.rs` | `rejects_over_quota_archives` | `cargo test --test unit_archives` | `tests/fixtures/archives/quota-latency.toml` | `assert_eq!(decision, Decision::Reject)` currently `Allow` | `ci-unit` | File Handling; Sandboxing
| MIME gatekeeper unit tests | `tests/unit/archives/test_mime_gatekeeper.rs` | `blocks_disallowed_mime_types` | `cargo test --test unit_archives` | `tests/fixtures/archives/quota-latency.toml` | `assert!(denied.contains("application/x-dosexec"))` fails | `ci-unit` | File Handling; Input Validation
| Overflow extraction integration | `tests/integration/archives/test_overflow_extraction.rs` | `aborts_overflow_with_latency_logging` | `cargo test --test integration_archives` | `tests/fixtures/archives/overflow-latency.tar.zst`, `tests/golden/archives/quota-throughput.jsonl` | `assert!(log.contains("quota_violation"))` missing | `ci-integration` | File Handling; Sandboxing
| Archive metadata fuzzing | `tests/fuzz/archives/fuzz_metadata.rs` | `fuzz_archive_metadata_inputs` | `cargo fuzz run archives_metadata` | `tests/golden/archives/quota-throughput.jsonl` | Expectation: `panic!("metadata violation")`; currently suppressed | `ci-fuzz` | File Handling; Input Validation
| Quota throughput guard | `tests/perf/archives/test_quota_throughput.rs` | `holds_quota_enforcement_budget` | `cargo bench --bench archives_quota_throughput` | `tests/fixtures/archives/bulk-sample/`, `tests/golden/archives/quota-throughput.jsonl` | `assert!(overhead_pct <= 0.02)` fails | `ci-performance` | File Handling; Sandboxing

**Fixture prerequisites**: Generate archives fixtures and goldens using the
archive builder workflow in the [Fixture Plan](./fixtures-plan.md#archive-scenarios).
Record SHA-256 manifests alongside large artifacts.

**CI wiring**: Archive suites execute before packaging any archive ingestion builds
so that quota regressions stop release candidates immediately.

## Encryption & TLS Controls

| Matrix focus | Test module | Test name | Harness | Fixtures & goldens | Initial failing assertion | CI job impact | Security checklist linkage |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Key rotation toggles (unit) | `tests/unit/security/test_key_toggle.rs` | `persists_toggle_policies` | `cargo test --test unit_security` | `tests/fixtures/security/encryption-toggles.json` | `assert!(policy_store.has(toggle))` false | `ci-unit` | Encryption; Key Management
| Encrypted store rebuild (integration) | `tests/integration/security/test_encrypted_store.rs` | `rebuilds_with_toggle_permutations` | `cargo test --test integration_security` | `tests/golden/security/encryption-toggle.trace` | `assert!(rebuild.audit.contains("toggle_applied"))` missing | `ci-integration` | Encryption; Key Management
| Toggle fuzz/perf combo | `tests/fuzz/security/fuzz_toggle_sequences.rs` | `fuzz_toggle_sequences` | `cargo fuzz run security_toggle_sequences` | `tests/golden/security/encryption-toggle.trace` | Expectation: `panic!("latency regression")` triggered when >5% drift; currently silent | `ci-fuzz` | Encryption; Key Management
| TLS cipher negotiation (unit) | `tests/unit/security/test_tls_negotiation.rs` | `enforces_mandatory_cipher_suites` | `cargo test --test unit_security` | `tests/fixtures/security/tls-config-matrix.yaml` | `assert!(negotiated.contains("TLS_AES_256_GCM_SHA384"))` fails | `ci-unit` | Encryption; Input Validation
| TLS handshake integration | `tests/integration/security/test_tls_handshake.rs` | `rejects_downgraded_clients` | `cargo test --test integration_security` | `tests/golden/security/tls-negotiation.trace`, `tests/golden/security/tls-performance.jsonl` | `assert_eq!(handshake.state, State::RejectedDowngrade)` currently accepts | `ci-integration` | Encryption; Authentication
| TLS fuzz/performance | `tests/fuzz/security/fuzz_tls_handshakes.rs` | `fuzz_tls_handshake_permutations` | `cargo fuzz run security_tls_handshakes` | `tests/golden/security/tls-performance.jsonl` | Expectation: `panic!("downgrade detected")`; currently no panic | `ci-fuzz` | Encryption; Authentication
| WSL transport handshake regression | `tests/integration/security/test_wsl_transport.rs` | `matches_wsl_dpapi_requirements` | `cargo test --test integration_security` | `tests/golden/transport/wsl-handshake-negotiation.trace`, `tests/golden/security/dpapi-recovery-audit.jsonl`, `tests/fixtures/security/dpapi-recovery/` | `assert!(audit.contains("DPAPI_RECOVERY_SUCCESS"))` fails | `ci-integration` (Windows runner) | Encryption; Key Management; Sandboxing
| DPAPI recovery coverage | `tests/unit/security/test_dpapi_recovery.rs` | `requires_recovery_policy_compliance` | `cargo test --test unit_security` | `tests/fixtures/security/dpapi-recovery/`, `tests/golden/security/dpapi-recovery-audit.jsonl` | `assert_eq!(recovery.status, Status::Compliant)` fails | `ci-unit` (Windows runner) | Key Management; Access Control

**Fixture prerequisites**: Capture TLS, DPAPI, and WSL traces following the
[Fixture Plan](./fixtures-plan.md#security-traces-encryption--tls). Add checksums
for trace outputs before enabling CI enforcement.

**CI wiring**: Extend `ci-integration` and `ci-unit` with Windows variants to cover
DPAPI and WSL-specific suites. Job names: `ci-unit-windows-security` and
`ci-integration-windows-security`.

## Multi-Repository Routing

| Matrix focus | Test module | Test name | Harness | Fixtures & goldens | Initial failing assertion | CI job impact | Security checklist linkage |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Routing table merges (unit) | `tests/unit/routing/test_routing_table.rs` | `merges_tenant_specific_routes` | `cargo test --test unit_routing` | `tests/fixtures/routing/latency-matrix.json` | `assert!(table.tenant("alpha").routes.contains("repo-a"))` fails | `ci-unit` | Access Control; Sandboxing
| Cross-repo latency (integration) | `tests/integration/routing/test_cross_repo_latency.rs` | `enforces_tenant_isolation_latency_budget` | `cargo test --test integration_routing` | `tests/golden/routing/multi-repo-latency.transcript`, `tests/fixtures/routing/high-fanout/` | `assert!(latency_stats.p99 <= fixture.p99_budget)` fails | `ci-integration` | Access Control; Sandboxing; Encryption
| Repository affinity fuzzing | `tests/fuzz/routing/fuzz_repo_affinity.rs` | `fuzz_repository_affinity_hints` | `cargo fuzz run routing_repo_affinity` | `tests/golden/routing/fuzz-affinity.jsonl` | Expectation: rejects cross-tenant leakage; currently passes | `ci-fuzz` | Access Control; Sandboxing
| Routing throughput guard | `tests/perf/routing/test_throughput.rs` | `maintains_multi_repo_throughput` | `cargo bench --bench routing_throughput` | `tests/golden/routing/fanout-throughput.jsonl`, `tests/fixtures/routing/high-fanout/` | `assert!(throughput >= baseline)` fails | `ci-performance` | Access Control; Sandboxing

**Fixture prerequisites**: Produce routing matrices and fanout corpora using the
routing generator steps in the [Fixture Plan](./fixtures-plan.md#routing-scenarios).

**CI wiring**: Routing suites execute alongside storage tests but isolated under
`ROUTING_ONLY=true` to aid triage when latency regressions appear.

## Offline Resilience & Replay

| Matrix focus | Test module | Test name | Harness | Fixtures & goldens | Initial failing assertion | CI job impact | Security checklist linkage |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Transport retry buffer integration | `tests/integration/transport/test_retry_buffer.rs` | `persists_air_gapped_sessions` | `cargo test --test integration_transport` | `tests/fixtures/transport/offline-queue/`, `tests/golden/transport/offline-buffer-replay.transcript` | `assert_eq!(buffer.snapshot(), golden.snapshot)` fails | `ci-integration` | Sandboxing; Encryption; Access Control
| Retry buffer fuzz | `tests/fuzz/transport/fuzz_retry_buffer.rs` | `fuzz_retry_buffer_bursts` | `cargo fuzz run transport_retry_buffer` | `tests/fixtures/transport/offline-queue/`, `tests/golden/transport/offline-buffer-replay.transcript` | Expectation: `panic!("bounded growth exceeded")`; currently silent | `ci-fuzz` | Sandboxing; Encryption
| Manifest replay integration | `tests/integration/ingestion/test_manifest_replay.rs` | `replays_manifest_with_ordered_audit` | `cargo test --test integration_ingestion` | `tests/fixtures/ingestion/delayed-ledger/`, `tests/golden/ingestion/manifest-replay.log` | `assert!(audit.is_ordered())` fails | `ci-integration` | Access Control; Sandboxing; Encryption
| Replay catch-up performance | `tests/perf/transport/test_replay_catch_up.rs` | `meets_replay_catch_up_sla` | `cargo bench --bench transport_replay_catch_up` | `tests/fixtures/ingestion/delayed-ledger/`, `tests/golden/ingestion/manifest-replay.log` | `assert!(catch_up_time <= 5_min)` fails | `ci-performance` | Sandboxing; Encryption; Access Control

**Fixture prerequisites**: Offline transport and manifest replay fixtures must be
populated per the [Fixture Plan](./fixtures-plan.md#offline-queue--replay-harnesses).
Include checksum manifests for large transcripts.

**CI wiring**: Integrate transport and ingestion suites into the same CI stage that
runs `cargo test --test integration_transport` and `cargo test --test integration_ingestion`.
Jobs fail prior to publishing replay artifacts to ensure red/green parity.

## CI Integration Summary

| CI job | Command focus | Expected initial state |
| --- | --- | --- |
| `ci-unit` | `cargo test --test unit_*` per subsystem | Fails on every new unit blueprint until implementation | 
| `ci-integration` | `cargo test --test integration_*` | Fails on subsystem integration tests awaiting features |
| `ci-fuzz` | `cargo fuzz run <target>` nightly | Fails by design when detection logic not implemented |
| `ci-performance` | `cargo bench --bench <suite>` scheduled | Fails until performance budgets satisfied |
| `ci-unit-windows-security` / `ci-integration-windows-security` | Windows runners covering DPAPI + WSL | Fails until Windows-specific encryption logic lands |
| `clients-matrix` | Composite job invoking CLI/SDK and IDE suites per [CI coverage plan](./ci-coverage.md) | Fails after replay diffing because blueprint tests assert missing behaviors |

These jobs must remain red until corresponding features meet their security and
performance guarantees, providing TDD evidence through failing logs.

## Security Checklist Traceability

Every suite above references the checklists in the [Threat Model](../security/threat-model.md):

- **Input Validation** – Required for tokenizer, CLI, IDE payload, archive MIME, and TLS negotiation tests.
- **Encryption & Key Management** – Required for storage, encryption toggles, TLS, and offline replay persistence.
- **Sandboxing** – Applies to ingestion pipelines, filesystem watchers, routing isolation, and offline transport.
- **Authentication & Access Control** – Required for client tooling, TLS, routing, and offline replay manifest enforcement.
- **File Handling** – Required for filesystem and archive suites to prove ingestion hygiene.

When authoring PRs that implement these tests or their associated features, include
checklist sign-off references in the PR description as mandated by
[`docs/process/pr-release-checklist.md`](../process/pr-release-checklist.md).
