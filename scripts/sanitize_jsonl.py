"""JSONL sanitizer for archive builder manifests."""

from __future__ import annotations

import argparse
import contextlib
import json
import sys
from dataclasses import dataclass
from itertools import count
from typing import IO, Dict, Iterator


class SchemaError(RuntimeError):
    """Raised when an input record violates the expected schema."""


@dataclass
class SanitizerState:
    aliases: Dict[str, str]
    counter: Iterator[int]

    def alias_for(self, tenant: str) -> str:
        if tenant not in self.aliases:
            self.aliases[tenant] = f"tenant-{next(self.counter):03d}"
        return self.aliases[tenant]


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Sanitize archive builder JSONL output")
    parser.add_argument(
        "--input",
        "-i",
        default="-",
        help="Input file (defaults to stdin)",
    )
    parser.add_argument(
        "--output",
        "-o",
        default="-",
        help="Output file (defaults to stdout)",
    )
    return parser


def sanitize_stream(source: IO[str], destination: IO[str]) -> None:
    state = SanitizerState(aliases={}, counter=count(1))
    for lineno, raw_line in enumerate(source, start=1):
        line = raw_line.strip()
        if not line:
            continue
        try:
            record = json.loads(line)
        except json.JSONDecodeError as exc:  # pragma: no cover - defensive
            raise SchemaError(f"line {lineno}: invalid JSON: {exc}") from exc
        sanitized = sanitize_record(record, state, lineno)
        json.dump(sanitized, destination, separators=(",", ":"))
        destination.write("\n")


def sanitize_record(record: object, state: SanitizerState, lineno: int) -> Dict[str, object]:
    if not isinstance(record, dict):
        raise SchemaError(f"line {lineno}: record must be a JSON object")

    scenario = record.get("scenario")
    if scenario not in {"fuzz", "quota-throughput"}:
        raise SchemaError(f"line {lineno}: unsupported scenario '{scenario}'")

    tenant = record.get("tenant")
    if not isinstance(tenant, str) or not tenant:
        raise SchemaError(f"line {lineno}: tenant must be a non-empty string")

    alias = state.alias_for(tenant)

    if scenario == "fuzz":
        return sanitize_fuzz(record, alias, lineno)
    return sanitize_throughput(record, alias, lineno)


def sanitize_fuzz(record: Dict[str, object], alias: str, lineno: int) -> Dict[str, object]:
    required_fields = {
        "operation": str,
        "request_count": int,
        "budget_bytes": int,
        "overflow_expected": bool,
    }
    for field, expected_type in required_fields.items():
        if field not in record:
            raise SchemaError(f"line {lineno}: missing '{field}' field")
        if not isinstance(record[field], expected_type):
            raise SchemaError(f"line {lineno}: '{field}' must be of type {expected_type.__name__}")

    request_count = record["request_count"]
    budget_bytes = record["budget_bytes"]
    if request_count < 0:
        raise SchemaError(f"line {lineno}: request_count must be non-negative")
    if budget_bytes < 0:
        raise SchemaError(f"line {lineno}: budget_bytes must be non-negative")

    operation = record["operation"]
    if operation not in {"ingest", "plan", "commit", "audit", "prune"}:
        raise SchemaError(f"line {lineno}: unsupported operation '{operation}'")

    return {
        "scenario": "fuzz",
        "tenant_alias": alias,
        "operation": operation,
        "request_count": request_count,
        "budget_bytes": budget_bytes,
        "overflow_expected": record["overflow_expected"],
    }


def sanitize_throughput(record: Dict[str, object], alias: str, lineno: int) -> Dict[str, object]:
    required_fields = {
        "window_seconds": int,
        "requests": int,
        "average_latency_ms": int,
        "saturation_ratio": (int, float),
    }
    for field, expected_type in required_fields.items():
        if field not in record:
            raise SchemaError(f"line {lineno}: missing '{field}' field")
        value = record[field]
        if isinstance(expected_type, tuple):
            if not isinstance(value, expected_type):
                raise SchemaError(f"line {lineno}: '{field}' must be numeric")
        elif not isinstance(value, expected_type):
            raise SchemaError(f"line {lineno}: '{field}' must be of type {expected_type.__name__}")

    if record["window_seconds"] <= 0:
        raise SchemaError(f"line {lineno}: window_seconds must be positive")
    if record["requests"] < 0:
        raise SchemaError(f"line {lineno}: requests must be non-negative")
    if record["average_latency_ms"] < 0:
        raise SchemaError(f"line {lineno}: average_latency_ms must be non-negative")

    ratio = float(record["saturation_ratio"])
    if not 0.0 <= ratio <= 1.5:
        raise SchemaError(f"line {lineno}: saturation_ratio outside expected bounds")

    return {
        "scenario": "quota-throughput",
        "tenant_alias": alias,
        "window_seconds": record["window_seconds"],
        "requests": record["requests"],
        "average_latency_ms": record["average_latency_ms"],
        "saturation_ratio": round(ratio, 3),
    }


def run_cli(args: argparse.Namespace) -> None:
    input_path = args.input
    output_path = args.output

    with contextlib.ExitStack() as stack:
        if input_path == "-":
            source = sys.stdin
        else:
            source = stack.enter_context(open(input_path, "r", encoding="utf-8"))

        if output_path == "-":
            destination = sys.stdout
        else:
            destination = stack.enter_context(open(output_path, "w", encoding="utf-8"))

        sanitize_stream(source, destination)


def main() -> None:
    parser = build_parser()
    args = parser.parse_args()
    try:
        run_cli(args)
    except SchemaError as exc:
        print(f"sanitize_jsonl: {exc}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":  # pragma: no cover - CLI entry point
    main()
