# Routing Goldens

Deterministic routing goldens generated via `scripts/routing_matrix.py` reside
here. They provide federation transcripts, latency notes, fan-out throughput
metrics, and seeded fuzz-affinity hints that keep CI runs reproducible.【F:scripts/routing_matrix.py†L187-L266】

## Generation commands

| Command | Required CLI flags | Produces | Dependencies / config |
| --- | --- | --- | --- |
| `python scripts/routing_matrix.py transcript` | `--output tests/golden/routing/mcp-federation.transcript --latency-output tests/golden/routing/multi-repo-latency.transcript` | Federation replay transcript and hop latency notes consumed by regression suites.【F:scripts/routing_matrix.py†L235-L266】 | Requires the deterministic topology emitted by the `matrix` and `latency` subcommands. |
| `python scripts/routing_matrix.py fanout` | `--metrics-output tests/golden/routing/fanout-throughput.jsonl` | Aggregated throughput metrics aligned with the high fan-out fixtures.【F:scripts/routing_matrix.py†L187-L233】【F:tests/golden/routing/fanout-throughput.jsonl†L1-L15】 | Optional `--output-dir` flag can refresh fixture JSONL payloads in the same run. |
| `python scripts/routing_matrix.py fuzz` | `--output tests/golden/routing/fuzz-affinity.jsonl` | Repository affinity hints for fuzzing tenant isolation.【F:scripts/routing_matrix.py†L268-L279】【F:tests/golden/routing/fuzz-affinity.jsonl†L1-L5】 | Uses a seeded RNG to keep permutations deterministic across CI jobs. |
