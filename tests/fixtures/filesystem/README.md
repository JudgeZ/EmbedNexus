# Filesystem Watcher Fixtures

The watcher fixtures exercise latency windows, debounce semantics, and replay
coverage for the ingestion pipeline. Regenerate the corpus after capturing a new
workload with the watcher harness so the observed counts stay aligned with the
golden metrics consumed by `watcher_latency.rs`.

## Regeneration workflow

1. Ensure Python 3.11 with `watchdog`, `pyyaml`, and `typer` is available
   (`python -m pip install -r scripts/requirements-watchers.txt`).
2. Record raw events into the scenario manifest:

   ```bash
   python scripts/record_fs_events.py \
     --scenario latency-burst \
     --output tests/fixtures/filesystem/mock-events.yaml \
     --replay-dir tests/fixtures/filesystem/workspace-replay/
   ```

3. Emit the aggregated latency windows used by the ingestion enumerator:

   ```bash
   python scripts/record_fs_events.py metrics \
     --scenario latency-burst \
     --latency-window-out tests/fixtures/filesystem/latency-window.yaml
   ```

   The metrics subcommand populates the `observed` and `max_latency_ms` fields so
   the tests can assert watcher throughput against
   `tests/golden/filesystem/watch-latency-burst.log`.

4. Rebuild the golden latency transcript to keep CI assertions deterministic:

   ```bash
   python scripts/record_fs_events.py transcript \
     --scenario latency-burst \
     --output tests/golden/filesystem/watch-latency-burst.log
   ```

5. Run `python scripts/verify_event_order.py tests/fixtures/filesystem/workspace-replay/`
   to confirm event ordering before committing.

These steps satisfy the Input Validation and Sandboxing checklist items called
out in `docs/security/threat-model.md`, ensuring watcher filters do not leak
restricted paths while replaying recorded events.
