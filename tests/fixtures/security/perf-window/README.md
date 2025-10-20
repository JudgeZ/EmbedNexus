# TLS Performance Window Fixtures

The performance window fixtures are regenerated automatically on Windows via:

```bash
scripts/trace_capture.sh --profile perf-window --output-dir tests/fixtures/security/perf-window
```

The directory contains:

- `rolling-window.json` — a deterministic snapshot of handshake/failure counts
  for the last 60 seconds of TLS activity.
- `notes.txt` — provenance information emitted by the capture helper.

These files replace the previous placeholder and allow Linux jobs to replay a
consistent workload when verifying throttling heuristics.
