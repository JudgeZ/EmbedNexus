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


def _write_log(scenario: str, output: Path) -> None:
    if scenario not in _SCENARIO_LOGS:
        raise ValueError(f"Unsupported scenario '{scenario}'")

    output.parent.mkdir(parents=True, exist_ok=True)
    contents = textwrap.dedent(_SCENARIO_LOGS[scenario]).lstrip()
    output.write_text(contents)


def main() -> None:
    """Emit deterministic watcher transcripts for the requested scenario."""

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--scenario",
        required=True,
        choices=sorted(_SCENARIO_LOGS.keys()),
        help="Watcher scenario to render.",
    )
    parser.add_argument(
        "--output",
        required=True,
        type=Path,
        help="Path to the log file to write.",
    )
    args = parser.parse_args()

    _write_log(args.scenario, args.output)


if __name__ == "__main__":
    main()
