# Scripts Implementation Plan

This plan documents how each placeholder script under `scripts/` will mature from stub to
production tooling. It summarizes responsibilities, third-party dependencies,
platform assumptions, acceptance criteria, and the failing tests or fixtures that
will consume each helper. The roadmap aligns implementation sequencing with the
`Regenerate Fixture Corpus` and `Regenerate Golden Artifacts` workflows so CI
automation can be enabled as individual tools land.【F:scripts/README.md†L3-L53】【F:docs/testing/test-matrix.md†L103-L114】【F:.github/workflows/regenerate-fixtures.yml†L1-L144】【F:.github/workflows/regenerate-goldens.yml†L1-L149】

## Phase Ordering

| Phase | Focus | Workflow steps unblocked | Guard removal & gating | Production verification |
| --- | --- | --- | --- | --- |
| 1. **Filesystem capture foundation** | Implement Python helpers that unblock deterministic filesystem fixtures so the Filesystem Watch Service tests can move from pending to active. (Scripts: `record_fs_events.py`, `verify_event_order.py`).【F:docs/testing/fixtures-plan.md†L93-L101】 | - `Generate filesystem event fixtures` (`Regenerate Fixture Corpus`).【F:.github/workflows/regenerate-fixtures.yml†L105-L118】<br>- `Generate filesystem watcher goldens` (`Regenerate Golden Artifacts`).【F:.github/workflows/regenerate-goldens.yml†L193-L202】 | - Delete the `grep -q 'Placeholder' scripts/record_fs_events.py` and nested `verify_event_order.py` guards once the capture and verifier ship.【F:.github/workflows/regenerate-fixtures.yml†L109-L117】<br>- Remove the `grep -q 'NotImplementedError'` short-circuit in the golden workflow after `record_fs_events.py` is feature-complete.【F:.github/workflows/regenerate-goldens.yml†L197-L201】 | - Activate the Filesystem Watch Service rows in the test matrix and document passing evidence for latency/fuzz replays.【F:docs/testing/test-matrix.md†L40-L48】<br>- Attach regenerated watcher logs from `tests/golden/filesystem/watch-fuzz.log` and `watch-latency-burst.log` with checksum proof.【F:tests/golden/filesystem/watch-fuzz.log†L1-L18】【F:tests/golden/filesystem/watch-latency-burst.log†L1-L17】 |
| 2. **Archive & checksum toolchain** | Deliver the Rust archive builder, JSONL sanitizer, and checksum automation to feed Archive Extraction quota coverage and unlock checksum verification in CI. (Scripts: `archive_builder.rs`, `sanitize_jsonl.py`, `checksums.sh`).【F:docs/testing/fixtures-plan.md†L102-L111】 | - `Generate archive scenarios` and downstream sanitizer piping in the fixture workflow.【F:.github/workflows/regenerate-fixtures.yml†L121-L134】<br>- `Generate archive-based goldens` in the golden workflow.【F:.github/workflows/regenerate-goldens.yml†L149-L158】<br>- `Verify fixture checksums` / `Run checksum verification` with the enforced manifests.【F:.github/workflows/regenerate-fixtures.yml†L204-L212】【F:.github/workflows/regenerate-goldens.yml†L216-L219】 | - Archive builder emits deterministic TOML, tar.zst, and JSONL streams with stable ordering; the sanitizer enforces schema rules and redacts tenants to deterministic aliases; checksum automation now ships strict `--update` / `--verify` modes and is wired into both workflows.【F:scripts/archive_builder.rs†L1-L417】【F:scripts/sanitize_jsonl.py†L1-L163】【F:scripts/checksums.sh†L1-L124】 | - Record passing Archive Extraction quota and overflow checks from the matrix, including the sanitized manifest outputs.【F:docs/testing/test-matrix.md†L50-L58】<br>- Publish regenerated archives such as `tests/fixtures/archives/quota-latency.toml` and the sanitized `tests/golden/archives/quota-throughput.jsonl` with checksum evidence.【F:tests/fixtures/archives/quota-latency.toml†L1-L2】【F:tests/golden/archives/quota-throughput.jsonl†L1-L1】 |
| 3. **Routing and shared fixture orchestration** | Build the routing matrix and shared bundle utilities so multi-repository routing suites and cross-subsystem fixtures can be provisioned. (Scripts: `routing_matrix.py`, `fixture_packager.py`).【F:docs/testing/fixtures-plan.md†L135-L151】 | - `Generate routing fixtures` in the fixture workflow.【F:.github/workflows/regenerate-fixtures.yml†L181-L191】<br>- `Generate routing goldens` in the golden workflow.【F:.github/workflows/regenerate-goldens.yml†L160-L170】<br>- `Update shared fixture bundles` to publish the curated bundle set.【F:.github/workflows/regenerate-fixtures.yml†L193-L202】 | - Drop the `grep -q 'Placeholder' scripts/routing_matrix.py` checks in both workflows to allow all subcommands to run.【F:.github/workflows/regenerate-fixtures.yml†L185-L190】【F:.github/workflows/regenerate-goldens.yml†L164-L170】<br>- Remove the placeholder guard for `fixture_packager.py` so build/validate executes with checksum enforcement.【F:.github/workflows/regenerate-fixtures.yml†L193-L202】 | - Capture routing federation, fan-out, and latency suite results listed in the matrix as part of the release evidence.【F:docs/testing/test-matrix.md†L74-L95】<br>- Share regenerated transcripts such as `tests/golden/routing/mcp-federation.transcript` and bundle validation logs for reviewers.【F:tests/golden/routing/mcp-federation.transcript†L1-L6】 |
| 4. **Offline resilience harnesses** | Implement the offline transport buffer and manifest replay harnesses to satisfy Offline Resilience & Replay scenarios and golden regeneration. (Scripts: `offline_transport_buffer.py`, `manifest_replay_harness.rs`).【F:docs/testing/fixtures-plan.md†L126-L133】 | - `Generate offline transport fixtures` and `Run manifest replay harness` in the fixture workflow.【F:.github/workflows/regenerate-fixtures.yml†L159-L179】<br>- `Rebuild transport replays` and `Recreate manifest replay log` in the golden workflow.【F:.github/workflows/regenerate-goldens.yml†L173-L191】 | - Remove the placeholder guards for `offline_transport_buffer.py` and `manifest_replay_harness.rs` so capture/verify/replay commands run end-to-end.【F:.github/workflows/regenerate-fixtures.yml†L163-L178】【F:.github/workflows/regenerate-goldens.yml†L177-L191】 | - Provide passing offline transport and manifest replay evidence from the matrix, including replay catch-up latency metrics.【F:docs/testing/test-matrix.md†L83-L90】<br>- Attach regenerated transcripts like `tests/golden/transport/offline-buffer-replay.transcript` and `tests/golden/ingestion/manifest-replay.log` with checksum validation output.【F:tests/golden/transport/offline-buffer-replay.transcript†L1-L1】【F:tests/golden/ingestion/manifest-replay.log†L1-L1】 |
| 5. **Encryption & TLS capture suite** | Land the encryption toggle generator, trace capture harness, DPAPI collector, and WSL proxy so Encryption & TLS Controls coverage and Windows-required workflows can execute. (Scripts: `generate_encryption_toggles.py`, `trace_capture.sh`, `collect_dpapi.ps1`, `wsl_transport_proxy.ps1`, `wsl/python-client.ps1`).【F:docs/testing/fixtures-plan.md†L112-L157】 | - Windows `Generate DPAPI recovery fixtures` and TLS capture steps within the fixture workflow.【F:.github/workflows/regenerate-fixtures.yml†L25-L40】<br>- Linux `Generate encryption toggle fixtures` and `Capture TLS and DPAPI traces` follow-up steps.【F:.github/workflows/regenerate-fixtures.yml†L136-L158】<br>- Golden workflow `Generate DPAPI audit export`, `Generate TLS negotiation goldens`, `Capture WSL bridge trace`, and `Capture TLS and DPAPI goldens`.【F:.github/workflows/regenerate-goldens.yml†L25-L94】【F:.github/workflows/regenerate-goldens.yml†L204-L214】 | - Remove placeholder markers from `generate_encryption_toggles.py` so the fixtures job no longer skips the toggle emission.【F:.github/workflows/regenerate-fixtures.yml†L136-L144】<br>- Ensure `trace_capture.sh`, `collect_dpapi.ps1`, and `wsl_transport_proxy.ps1` emit checksums referenced by the workflows, documenting readiness for Windows runners.【F:.github/workflows/regenerate-fixtures.yml†L25-L40】【F:.github/workflows/regenerate-goldens.yml†L25-L94】 | - Supply Encryption & TLS Controls verification runs from the matrix, covering latency, negotiation, and DPAPI recovery checks.【F:docs/testing/test-matrix.md†L60-L71】<br>- Include regenerated traces such as `tests/golden/security/tls-performance.jsonl` and WSL bridge metadata with checksum manifests for reviewer confirmation.【F:tests/golden/security/tls-performance.jsonl†L1-L10】【F:tests/golden/security/wsl-bridge.json†L1-L6】 |
| 6. **CI bridge hardening** | Finalize remaining glue code once upstream tools exist so Actions jobs can flip from placeholder detection to full execution. (Scripts: `checksums.sh`, cross-language lint gates).【F:docs/testing/fixtures-plan.md†L124-L165】【F:scripts/README.md†L32-L43】 | - Enforce checksum verification for every regeneration run (fixture + golden jobs).【F:.github/workflows/regenerate-fixtures.yml†L204-L212】【F:.github/workflows/regenerate-goldens.yml†L216-L219】<br>- Promote cross-language lint gates and workflow assertions documented in the scripting overview.【F:scripts/README.md†L32-L43】 | - Convert checksum scripts from placeholder echoes to strict exit-on-diff behavior and remove all guard checks that bypass verification.【F:.github/workflows/regenerate-fixtures.yml†L204-L212】【F:.github/workflows/regenerate-goldens.yml†L216-L219】<br>- Wire remaining CI linting hooks so the scripting README automation table reflects active enforcement.【F:scripts/README.md†L32-L43】 | - Share checksum verification transcripts alongside regeneration artifacts per the automation runbook, demonstrating no drift across fixtures/goldens.【F:docs/testing/test-matrix.md†L103-L114】【F:docs/testing/fixtures-plan.md†L157-L165】<br>- Capture lint gate outputs that confirm cross-language formatting and static analysis now fail fast inside the workflows.【F:scripts/README.md†L32-L43】 |

