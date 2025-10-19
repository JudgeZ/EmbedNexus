# Scripts Implementation Plan

This plan documents how each placeholder script under `scripts/` will mature from stub to
production tooling. It summarizes responsibilities, third-party dependencies,
platform assumptions, acceptance criteria, and the failing tests or fixtures that
will consume each helper. The roadmap aligns implementation sequencing with the
`Regenerate Fixture Corpus` and `Regenerate Golden Artifacts` workflows so CI
automation can be enabled as individual tools land.【F:scripts/README.md†L3-L53】【F:docs/testing/test-matrix.md†L103-L114】【F:.github/workflows/regenerate-fixtures.yml†L1-L144】【F:.github/workflows/regenerate-goldens.yml†L1-L149】

## Phase Ordering

1. **Filesystem capture foundation** – Implement Python helpers that unblock
   deterministic filesystem fixtures so the Filesystem Watch Service tests can
   move from pending to active. (Scripts: `record_fs_events.py`,
   `verify_event_order.py`).【F:docs/testing/fixtures-plan.md†L93-L101】【F:docs/testing/test-matrix.md†L40-L48】
2. **Archive & checksum toolchain** – Deliver the Rust archive builder, JSONL
   sanitizer, and checksum automation to feed Archive Extraction quota coverage
   and unlock checksum verification in CI. (Scripts: `archive_builder.rs`,
   `sanitize_jsonl.py`, `checksums.sh`).【F:docs/testing/fixtures-plan.md†L102-L111】【F:docs/testing/test-matrix.md†L50-L58】
3. **Routing and shared fixture orchestration** – Build the routing matrix and
   shared bundle utilities so multi-repository routing suites and cross-subsystem
   fixtures can be provisioned. (Scripts: `routing_matrix.py`,
   `fixture_packager.py`).【F:docs/testing/fixtures-plan.md†L135-L151】【F:docs/testing/test-matrix.md†L74-L95】
4. **Offline resilience harnesses** – Implement the offline transport buffer and
   manifest replay harnesses to satisfy Offline Resilience & Replay scenarios and
   golden regeneration. (Scripts: `offline_transport_buffer.py`,
   `manifest_replay_harness.rs`).【F:docs/testing/fixtures-plan.md†L126-L133】【F:docs/testing/test-matrix.md†L83-L90】
5. **Encryption & TLS capture suite** – Land the encryption toggle generator,
   trace capture harness, DPAPI collector, and WSL proxy so Encryption & TLS
   Controls coverage and Windows-required workflows can execute. (Scripts:
   `generate_encryption_toggles.py`, `trace_capture.sh`, `collect_dpapi.ps1`,
   `wsl_transport_proxy.ps1`, `wsl/python-client.ps1`).【F:docs/testing/fixtures-plan.md†L112-L157】【F:docs/testing/test-matrix.md†L60-L71】
6. **CI bridge hardening** – Finalize remaining glue code once upstream tools
   exist so Actions jobs can flip from placeholder detection to full execution.
   (Scripts: `checksums.sh`, cross-language lint gates).【F:docs/testing/fixtures-plan.md†L124-L165】【F:scripts/README.md†L32-L43】

The sequence prioritizes Linux-friendly Python tooling first, then Rust binaries,
and finally Windows/WSL-specific capture steps that depend on the earlier
artifacts.

## Script Breakdown

Each subsection captures the ownership surface, acceptance criteria, and CI
handoff for the corresponding script.

### `record_fs_events.py`
- **Purpose & scope**: Capture filesystem event traces using watchdog observers
  and emit deterministic YAML fixtures referenced by Filesystem Watch Service
  tests.【F:scripts/README.md†L10-L13】【F:docs/testing/fixtures-plan.md†L93-L101】【F:docs/testing/test-matrix.md†L40-L48】
- **Dependencies**: Python 3.11, `watchdog`, `pyyaml`; supports Linux, macOS,
  and Windows adapters via watchdog backends.【F:scripts/record_fs_events.py†L1-L22】
- **Acceptance criteria**:
  - Inputs: `--config <yaml>` describing watch targets, optional
    `--replay-dir` for per-scenario exports.
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
- **Dependencies**: Python 3.11, `pyyaml`; cross-platform.
- **Acceptance criteria**:
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
