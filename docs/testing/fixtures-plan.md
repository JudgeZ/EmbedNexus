# Fixture and Golden Data Management Plan

## Scope and Goals

This plan governs the creation, maintenance, and review of test fixtures stored under `tests/fixtures/` and golden artifacts under `tests/golden/`. It standardizes directory layout, naming conventions, and update workflows so that teams can confidently extend the shared corpus while preserving deterministic test outcomes.

> **Placeholder notice**: The initial repository population introduces empty shell files and checksum notes that document the
> forthcoming generation commands. These placeholders satisfy the documented layout and allow the test matrix to reference stable
> paths while TDD work is staged. Replace each placeholder with the captured asset and corresponding checksum once the generation
> scripts have been executed.

## Directory Layout Overview

```
tests/
├── fixtures/
│   ├── archives/
│   │   ├── bulk-sample/
│   │   ├── overflow-case.tar.zst
│   │   └── quota-scenarios.toml
│   ├── filesystem/
│   │   ├── mock-events.yaml
│   │   └── workspace-replay/
│   ├── ingestion/
│   │   └── delayed-ledger/
│   ├── routing/
│   │   ├── high-fanout/
│   │   └── multi-repo-matrix.json
│   ├── security/
│   │   ├── dpapi-recovery/
│   │   ├── encryption-toggles.json
│   │   └── perf-window/
│   ├── transport/
│   │   └── offline-queue/
│   └── shared/
│       └── README.md
└── golden/
    ├── archives/
    │   └── fuzzed-manifests.jsonl
    ├── filesystem/
    │   └── watch-fuzz.log
    ├── mcp/
    ├── routing/
    │   ├── fuzz-affinity.jsonl
    │   └── mcp-federation.transcript
    ├── security/
    │   ├── dpapi-recovery-audit.jsonl
    │   ├── tls-fuzz.log
    │   └── tls-handshake.trace
    ├── transport/
    │   ├── offline-buffer-replay.transcript
    │   └── wsl-handshake-negotiation.trace
    └── ingestion/
        └── manifest-replay.log
```

The tree captures current references in the [test matrix](./test-matrix.md). Subdirectories remain dedicated to subsystem domains so that fixtures scale without cross-contamination.

## Naming Conventions

| Asset Type | Convention | Example |
| --- | --- | --- |
| Directory | `kebab-case` describing the scenario or corpus | `workspace-replay/`, `high-fanout/` |
| Static file fixture | `kebab-case.ext` using descriptive nouns | `mock-events.yaml`, `quota-scenarios.toml` |
| Golden transcript/log | `kebab-case.suffix` where suffix captures format (`.transcript`, `.log`, `.jsonl`) | `watch-fuzz.log`, `mcp-federation.transcript` |
| Generated bundle | `scenario-name.format` with compression suffixes appended (`.tar.zst`, `.zip`) | `overflow-case.tar.zst` |

Every fixture directory must contain a short `README.md` when multiple files coexist. The README documents:

1. Purpose of the fixture corpus.
2. Steps to regenerate the data.
3. Ownership contact for approvals.

## Update Workflows

### Adding or Modifying Fixtures

