# Filesystem Watcher Fixtures

The watcher fixtures exercise latency windows, debounce semantics, and replay
coverage for the ingestion pipeline. Regenerate the corpus after capturing a new
workload with the watcher harness so the observed counts stay aligned with the
golden metrics consumed by `watcher_latency.rs`.

## Regeneration workflow

1. Ensure Python 3.11 with `pyyaml` is available (`python -m pip install pyyaml`).
2. Populate the replay workspace and scenario manifests:

   ```bash
   python scripts/record_fs_events.py \
     --config tests/fixtures/filesystem/mock-events.yaml \
     --replay-dir tests/fixtures/filesystem/workspace-replay/
   ```

   The stub emits deterministic outputs: configuration metadata (`mode: config`),
   filesystem replay transcripts (`mode: replay`), and latency metrics
   (`mode: metrics`).

3. Refresh the fuzz golden transcript consumed by regression tests:

   ```bash
   python scripts/record_fs_events.py \
     --scenario fuzz \
     --output tests/golden/filesystem/watch-fuzz.log
   ```

   Repeat the command with `--scenario latency-burst` to refresh the latency
   golden.

4. Run `python scripts/verify_event_order.py tests/fixtures/filesystem/workspace-replay/`
   to confirm replay ordering before committing. The verifier prints `OK ✓` lines
   for each valid transcript and exits with one of four codes (`0` success, `1`
   load/parsing failure, `2` schema violation, `3` ordering/dependency failure).

These steps satisfy the Input Validation and Sandboxing checklist items called
out in `docs/security/threat-model.md`, ensuring watcher filters do not leak
restricted paths while replaying recorded events.
