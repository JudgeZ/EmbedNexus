"""Utility to emit deterministic filesystem watcher transcripts.

The real filesystem watcher harness is still under development.  Until the
implementation lands we provide a lightweight stub that produces deterministic
log transcripts for the golden regeneration workflow.  This keeps the CI job
green while documenting the expected interface for the future recorder.
"""

from __future__ import annotations

import argparse
from pathlib import Path
import textwrap
from typing import Dict


_SCENARIO_LOGS: Dict[str, str] = {
    "fuzz": """
        # Filesystem Watch Harness Transcript
        scenario: fuzz
        source: scripts/record_fs_events.py
        notes:
          - deterministic stub output until watcher harness is implemented
        events:
          - ts_ms: 0
            action: created
            path: repo/docs/README.md
          - ts_ms: 12
            action: modified
            path: repo/docs/README.md
          - ts_ms: 24
            action: removed
            path: repo/docs/tmp.out
        summary:
          envelopes_emitted: 3
          anomalies_detected: false
    """,
    "latency-burst": """
        # Filesystem Watch Harness Latency Burst Metrics
        scenario: latency-burst
        source: scripts/record_fs_events.py
        notes:
          - deterministic stub output until watcher harness is implemented
        buckets:
          - window_ms: 25
            events_observed: 16
            max_latency_ms: 18
          - window_ms: 50
            events_observed: 32
            max_latency_ms: 41
          - window_ms: 100
            events_observed: 64
            max_latency_ms: 172
        summary:
          sustained_latency_ms_p99: 172
          max_queue_depth: 9
    """,
}


_MOCK_EVENTS_TEMPLATE = """# Filesystem Recorder Stub
source: scripts/record_fs_events.py
mode: config
config: {config_value}
scenarios:
{scenarios}
notes:
  - deterministic stub output until watcher harness is implemented
"""


def _write_log(scenario: str, output: Path) -> None:
    if scenario not in _SCENARIO_LOGS:
        raise ValueError(f"Unsupported scenario '{scenario}'")

    output.parent.mkdir(parents=True, exist_ok=True)
    contents = textwrap.dedent(_SCENARIO_LOGS[scenario]).lstrip()
    output.write_text(contents)


def _write_config_mode_outputs(config: Path, replay_dir: Path) -> None:
    replay_dir.mkdir(parents=True, exist_ok=True)

    scenario_lines = "\n".join(f"  - {name}" for name in sorted(_SCENARIO_LOGS.keys()))
    mock_events_path = replay_dir / "mock-events.yaml"
    mock_events_contents = _MOCK_EVENTS_TEMPLATE.format(
        config_value=config.as_posix(),
        scenarios=scenario_lines,
    ).rstrip()
    mock_events_path.write_text(f"{mock_events_contents}\n")

    for scenario in sorted(_SCENARIO_LOGS.keys()):
        _write_log(scenario, replay_dir / f"{scenario}.yaml")


def main() -> None:
    """Emit deterministic watcher transcripts for golden or fixture workflows."""

    parser = argparse.ArgumentParser(description=__doc__)
    mode_group = parser.add_mutually_exclusive_group(required=True)
    mode_group.add_argument(
        "--scenario",
        choices=sorted(_SCENARIO_LOGS.keys()),
        help="Watcher scenario to render.",
    )
    mode_group.add_argument(
        "--config",
        type=Path,
        help="Fixture configuration file to replay (stubbed).",
    )
    parser.add_argument(
        "--output",
        type=Path,
        help="Path to the log file to write.",
    )
    parser.add_argument(
        "--replay-dir",
        type=Path,
        help="Directory to populate with deterministic replay artifacts.",
    )
    args = parser.parse_args()

    if args.scenario:
        if args.output is None:
            parser.error("--output is required when --scenario is provided.")
        _write_log(args.scenario, args.output)
        return

    if args.config:
        if args.replay_dir is None:
            parser.error("--replay-dir is required when --config is provided.")
        _write_config_mode_outputs(args.config, args.replay_dir)
        return

    parser.error("One of --scenario/--output or --config/--replay-dir must be specified.")


if __name__ == "__main__":
    main()
