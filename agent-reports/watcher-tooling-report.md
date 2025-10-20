# Filesystem Watcher Tooling Summary

## Implemented Scripts and CLI Contracts
- `scripts/record_fs_events.py` is the only watcher helper with functional code today. Its command line interface now exposes two mutually exclusive modes: (1) `--scenario/--output` for golden regeneration and (2) `--config/--replay-dir` for fixture workflows. Both paths emit deterministic stub data so downstream tooling can rely on stable schemas until the real recorder ships.【F:scripts/record_fs_events.py†L1-L118】
- `scripts/verify_event_order.py` remains a stub and raises `NotImplementedError`, although the module docstring sketches a future `python scripts/verify_event_order.py <path>` interface for validating captured transcripts once implemented.【F:scripts/verify_event_order.py†L1-L19】

## Referenced Configuration and Fixture Assets
- The recorder workflows document `tests/fixtures/filesystem/mock-events.yaml` as the configuration placeholder that will eventually replace historical references to `configs/filesystem/mock-events.yaml`. The file currently only contains comments noting its placeholder status until real captures land.【F:tests/fixtures/filesystem/mock-events.yaml†L1-L2】
- Placeholder latency metrics and replay directories live under `tests/fixtures/filesystem/latency-window.yaml` and `tests/fixtures/filesystem/workspace-replay/` to receive future outputs from extended recorder options (e.g., `--metrics`, `--replay-dir`).【F:tests/fixtures/filesystem/latency-window.yaml†L1-L3】【F:tests/fixtures/filesystem/workspace-replay/README.md†L1-L12】

## Runtime Dependencies
- The scripts catalog calls for Python 3.11 plus the `watchdog` and `pyyaml` packages to support the filesystem recorder stub, while the planned verifier will rely on Python 3.11 and `pyyaml`. These dependencies should be installed in any workflow invoking the watcher utilities.【F:scripts/README.md†L7-L34】

## Emitted Output Paths
- `record_fs_events.py` writes YAML transcripts wherever the caller directs `--output`. The deterministic golden captures that mirror the embedded scenarios are stored under `tests/golden/filesystem/` as `watch-fuzz.log` and `watch-latency-burst.log`. These outputs document the expected structure for future real captures.【F:scripts/record_fs_events.py†L60-L118】【F:tests/golden/filesystem/watch-fuzz.log†L1-L18】【F:tests/golden/filesystem/watch-latency-burst.log†L1-L17】
- In fixture mode (`--config`/`--replay-dir`) the script populates the requested replay directory with a deterministic `mock-events.yaml` summary plus per-scenario YAML transcripts derived from `_SCENARIO_LOGS`, enabling fixture regeneration workflows to proceed without live captures.【F:scripts/record_fs_events.py†L80-L118】
