# Encryption & TLS Validation Logging Template

Use this template to document encryption-at-rest and TLS negotiation validation runs associated with the Encryption & TLS Controls scenarios in the test matrix. Store completed artifacts under `tests/fixtures/security/metrics/` using filenames like `encryption-tls-validation-<scenario>-<yyyymmdd>.jsonl`.

## Scenario Details
- **Matrix link**: Encryption & TLS Controls – see [`docs/testing/test-matrix.md`](../test-matrix.md#encryption--tls-controls)
- **Scenario focus**: (encryption toggle, TLS handshake downgrade defense, etc.)
- **Environment details**: (OS, hardware, certificate chain info)
- **Key material references**: (fixture names, rotation schedule)

## Instrumentation Checklist
- [ ] Enable detailed encryption logging (key rotation events, cipher negotiation, storage writes) with timestamps.
- [ ] Capture TLS handshake traces, including selected cipher suites and certificate validation results.
- [ ] Record error paths (e.g., failed certificate validation, key rotation retries) with correlation IDs.
- [ ] Collect system resource metrics if available (CPU, memory) during encryption toggles.

## Required Tooling
- Encryption logging hooks or tracing infrastructure built into the storage subsystem.
- TLS inspection tool capable of capturing handshake details (`openssl s_client`, `nmap --script ssl-enum-ciphers`, `testssl.sh`).
- Checksum utilities for verifying encrypted artifact integrity (`sha256sum`, `b3sum`).
- Secure log storage supporting tamper-evident archival (append-only logs or signed bundles).

## Metrics to Record
- Latency of encryption operations (key rotation, encryption/decryption) with percentile breakdowns.
- TLS handshake duration and success/failure counts per cipher suite.
- Validation status for certificate chain and hostname checks.
- Integrity verification results for encrypted payloads and rotated keys.

Store results as JSON Lines (`.jsonl`) with individual records containing timestamps, operation type, metrics, and validation verdicts. For large handshake traces, compress them (`.zst`) and include an index record referencing the compressed asset under `tests/fixtures/security/metrics/artifacts/`.

## Validation Steps
- Verify encrypted artifact and log bundle checksums under `tests/fixtures/security/metrics/checksums/`.
- Document TLS certificate fingerprints and expiry data for auditing.
- Reference the failing tests or CI pipelines exercising the scenario once present.
- Note any deviations from expected cipher suite policies or encryption toggles and provide remediation plans.
