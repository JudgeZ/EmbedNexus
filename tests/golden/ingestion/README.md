# Manifest Replay Golden

The manifest replay golden is emitted by the same harness that populates the
delayed ledger fixtures:

```bash
cargo run --bin manifest_replay_harness -- \
  --input-dir tests/fixtures/ingestion/delayed-ledger/ \
  --golden-out tests/golden/ingestion/manifest-replay.log \
  --delay-ms 45000
```

Retain the Encryption checklist notes (at-rest encryption profile, ledger
sequence verification) in the associated pull request whenever this golden is
updated so reviewers can confirm replay fidelity.