The sequence prioritizes Linux-friendly Python tooling first, then Rust binaries,
and finally Windows/WSL-specific capture steps that depend on the earlier
artifacts.

## Latency & Throughput Command Matrix

The following matrix links each latency or throughput-oriented helper to the
expected invocation pattern and the placeholder artifacts it will eventually
populate. This keeps the archive and performance workflows in sync across the
fixture plan and regeneration jobs.

| Tool | Invocation (relative to repo root) | Output artifact(s) | Placeholder reference |
| --- | --- | --- | --- |
| `scripts/trace_capture.sh` | `scripts/trace_capture.sh --profile encryption-latency --output <path>` | JSON capture of encrypted vs. unencrypted latency samples | `tests/fixtures/security/encryption-latency.json` (generated by workflow)【F:scripts/trace_capture.sh†L13-L176】【F:.github/workflows/regenerate-fixtures.yml†L146-L156】 |
| `scripts/trace_capture.sh` | `scripts/trace_capture.sh --profile perf-window --output-dir <dir>` | `rolling-window.json` plus notes covering handshake counts (throughput) | `tests/fixtures/security/perf-window/` (workflow target directory)【F:scripts/trace_capture.sh†L15-L224】【F:.github/workflows/regenerate-fixtures.yml†L146-L155】 |
| `scripts/trace_capture.sh` | `scripts/trace_capture.sh --profile perf-baseline --output <path>` | JSONL stream of TLS stage durations (latency baseline) | `tests/golden/security/tls-performance.jsonl` (workflow target file)【F:scripts/trace_capture.sh†L16-L235】【F:.github/workflows/regenerate-fixtures.yml†L146-L156】 |
| `scripts/record_fs_events.py` | `python scripts/record_fs_events.py --scenario latency-burst --output <path>` or `python scripts/record_fs_events.py --config tests/fixtures/filesystem/mock-events.yaml --replay-dir <dir>` | YAML latency buckets, fuzz transcripts, and deterministic fixture stubs | `tests/golden/filesystem/watch-latency-burst.log` and replay assets under `tests/fixtures/filesystem/`【F:scripts/record_fs_events.py†L38-L118】【F:.github/workflows/regenerate-goldens.yml†L193-L200】 |
| `cargo run --manifest-path scripts/Cargo.toml --bin archive_builder` | `cargo run --manifest-path scripts/Cargo.toml --bin archive_builder -- --scenario quota-latency --output tests/fixtures/archives/quota-latency.toml` | TOML calibration for quota latency budgets | `tests/fixtures/archives/quota-latency.toml`【F:.github/workflows/regenerate-fixtures.yml†L121-L133】【F:tests/fixtures/archives/quota-latency.toml†L1-L2】 |
| `cargo run --manifest-path scripts/Cargo.toml --bin archive_builder` | `cargo run --manifest-path scripts/Cargo.toml --bin archive_builder -- --scenario overflow-latency --output tests/fixtures/archives/overflow-latency.tar.zst` | Compressed archive capturing overflow latency scenario | `tests/fixtures/archives/overflow-latency.tar.zst` (placeholder string notes pending real asset)【F:.github/workflows/regenerate-fixtures.yml†L121-L133】【06720e†L1-L1】 |
| `cargo run --manifest-path scripts/Cargo.toml --bin archive_builder` | `cargo run --manifest-path scripts/Cargo.toml --bin archive_builder -- --scenario quota-throughput \| python scripts/sanitize_jsonl.py > tests/golden/archives/quota-throughput.jsonl` | Sanitized JSONL throughput metrics for archive quotas | `tests/golden/archives/quota-throughput.jsonl`【F:.github/workflows/regenerate-goldens.yml†L149-L158】【F:tests/golden/archives/quota-throughput.jsonl†L1-L1】 |

