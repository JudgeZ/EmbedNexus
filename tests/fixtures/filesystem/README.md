# Filesystem Fixture Placeholders

Placeholder YAML and directory structures live here until the filesystem watcher
tests are authored. Real datasets will be captured via
`scripts/record_fs_events.py` and related tooling referenced in
`docs/testing/fixtures-plan.md`.

The recorder stub supports two invocation paths:

- `python scripts/record_fs_events.py --scenario <name> --output <file>` writes a
  deterministic golden based on the embedded `_SCENARIO_LOGS` entries.
- `python scripts/record_fs_events.py --config tests/fixtures/filesystem/mock-events.yaml --replay-dir <dir>` populates this
  directory with a deterministic `mock-events.yaml` manifest plus per-scenario
  replay captures derived from the same embedded data.

Update the configuration and replay artifacts when recording new event traces so
that regeneration workflows and CI jobs stay aligned with the documented schema.
