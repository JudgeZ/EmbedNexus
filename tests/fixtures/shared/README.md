# Shared Fixture Library

Run `python scripts/fixture_packager.py build --output tests/fixtures/shared`
to assemble the routing bundle consumed by downstream tests. The command copies
the routing matrices, fan-out corpora, throughput metrics, and fuzz-affinity
datasets into the bundle directory and writes `bundle.json` with SHA-256
digests. Validate the bundle with
`python scripts/fixture_packager.py validate tests/fixtures/shared` before
committing regenerated fixtures.【F:scripts/fixture_packager.py†L53-L143】
