# Security Golden Artifacts

The security goldens are now generated automatically via the Windows capture
workflow. Running `scripts/trace_capture.sh` alongside the PowerShell helpers
produces deterministic TLS traces, fuzz logs, DPAPI audit exports, and
performance samples so Linux jobs can replay the scenarios without manual
uploads.

Key generators:

- `scripts/collect_dpapi.ps1` — seeds `dpapi-recovery-audit.jsonl` and fixture
  metadata.
- `scripts/trace_capture.sh` — emits TLS handshakes, negotiation transcripts,
  fuzzing logs, encryption toggles, and performance baselines.
- `scripts/wsl_transport_proxy.ps1` — records the WSL bridge negotiation trace
  and associated metadata consumed by transport tests.

Checksum files under this directory are updated as part of the automated Windows
jobs so reviewers can download and verify captures from each workflow run.
