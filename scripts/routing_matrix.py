"""Generate deterministic routing fixtures and goldens.

The CLI provided by this module emits the small, deterministic datasets used by the
test suite and CI regeneration workflows.  Each subcommand owns a specific slice
of routing evidence:

* ``matrix`` – multi-repository adjacency metadata consumed by unit tests.
* ``latency`` – hop latency budgets paired with control-plane metrics.
* ``fanout`` – high fan-out fixture corpora and their corresponding throughput
  summaries.
* ``transcript`` – federation transcripts used by the MCP compatibility harness.
* ``fuzz`` – seeded repository affinity hints for isolation fuzzing.

All payloads are deterministic and tied to the constants defined in this module so
that regenerations stay stable across environments.  The command outputs are kept
intentionally small (a few dozen lines) to simplify the assertions in the test
suite while still exercising realistic data shapes.
"""

from __future__ import annotations

import json
from dataclasses import dataclass
from hashlib import sha256
from pathlib import Path
from random import Random
from typing import Dict, Iterable, List, Sequence

import typer


app = typer.Typer(add_completion=False, help="Routing matrix and transcript generator")


GRAPH_VERSION = "2024.01"
GENERATED_AT = "2024-01-01T00:00:00Z"


@dataclass(frozen=True)
class Node:
    """Single routing node definition."""

    identifier: str
    tier: str
    replicas: int


@dataclass(frozen=True)
class Edge:
    """Directional routing edge between nodes."""

    source: str
    target: str
    weight: int


NODES: Sequence[Node] = (
    Node("ingest-api", "edge", 3),
    Node("routing-control", "control", 5),
    Node("artifact-store", "storage", 4),
    Node("code-search", "compute", 6),
    Node("audit-log", "audit", 2),
)

EDGES: Sequence[Edge] = (
    Edge("ingest-api", "routing-control", 3),
    Edge("routing-control", "artifact-store", 4),
    Edge("routing-control", "code-search", 5),
    Edge("code-search", "artifact-store", 2),
    Edge("artifact-store", "audit-log", 1),
)

LATENCY_PROFILES = {
    ("ingest-api", "routing-control"): {"p50": 11, "p95": 23, "p99": 37},
    ("routing-control", "artifact-store"): {"p50": 15, "p95": 32, "p99": 48},
    ("routing-control", "code-search"): {"p50": 19, "p95": 35, "p99": 51},
    ("code-search", "artifact-store"): {"p50": 13, "p95": 26, "p99": 39},
    ("artifact-store", "audit-log"): {"p50": 8, "p95": 14, "p99": 20},
}

FANOUT_SCENARIOS = {
    "burst-ingest.jsonl": (
        {
            "scenario": "burst-ingest",
            "tenant": "tenant-alpha",
            "routes": ["routing-control", "artifact-store"],
            "requests": 128,
            "expected_p99_latency_ms": 185,
        },
        {
            "scenario": "burst-ingest",
            "tenant": "tenant-beta",
            "routes": ["routing-control", "code-search", "artifact-store"],
            "requests": 96,
            "expected_p99_latency_ms": 212,
        },
    ),
    "federated-search.jsonl": (
        {
            "scenario": "federated-search",
            "tenant": "tenant-gamma",
            "routes": ["routing-control", "code-search"],
            "queries_per_second": 42,
            "replica_spread": 3,
        },
        {
            "scenario": "federated-search",
            "tenant": "tenant-delta",
            "routes": ["routing-control", "code-search", "artifact-store"],
            "queries_per_second": 38,
            "replica_spread": 2,
        },
    ),
}

TRANSCRIPT_HEADER = "# Zaevrynth Multi-Repo Federation Transcript v1"
TRANSCRIPT_EVENTS = (
    "000 ingress-api -> routing-control [route:init] latency=11ms",
    "001 routing-control -> artifact-store [prefetch] latency=15ms",
    "002 routing-control -> code-search [span:compute] latency=19ms",
    "003 code-search -> artifact-store [hydrate] latency=13ms",
    "004 artifact-store -> audit-log [archive] latency=8ms",
)

FUZZ_TENANTS = (
    "tenant-alpha",
    "tenant-beta",
    "tenant-gamma",
    "tenant-delta",
    "tenant-epsilon",
)
FUZZ_SEED = 71023


