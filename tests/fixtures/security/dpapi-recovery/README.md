# DPAPI Recovery Fixtures

These fixtures are generated automatically by running the Windows capture helper:

```powershell
pwsh -File scripts/collect_dpapi.ps1 -EmitChecksums
```

The script emits synthetic but deterministic DPAPI recovery metadata so CI jobs
and integration tests can validate recovery workflows without needing access to
a domain-joined workstation. The key outputs are:

- `manifest.json` — describes the capture environment, recovery agent, and
  artifact inventory.
- `master-key-inventory.json` — lists recovered DPAPI master keys with rotation
  metadata.
- `certificate-inventory.json` — enumerates active and legacy recovery agent
  certificates.
- `recovery-events.jsonl` — the audit export that is copied to
  `tests/golden/security/dpapi-recovery-audit.jsonl`.

Checksums for the JSONL export are written to `recovery-events.jsonl.sha256` so
Linux workflows can verify integrity before replaying the audit log.