1. **Plan alignment** – Confirm the change is referenced in `docs/testing/test-matrix.md` or file a matrix update alongside the fixture addition.
2. **Local branch** – Create a dedicated branch named `fixtures/<subsystem>/<change>`. Avoid reusing branches between subsystems to preserve audit clarity.
3. **Author fixtures** – Generate data following the processes in [Fixture Generation](#fixture-generation) and record regeneration steps in the fixture-level `README.md`.
4. **Checksum capture** – Produce SHA-256 hashes for binary or large artifacts using `scripts/checksums.sh` (see [Review Process](#review-process)). Commit checksum manifests next to the assets (e.g., `overflow-case.tar.zst.sha256`).
5. **Golden refresh** – When updating golden outputs, run the relevant deterministic test harness (`cargo test --test <suite> -- --update-golden`) to ensure code-generated outputs remain synchronized. Capture the command in commit messages.
6. **Documentation** – Update this plan or subsystem-specific docs if workflows evolve.

### Checksum Manifest Coverage

Maintain deterministic checksum tracking for every regenerated artifact. The following manifests must be updated together with their paired assets:

- **Archive artifacts** – Refresh both golden and fixture manifests after rebuilding bundles:
  - `tests/golden/archives/fuzzed-manifests.jsonl.sha256`
  - `tests/golden/archives/quota-throughput.jsonl.sha256`
  - `tests/fixtures/archives/overflow-case.tar.zst.sha256`
  - `tests/fixtures/archives/overflow-latency.tar.zst.sha256`
- **Routing artifacts** – Regenerate manifests whenever transcripts or throughput captures change:
  - `tests/golden/routing/fanout-throughput.jsonl.sha256`
  - `tests/golden/routing/fuzz-affinity.jsonl.sha256`
  - `tests/golden/routing/mcp-federation.transcript.sha256`
  - `tests/golden/routing/multi-repo-latency.transcript.sha256`
- **Filesystem artifacts** – Add and maintain checksum companions for watcher outputs and configuration drops when they become deterministic:
  - `tests/golden/filesystem/watch-fuzz.log.sha256`
  - `tests/golden/filesystem/watch-latency-burst.log.sha256`
  - Future watcher configuration exports under `tests/fixtures/filesystem/` should follow the same `.sha256` pattern once populated.
- **Security artifacts** – Update both golden and fixture manifests after rerunning TLS, DPAPI, or WSL capture workflows:
  - `tests/golden/security/dpapi-recovery-audit.jsonl.sha256`
  - `tests/golden/security/encryption-toggle.trace.sha256`
  - `tests/golden/security/tls-fuzz.log.sha256`
  - `tests/golden/security/tls-handshake.trace.sha256`
  - `tests/golden/security/tls-negotiation.trace.sha256`
  - `tests/golden/security/tls-performance.jsonl.sha256`
  - `tests/golden/security/wsl-bridge.json.sha256`
  - `tests/fixtures/security/dpapi-recovery/recovery-events.jsonl.sha256`

**Synchronization workflow**:

1. Regenerate artifacts via the documented capture harnesses (see subsystem READMEs) so assets and manifests remain aligned.
2. Run `scripts/checksums.sh --update <paths>` once the helper is implemented; until then, invoke `sha256sum <artifact> > <artifact>.sha256` manually, keeping manifests co-located with their assets.
3. Verify the results with `scripts/checksums.sh --verify <paths>` (or `sha256sum --check`) before committing. Update fixture and golden trees in the same change set to avoid checksum drift between paired corpora.

### Deleting Fixtures

1. Verify no active tests reference the asset via `rg <fixture-name> tests/ src/`.
2. Remove associated checksum manifests and README entries.
3. Note the removal rationale in the pull request description, including downstream cleanup tasks.

### GitHub Actions regeneration

Two manual GitHub Actions workflows keep long-running fixture and golden captures reproducible: [`regenerate-fixtures.yml`](../../.github/workflows/regenerate-fixtures.yml) and [`regenerate-goldens.yml`](../../.github/workflows/regenerate-goldens.yml). Each workflow fans out into a Windows capture job that uploads raw security material followed by a Linux consolidation job that recreates cross-platform assets.

| Workflow | Job | Runner | Required tooling | Consumed artifacts | Published artifacts | Dependency flow |
| --- | --- | --- | --- | --- | --- | --- |
| Regenerate Fixture Corpus | `fixtures-windows` | `windows-latest` | PowerShell 7, repository scripts (`collect_dpapi.ps1`, `trace_capture.sh`) | – | `windows-security-fixtures` (DPAPI recovery trees, TLS configs, perf-window captures) | Entry point |
| Regenerate Fixture Corpus | `fixtures-linux` | `ubuntu-latest` | Python 3.11 (`watchdog`, `pyyaml`, `typer`, `rich`, `click`, `cryptography`, `networkx`), `openssl`, `tshark`, `jq`, `shellcheck`, `shfmt`, `zstd`, Rust toolchain | Downloads `windows-security-fixtures` via `actions/download-artifact` | `fixture-regeneration-output` (full `tests/fixtures/**`, refreshed goldens referenced by fixtures) | `needs: fixtures-windows` |
| Regenerate Golden Artifacts | `goldens-windows` | `windows-latest` | PowerShell 7, repository scripts (`collect_dpapi.ps1`, `trace_capture.sh`, `wsl_transport_proxy.ps1`) | – | `windows-security-goldens` (TLS traces, DPAPI audit, WSL bridge bundle) | Entry point |
| Regenerate Golden Artifacts | `goldens-linux` | `ubuntu-latest` | Python 3.11 (`watchdog`, `pyyaml`, `typer`, `rich`, `click`, `cryptography`, `networkx`), `openssl`, `tshark`, `jq`, `shellcheck`, `shfmt`, `zstd`, Rust toolchain | Downloads `windows-security-goldens` | `golden-regeneration-output` (complete `tests/golden/**` refresh) | `needs: goldens-windows` |

The Windows stage always runs first to collect platform-specific traces and upload the `windows-security-*` artifact. The Linux stage is blocked until the Windows artifact is present, then expands the corpus with the toolchains that are only available on Ubuntu runners.

#### Workflow toggles

Both workflows expose two manual inputs when dispatched from the Actions UI:

- `dry_run=true` – Executes the job pre-checks (checkout, dependency installation) and skips every generation, capture, and upload step guarded by `if: inputs.dry_run != 'true'`. Use this when verifying tooling availability or runner permissions before spending time on long captures.
- `skip_artifact_upload=true` – Runs generation tasks but omits the `actions/upload-artifact` steps. Enable this while iterating locally on scripts to avoid consuming artifact storage quotas. Always rerun without this flag before finalizing fixtures so reviewers can download evidence.

If the Windows job succeeds but the Linux job fails (for example, waiting on a manual fix), use the **Re-run failed jobs** button once the issue is corrected. This reuses the existing `windows-security-*` artifact so the dependent Linux job can resume without regenerating the Windows captures.

#### Promotion checklist

After the Linux job uploads its consolidated artifact:

1. Download the relevant artifact (`fixture-regeneration-output` or `golden-regeneration-output`) from the workflow run page.
2. Extract the bundle locally and run `bash scripts/checksums.sh --verify <extracted paths>` to confirm every `.sha256` manifest matches the regenerated payloads.
3. Record the workflow run URL and the artifact name in the updated fixture README(s) so future reviewers can trace provenance back to the GitHub Actions evidence.

## Fixture Generation

### Filesystem Events Corpus

- **Tooling**: `scripts/record_fs_events.py` (requires Python 3.11, `watchdog`, and `pyyaml`).
- **Workflow**:
  1. Start the recorder in a clean workspace snapshot: `python scripts/record_fs_events.py --config tests/fixtures/filesystem/mock-events.yaml`.
  2. Reproduce the target actions (create/edit/delete) to populate `tests/fixtures/filesystem/mock-events.yaml`.
  3. Export bulk sequences via `--replay-dir tests/fixtures/filesystem/workspace-replay/` to keep per-scenario YAML files deterministic.
  4. Validate ordering with `python scripts/verify_event_order.py tests/fixtures/filesystem/workspace-replay/` before committing.

### Archive Scenarios

- **Tooling**: `scripts/archive_builder.rs` (Rust binary) and `scripts/checksums.sh`.
- **Workflow**:
  1. Run `cargo run --bin archive_builder -- --scenario quota --output tests/fixtures/archives/quota-scenarios.toml`.
  2. Generate overflow bundles: `cargo run --bin archive_builder -- --scenario overflow --output tests/fixtures/archives/overflow-case.tar.zst`.
  3. Populate the bulk sample corpus using `cargo run --bin archive_builder -- --scenario bulk --output-dir tests/fixtures/archives/bulk-sample/`.
  4. Update fuzzed manifests by piping the builder output through the sanitizer: `cargo run --bin archive_builder -- --scenario fuzz | python scripts/sanitize_jsonl.py > tests/golden/archives/fuzzed-manifests.jsonl`.
  5. Record SHA-256 hashes with `scripts/checksums.sh tests/fixtures/archives/ tests/golden/archives/`.

### Security Traces (Encryption & TLS)

- **Tooling**: `scripts/trace_capture.sh` (depends on `openssl`, `tshark`, and `jq`).

#### Profile matrix

| Profile | Command stub | Artifact(s) | Destination | Notes |
| --- | --- | --- | --- | --- |
| `tls-handshake` | `TRACE_OUT=<file> --profile tls-handshake` | `tls-handshake.trace` | `tests/golden/security/` | Deterministic TLS 1.3 handshake transcript. |
| `tls-negotiation` | `TRACE_OUT=<file> --profile tls-negotiation` | `tls-negotiation.trace` | `tests/golden/security/` | Captures negotiated cipher/ALPN against proxy endpoint. |
| `tls-fuzz` | `TRACE_OUT=<file> --profile tls-fuzz` | `tls-fuzz.log` | `tests/golden/security/` | Logs downgrade rejection and GREASE validation. |
| `tls-config-matrix` | `TRACE_OUT=<file> --profile tls-config-matrix` | `tls-config-matrix.yaml` | `tests/fixtures/security/` | Enumerates supported TLS versions and ciphers. |
| `encryption-toggle` | `TRACE_OUT=<file> --profile encryption-toggle` | `encryption-toggle.trace` | `tests/golden/security/` | Mirrors toggle state machine emitted during capture. |
| `encryption-latency` | `--profile encryption-latency --output <file>` | `encryption-latency.json` | `tests/fixtures/security/` | Structured latency samples plus summary stats. |
| `perf-window` | `--profile perf-window --output-dir <dir>` | `rolling-window.json`, `notes.txt` | `tests/fixtures/security/perf-window/` | Directory payload with throughput observations. |
| `perf-baseline` | `TRACE_OUT=<file> --profile perf-baseline` | `tls-performance.jsonl` | `tests/golden/security/` | Baseline TLS stage durations. |
| `dpapi-audit` | `TRACE_OUT=<file> --profile dpapi-audit` | `dpapi-recovery-audit.jsonl` | `tests/golden/security/` | Copies audit export seeded by `collect_dpapi.ps1`. |
| `wsl-transport` | `TRACE_OUT=<file> --profile wsl-transport --proxy-host <host> --proxy-port <port>` | `wsl-handshake-negotiation.trace`, `wsl-bridge.json` | `tests/golden/transport/`, `tests/golden/security/` | Aligns with metadata from `wsl_transport_proxy.ps1`. |

#### Non-interactive prerequisites

- **Runtime dependencies**: Ensure `openssl`, `tshark`, and `jq` are installed for Bash captures. PowerShell helpers (`collect_dpapi.ps1`, `wsl_transport_proxy.ps1`) require PowerShell 7.3+, DPAPI tooling/EventLog access, and `wsl.exe` on Windows 11 22H2+ hosts.
- **Output targeting**: Provide `--output`, `--output-dir`, or `TRACE_OUT` so the script writes deterministically without prompts. Profiles emitting directories (`perf-window`) must point at an existing writable directory.
- **DPAPI capture chain**: Run `pwsh -File scripts/collect_dpapi.ps1 -OutputDir tests/fixtures/security/dpapi-recovery/ -EmitChecksums` before invoking `--profile dpapi-audit`. The host must be domain-joined with the recovery-agent certificate installed; keep the generated audit export local for the Bash script to promote into goldens.
- **WSL relay workflow**: Start `wsl_transport_proxy.ps1` with administrative privileges on Windows and disable VPN clients to avoid skewing timestamps. From WSL, invoke the `wsl-transport` profile with matching proxy host/port and set `TRACE_OUT=tests/golden/transport/wsl-handshake-negotiation.trace`. Preserve the paired `wsl-bridge.json` metadata in `tests/golden/security/`.
- **Integrity artifacts**: Pass `-EmitChecksums` when available so CI can validate emitted traces. Record host metadata (certificate thumbprints, Windows build, WSL distribution) inside fixture-level READMEs to meet matrix reproducibility requirements.

#### Capture workflow

1. Establish output paths (`TRACE_OUT`, `--output`, or `--output-dir`) per the profile matrix and verify prerequisite tooling is installed.
2. Collect TLS transcripts: run the `tls-handshake`, `tls-negotiation`, and `tls-fuzz` profiles to refresh golden traces, then execute `tls-config-matrix` for the supporting YAML fixture.
3. Generate encryption datasets: capture `encryption-toggle` and `encryption-latency` artifacts, and refresh synthesized toggles via `python scripts/generate_encryption_toggles.py --output tests/fixtures/security/encryption-toggles.json` (requires `click` and `cryptography`).
4. Produce performance corpora: run `--profile perf-window --output-dir tests/fixtures/security/perf-window/` and `TRACE_OUT=tests/golden/security/tls-performance.jsonl scripts/trace_capture.sh --profile perf-baseline`, updating README size thresholds as needed.
5. Execute the DPAPI workflow on a Windows host: capture recovery materials with `collect_dpapi.ps1`, emit checksums, then promote the audit export through `--profile dpapi-audit` so `tests/golden/security/dpapi-recovery-audit.jsonl` aligns with `tests/fixtures/security/dpapi-recovery/`.
6. Coordinate the WSL transport replay: keep the Windows relay active, then from WSL run `TRACE_OUT=tests/golden/transport/wsl-handshake-negotiation.trace scripts/trace_capture.sh --profile wsl-transport --proxy-host localhost --proxy-port 5173` to emit the handshake transcript and bridge metadata.
7. Update fixture-level READMEs with environment notes (certificate thumbprints, proxy ports, VPN status) and confirm DPAPI audits correlate with transport timestamps before committing refreshed assets.

### Offline Queue & Replay Harnesses

- **Tooling**: `scripts/offline_transport_buffer.py` (Python 3.11 + `typer`, `rich`) and `scripts/manifest_replay_harness.rs` (Rust integration harness).
- **Workflow**:
  1. Capture air-gapped transport sessions by running `python scripts/offline_transport_buffer.py capture --output-dir tests/fixtures/transport/offline-queue/ --profile air-gapped`. This produces deterministic queue snapshots (`air-gapped-session.yaml`, `burst-drain.jsonl`) that model retry buffer saturation without remote connectivity.
  2. Validate bounded growth with `python scripts/offline_transport_buffer.py verify tests/fixtures/transport/offline-queue/ --max-buffer 512` to ensure replay scenarios respect the transport backpressure thresholds documented in [Transport Adapter Specification](../design/transport.md#offline-backpressure).
  3. Generate golden replays by invoking `python scripts/offline_transport_buffer.py replay --input tests/fixtures/transport/offline-queue/burst-drain.jsonl --transcript tests/golden/transport/offline-buffer-replay.transcript`, capturing deterministic dequeue ordering for the failing integration test.
  4. Simulate delayed storage availability with `cargo run --bin manifest_replay_harness -- --input-dir tests/fixtures/ingestion/delayed-ledger/ --golden-out tests/golden/ingestion/manifest-replay.log --delay-ms 45000`. The harness should emit manifest diff batches (`retry-window-*.jsonl`) and ledger checkpoints replicating the recovery workflow.
  5. Record checksum manifests for large queue snapshots and replay logs using `scripts/checksums.sh tests/fixtures/transport/offline-queue/ tests/fixtures/ingestion/delayed-ledger/ tests/golden/transport/offline-buffer-replay.transcript tests/golden/ingestion/manifest-replay.log`.
  6. Update the fixture-level README files with air-gapped host prerequisites (e.g., firewall rules, telemetry capture sinks) and storage reconnection choreography so reviewers can reproduce the capture sessions.

### Routing Scenarios

- **Tooling**: `python scripts/routing_matrix.py` (depends on `networkx`) and `scripts/checksums.sh`.

| Subcommand | Required flags | Outputs | Dependencies / config |
| --- | --- | --- | --- |
| `matrix` | `--output tests/fixtures/routing/multi-repo-matrix.json`<br>`--latency-output tests/fixtures/routing/latency-matrix.json` (planned) | Routing adjacency matrix plus latency budget lookup for unit coverage.【F:tests/fixtures/routing/multi-repo-matrix.json†L1-L3】【F:tests/fixtures/routing/latency-matrix.json†L1-L4】 | Python 3.11 + `networkx`; execute before other subcommands so graph topology stays consistent.【F:scripts/README.md†L19-L24】 |
| `fanout` | `--output-dir tests/fixtures/routing/high-fanout/`<br>`--metrics-out tests/golden/routing/fanout-throughput.jsonl` (planned) | High fan-out corpora and throughput guard metrics consumed by performance tests.【F:tests/fixtures/routing/high-fanout/README.md†L1-L5】【F:tests/golden/routing/fanout-throughput.jsonl†L1-L1】 | Reuses the matrix topology; ensure output directory exists so scenario IDs map cleanly to throughput data.【F:docs/testing/test-blueprints.md†L140-L142】 |
| `transcript` | `--output tests/golden/routing/mcp-federation.transcript`<br>`--latency-output tests/golden/routing/multi-repo-latency.transcript` (planned) | Golden transcripts for MCP federation and cross-repo latency validation.【F:tests/golden/routing/mcp-federation.transcript†L1-L1】【F:docs/testing/test-matrix.md†L77-L80】 | Depends on regenerated matrices and fan-out corpora to align hop timing and tenant affinity.【F:docs/testing/test-blueprints.md†L139-L142】 |
| `fuzz` | `--output tests/golden/routing/fuzz-affinity.jsonl` | Repository affinity datasets backing fuzz scenarios.【F:tests/golden/routing/fuzz-affinity.jsonl†L1-L1】【F:docs/testing/test-blueprints.md†L139-L142】 | Provide deterministic seed handling to keep CI results reproducible across reruns.【F:docs/testing/scripts-implementation-plan.md†L214-L220】 |

- **Workflow**:
  1. Produce multi-repo matrices: `python scripts/routing_matrix.py matrix --output tests/fixtures/routing/multi-repo-matrix.json`.
  2. Emit high-fanout corpora: `python scripts/routing_matrix.py fanout --output-dir tests/fixtures/routing/high-fanout/`.
  3. Generate golden transcripts via the federation harness: `python scripts/routing_matrix.py transcript --output tests/golden/routing/mcp-federation.transcript`.
  4. Refresh fuzz-affinity hints: `python scripts/routing_matrix.py fuzz --output tests/golden/routing/fuzz-affinity.jsonl`.
  5. Record checksums for binary payloads or large JSONL files to detect drift.

### Shared Fixture Library

- **Tooling**: Maintained through `scripts/fixture_packager.py` (Python 3.11 + `typer`).
- **Workflow**:
  1. Update shared bundles with `python scripts/fixture_packager.py build --output tests/fixtures/shared/`.
  2. Validate schema compatibility by running `python scripts/fixture_packager.py validate tests/fixtures/shared/`.
  3. Document version increments in `tests/fixtures/shared/README.md`.

### Platform-Specific Setup Notes

- **Windows + WSL handshake replay**: Ensure the Windows host and the WSL distribution share the same user profile for DPAPI key material. The Windows relay service defined in `scripts/wsl_transport_proxy.ps1` must run with administrative privileges to forward loopback traffic to WSL. Disable third-party VPN clients during capture to avoid skewing latency metrics in `tests/golden/transport/wsl-handshake-negotiation.trace`.
- **DPAPI recovery agent rotation**: Prior to regenerating `tests/fixtures/security/dpapi-recovery/`, validate that the recovery agent certificate has not expired and that the Windows event subscription to the Security log is active. Record certificate thumbprints and rotation dates in the fixture README to keep parity with the failing coverage described in `docs/testing/test-matrix.md#encryption--tls-controls`.

## Review Process

1. **Checksum verification** – Run `scripts/checksums.sh --verify` across touched directories. CI must include this step to ensure committed hashes match actual contents.
2. **Structured diffs** – Use `git diff --stat` for quick scope validation and `git difftool --tool diffoscope <path>` when binary artifacts change. Attach diffoscope reports for reviewer context.
3. **Golden change approval** – Flag pull requests updating `tests/golden/` with the `needs-golden-review` label. Require approval from the owning subsystem lead.
4. **Traceability** – Reference the originating test plan (e.g., subsystem matrix entry) in commit messages and PR descriptions. Include regeneration commands and tooling versions.
5. **CI gating** – Extend pipelines with the checksum verification step plus smoke tests that replay updated fixtures (e.g., `cargo test --test filesystem_watch -- --replay-dir tests/fixtures/filesystem/workspace-replay/`).

## Change Audit Checklist

Before merging fixture updates, confirm:

- [ ] Checksums added/verified for all binary or large assets.
- [ ] Fixture README files updated with regeneration steps.
- [ ] Test matrix references aligned or updated.
- [ ] Review artifacts (diffoscope outputs, command logs) attached to the PR.
- [ ] Regeneration commands executed on supported tool versions.

Maintaining this checklist ensures fixture growth remains deliberate and auditable across teams.
