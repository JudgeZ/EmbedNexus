"""Validate causal ordering of recorded filesystem event transcripts."""

from __future__ import annotations

import argparse
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, List, Sequence

import yaml

EXIT_SUCCESS = 0
EXIT_ERR_LOAD = 1
EXIT_ERR_SCHEMA = 2
EXIT_ERR_ORDER = 3


@dataclass(frozen=True)
class Issue:
    """Represents a single validation problem."""

    message: str
    exit_code: int


@dataclass(frozen=True)
class VerificationResult:
    """Outcome of validating one replay file."""

    path: Path
    issues: List[Issue]
    skipped: bool = False

    @property
    def ok(self) -> bool:
        return not self.issues and not self.skipped


@dataclass(frozen=True)
class VerificationSummary:
    """Aggregated verification results."""

    results: List[VerificationResult]

    @property
    def exit_code(self) -> int:
        codes = [issue.exit_code for result in self.results for issue in result.issues]
        return max(codes, default=EXIT_SUCCESS)


def _yaml_paths(path: Path) -> List[Path]:
    if path.is_dir():
        files = sorted(p for p in path.iterdir() if p.suffix in {".yaml", ".yml"})
        if not files:
            return [path]
        return files
    return [path]


def _load_yaml(path: Path) -> tuple[object, list[Issue]]:
    try:
        text = path.read_text()
    except FileNotFoundError:
        return None, [Issue(f"Replay file not found: {path}", EXIT_ERR_LOAD)]
    except OSError as exc:  # pragma: no cover - defensive
        return None, [Issue(f"Unable to read replay file {path}: {exc}", EXIT_ERR_LOAD)]

    try:
        data = yaml.safe_load(text) if text.strip() else {}
    except yaml.YAMLError as exc:
        return None, [Issue(f"Failed to parse YAML for {path}: {exc}", EXIT_ERR_LOAD)]

    return data, []


def _validate_replay(data: object) -> tuple[list[Issue], bool]:
    if not isinstance(data, dict):
        return [Issue("Replay root must be a mapping", EXIT_ERR_SCHEMA)], False

    mode = data.get("mode")
    if mode == "config" and "events" not in data:
        return [], True
    if mode not in (None, "replay") and "events" not in data:
        return [], True

    issues: list[Issue] = []
    if mode not in (None, "replay"):
        issues.append(Issue(f"Unsupported replay mode '{mode}'", EXIT_ERR_SCHEMA))

    scenario = data.get("scenario")
    if not isinstance(scenario, str) or not scenario:
        issues.append(Issue("Missing string 'scenario' metadata", EXIT_ERR_SCHEMA))

    source = data.get("source")
    if not isinstance(source, str) or not source:
        issues.append(Issue("Missing string 'source' metadata", EXIT_ERR_SCHEMA))

    events = data.get("events")
    if not isinstance(events, list) or not events:
        issues.append(Issue("Replay must provide a non-empty 'events' list", EXIT_ERR_SCHEMA))
        return issues, False

    seen_ids: set[str] = set()
    previous_ts: int | None = None

    for index, event in enumerate(events):
        if not isinstance(event, dict):
            issues.append(Issue(f"Event {index} must be a mapping", EXIT_ERR_SCHEMA))
            continue

        event_id = event.get("event_id")
        if not isinstance(event_id, str) or not event_id:
            issues.append(Issue(f"Event {index} is missing a string 'event_id'", EXIT_ERR_SCHEMA))
        elif event_id in seen_ids:
            issues.append(Issue(f"Duplicate event_id '{event_id}'", EXIT_ERR_SCHEMA))
        else:
            seen_ids.add(event_id)

        ts_ms = event.get("ts_ms")
        if not isinstance(ts_ms, int):
            issues.append(Issue(f"Event {event_id or index} must declare integer 'ts_ms'", EXIT_ERR_SCHEMA))
        else:
            if previous_ts is not None and ts_ms < previous_ts:
                issues.append(
                    Issue(
                        f"Event {event_id or index} timestamp {ts_ms} is not non-decreasing (previous {previous_ts})",
                        EXIT_ERR_ORDER,
                    )
                )
            previous_ts = ts_ms

        deps = event.get("depends_on", [])
        if deps:
            if not isinstance(deps, list) or not all(isinstance(dep, str) for dep in deps):
                issues.append(
                    Issue(
                        f"Event {event_id or index} has an invalid 'depends_on' list",
                        EXIT_ERR_SCHEMA,
                    )
                )
            else:
                missing = [dep for dep in deps if dep not in seen_ids]
                if missing:
                    issues.append(
                        Issue(
                            f"Event {event_id or index} depends on unknown events: {', '.join(missing)}",
                            EXIT_ERR_ORDER,
                        )
                    )

    return issues, False


def verify_paths(paths: Iterable[Path]) -> VerificationSummary:
    results: list[VerificationResult] = []
    for raw in paths:
        path = Path(raw)
        for candidate in _yaml_paths(path):
            data, load_issues = _load_yaml(candidate)
            if load_issues:
                results.append(VerificationResult(candidate, load_issues))
                continue

            issues, skipped = _validate_replay(data)
            results.append(VerificationResult(candidate, issues, skipped=skipped))

    return VerificationSummary(results)


def _parse_args(argv: Sequence[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "paths",
        nargs="+",
        help="Replay YAML file(s) or directories containing replay assets.",
    )
    return parser.parse_args(argv)


def main(argv: Sequence[str] | None = None) -> int:
    args = _parse_args(argv)
    summary = verify_paths(Path(p) for p in args.paths)

    for result in summary.results:
        if result.skipped:
            continue
        if not result.issues:
            print(f"OK \u2713 {result.path}")
            continue
        for issue in result.issues:
            print(f"ERROR ({issue.exit_code}) {result.path}: {issue.message}")

    return summary.exit_code


if __name__ == "__main__":  # pragma: no cover - CLI execution
    raise SystemExit(main())
