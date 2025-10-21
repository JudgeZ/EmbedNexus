from __future__ import annotations

import json
from pathlib import Path

from typer.testing import CliRunner

from scripts import routing_matrix


runner = CliRunner()


def test_matrix_generation(tmp_path: Path) -> None:
    destination = tmp_path / "matrix.json"
    result = runner.invoke(routing_matrix.app, ["matrix", "--output", str(destination)])
    assert result.exit_code == 0, result.stdout
    payload = json.loads(destination.read_text())
    assert payload["schema"] == "zaevrynth.routing.matrix"
    assert payload["version"] == routing_matrix.GRAPH_VERSION
    assert payload["adjacency"]["routing-control"]["artifact-store"] == 4


def test_latency_generation(tmp_path: Path) -> None:
    destination = tmp_path / "latency.json"
    result = runner.invoke(routing_matrix.app, ["latency", "--output", str(destination)])
    assert result.exit_code == 0, result.stdout
    payload = json.loads(destination.read_text())
    assert payload["schema"] == "zaevrynth.routing.latency"
    hop = next(item for item in payload["links"] if item["from"] == "artifact-store")
    assert hop["p99"] == routing_matrix.LATENCY_PROFILES[("artifact-store", "audit-log")]["p99"]


def test_fanout_outputs(tmp_path: Path) -> None:
    output_dir = tmp_path / "fanout"
    metrics_path = tmp_path / "fanout-metrics.json"
    result = runner.invoke(
        routing_matrix.app,
        [
            "fanout",
            "--output-dir",
            str(output_dir),
            "--metrics-output",
            str(metrics_path),
        ],
    )
    assert result.exit_code == 0, result.stdout
    manifest = json.loads((output_dir / "manifest.json").read_text())
    assert "burst-ingest.jsonl" in manifest["files"]
    metrics = json.loads(metrics_path.read_text())
    assert metrics["aggregate"]["total_requests"] == 224


def test_transcript_generation(tmp_path: Path) -> None:
    transcript_path = tmp_path / "transcript.log"
    latency_path = tmp_path / "latency.log"
    result = runner.invoke(
        routing_matrix.app,
        [
            "transcript",
            "--output",
            str(transcript_path),
            "--latency-output",
            str(latency_path),
        ],
    )
    assert result.exit_code == 0, result.stdout
    content = transcript_path.read_text()
    assert routing_matrix.TRANSCRIPT_HEADER in content
    latency_content = latency_path.read_text()
    assert "routing-control -> code-search" in latency_content


def test_fuzz_affinity(tmp_path: Path) -> None:
    output = tmp_path / "fuzz.jsonl"
    result = runner.invoke(routing_matrix.app, ["fuzz", "--output", str(output)])
    assert result.exit_code == 0, result.stdout
    lines = [json.loads(line) for line in output.read_text().splitlines()]
    assert lines[0]["tenant"] == "tenant-alpha"
    assert len(lines[0]["preferred_routes"]) == 3
    assert all(abs(sum(entry["weights"]) - 1.0) < 1e-6 for entry in lines)


def test_fanout_requires_destination(tmp_path: Path) -> None:
    result = runner.invoke(routing_matrix.app, ["fanout"])
    assert result.exit_code != 0
    assert "At least one" in result.stderr
