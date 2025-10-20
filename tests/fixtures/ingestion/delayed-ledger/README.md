# Delayed Ledger Replay Fixtures

The delayed ledger corpus feeds the manifest replay harness and validates the
offline buffer plumbing. Regenerate the inputs when capturing a new replay
window:

```bash
cargo run --bin manifest_replay_harness -- \
  --input-dir tests/fixtures/ingestion/delayed-ledger/ \
  --golden-out tests/golden/ingestion/manifest-replay.log \
  --delay-ms 45000
```

Populate `placeholder.jsonl` with the captured sequence backlog prior to running
the harness. Each JSONL record should include `sequence`, `repo_id`,
`delayed_ms`, and manifest checksums as shown in the seeded fixture. When
refreshing the corpus, document the harness command and the storage outage
parameters in the pull request so reviewers can cross-reference the Encryption
checklist from `docs/security/threat-model.md`.
