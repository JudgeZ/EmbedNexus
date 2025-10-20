# Filesystem Fixture Placeholders

Placeholder YAML and directory structures live here until the filesystem watcher
tests are authored. Real datasets will be captured via
`scripts/record_fs_events.py` and related tooling referenced in
`docs/testing/fixtures-plan.md`.

The command scaffolding expects the configuration at
`tests/fixtures/filesystem/mock-events.yaml`; update this file when recording new
event traces so that regeneration workflows and CI jobs stay aligned.
