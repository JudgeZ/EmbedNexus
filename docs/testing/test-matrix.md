# Test Matrix for Planned Feature Work

This matrix catalogues the intentionally failing tests that must be in place before implementation begins on each subsystem. The objective is to drive test-first delivery so that every planned capability is protected by unit, integration, fuzz, and performance coverage ahead of landing code changes.

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
- **Planned feature focus**: Workspace filesystem watching with debounce and ignore rules.
- **Required failing tests**:
  - **Unit** – watcher debounce helpers using `tests/fixtures/filesystem/mock-events.yaml`.
  - **Integration** – end-to-end change propagation through `tests/fixtures/filesystem/workspace-replay/` driving the ingest queue.
  - **Fuzz** – event storm permutations generated from `tests/golden/filesystem/watch-fuzz.log` to validate ignore glob safety.
  - **Performance** – sustained burst benchmark guarding CPU usage under 5k events/min replay.
- **Security traceability** – Covers sandboxing and input-handling mitigations per the [Sandboxing Checklist](../security/threat-model.md#sandboxing-checklist) and [Input Validation Checklist](../security/threat-model.md#input-validation-checklist).

### Archive Extraction Quotas
- **Planned feature focus**: Controlled archive ingestion with byte quotas and type filters.
- **Required failing tests**:
  - **Unit** – quota calculators and MIME gatekeepers using `tests/fixtures/archives/quota-scenarios.toml`.
  - **Integration** – staged tarball ingestion from `tests/fixtures/archives/overflow-case.tar.zst` asserting quota enforcement errors.
  - **Fuzz** – randomized archive metadata from `tests/golden/archives/fuzzed-manifests.jsonl` stressing parser hardening.
  - **Performance** – extraction throughput guard maintaining quota checks under 2% overhead on the `tests/fixtures/archives/bulk-sample/` corpus.
- **Security traceability** – Maps to the [Input Validation Checklist](../security/threat-model.md#input-validation-checklist) for size gating and the [Sandboxing Checklist](../security/threat-model.md#sandboxing-checklist) for extractor isolation requirements.

### Encryption & TLS Controls
- **Planned feature focus**: Toggleable encryption-at-rest and TLS-enforced sync endpoints.
- **Required failing tests**:
  - **Unit** – key negotiation toggles and TLS configuration validators using `tests/fixtures/security/encryption-toggles.json`.
  - **Integration** – encrypted sync round-trip employing `tests/golden/security/tls-handshake.trace` with toggled endpoints.
  - **Fuzz** – randomized cipher-suite negotiation transcripts from `tests/golden/security/tls-fuzz.log` ensuring downgrade protection.
  - **Performance** – encryption overhead guard measuring index rebuild latency with toggles flipped across `tests/fixtures/security/perf-window/` datasets.
- **Security traceability** – Directly linked to the [Encryption Checklist](../security/threat-model.md#encryption-checklist) for configuration validation and the [Input Validation Checklist](../security/threat-model.md#input-validation-checklist) for handshake parameter vetting.

### Multi-Repository Routing
- **Planned feature focus**: Routing embeddings and retrieval across federated repository workspaces.
- **Required failing tests**:
  - **Unit** – routing table merge helpers referencing `tests/fixtures/routing/multi-repo-matrix.json`.
  - **Integration** – cross-repo retrieval using the golden session `tests/golden/routing/mcp-federation.transcript` to confirm correct tenant isolation.
  - **Fuzz** – randomized repository affinity hints derived from `tests/golden/routing/fuzz-affinity.jsonl`.
  - **Performance** – routing throughput guard validating scheduler latency across the `tests/fixtures/routing/high-fanout/` scenario pack.
- **Security traceability** – Upholds isolation controls aligned with the [Sandboxing Checklist](../security/threat-model.md#sandboxing-checklist) and [Encryption Checklist](../security/threat-model.md#encryption-checklist) for cross-tenant data flows.

## Shared Testing Assets

- **Fixture Library**: Reuse the shared fixture bundle under `tests/fixtures/shared/` for cross-subsystem consistency. Extend fixtures instead of duplicating data and document each addition inside the fixture README files.
- **Golden MCP Messages**: Client scripts and IDE validation must rely on the curated golden message transcripts in `tests/golden/mcp/`. Update transcripts when protocol changes occur and ensure replay harnesses remain deterministic.

## Execution Notes

1. Add each failing test to the appropriate suite before feature implementation begins. Reference the planned feature label in the test description to maintain traceability.
2. Link new test coverage to existing CI job definitions so that failures surface immediately while the features remain pending.
3. Document deviations or temporary skips inside the subsystem-specific testing READMEs with rationale and target removal milestones.
