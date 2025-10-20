# Routing Fixture Placeholders

Placeholder routing matrices and high-fanout corpora reside here. They will be
materialized via `python scripts/routing_matrix.py` once routing-focused tests
are authored.【F:docs/testing/fixtures-plan.md†L164-L168】

## Generation commands

| Command | Required CLI flags | Produces | Dependencies / config |
| --- | --- | --- | --- |
| `python scripts/routing_matrix.py matrix` | `--output tests/fixtures/routing/multi-repo-matrix.json` | Multi-repository routing adjacency matrix used by unit tests.【F:docs/testing/fixtures-plan.md†L164-L165】 | Python 3.11 with `networkx`; run before other subcommands so downstream assets share topology.【F:scripts/README.md†L19-L24】 |
| `python scripts/routing_matrix.py matrix` | `--latency-output tests/fixtures/routing/latency-matrix.json` (planned flag) | Latency budget matrix consumed by routing governance checks.【F:tests/fixtures/routing/latency-matrix.json†L1-L4】 | Shares the same graph definition as the primary matrix; keep CLI invocation in the same run as the `--output` flag so artifacts stay consistent.【F:docs/testing/test-matrix.md†L77-L80】 |
| `python scripts/routing_matrix.py fanout` | `--output-dir tests/fixtures/routing/high-fanout/` | High fan-out corpora for performance and integration scenarios.【F:docs/testing/fixtures-plan.md†L165-L166】 | Requires fixtures directory to exist; generates scenario-specific JSONL captures referenced by throughput guard tests.【F:docs/testing/test-blueprints.md†L140-L142】 |
