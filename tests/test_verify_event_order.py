import textwrap
from pathlib import Path

from scripts import verify_event_order


def _write_yaml(path: Path, contents: str) -> None:
    path.write_text(textwrap.dedent(contents).lstrip())


def test_verify_single_file_success(tmp_path: Path) -> None:
    replay = tmp_path / "watch-success.yaml"
    _write_yaml(
        replay,
        """
        scenario: fuzz
        source: scripts/record_fs_events.py
        events:
          - event_id: evt-1
            ts_ms: 0
            action: created
            path: repo/docs/README.md
          - event_id: evt-2
            ts_ms: 5
            action: modified
            path: repo/docs/README.md
            depends_on:
              - evt-1
        summary:
          envelopes_emitted: 2
          anomalies_detected: false
        """,
    )

    summary = verify_event_order.verify_paths([replay])

    assert summary.exit_code == verify_event_order.EXIT_SUCCESS
    assert summary.results[0].issues == []


def test_verify_reports_schema_violations(tmp_path: Path) -> None:
    replay = tmp_path / "watch-missing-fields.yaml"
    _write_yaml(
        replay,
        """
        scenario: fuzz
        source: scripts/record_fs_events.py
        events:
          - ts_ms: "fast"
            action: created
            path: repo/docs/README.md
        """,
    )

    summary = verify_event_order.verify_paths([replay])

    assert summary.exit_code == verify_event_order.EXIT_ERR_SCHEMA
    assert summary.results[0].issues
    assert any("event_id" in issue.message for issue in summary.results[0].issues)


def test_verify_reports_ordering_violations(tmp_path: Path) -> None:
    replay_dir = tmp_path / "workspace-replay"
    replay_dir.mkdir()
    _write_yaml(
        replay_dir / "mock-events.yaml",
        """
        mode: config
        scenarios:
          - fuzz
        notes:
          - placeholder
        """,
    )
    _write_yaml(
        replay_dir / "watch-out-of-order.yaml",
        """
        scenario: fuzz
        source: scripts/record_fs_events.py
        events:
          - event_id: evt-1
            ts_ms: 10
            action: created
            path: repo/docs/README.md
          - event_id: evt-2
            ts_ms: 5
            action: modified
            path: repo/docs/README.md
            depends_on:
              - evt-1
        """,
    )

    summary = verify_event_order.verify_paths([replay_dir])

    assert summary.exit_code == verify_event_order.EXIT_ERR_ORDER
    (result,) = [r for r in summary.results if not r.skipped]
    assert any("non-decreasing" in issue.message for issue in result.issues)
