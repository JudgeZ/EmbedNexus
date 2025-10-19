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

## Shared Testing Assets

- **Fixture Library**: Reuse the shared fixture bundle under `tests/fixtures/shared/` for cross-subsystem consistency. Extend fixtures instead of duplicating data and document each addition inside the fixture README files.
- **Golden MCP Messages**: Client scripts and IDE validation must rely on the curated golden message transcripts in `tests/golden/mcp/`. Update transcripts when protocol changes occur and ensure replay harnesses remain deterministic.

## Execution Notes

1. Add each failing test to the appropriate suite before feature implementation begins. Reference the planned feature label in the test description to maintain traceability.
2. Link new test coverage to existing CI job definitions so that failures surface immediately while the features remain pending.
3. Document deviations or temporary skips inside the subsystem-specific testing READMEs with rationale and target removal milestones.
