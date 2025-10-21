from __future__ import annotations

import json
from pathlib import Path

from typer.testing import CliRunner

from scripts import fixture_packager, routing_matrix


runner = CliRunner()


def _prime_routing_fixtures(root: Path) -> None:
    fixtures_root = root / "fixtures" / "routing"
    golden_root = root / "golden" / "routing"
    runner.invoke(routing_matrix.app, ["matrix", "--output", str(fixtures_root / "multi-repo-matrix.json")])
    runner.invoke(routing_matrix.app, ["latency", "--output", str(fixtures_root / "latency-matrix.json")])
    runner.invoke(
        routing_matrix.app,
        [
            "fanout",
            "--output-dir",
            str(fixtures_root / "high-fanout"),
            "--metrics-output",
            str(fixtures_root / "fanout-metrics.json"),
        ],
    )
    runner.invoke(routing_matrix.app, ["fuzz", "--output", str(golden_root / "fuzz-affinity.jsonl")])


def test_build_and_validate(tmp_path: Path) -> None:
    data_root = tmp_path / "data"
    data_root.mkdir()
    _prime_routing_fixtures(data_root)

    bundle_dir = tmp_path / "bundle"
    result = runner.invoke(
        fixture_packager.app,
        [
            "build",
            "--output",
            str(bundle_dir),
            "--source-root",
            str(data_root),
        ],
    )
    assert result.exit_code == 0, result.stdout
    manifest = json.loads((bundle_dir / "bundle.json").read_text())
    assert manifest["schema"] == fixture_packager.BUNDLE_SCHEMA
    assert len(manifest["artifacts"]) == len(fixture_packager.DEFAULT_ARTIFACTS)

    validate_result = runner.invoke(fixture_packager.app, ["validate", str(bundle_dir)])
    assert validate_result.exit_code == 0, validate_result.stdout


def test_validate_rejects_checksum_mismatch(tmp_path: Path) -> None:
    data_root = tmp_path / "data"
    data_root.mkdir()
    _prime_routing_fixtures(data_root)
    bundle_dir = tmp_path / "bundle"
    runner.invoke(
        fixture_packager.app,
        [
            "build",
            "--output",
            str(bundle_dir),
            "--source-root",
            str(data_root),
        ],
    )

    manifest_path = bundle_dir / "bundle.json"
    manifest = json.loads(manifest_path.read_text())
    manifest["artifacts"][0]["sha256"] = "deadbeef"
    manifest_path.write_text(json.dumps(manifest, indent=2))

    result = runner.invoke(fixture_packager.app, ["validate", str(bundle_dir)])
    assert result.exit_code != 0
    assert "Checksum mismatch" in result.stderr


def test_validate_rejects_schema(tmp_path: Path) -> None:
    data_root = tmp_path / "data"
    data_root.mkdir()
    _prime_routing_fixtures(data_root)
    bundle_dir = tmp_path / "bundle"
    runner.invoke(
        fixture_packager.app,
        [
            "build",
            "--output",
            str(bundle_dir),
            "--source-root",
            str(data_root),
        ],
    )

    manifest_path = bundle_dir / "bundle.json"
    manifest = json.loads(manifest_path.read_text())
    manifest["version"] = "2099.99"
    manifest_path.write_text(json.dumps(manifest, indent=2))

    result = runner.invoke(fixture_packager.app, ["validate", str(bundle_dir)])
    assert result.exit_code != 0
    assert "Unsupported bundle version" in result.stderr