Each entry aligns the stubbed generation commands with the placeholders tracked
under `tests/fixtures/archives/` or `tests/golden/archives/`, ensuring future
implementations replace the documented artifacts without path drift.

## Script Breakdown

Each subsection captures the ownership surface, acceptance criteria, and CI
handoff for the corresponding script.

### `record_fs_events.py`
- **Purpose & scope**: Capture filesystem event traces using watchdog observers
  and emit deterministic YAML fixtures referenced by Filesystem Watch Service
  tests.【F:scripts/README.md†L10-L13】【F:docs/testing/fixtures-plan.md†L93-L101】【F:docs/testing/test-matrix.md†L40-L48】
- **Dependencies**:
  - **Stub**: Python 3.11 standard library only (`argparse`, `pathlib`,
    `textwrap`, typing helpers) because the placeholder emits embedded
    transcripts without touching live watchers.【F:scripts/record_fs_events.py†L1-L59】
  - **Target implementation**: Python 3.11 with `watchdog` and `pyyaml` so the
    real harness can stream filesystem events across Linux, macOS, and Windows
    observers as described in the scripting overview.【F:scripts/README.md†L10-L21】
- **Current stub behavior**:
  - CLI offers two mutually exclusive paths. Golden regeneration uses
    `--scenario {fuzz, latency-burst}` with `--output <path>`, while fixture
    workflows use `--config <file>` plus `--replay-dir <dir>` to populate
    deterministic stub assets. Both routes rely on `_SCENARIO_LOGS` for their
    embedded content.【F:scripts/record_fs_events.py†L38-L118】
  - Deterministic log content mirrors the golden captures in
    `tests/golden/filesystem/` and documents the expected schema for future real
    recordings.【F:scripts/record_fs_events.py†L60-L118】【F:tests/golden/filesystem/watch-fuzz.log†L1-L18】【F:tests/golden/filesystem/watch-latency-burst.log†L1-L17】
  - Fixture mode writes a `mock-events.yaml` summary and per-scenario replay
    files to the provided directory so workflows referencing
    `tests/fixtures/filesystem/` can run without live captures.【F:scripts/record_fs_events.py†L80-L118】
