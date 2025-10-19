# Fixture and Golden Data Management Plan

## Scope and Goals

This plan governs the creation, maintenance, and review of test fixtures stored under `tests/fixtures/` and golden artifacts under `tests/golden/`. It standardizes directory layout, naming conventions, and update workflows so that teams can confidently extend the shared corpus while preserving deterministic test outcomes.

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
│   ├── routing/
│   │   ├── high-fanout/
│   │   └── multi-repo-matrix.json
│   ├── security/
│   │   ├── encryption-toggles.json
│   │   └── perf-window/
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
    └── security/
        ├── tls-fuzz.log
        └── tls-handshake.trace
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

### Deleting Fixtures

1. Verify no active tests reference the asset via `rg <fixture-name> tests/ src/`.
2. Remove associated checksum manifests and README entries.
3. Note the removal rationale in the pull request description, including downstream cleanup tasks.

## Fixture Generation

### Filesystem Events Corpus

- **Tooling**: `scripts/record_fs_events.py` (requires Python 3.11, `watchdog`, and `pyyaml`).
- **Workflow**:
  1. Start the recorder in a clean workspace snapshot: `python scripts/record_fs_events.py --config configs/filesystem/mock-events.yaml`.
  2. Reproduce the target actions (create/edit/delete) to populate `mock-events.yaml`.
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
- **Workflow**:
  1. Start the capture harness: `TRACE_OUT=tests/golden/security/tls-handshake.trace scripts/trace_capture.sh --profile tls-handshake`.
  2. Replay negative scenarios with `--profile tls-fuzz` to build `tests/golden/security/tls-fuzz.log`.
  3. Generate encryption toggle fixtures by running the configuration synthesizer: `python scripts/generate_encryption_toggles.py --output tests/fixtures/security/encryption-toggles.json` (requires `python -m pip install click cryptography`).
  4. Materialize performance datasets through `scripts/trace_capture.sh --profile perf-window --output-dir tests/fixtures/security/perf-window/` and confirm dataset size thresholds in the README.

### Routing Scenarios

- **Tooling**: `python scripts/routing_matrix.py` (depends on `networkx`) and `scripts/checksums.sh`.
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
