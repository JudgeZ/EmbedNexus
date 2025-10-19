"""Placeholder for the filesystem event recorder.

Planned responsibilities:
- Load YAML configuration describing watched directories and ignore rules.
- Stream file system events using watchdog observers and normalize them into
  deterministic records for fixture regeneration.
- Persist outputs under `tests/fixtures/filesystem/` with optional replay exports.

Runtime requirements:
- Python 3.11
- Third-party packages: `watchdog`, `pyyaml`

Implementation notes:
- Consider supporting inotify, FSEvents, and ReadDirectoryChangesW via watchdog
  adapters so captures work across Linux, macOS, and Windows hosts.
- Emit structured logging for traceability and include a dry-run mode for test
  validation without disk writes.
"""


def main() -> None:
    """CLI entry point placeholder."""
    raise NotImplementedError("Filesystem event recording is not implemented yet.")


if __name__ == "__main__":  # pragma: no cover - stub only
    main()