- **Dependencies**: Python 3.11, `watchdog`, `pyyaml`; supports Linux, macOS,
  and Windows adapters via watchdog backends.【F:scripts/record_fs_events.py†L1-L22】
- **Current stub behavior**:
  - CLI exposes `--scenario/--output` for golden regeneration and
    `--config/--replay-dir` for fixture workflows, ensuring exactly one path is
    selected at runtime via mutually exclusive arguments.
  - Deterministic log content mirrors the golden captures in
    `tests/golden/filesystem/` and documents the expected schema for future
    real recordings.【F:scripts/record_fs_events.py†L60-L118】【F:tests/golden/filesystem/watch-fuzz.log†L1-L18】【F:tests/golden/filesystem/watch-latency-burst.log†L1-L17】
  - Fixture mode produces a `mock-events.yaml` manifest plus per-scenario YAML
    replays wherever `--replay-dir` points, keeping
    `tests/fixtures/filesystem/` synchronized with the stubbed expectations.【F:scripts/record_fs_events.py†L80-L118】
- **Acceptance criteria (target implementation)**:
  - Inputs: Accept configuration files that describe watch targets, latency
    metrics, and optional replay destinations so fixture captures can be
    regenerated without editing code.
  - Outputs: Deterministic YAML files under `tests/fixtures/filesystem/` and
    structured logs for CI; exit 0 when capture completes, non-zero on config or
    runtime errors.
  - Error handling: Surface validation errors for unsupported platforms,
    missing directories, or watcher failures with actionable messages.
- **Consuming tests/fixtures**: Drives fixture regeneration for
  `tests/fixtures/filesystem/workspace-replay/` and enables latency regression
  tests to replay captured events.【F:docs/testing/fixtures-plan.md†L97-L101】【F:docs/testing/test-matrix.md†L44-L47】
- **CI enablement**: Remove placeholder guard in `Regenerate Fixture Corpus`
  once implemented so the filesystem step runs on Ubuntu; surface lint results
  via `ruff`/`black` per directory standards.【F:.github/workflows/regenerate-fixtures.yml†L34-L57】【F:scripts/README.md†L32-L38】

