#!/usr/bin/env python3
"""Normalize transcript JSON files for IDE and client fixtures.

This tool enforces a canonical layout for transcript entries so that captured
client exchanges can be diffed reliably across platforms. It normalizes
primitive fields (direction, timestamps) and recursively sorts all envelope
objects before writing the result with deterministic indentation.
"""
from __future__ import annotations

import argparse
import json
import sys
from dataclasses import dataclass
from datetime import datetime, timezone, timedelta
from pathlib import Path
from typing import Any, Iterable, List, Sequence

_VALID_DIRECTIONS = {"send", "receive"}


@dataclass
class TranscriptNormalizationError(Exception):
    """Raised when a transcript cannot be normalized."""

    message: str

    def __str__(self) -> str:  # pragma: no cover - trivial
        return self.message


def parse_args(argv: Sequence[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Normalize MCP transcript JSON files for deterministic diffs.",
    )
    parser.add_argument(
        "paths",
        nargs="+",
        help="Transcript JSON files to normalize.",
    )
    parser.add_argument(
        "--in-place",
        action="store_true",
        help="Rewrite each transcript in place instead of printing to stdout.",
    )
    parser.add_argument(
        "--output",
        type=Path,
        help="Write the normalized transcript to this path (single input only).",
    )
    parser.add_argument(
        "--check",
        action="store_true",
        help="Exit with status 1 if any transcript would change after normalization.",
    )
    return parser.parse_args(argv)


def normalize_transcript(records: Any) -> List[dict[str, Any]]:
    if not isinstance(records, list):
        raise TranscriptNormalizationError(
            "Transcript must be a JSON array of exchange records.",
        )

    normalized: List[dict[str, Any]] = []
    for index, record in enumerate(records):
        if not isinstance(record, dict):
            raise TranscriptNormalizationError(
                f"Record {index} must be a JSON object, found {type(record).__name__}.",
            )
        normalized.append(_normalize_record(record, index))
    return normalized


def _normalize_record(record: dict[str, Any], index: int) -> dict[str, Any]:
    try:
        direction_raw = record["direction"]
    except KeyError as exc:  # pragma: no cover - defensive
        raise TranscriptNormalizationError(
            f"Record {index} is missing required field 'direction'.",
        ) from exc
    direction = str(direction_raw).lower()
    if direction not in _VALID_DIRECTIONS:
        raise TranscriptNormalizationError(
            f"Record {index} direction must be one of {_VALID_DIRECTIONS}, got {direction_raw!r}.",
        )

    try:
        timestamp_raw = record["timestamp"]
    except KeyError as exc:  # pragma: no cover - defensive
        raise TranscriptNormalizationError(
            f"Record {index} is missing required field 'timestamp'.",
        ) from exc
    timestamp = _normalize_timestamp(str(timestamp_raw))

    try:
        envelope_raw = record["envelope"]
    except KeyError as exc:  # pragma: no cover - defensive
        raise TranscriptNormalizationError(
            f"Record {index} is missing required field 'envelope'.",
        ) from exc
    envelope = _sort_json_tree(envelope_raw)

    normalized: dict[str, Any] = {
        "direction": direction,
        "timestamp": timestamp,
        "envelope": envelope,
    }

    # Preserve any additional metadata keys in sorted order for determinism.
    for key in sorted(k for k in record.keys() if k not in {"direction", "timestamp", "envelope"}):
        normalized[key] = _sort_json_tree(record[key])

    return normalized


def _normalize_timestamp(value: str) -> str:
    # Accept timestamps with or without timezone suffixes. Default to UTC when absent.
    parsed = _parse_datetime(value)
    parsed_utc = parsed.astimezone(timezone.utc)

    # Round to the nearest millisecond for stable comparisons.
    microseconds = parsed_utc.microsecond
    milliseconds = int(round(microseconds / 1000.0))
    if milliseconds == 1000:
        parsed_utc += timedelta(seconds=1)
        milliseconds = 0
    parsed_utc = parsed_utc.replace(microsecond=milliseconds * 1000)

    return parsed_utc.isoformat(timespec="milliseconds").replace("+00:00", "Z")


def _parse_datetime(value: str) -> datetime:
    # Handle the `Z` suffix and fallback to naive parsing if needed.
    if value.endswith("Z"):
        value = value[:-1] + "+00:00"
    try:
        parsed = datetime.fromisoformat(value)
    except ValueError as exc:
        raise TranscriptNormalizationError(
            f"Invalid ISO-8601 timestamp: {value!r}",
        ) from exc
    if parsed.tzinfo is None:
        parsed = parsed.replace(tzinfo=timezone.utc)
    return parsed


def _sort_json_tree(node: Any) -> Any:
    if isinstance(node, dict):
        return {key: _sort_json_tree(node[key]) for key in sorted(node.keys())}
    if isinstance(node, list):
        return [_sort_json_tree(item) for item in node]
    return node


def _serialize(records: Iterable[dict[str, Any]]) -> str:
    return json.dumps(list(records), indent=2, ensure_ascii=False) + "\n"


def process_file(path: Path, *, in_place: bool, output: Path | None, check: bool) -> bool:
    content = path.read_text(encoding="utf-8")
    try:
        data = json.loads(content)
        normalized = normalize_transcript(data)
    except (json.JSONDecodeError, TranscriptNormalizationError) as exc:
        raise TranscriptNormalizationError(f"{path}: {exc}")

    serialized = _serialize(normalized)
    if check:
        if content != serialized:
            return True
        return False

    if output is not None:
        output.write_text(serialized, encoding="utf-8")
    elif in_place:
        path.write_text(serialized, encoding="utf-8")
    else:
        sys.stdout.write(serialized)
    return content != serialized


def main(argv: Sequence[str]) -> int:
    args = parse_args(argv)
    paths = [Path(p) for p in args.paths]

    if args.output and len(paths) != 1:
        raise SystemExit("--output can only be used with a single input path")
    if not args.in_place and not args.check and not args.output and len(paths) > 1:
        raise SystemExit("provide --in-place or --output when normalizing multiple files")

    any_changes = False
    for path in paths:
        changed = process_file(
            path,
            in_place=args.in_place,
            output=args.output,
            check=args.check,
        )
        any_changes = any_changes or changed

    if args.check and any_changes:
        return 1
    return 0


if __name__ == "__main__":  # pragma: no cover - CLI entry point
    sys.exit(main(sys.argv[1:]))
