"""Tests for the deterministic filesystem recorder stub."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

import textwrap


REPO_ROOT = Path(__file__).resolve().parent.parent
SCRIPT = REPO_ROOT / "scripts" / "record_fs_events.py"


def read_text(path: Path) -> str:
    return path.read_text()


def test_scenario_mode(tmp_path: Path) -> None:
    output_path = tmp_path / "watch.log"

    result = subprocess.run(
        [sys.executable, str(SCRIPT), "--scenario", "fuzz", "--output", str(output_path)],
        check=True,
        capture_output=True,
        text=True,
    )

    assert result.returncode == 0
    expected = textwrap.dedent(
        """
        # Filesystem Watch Harness Transcript
        mode: replay
        scenario: fuzz
        source: scripts/record_fs_events.py
        notes:
          - deterministic stub output until watcher harness is implemented
        events:
          - event_id: evt-1
            ts_ms: 0
            action: created
            path: repo/docs/README.md
          - event_id: evt-2
            ts_ms: 12
            action: modified
            path: repo/docs/README.md
            depends_on:
              - evt-1
          - event_id: evt-3
            ts_ms: 24
            action: removed
            path: repo/docs/tmp.out
        summary:
          envelopes_emitted: 3
          event_count: 3
          duration_ms: 24
          anomalies_detected: false
        """
    ).lstrip()
    assert read_text(output_path) == expected


def test_config_mode(tmp_path: Path) -> None:
    config_path = tmp_path / "mock-events.yaml"
    config_path.write_text("# config stub\n")
    replay_dir = tmp_path / "replay"

    result = subprocess.run(
        [
            sys.executable,
            str(SCRIPT),
            "--config",
            str(config_path),
            "--replay-dir",
            str(replay_dir),
        ],
        check=True,
        capture_output=True,
        text=True,
    )

    assert result.returncode == 0

    mock_events = textwrap.dedent(
        f"""
        # Filesystem Recorder Stub
        source: scripts/record_fs_events.py
        mode: config
        config: {config_path.as_posix()}
        scenarios:
          - fuzz
          - latency-burst
        notes:
          - deterministic stub output until watcher harness is implemented
        """
    ).lstrip()
    assert read_text(replay_dir / "mock-events.yaml") == mock_events

    fuzz_replay = replay_dir / "fuzz.yaml"
    latency_replay = replay_dir / "latency-burst.yaml"
    assert fuzz_replay.exists()
    assert latency_replay.exists()
    assert read_text(fuzz_replay).startswith("# Filesystem Watch Harness Transcript")
    assert read_text(latency_replay).startswith(
        "# Filesystem Watch Harness Latency Burst Metrics"
    )