### `verify_event_order.py`
- **Purpose & scope**: Validate ordering and integrity of captured filesystem
  events before promoting fixtures.【F:scripts/README.md†L10-L14】【F:docs/testing/fixtures-plan.md†L95-L101】
- **Dependencies**:
  - **Stub**: Python 3.11 standard library only; the placeholder does not import
    `pyyaml` yet because it raises `NotImplementedError` immediately.【F:scripts/verify_event_order.py†L1-L26】
  - **Target implementation**: Python 3.11 plus `pyyaml` to parse capture YAML on
    every platform covered by the fixture workflows.【F:scripts/README.md†L10-L16】
- **Current stub behavior**:
  - Module only raises `NotImplementedError` while documenting the intended
    `python scripts/verify_event_order.py <path>` CLI and importable API surface
    for future pytest integration.【F:scripts/verify_event_order.py†L1-L26】
  - No runtime dependencies beyond the standard library should be installed for
    the stub; workflows must skip verifier execution until the parser logic
    lands. Capture any temporary expectations in
    `agent-reports/watcher-tooling-report.md` so consumers stay aligned with the
    stub’s limitations.【F:agent-reports/watcher-tooling-report.md†L1-L15】
- **Dependencies**: Python 3.11, `pyyaml`; cross-platform.
- **Current stub behavior**: Module only raises `NotImplementedError` while
  documenting the intended `python scripts/verify_event_order.py <path>` CLI and
  importable API surface for future pytest integration.【F:scripts/verify_event_order.py†L1-L26】
- **Acceptance criteria (target implementation)**:
  - Inputs: Path to a replay directory or YAML file; optional flags for
    normalization and strictness.
  - Outputs: Exit 0 when ordering matches expectations, emit diff summaries on
    stderr/stdout when discrepancies occur.
  - Error handling: Distinguish malformed YAML, missing fields, and causal order
    violations with specific exit codes for CI triage.
- **Consuming tests/fixtures**: Provides validation for the Filesystem Watch
  Service integration and fuzz suites by ensuring captured corpora remain
  deterministic.【F:docs/testing/test-matrix.md†L44-L47】
- **CI enablement**: Turn on the verification step in `Regenerate Fixture
  Corpus`; integrate into pytest fixtures as part of failing coverage activation
  once outputs exist.【F:.github/workflows/regenerate-fixtures.yml†L46-L70】

### `archive_builder.rs`
- **Purpose & scope**: Generate archive fixtures and manifest streams for
  ingestion scenarios.【F:scripts/README.md†L15-L20】【F:docs/testing/fixtures-plan.md†L102-L110】
- **Dependencies**: Rust 1.76+, crates `clap`, `serde`, `tar`, `zstd` (finalize
  during implementation); Linux/macOS/Windows with Cargo.
- **Acceptance criteria**:
  - Inputs: CLI flags for scenario selection, output paths, compression options.
  - Outputs: Deterministic archives (`.tar.zst`) and TOML manifests with
    reproducible metadata; streaming mode for piping into sanitization.
  - Error handling: Meaningful errors for missing inputs, I/O failures, and
    checksum mismatches; exit codes align with CI gating.
- **Consuming tests/fixtures**: Feeds Archive Extraction quota tests and golden
  manifests consumed by ingestion pipelines.【F:docs/testing/test-matrix.md†L50-L58】
- **CI enablement**: Once stable, remove placeholder detection in both
  regeneration workflows and add `cargo fmt`/`clippy` checks per standards.【F:.github/workflows/regenerate-fixtures.yml†L70-L90】【F:.github/workflows/regenerate-goldens.yml†L61-L90】【F:scripts/README.md†L37-L39】

### `sanitize_jsonl.py`
- **Purpose & scope**: Scrub archive builder streams before promoting to golden
  artifacts.【F:scripts/README.md†L16-L17】【F:docs/testing/fixtures-plan.md†L104-L110】
- **Dependencies**: Python 3.11, `click`, `pyyaml`, `jsonschema` (final list
  confirmed during build).
- **Acceptance criteria**:
  - Inputs: JSONL via stdin or file path, CLI flags for schema enforcement and
    remediation output.
  - Outputs: Sanitized JSONL with deterministic ordering plus optional reject
    reports; exit non-zero when schema violations remain.
  - Error handling: Separate codes for schema validation failures vs. I/O
    issues; descriptive messages for CI logs.
- **Consuming tests/fixtures**: Ensures `tests/golden/archives/fuzzed-manifests.jsonl`
  satisfies Archive Extraction quota fuzz coverage.【F:docs/testing/fixtures-plan.md†L108-L110】【F:docs/testing/test-matrix.md†L55-L57】
