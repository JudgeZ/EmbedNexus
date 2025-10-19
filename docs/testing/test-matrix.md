# Test Matrix for Planned Feature Work

This matrix captures the intentionally failing tests that must exist prior to implementing each major subsystem feature. The goal is to drive test-driven development by ensuring every capability is guarded by unit, integration, fuzz, and performance coverage before code lands.

## Subsystem Overview

| Subsystem | Planned Feature Focus | Required Failing Tests |
| --- | --- | --- |
| Embedding Engine Core | Adaptive chunking, multi-modal embedding expansion | - Unit: tokenizer boundary handling, embedding dimensionality negotiation<br>- Integration: ingestion pipeline end-to-end run with adaptive chunking enabled<br>- Fuzz: malformed document payloads targeting tokenizer and encoder surfaces<br>- Performance: throughput regression guarding 10k document batch window |
| Secure Storage & Retrieval | Encrypted vector persistence, rotated key support | - Unit: key rotation helpers, encryption envelope edge cases<br>- Integration: encrypted round-trip search across sharded stores<br>- Fuzz: randomized key schedules and ciphertext tampering inputs<br>- Performance: latency guard on rotated index rebuild cycle |
| Client Tooling (CLI & SDK) | Offline sync script, custom embedding hooks | - Unit: CLI flag parsing for embedding hooks, SDK request builders<br>- Integration: offline sync against staging MCP endpoint using shared fixtures<br>- Fuzz: CLI argument permutations leveraging fixture corpus<br>- Performance: sync duration guard leveraging large fixture dataset |
| IDE Extensions | Inline suggestion surfacing, MCP golden message replay | - Unit: adapter transformers for IDE protocol messages<br>- Integration: replay of golden MCP conversations against extension bridge<br>- Fuzz: randomized suggestion payload ordering via golden transcript mutator<br>- Performance: response time guard for multi-file suggestion batches |

## Shared Testing Assets

- **Fixture Library**: Reuse the shared fixture bundle under `tests/fixtures/shared/` for cross-subsystem consistency. Extend fixtures rather than duplicating data, and document additions in fixture README files.
- **Golden MCP Messages**: Client scripts and IDE validation must rely on the curated golden message transcripts located in `tests/golden/mcp/`. Update these transcripts when protocol changes occur and ensure replay harnesses remain deterministic.

## Execution Notes

1. Add each failing test to the appropriate suite before feature implementation begins. Reference the planned feature label in test descriptions to maintain traceability.
2. Link new test coverage to existing CI job definitions so that failures surface immediately when features are still pending.
3. Document deviations or temporary skips directly in subsystem-specific testing READMEs with rationale and target removal milestone.

