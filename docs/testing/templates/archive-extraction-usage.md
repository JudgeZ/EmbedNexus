# Archive Extraction Usage Template

Document archive extraction quota measurements for the Archive Extraction Quotas scenarios in the test matrix using this template. Store populated templates under `tests/fixtures/archives/metrics/` with descriptive filenames such as `archive-usage-<dataset>-<yyyymmdd>.yaml`.

## Scenario Details
- **Matrix link**: Archive Extraction Quotas – see [`docs/testing/test-matrix.md`](../test-matrix.md#archive-extraction-quotas)
- **Archive corpus**:
- **Quota configuration**:
- **Extraction tooling versions**:

## Instrumentation Checklist
- [ ] Capture start/end timestamps for each archive extraction using a monotonic timer.
- [ ] Record byte counts, file counts, and MIME types emitted per archive.
- [ ] Track quota threshold crossings and enforcement actions (e.g., skipped files, truncation events).
- [ ] Log compression/decompression stages and intermediate storage usage.

## Required Tooling
- High-resolution timer or benchmarking utility (e.g., `hyperfine`, `time`, built-in profiler).
- Checksum utility for verifying extracted outputs (`sha256sum`, `b3sum`).
- Archive inspection tools capable of emitting structured logs (`bsdtar`, `zstd`, `tar --verbose --to-command`).

## Metrics to Record
- Extraction latency per archive and aggregate percentiles (p50/p95/p99).
- Bytes extracted vs. quota limit and enforcement effectiveness (% truncated, % skipped).
- Decompression throughput (MB/s) and CPU utilization if available.
- Checksum verification results for representative files.

Store measurements in YAML with sections for `timing`, `quota_enforcement`, and `validation`. Large per-file listings can be compressed and stored under `tests/fixtures/archives/metrics/<scenario>/` with an accompanying index file referencing the compressed assets.

## Validation Steps
- Persist checksum manifests under `tests/fixtures/archives/metrics/checksums/` and include command output used to generate them.
- Note any anomalies (unexpected MIME types, extraction errors) and remediation steps.
- Reference associated failing tests or CI jobs validating quota enforcement once implemented.