- **CI enablement**: Enable sanitizer invocation in both workflows and wire into
  linting (`ruff`/`black`).【F:.github/workflows/regenerate-fixtures.yml†L70-L90】【F:.github/workflows/regenerate-goldens.yml†L61-L68】【F:scripts/README.md†L32-L38】

### `checksums.sh`
- **Purpose & scope**: Generate and verify SHA-256 manifests for large artifacts
  across fixtures and goldens.【F:scripts/README.md†L23-L24】【F:docs/testing/fixtures-plan.md†L78-L83】【F:docs/testing/fixtures-plan.md†L130-L133】
- **Dependencies**: POSIX shell, `sha256sum`, `find`, `sort` (coreutils); Linux
  primary, macOS via GNU coreutils.
- **Acceptance criteria**:
  - Inputs: Paths or `--verify` flag; optional `--update` to rewrite manifests.
  - Outputs: Deterministic manifest files and verification reports; exit 0 on
    success, non-zero on mismatches.
  - Error handling: Distinct messages for missing manifests vs. checksum drift.
- **Consuming tests/fixtures**: Required by fixture update checklist and CI
  verification across archives, transport, and ingestion assets.【F:docs/testing/fixtures-plan.md†L80-L83】【F:docs/testing/fixtures-plan.md†L132-L133】
- **CI enablement**: Once implemented, enable verification steps in both Actions
  workflows and integrate `shellcheck`/`shfmt` linting per standards.【F:.github/workflows/regenerate-fixtures.yml†L118-L133】【F:.github/workflows/regenerate-goldens.yml†L115-L133】【F:scripts/README.md†L37-L40】

### `routing_matrix.py`
- **Purpose & scope**: Build routing matrices, fan-out corpora, and federation
  transcripts for multi-repository routing scenarios.【F:scripts/README.md†L21-L22】【F:docs/testing/fixtures-plan.md†L135-L143】
- **Dependencies**: Python 3.11, `networkx`, `typer`; Linux/macOS/Windows.
- **Acceptance criteria**:
  - Inputs: Subcommands (`matrix`, `fanout`, `transcript`, `fuzz`) with target
    paths and optional seed parameters for deterministic graphs.
  - Outputs: Deterministic JSON/JSONL fixtures and transcripts; summary metrics
    for CI logs.
  - Error handling: Validation errors for inconsistent topology definitions or
    unsupported combinations emit actionable messages.
- **Consuming tests/fixtures**: Powers multi-repo routing unit/integration/fuzz
  suites and golden transcripts referenced in the test matrix.【F:docs/testing/test-matrix.md†L74-L80】
- **CI enablement**: Activate routing steps in both workflows and enforce
  `ruff`/`black` linting when script is live.【F:.github/workflows/regenerate-fixtures.yml†L104-L117】【F:.github/workflows/regenerate-goldens.yml†L71-L80】【F:scripts/README.md†L32-L38】

### `fixture_packager.py`
- **Purpose & scope**: Assemble and validate shared fixture bundles for reuse
  across subsystems.【F:scripts/README.md†L22-L23】【F:docs/testing/fixtures-plan.md†L145-L151】
- **Dependencies**: Python 3.11, `typer`, `pyyaml`, `rich`; cross-platform.
- **Acceptance criteria**:
  - Inputs: `build` and `validate` subcommands, version metadata, optional
    manifest overrides.
  - Outputs: Structured bundle directories, validation reports, and semantic
    version tagging to record compatibility.
  - Error handling: Non-zero exit when schema drift or version conflicts occur;
    descriptive CLI messaging.
- **Consuming tests/fixtures**: Supplies shared dataset updates consumed across
  subsystem test suites per the matrix’s Shared Testing Assets guidance.【F:docs/testing/test-matrix.md†L92-L95】
- **CI enablement**: Hook into fixture regeneration job to rebuild bundles and
  surface linting requirements (`ruff`/`black`).【F:.github/workflows/regenerate-fixtures.yml†L118-L133】【F:scripts/README.md†L32-L38】

### `offline_transport_buffer.py`
- **Purpose & scope**: Capture, verify, and replay offline transport queues for
  retry buffer scenarios.【F:scripts/README.md†L19-L20】【F:docs/testing/fixtures-plan.md†L126-L133】
- **Dependencies**: Python 3.11, `typer`, `rich`, `pyyaml`; Linux/macOS/Windows.
- **Acceptance criteria**:
  - Inputs: Subcommands `capture`, `verify`, `replay` with profile selection and
    thresholds.
  - Outputs: Deterministic YAML/JSONL fixtures and transcripts with telemetry
    summaries; exit codes separate capture, verification, and replay failures.
  - Error handling: Provide clear diagnostics for buffer overflows, schema
    mismatches, and filesystem access issues.
