"""Shared fixture bundle builder and validator.

The routing fixture suite emits several small artifacts (matrices, transcripts, fan-out
corpuses, and fuzz hints).  CI workflows package these assets into a shared bundle so
that other subsystems can consume the data without needing to call the generators
directly.  This module provides a Typer-powered CLI with two subcommands:

``build``
    Copies the curated set of routing assets into an output directory and records a
    manifest with SHA-256 digests.  The command enforces a schema/version contract so
    downstream tooling can reject incompatible bundles.

``validate``
    Loads an existing bundle and verifies the schema, version, and checksums.  The
    validator is used both in CI and within the unit tests to guarantee deterministic
    outputs.
"""

from __future__ import annotations

import json
import shutil
from dataclasses import dataclass
from hashlib import sha256
from pathlib import Path
from typing import Iterable, List

import typer


app = typer.Typer(add_completion=False, help="Shared fixture bundle utilities")

BUNDLE_SCHEMA = "zaevrynth.fixtures.shared"
BUNDLE_VERSION = "2024.01"
GENERATED_AT = "2024-01-01T00:00:00Z"


@dataclass
class ArtifactSpec:
    """Describe a source artifact that should be included in a bundle."""

    name: str
    source: Path
    destination: Path


DEFAULT_ARTIFACTS: List[ArtifactSpec] = [
    ArtifactSpec(
        name="routing-matrix",
        source=Path("fixtures/routing/multi-repo-matrix.json"),
        destination=Path("routing/multi-repo-matrix.json"),
    ),
    ArtifactSpec(
        name="routing-latency",
        source=Path("fixtures/routing/latency-matrix.json"),
        destination=Path("routing/latency-matrix.json"),
    ),
    ArtifactSpec(
        name="routing-fanout-manifest",
        source=Path("fixtures/routing/high-fanout/manifest.json"),
        destination=Path("routing/high-fanout/manifest.json"),
    ),
    ArtifactSpec(
        name="routing-fanout-burst",
        source=Path("fixtures/routing/high-fanout/burst-ingest.jsonl"),
        destination=Path("routing/high-fanout/burst-ingest.jsonl"),
    ),
    ArtifactSpec(
        name="routing-fanout-search",
        source=Path("fixtures/routing/high-fanout/federated-search.jsonl"),
        destination=Path("routing/high-fanout/federated-search.jsonl"),
    ),
    ArtifactSpec(
        name="routing-fanout-metrics",
        source=Path("fixtures/routing/fanout-metrics.json"),
        destination=Path("routing/fanout-metrics.json"),
    ),
    ArtifactSpec(
        name="routing-fuzz-affinity",
        source=Path("golden/routing/fuzz-affinity.jsonl"),
        destination=Path("routing/fuzz-affinity.jsonl"),
    ),
]


def _sha256(path: Path) -> str:
    digest = sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(65536), b""):
            digest.update(chunk)
    return digest.hexdigest()


def _copy_artifacts(artifacts: Iterable[ArtifactSpec], source_root: Path, output_root: Path) -> List[dict]:
    manifest_entries: List[dict] = []
    for artifact in artifacts:
        source = source_root / artifact.source
        destination = output_root / artifact.destination
        if not source.exists():
            raise typer.BadParameter(f"Missing source artifact: {source}")
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, destination)
        manifest_entries.append(
            {
                "name": artifact.name,
                "path": str(artifact.destination.as_posix()),
                "sha256": _sha256(destination),
            }
        )
    return manifest_entries


def _write_bundle_manifest(output_root: Path, entries: List[dict]) -> Path:
    manifest_path = output_root / "bundle.json"
    payload = {
        "schema": BUNDLE_SCHEMA,
        "version": BUNDLE_VERSION,
        "generated_at": GENERATED_AT,
        "artifacts": sorted(entries, key=lambda item: item["name"]),
    }
    manifest_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return manifest_path


@app.command()
def build(
    output: Path = typer.Option(..., exists=False, file_okay=False, dir_okay=True, help="Directory to populate"),
    source_root: Path = typer.Option(Path("tests"), file_okay=False, dir_okay=True, exists=True, help="Fixture root"),
) -> None:
    """Assemble the shared routing fixture bundle."""

    output.mkdir(parents=True, exist_ok=True)
    entries = _copy_artifacts(DEFAULT_ARTIFACTS, source_root, output)
    manifest_path = _write_bundle_manifest(output, entries)
    typer.secho(f"Bundle written to {manifest_path}", fg=typer.colors.GREEN)


def _load_manifest(path: Path) -> dict:
    try:
        content = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:  # pragma: no cover - defensive programming
        raise typer.BadParameter(f"Invalid JSON in manifest {path}: {exc}") from exc
    return content


def _validate_entry(bundle_root: Path, entry: dict) -> None:
    expected_path = bundle_root / entry["path"]
    if not expected_path.exists():
        raise typer.BadParameter(f"Missing artifact: {expected_path}")
    if _sha256(expected_path) != entry["sha256"]:
        raise typer.BadParameter(f"Checksum mismatch for {expected_path}")


@app.command()
def validate(bundle_path: Path = typer.Argument(..., exists=True, file_okay=True, dir_okay=True)) -> None:
    """Validate an existing bundle."""

    if bundle_path.is_dir():
        manifest_path = bundle_path / "bundle.json"
    else:
        manifest_path = bundle_path
        bundle_path = manifest_path.parent

    if not manifest_path.exists():
        raise typer.BadParameter(f"Bundle manifest missing: {manifest_path}")

    manifest = _load_manifest(manifest_path)
    if manifest.get("schema") != BUNDLE_SCHEMA:
        raise typer.BadParameter("Unsupported bundle schema")
    if manifest.get("version") != BUNDLE_VERSION:
        raise typer.BadParameter("Unsupported bundle version")

    artifacts = manifest.get("artifacts")
    if not isinstance(artifacts, list):
        raise typer.BadParameter("Manifest artifacts field is invalid")

    for entry in artifacts:
        _validate_entry(bundle_path, entry)

    typer.secho("Bundle validation succeeded", fg=typer.colors.GREEN)


def main() -> None:  # pragma: no cover - exercised via Typer
    app()


if __name__ == "__main__":  # pragma: no cover - CLI entry point
    main()
