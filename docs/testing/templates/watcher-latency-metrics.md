# Filesystem Watcher Latency Metrics Template

Use this template when capturing filesystem watcher latency measurements for the Filesystem Watch Service scenarios in the test matrix. Fill out each section and store completed records under `tests/fixtures/filesystem/metrics/` with a filename that references the scenario and date (for example, `watcher-latency-<scenario>-<yyyymmdd>.json`).

## Scenario Details
- **Matrix link**: Filesystem Watch Service – see [`docs/testing/test-matrix.md`](../test-matrix.md#filesystem-watch-service)
- **Scenario name**:
- **Watch path(s)**:
- **Event mix**: (e.g., create/update/delete ratios, ignored patterns)

## Instrumentation Checklist
- [ ] Enable high-resolution timers around filesystem event emission and ingestion queue acknowledgement.
- [ ] Synchronize system clocks or include offset data for multi-host captures.
- [ ] Capture debounce parameters and ignore rule configuration applied during the run.
- [ ] Record number of events processed and sustained burst duration.

## Required Tooling
- High-resolution timer or tracing facility (e.g., `perf`, `hyperfine`, monotonic clock profiler).
- Filesystem event stream inspector capable of exporting timestamps.
- Optional: clock skew tracer if multiple machines are involved.

## Metrics to Record
- Minimum, median, 95th percentile, and maximum watcher-to-queue latency (milliseconds).
- Event throughput (events per second) during steady state and burst windows.
- Debounce effectiveness (number of collapsed events vs. emitted events).
- Clock skew measurement when applicable.

Store raw measurements in JSON or YAML and include a `summary` section with computed percentiles. Compress large payloads using `.zst` if they exceed 5 MB.

## Validation Steps
- Verify dataset integrity with checksum utilities (e.g., `sha256sum`) and commit checksum outputs alongside the fixture under `tests/fixtures/filesystem/metrics/checksums/`.
- Add notes about anomalies or data gaps.
- Link to relevant failing tests once they are implemented.