- **Consuming tests/fixtures**: Enables Offline Resilience integration and fuzz
  tests plus golden transcript regeneration for transport replay.【F:docs/testing/test-matrix.md†L83-L88】
- **CI enablement**: Allow Actions jobs to capture and replay offline assets;
  ensure Python lint tooling passes before enabling step.【F:.github/workflows/regenerate-fixtures.yml†L96-L117】【F:.github/workflows/regenerate-goldens.yml†L82-L90】【F:scripts/README.md†L32-L38】

### `manifest_replay_harness.rs`
- **Purpose & scope**: Reproduce ingestion manifest replays with configurable
  delays for offline recovery tests.【F:scripts/README.md†L20-L21】【F:docs/testing/fixtures-plan.md†L126-L133】
- **Dependencies**: Rust 1.76+, crates `tokio`, `serde`, `clap`, `tracing`; cross
  platform via Cargo.
- **Acceptance criteria**:
  - Inputs: CLI flags for input directories, delay profiles, fault injection, and
    output destinations.
  - Outputs: Deterministic replay logs and checkpoints; structured telemetry for
    tests.
  - Error handling: Distinct exit codes for I/O errors, schema mismatches, and
    timeout breaches; log warnings for recoverable issues.
- **Consuming tests/fixtures**: Required for Offline Resilience performance and
  ingestion recovery tests using delayed-ledger fixtures and manifest logs.【F:docs/testing/test-matrix.md†L86-L89】
- **CI enablement**: Trigger Cargo runs in both workflows and enforce `cargo
  fmt`/`clippy` gating once implemented.【F:.github/workflows/regenerate-fixtures.yml†L90-L101】【F:.github/workflows/regenerate-goldens.yml†L92-L100】【F:scripts/README.md†L37-L39】

### `generate_encryption_toggles.py`
- **Purpose & scope**: Produce encryption toggle datasets referenced by TLS and
  DPAPI workflows.【F:scripts/README.md†L24-L25】【F:docs/testing/fixtures-plan.md†L112-L123】
- **Dependencies**: Python 3.11, `click`, `cryptography`, `pyyaml`; Linux/macOS;
  runs on Windows for parity.
- **Acceptance criteria**:
  - Inputs: Output path, profile selectors, optional metadata (cert thumbprints,
    rotation cadence).
  - Outputs: Deterministic JSON/YAML toggle files plus metadata logs; exit 0 on
    success.
  - Error handling: Detect invalid toggle definitions, missing certificates, and
    crypto API failures with actionable guidance.
- **Consuming tests/fixtures**: Powers encryption-at-rest tests and TLS toggle
  scenarios referenced in the Encryption & TLS Controls matrix section.【F:docs/testing/test-matrix.md†L60-L69】
- **CI enablement**: Enable generation step in fixture workflow and enforce
  Python linting before toggles feed downstream captures.【F:.github/workflows/regenerate-fixtures.yml†L90-L109】【F:scripts/README.md†L32-L38】

### `trace_capture.sh`
- **Purpose & scope**: Automate TLS, Noise, and DPAPI trace captures across
  profiles, coordinating with Windows tooling as needed.【F:scripts/README.md†L17-L18】【F:docs/testing/fixtures-plan.md†L112-L123】
- **Dependencies**: POSIX shell, `openssl`, `tshark`, `jq`; Linux/WSL primary.
- **Acceptance criteria**:
  - Inputs: Profile flags (`--profile <name>`), output paths, optional proxy
    settings.
  - Outputs: Captured traces/logs with deterministic naming; structured logs for
    CI.
  - Error handling: Non-zero exit for capture failures, missing dependencies, or
    profile misconfiguration; fallback guidance for manual runs.
- **Consuming tests/fixtures**: Feeds TLS handshake, fuzz, perf, and DPAPI audit
  goldens plus transport handshake traces in the matrix.【F:docs/testing/test-matrix.md†L60-L71】
- **CI enablement**: Activate capture steps in both workflows once script is
  reliable; ensure `shellcheck`/`shfmt` compliance for linting.【F:.github/workflows/regenerate-fixtures.yml†L109-L117】【F:.github/workflows/regenerate-goldens.yml†L102-L113】【F:scripts/README.md†L37-L40】

### `collect_dpapi.ps1`
- **Purpose & scope**: Gather DPAPI recovery telemetry on domain-joined Windows
  hosts for encryption recovery coverage.【F:scripts/README.md†L18-L19】【F:docs/testing/fixtures-plan.md†L118-L123】
- **Dependencies**: PowerShell 7.3+, Windows 11 22H2+, EventLog APIs, DPAPI
  tooling.
