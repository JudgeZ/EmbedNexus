# Encryption Engine Specification

This document extends the encrypted storage flows in the [architecture overview](./overview.md), defining the dedicated encryption services that safeguard embeddings, manifests, and audit metadata.

## Module Responsibilities
- Manage key lifecycles, including creation, sealing, rotation, and revocation across repositories and workspaces.
- Provide streaming encryption/decryption primitives for ingestion writers and vector store readers.
- Maintain policy enforcement for cipher suite selection, nonce management, and integrity tagging.
- Expose attestation and compliance hooks for audit trails and external policy verification.
- Support cross-platform secure enclave integration (Linux, macOS Keychain, Windows DPAPI/WSL interop) with explicit downgrade procedures when certain facilities are unavailable offline.

## Public Interfaces

| Interface | Description | Inputs | Outputs |
|-----------|-------------|--------|---------|
| `KeyManager::provision(repo_id)` | Create or retrieve per-repository key material | Repository identifier, policy tag | `KeyHandle` |
| `KeyManager::rotate(schedule)` | Rotate keys according to policy | Rotation schedule, audit context | Rotation report |
| `EncryptionEngine::seal(batch, key_handle)` | Encrypt payloads with authenticated metadata | Plaintext payload, key handle, nonce strategy | Ciphertext blob + integrity tag |
| `EncryptionEngine::open(blob, key_handle)` | Decrypt and verify payloads | Ciphertext blob, key handle | Plaintext payload, verification report |
| `Attestor::stamp(event)` | Generate attestation artifacts for compliance | Event metadata, hash chain pointer | Signed attestation record |

## Data Models
- **`KeyHandle`**: `{ key_id, repo_id, version, sealed_reference, expiry, policy }`.
- **`CiphertextBlob`**: `{ payload, nonce, tag, manifest_pointer, rotation_epoch }`.
- **`AttestationRecord`**: `{ event_id, timestamp, hash, policy_tag, verifier }`.
- **`RotationReport`**: `{ rotation_epoch, key_ids[], succeeded[], failed[], remediation_steps }`.
- **`VerificationReport`**: `{ blob_id, verified, failure_reason?, audit_pointer }`.

## Sequencing

```mermaid
sequenceDiagram
    participant Pipeline as Ingestion Pipeline
    participant KM as Key Manager
    participant Engine as Encryption Engine
    participant Store as Secure Storage
    participant Ledger as Audit Ledger

    Note over Pipeline,KM: Preconditions: threat model policies loaded, secure enclave online
    Pipeline->>KM: provision(repo_id)
    KM-->>Pipeline: KeyHandle
    Pipeline->>Engine: seal(batch, KeyHandle)
    Engine->>Engine: Derive nonce + apply policy checks
    Engine-->>Store: CiphertextBlob
    Engine->>Ledger: Attestation record
    Ledger-->>KM: Confirm attestation hash chain
    Note over Engine,Store: Postconditions: Ciphertext persisted, attestation durable, KeyHandle usage logged
```

## Preconditions & Postconditions
- **Preconditions**
  - Local secure enclave or keystore is initialized and unlockable for the session.
  - Threat-model driven policies (cipher suites, rotation cadence, attestation requirements) are loaded and validated.
  - Audit ledger is writable and hash-chain anchors are synchronized.
- **Postconditions**
  - Every encryption operation emits an attestation record chained to the audit ledger.
  - Key usage metrics update rotation schedules and revocation triggers.
  - Decryption failures surface actionable remediation paths without exposing key material.

## Cross-Cutting Concerns
- **Error Handling**: Differentiate between recoverable keystore timeouts and critical key compromise signals; escalate via incident hooks when compromise indicators surface.
- **Concurrency**: Serialize key rotations while allowing parallel seal/open operations through read-only handles with reference counting.
- **Resource Limits**: Monitor enclave session quotas, key cache sizes, and attestation storage growth to avoid exhausting offline workstations, including guidance for WSL users where host disk quotas apply.
- **Security Alignment**: Map all enforcement back to the [Encryption Checklist](../security/threat-model.md#encryption-checklist) and [Key Management Checklist](../security/threat-model.md#key-management-checklist); ensure policy exceptions are documented with compensating controls.
- **Offline Expectations**: Provide deterministic key caching and attestation batching for air-gapped sessions, replaying audit events once connectivity resumes.
- **Platform Notes**: Document per-platform key derivation, secure storage prerequisites, and fallback behavior when native enclaves are unavailable.

## Required Failing Tests
In alignment with the [test matrix](../testing/test-matrix.md#encryption--tls-controls), implement failing tests that cover:
- Unit tests for key negotiation toggles, rotation edge cases, and TLS configuration validators when sync endpoints are enabled.
- Integration tests for encrypted sync and local persistence demonstrating rotation-aware unlock flows.
- Fuzz tests mutating handshake transcripts and ciphertext blobs for downgrade and tampering resilience.
- Performance tests capturing encryption overhead during index rebuilds across representative datasets.
- Each suite must respect the TDD process by landing failing tests prior to implementation and referencing `docs/process/pr-release-checklist.md` when summarizing coverage.
