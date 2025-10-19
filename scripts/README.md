# Fixture Generation Scripts Overview

This directory collects the multi-language tooling referenced throughout the fixture
and golden-data workflows. Each entry below captures the intended responsibilities,
required runtimes, and external dependencies so future implementations stay aligned
with the guidance in `docs/testing/fixtures-plan.md` and the CI coverage plan.

## Runtime & Dependency Matrix

| Script | Language / Runtime | Key Dependencies | Planned Responsibilities |
| --- | --- | --- | --- |
| `record_fs_events.py` | Python 3.11 | `watchdog`, `pyyaml` | Capture file system events using per-scenario configs and emit deterministic YAML traces. |
| `verify_event_order.py` | Python 3.11 | `pyyaml` | Validate ordering and integrity of previously captured file system event sequences. |
| `archive_builder.rs` | Rust (binary crate) | `cargo` toolchain, compression & serde crates (TBD) | Produce archive fixtures, scenario-specific bundles, and manifest streams for downstream sanitizers. |
| `sanitize_jsonl.py` | Python 3.11 | `click`, `pyyaml`, `jsonschema` (TBD) | Normalize and scrub JSONL manifests emitted by the archive builder prior to promotion to goldens. |
| `trace_capture.sh` | Bash | `openssl`, `tshark`, `jq`, POSIX utilities | Drive TLS/Noise capture sessions and stream traces to fixture directories. |
| `collect_dpapi.ps1` | PowerShell 7+ | Windows DPAPI tooling, EventLog APIs | Collect DPAPI recovery telemetry on domain-joined Windows hosts. |
| `offline_transport_buffer.py` | Python 3.11 | `typer`, `rich`, `pyyaml` | Simulate offline transport buffers, verify queue boundaries, and replay sessions into transcripts. |
| `manifest_replay_harness.rs` | Rust (binary crate) | `cargo` toolchain, async runtime (TBD) | Reproduce ingestion manifest replays with configurable delay profiles for deterministic regression testing. |
| `routing_matrix.py` | Python 3.11 | `networkx`, `typer` | Generate routing matrices, fan-out scenarios, and transcript data for federation harnesses. |
| `fixture_packager.py` | Python 3.11 | `typer`, `pyyaml`, `rich` | Assemble shared fixture bundles and validate schema compatibility across subsystems. |
| `checksums.sh` | Bash | `coreutils` (`sha256sum`), `find`, `xargs` | Generate and verify SHA-256 manifest files for large artifacts. |
| `generate_encryption_toggles.py` | Python 3.11 | `click`, `cryptography`, `pyyaml` | Produce encryption toggle datasets referenced by TLS and DPAPI fixture workflows. |
| `wsl_transport_proxy.ps1` | PowerShell 7+ | Windows `wsl.exe`, networking cmdlets | Bridge Windows transports into WSL for handshake replay and transcript capture. |
| `wsl/python-client.ps1` | Windows PowerShell | `wsl.exe`, client runtime installers | Launch Python client workflows inside WSL from Windows-hosted CI jobs. |

> **Note:** Dependency lists marked `TBD` signal areas where the owning subsystem will
> finalize crate or package selections during implementation. Capture the final decision
> in the script docstring and update this table when the concrete set is known.

## Linting & Formatting Expectations

To keep the multi-language scripts consistent, enforce the following tooling when the
implementations land:

- **Python:** `ruff` for linting and `black` for formatting, targeting Python 3.11.
- **Rust:** `cargo fmt` (rustfmt) and `cargo clippy --all-targets --all-features`.
- **Shell:** `shfmt` for formatting and `shellcheck` for static analysis.
- **PowerShell:** `PSScriptAnalyzer` with the default rule set; format using `Invoke-Formatter`.

Document lint outcomes in pull requests touching this directory and integrate the
checks into CI once the scripts move beyond placeholders.

## Implementation Status

All scripts currently exist as descriptive stubs. Populate the module docstrings or
comment blocks with additional requirements as subsystem owners refine the fixture
workflows. When promoting a stub to a real implementation, replace the placeholder
main entry point with functional code and wire the script into the relevant `Makefile`
or CI workflow to maintain traceability.
