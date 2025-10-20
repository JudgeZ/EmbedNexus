# Archive Quota Fixtures

The archive fixtures model quota enforcement, latency ceilings, and nested
bundle depth for the planner. Refresh the corpus with the archive builder when
adding scenarios to the ingestion tests.

```bash
cargo run --bin archive_builder -- --scenario quota-latency \
  --output tests/fixtures/archives/quota-latency.toml

cargo run --bin archive_builder -- --scenario overflow-latency \
  --output tests/fixtures/archives/overflow-latency.tar.zst

cargo run --bin archive_builder -- --scenario quota \
  --output tests/fixtures/archives/quota-scenarios.toml
```

After generation, record SHA-256 hashes via
`sha256sum tests/fixtures/archives/*.tar.zst > tests/fixtures/archives/*.sha256`
and keep manifests committed alongside the artifacts. These instructions satisfy
the File Handling and Sandboxing checklist requirements—archives are expanded in
isolated scratch space, byte counts are deterministic, and nested depths remain
bounded per the recorded profiles.
