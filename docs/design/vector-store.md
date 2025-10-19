# Vector Store Specification

This specification complements the encrypted persistence path described in the [architecture overview](./overview.md) by detailing the logical vector store that persists and retrieves embeddings.

## Module Responsibilities
- Persist embedding batches in encrypted shards keyed by repository and workspace lineage.
- Expose retrieval primitives supporting similarity search, filtering, and manifest-backed pagination.
- Maintain compaction and rotation routines that respect resource budgets and preserve audit trails, including DPAPI-backed key recovery workflows for WSL-hosted shards.
- Provide transactional hooks for ingestion and transport layers to coordinate writes and reads.
- Surface cross-platform storage guidance so Linux, macOS, and WSL deployments maintain consistent file permissions, case handling, and path normalization.

## Public Interfaces

| Interface | Description | Inputs | Outputs |
|-----------|-------------|--------|---------|
| `VectorStore::put(batch, manifest)` | Persist an embedding batch into a shard | `EmbeddingBatch`, `ManifestDiff`, encryption context | `StoreWriteReceipt` |
| `VectorStore::query(criteria)` | Execute similarity search and filtering | Query criteria, tenant scope, consistency hints | `QueryResultSet` |
| `VectorStore::compact(policy)` | Run shard compaction and index optimization | Compaction policy, resource budget | `CompactionReport` |
| `VectorStore::rotate_keys(schedule)` | Trigger key rotation for shards | Rotation schedule, key handles | Updated shard descriptors |
| `VectorStore::export(manifest_cursor)` | Stream embeddings and metadata for backup | Manifest cursor, export policy | Stream of encrypted payloads |

## Data Models
- **`ShardDescriptor`**: `{ shard_id, repo_id, workspace_ids[], key_id, size_bytes, last_compacted_at }`.
- **`StoreWriteReceipt`**: `{ shard_id, batch_id, commit_ts, checksum, audit_pointer }`.
- **`QueryCriteria`**: `{ repo_scope, filters{}, vector, k, consistency_mode }`.
- **`QueryResultSet`**: `{ hits[], latency_ms, shard_scan_count, manifest_cursor }`.
- **`CompactionReport`**: `{ shard_id, reclaimed_bytes, segments_merged, warnings[] }`.

## Sequencing

```mermaid
sequenceDiagram
    participant Pipeline as Ingestion Pipeline
    participant Store as Vector Store
    participant Encryptor as Encryption Engine
    participant Ledger as Audit Ledger
    participant Client as Query Client

    Note over Pipeline,Store: Preconditions: Shard descriptors loaded, encryption keys active, manifest cursor consistent
    Pipeline->>Store: put(batch, manifest)
    Store->>Encryptor: Encrypt vectors + metadata
    Encryptor-->>Store: Ciphertext payload + key reference
    Store->>Ledger: Log write receipt and checksums
    Store-->>Pipeline: StoreWriteReceipt
    Client->>Store: query(criteria)
    Store->>Encryptor: Decrypt relevant shard slices
    Store-->>Client: QueryResultSet with manifest cursor
    Note over Store,Client: Postconditions: Query statistics recorded, decrypted buffers scrubbed
```

## Preconditions & Postconditions
- **Preconditions**
  - Encryption engine exposes valid key handles for targeted shards.
  - Manifest cursors from ingestion align with the latest committed shard state.
  - Disk quotas and file descriptors are within configured budgets before writes or compactions begin.
- **Postconditions**
  - Every write produces a durable receipt referenced by the audit ledger.
  - Compaction operations emit reports and do not violate key rotation policies.
  - Query responses redact sensitive diagnostics while exposing necessary latency metrics.

## Cross-Cutting Concerns
- **Error Handling**: Differentiate between transient IO failures (retry) and shard corruption (quarantine + operator alert). Surface actionable diagnostics without exposing key material.
- **Concurrency**: Support concurrent readers and writers through optimistic MVCC, ensuring ingestion commits do not block read paths beyond configured contention windows.
- **Resource Limits**: Enforce shard size ceilings, compaction throttles, and buffer pool limits to maintain offline operation without exhausting local storage, documenting WSL-specific disk passthrough considerations.
- **Offline Expectations**: Buffer query analytics and rotation telemetry locally when disconnected from optional sync endpoints, replaying once connectivity is restored.
- **Platform Notes**: Specify how case sensitivity, filesystem notifications, and file-lock semantics differ across supported platforms so implementations remain portable.
- **WSL Telemetry Expectations**: Capture DPAPI key recovery events, shard unlock timings, and encryption envelope integrity when the store runs under WSL; align telemetry schemas with the failing coverage introduced in the [Encryption & TLS Controls matrix](../testing/test-matrix.md#encryption--tls-controls).
- **Security Alignment**: Integrate with encryption policies defined in [encryption.md](./encryption.md) and the [Encryption Checklist](../security/threat-model.md#encryption-checklist), and map query authorization flows to the [Access Control Checklist](../security/threat-model.md#access-control-checklist).

## Test hooks
Vector store hardening requires failing coverage from both the [Secure Storage & Retrieval matrix](../testing/test-matrix.md#secure-storage--retrieval) and the [Multi-Repository Routing matrix](../testing/test-matrix.md#multi-repository-routing). Capture their findings against the [Encryption Checklist](../security/threat-model.md#encryption-checklist) and [Sandboxing Checklist](../security/threat-model.md#sandboxing-checklist):
- **Shard integrity hook** – Unit and fuzz tests validating key rotation helpers, tamper detection, and receipt generation with `tests/fixtures/security/encryption-latency.json` and `tests/golden/security/encryption-toggle.trace` to demonstrate encryption guarantees and sandbox isolation of compromised shards.
- **Cross-repo routing hook** – Integration tests replaying `tests/golden/routing/multi-repo-latency.transcript` to confirm tenant isolation and routing-table merges remain within policy while enforcing encryption on every hop.
- **Routing throughput guard hook** – Performance coverage executing `tests/golden/routing/fanout-throughput.jsonl` alongside `tests/fixtures/routing/high-fanout/` to ensure scheduling latency stays within guardrails without violating sandbox-imposed resource ceilings.
- **Compaction resilience hook** – Performance and integration tests from the secure storage matrix verifying rotation-aware compaction against `tests/golden/security/encryption-toggle.trace` does not leak plaintext diagnostics and respects encryption attestations.
- **WSL DPAPI recovery hook** – Unit and integration tests targeting `tests/fixtures/security/dpapi-recovery/` with audit validation against `tests/golden/security/dpapi-recovery-audit.jsonl` to guarantee DPAPI-backed key handles are honored when shards migrate between Windows hosts and WSL distributions.
- Register each hook as a failing test ahead of implementation and cross-reference `docs/process/pr-release-checklist.md` with the corresponding security checklist evidence.
