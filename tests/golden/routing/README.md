# Routing Golden Placeholder

Placeholder transcripts and affinity datasets for routing live here. Real
artifacts will be produced by `python scripts/routing_matrix.py` once routing
regression suites are in place.【F:docs/testing/fixtures-plan.md†L164-L168】

## Generation commands

| Command | Required CLI flags | Produces | Dependencies / config |
| --- | --- | --- | --- |
| `python scripts/routing_matrix.py transcript` | `--output tests/golden/routing/mcp-federation.transcript` | Federation replay transcript for MCP interoperability validation.【F:docs/testing/fixtures-plan.md†L166-L167】 | Requires routing matrices from the `matrix` subcommand to provide topology input.【F:docs/testing/fixtures-plan.md†L164-L167】 |
| `python scripts/routing_matrix.py transcript` | `--latency-output tests/golden/routing/multi-repo-latency.transcript` (planned flag) | Cross-repository latency transcript referenced by routing integration tests.【F:tests/golden/routing/multi-repo-latency.transcript†L1-L1】【F:docs/testing/test-matrix.md†L77-L80】 | Shares configuration with the primary transcript; capture after refreshing fan-out fixtures to keep hop timings consistent.【F:docs/testing/test-blueprints.md†L140-L142】 |
| `python scripts/routing_matrix.py fanout` | `--metrics-out tests/golden/routing/fanout-throughput.jsonl` (planned flag) | Throughput guard metrics aligned with high fan-out corpora.【F:tests/golden/routing/fanout-throughput.jsonl†L1-L1】【F:docs/testing/test-blueprints.md†L140-L142】 | Consume the same output directory as the fixtures command so scenario IDs remain stable.【F:docs/testing/test-matrix.md†L80-L80】 |
| `python scripts/routing_matrix.py fuzz` | `--output tests/golden/routing/fuzz-affinity.jsonl` | Repository affinity hints for fuzzing tenant isolation.【F:docs/testing/fixtures-plan.md†L166-L167】【F:tests/golden/routing/fuzz-affinity.jsonl†L1-L1】 | Requires seeded RNG support to reproduce deterministic affinity permutations across CI runs.【F:docs/testing/test-blueprints.md†L139-L142】 |