def _write_json(path: Path, payload: Dict[str, object]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def _write_jsonl(path: Path, rows: Iterable[Dict[str, object]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as handle:
        for row in rows:
            json.dump(row, handle, sort_keys=True)
            handle.write("\n")


def _build_adjacency() -> Dict[str, Dict[str, int]]:
    adjacency: Dict[str, Dict[str, int]] = {node.identifier: {} for node in NODES}
    for edge in EDGES:
        adjacency[edge.source][edge.target] = edge.weight
    return adjacency


def _latency_metrics() -> Dict[str, object]:
    longest = max(profile["p95"] for profile in LATENCY_PROFILES.values())
    average = round(sum(profile["p50"] for profile in LATENCY_PROFILES.values()) / len(LATENCY_PROFILES), 2)
    return {
        "max_hop_p95_ms": longest,
        "mean_p50_ms": average,
        "budget_ms": 275,
    }


def _fanout_metrics(rows: Iterable[Dict[str, object]]) -> Dict[str, object]:
    tenants: List[str] = []
    total_requests = 0
    total_qps = 0
    for row in rows:
        tenants.append(row["tenant"])
        if "requests" in row:
            total_requests += int(row["requests"])
        if "queries_per_second" in row:
            total_qps += int(row["queries_per_second"])
    return {
        "tenants": sorted(set(tenants)),
        "total_requests": total_requests,
        "total_qps": total_qps,
    }


def _fuzz_rows() -> List[Dict[str, object]]:
    rng = Random(FUZZ_SEED)
    repositories = [node.identifier for node in NODES if node.identifier != "ingest-api"]
    rows: List[Dict[str, object]] = []
    for tenant in FUZZ_TENANTS:
        shuffled = repositories.copy()
        rng.shuffle(shuffled)
        preferred = shuffled[:3]
        weights = [round(value, 2) for value in _normalise_weights(len(preferred))]
        rows.append(
            {
                "tenant": tenant,
                "preferred_routes": preferred,
                "weights": weights,
                "seed": FUZZ_SEED,
            }
        )
    return rows


def _normalise_weights(length: int) -> List[float]:
    base = [float(index + 1) for index in range(length)]
    total = sum(base)
    return [value / total for value in base]


@app.command()
def matrix(
    output: Path = typer.Option(..., exists=False, dir_okay=False, writable=True, help="Destination file for the adjacency matrix"),
) -> None:
    """Emit the deterministic routing adjacency matrix."""

    payload = {
        "schema": "zaevrynth.routing.matrix",
        "version": GRAPH_VERSION,
        "generated_at": GENERATED_AT,
        "nodes": [
            {
                "id": node.identifier,
                "tier": node.tier,
                "replicas": node.replicas,
            }
            for node in NODES
        ],
        "edges": [
            {
                "from": edge.source,
                "to": edge.target,
                "weight": edge.weight,
            }
            for edge in EDGES
        ],
        "adjacency": _build_adjacency(),
    }
    _write_json(output, payload)


@app.command()
def latency(
    output: Path = typer.Option(..., exists=False, dir_okay=False, writable=True, help="Destination file for the latency matrix"),
) -> None:
    """Emit latency expectations for every routing hop."""

    payload = {
        "schema": "zaevrynth.routing.latency",
        "version": GRAPH_VERSION,
        "generated_at": GENERATED_AT,
        "links": [
            {
                "from": source,
                "to": target,
                **metrics,
            }
            for (source, target), metrics in LATENCY_PROFILES.items()
        ],
        "summary": _latency_metrics(),
    }
    _write_json(output, payload)


@app.command()
def fanout(
    output_dir: Path = typer.Option(None, dir_okay=True, file_okay=False, help="Fixture directory for high fan-out corpora"),
    metrics_output: Path = typer.Option(None, dir_okay=False, file_okay=True, help="Optional throughput metrics output path"),
) -> None:
    """Generate high fan-out fixture corpora and optional throughput metrics."""

    if output_dir is None and metrics_output is None:
        raise typer.BadParameter("At least one of --output-dir or --metrics-output must be provided.")

    metrics_rows: List[Dict[str, object]] = []
    if output_dir is not None:
        output_dir.mkdir(parents=True, exist_ok=True)
        for filename, rows in FANOUT_SCENARIOS.items():
            path = output_dir / filename
            _write_jsonl(path, rows)
            metrics_rows.extend(rows)
        manifest = {
            "schema": "zaevrynth.routing.fanout",
            "version": GRAPH_VERSION,
            "files": sorted(str((output_dir / name).name) for name in FANOUT_SCENARIOS),
        }
        _write_json(output_dir / "manifest.json", manifest)

    if metrics_output is not None:
        if not metrics_rows and output_dir is None:
            # When only metrics are requested we derive them from the static scenarios.
            for rows in FANOUT_SCENARIOS.values():
                metrics_rows.extend(rows)
        metrics_payload = {
            "schema": "zaevrynth.routing.fanout-metrics",
            "version": GRAPH_VERSION,
            "generated_at": GENERATED_AT,
            "aggregate": _fanout_metrics(metrics_rows),
        }
        _write_json(metrics_output, metrics_payload)


@app.command()
def transcript(
    output: Path = typer.Option(..., exists=False, dir_okay=False, writable=True, help="Primary transcript output path"),
    latency_output: Path = typer.Option(None, dir_okay=False, file_okay=True, help="Optional latency transcript output"),
) -> None:
    """Emit deterministic federation transcripts."""

    output.parent.mkdir(parents=True, exist_ok=True)
    lines = [TRANSCRIPT_HEADER, f"generated_at: {GENERATED_AT}"]
    lines.extend(TRANSCRIPT_EVENTS)
    output.write_text("\n".join(lines) + "\n", encoding="utf-8")

    if latency_output is not None:
        latency_output.parent.mkdir(parents=True, exist_ok=True)
        hop_lines = ["# Multi-repo latency transcript", f"generated_at: {GENERATED_AT}"]
        for (source, target), metrics in LATENCY_PROFILES.items():
            hop_lines.append(
                f"{source} -> {target} p50={metrics['p50']}ms p95={metrics['p95']}ms p99={metrics['p99']}ms"
            )
        latency_output.write_text("\n".join(hop_lines) + "\n", encoding="utf-8")


@app.command()
def fuzz(
    output: Path = typer.Option(..., exists=False, dir_okay=False, writable=True, help="Destination for fuzz affinity hints"),
) -> None:
    """Emit seeded routing affinity hints for fuzzing isolation tests."""

    rows = _fuzz_rows()
    _write_jsonl(output, rows)


def _sha256_file(path: Path) -> str:
    digest = sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(65536), b""):
            digest.update(chunk)
    return digest.hexdigest()


@app.command()
def inspect(
    path: Path = typer.Argument(..., exists=True, readable=True, help="File generated by this script"),
) -> None:
    """Helper command used in development to view payload digests."""

    digest = _sha256_file(path)
    typer.echo(f"sha256:{digest}  {path}")


def main() -> None:  # pragma: no cover - exercised via Typer CLI
    app()


if __name__ == "__main__":  # pragma: no cover - CLI entry point
    main()