- **Acceptance criteria**:
  - Inputs: Output directory, optional credential scope parameters, logging
    verbosity.
  - Outputs: Structured JSONL audit files and checksum manifests.
  - Error handling: Descriptive errors for missing privileges, certificate
    issues, and EventLog access failures.
- **Consuming tests/fixtures**: Enables encrypted storage DPAPI recovery tests
  and TLS regression coverage tied to DPAPI audits.【F:docs/testing/test-matrix.md†L70-L71】
- **CI enablement**: Integrate with Windows guidance job and eventually attach
  automated runs when Windows capture agents are available; enforce
  `PSScriptAnalyzer`/`Invoke-Formatter` compliance.【F:.github/workflows/regenerate-fixtures.yml†L118-L144】【F:.github/workflows/regenerate-goldens.yml†L102-L149】【F:scripts/README.md†L39-L43】

### `wsl_transport_proxy.ps1`
- **Purpose & scope**: Bridge Windows transport sessions into WSL for handshake
  replay during TLS and transport captures.【F:scripts/README.md†L25-L26】【F:docs/testing/fixtures-plan.md†L121-L157】
- **Dependencies**: PowerShell 7.3+, Windows 11, `wsl.exe`, networking/firewall
  cmdlets.
- **Acceptance criteria**:
  - Inputs: Proxy port, target distribution, optional authentication.
  - Outputs: Structured logs indicating tunnel status; exit 0 on success.
  - Error handling: Detailed messages for firewall restrictions, missing WSL
    distribution, or port binding issues.
- **Consuming tests/fixtures**: Required for WSL handshake regression coverage in
  Encryption & TLS Controls tests.【F:docs/testing/test-matrix.md†L70-L71】
- **CI enablement**: Document manual invocation in Windows workflow until fully
  automated; enforce PowerShell lint/formatting once implemented.【F:.github/workflows/regenerate-goldens.yml†L102-L149】【F:scripts/README.md†L39-L43】

### `wsl/python-client.ps1`
- **Purpose & scope**: Launch Python client workflows from Windows CI into a WSL
  distribution to mirror IDE bridge scenarios.【F:scripts/README.md†L26-L27】【F:docs/testing/fixtures-plan.md†L153-L157】
- **Dependencies**: Windows PowerShell 5.1+, `wsl.exe`, configured Python 3.11 in
  target distribution.
- **Acceptance criteria**:
  - Inputs: Distribution name, script/module to execute, environment bootstrap
    options.
  - Outputs: Proxy logs and exit status propagated back to Windows runner.
  - Error handling: Provide actionable errors for missing distribution, failed
    command execution, or environment provisioning.
- **Consuming tests/fixtures**: Supports Windows/WSL capture flows and upcoming
  client tooling tests that rely on shared fixtures.【F:docs/testing/test-matrix.md†L60-L71】【F:docs/testing/test-matrix.md†L92-L95】
- **CI enablement**: Surface invocation guidance in Windows workflow and enforce
  PowerShell linting once script is functional.【F:.github/workflows/regenerate-goldens.yml†L135-L149】【F:scripts/README.md†L39-L43】

## CI Roadmap & Lint Integration

- **Progressive enablement**: After each phase, remove the corresponding
  placeholder checks in the Actions workflows so regeneration steps execute on
  main branches. Document the activation in workflow history and subsystem
  fixture READMEs.【F:.github/workflows/regenerate-fixtures.yml†L38-L133】【F:.github/workflows/regenerate-goldens.yml†L32-L133】
- **Lint/format gating**: Ensure Python scripts run through `ruff` and `black`,
  Rust targets through `cargo fmt`/`clippy`, shell scripts via `shfmt` and
  `shellcheck`, and PowerShell scripts via `PSScriptAnalyzer` and
  `Invoke-Formatter` before toggling CI steps from dry-run to active.【F:scripts/README.md†L32-L43】
- **Checksum verification**: Enable `scripts/checksums.sh --verify` immediately
  after checksum helper lands to provide automated drift detection for both
  fixture and golden assets.【F:docs/testing/fixtures-plan.md†L160-L165】【F:.github/workflows/regenerate-fixtures.yml†L118-L133】【F:.github/workflows/regenerate-goldens.yml†L115-L133】
- **Evidence capture**: When CI steps become active, attach workflow run URLs and
  checksum outputs to fixture README updates, maintaining traceability per the
  fixture management plan.【F:docs/testing/test-matrix.md†L103-L114】【F:docs/testing/fixtures-plan.md†L132-L165】

This implementation plan establishes a cross-language roadmap that keeps
placeholder scripts aligned with the test-first mandate and CI automation.
