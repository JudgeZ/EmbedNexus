# High-Fanout Scenarios

`python scripts/routing_matrix.py fanout` emits the JSONL corpora in this
directory. The fixtures cover burst-ingest and federated-search workloads and
include a generated `manifest.json` enumerating the payloads. Pair the command
with `--metrics-output` to capture aggregate throughput statistics alongside the
fixtures so the shared bundle remains synchronized.【F:scripts/routing_matrix.py†L187-L233】
