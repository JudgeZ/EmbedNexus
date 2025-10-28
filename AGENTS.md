# Mandatory Contributor Workflows

All contributors **must** review and follow this plan before beginning any coding task. Whenever repository requirements evolve (e.g., encryption enhancements, expanded platform support, new ingestion rules), update this document immediately to keep the workflow authoritative.

## Plan-Before-Code & TDD Discipline
- Produce a written implementation plan and obtain agreement before modifying source files. Cross-reference the latest design specs in `docs/design/` and update them when planning introduces new workflows, encryption models, or client touchpoints.
- Review and follow the documentation workflow in `docs/process/doc-review.md` so that design specs, security narratives, and integration guides are approved before implementation work begins.
- Follow test-driven development (TDD): write or update failing tests that capture the desired behavior prior to implementing code changes, referencing the required coverage in `docs/testing/test-matrix.md`.
- Preserve evidence of TDD in PRs (e.g., failing test logs, CI run links) so reviewers can confirm red-green cycles.

## Rust Engineering Standards
- Conform to idiomatic Rust style; run `rustfmt` and `cargo clippy --all-targets --all-features` before submitting changes.
- Treat security as a first-class concern. Perform threat modeling and document mitigation notes in PR discussions for any change that could affect data handling, encryption, authentication, IDE/CLI integrations, or platform interfaces.
- Consult `docs/security/threat-model.md` before and during implementation. Complete the applicable security review checklists (input validation, encryption, sandboxing) and link the checklist outcomes in the PR description.
- When a change introduces new encryption paths or expands platform coverage, update the design docs and call out the impact in the PR summary.

## Documentation Requirements (4C Framework)
- Documentation must be **Clear, Concise, Complete, and Correct** (4C).
- Maintain the full C4 architecture stack (Context, Containers, Components, Code) as living documentation alongside Mermaid diagrams. Each level must be versioned under `docs/design/` (see `docs/design/overview.md` for entry points and the `docs/design/c4/` directory for level-specific assets) and updated whenever implementations shift responsibilities or interfaces.
- Include Mermaid diagrams when visualizing flows, data pipelines, or architectural relationships, and keep them synchronized with the latest design updates and the authoritative C4 narratives.
- Reference or update client integration scripts and IDE compatibility work where relevant to the change, including delivery expectations for CLI packages, SDKs, and IDE extensions.

## Pull Request Expectations
- Each PR must include:
  - A completed [PR release checklist](docs/process/pr-release-checklist.md) covering plan review, TDD adherence, `rustfmt`, `clippy`, security assessment, documentation updates, and confirmation that relevant items from `docs/security/threat-model.md` were evaluated.
  - Evidence of peer review before merge; at least one reviewer must sign off on security considerations.
  - Release tagging notes when the change affects public interfaces or deployment artifacts.

## Documentation Conventions
- Keep README and supplementary docs synchronized with implemented features.
- Link relevant Mermaid diagrams and outline integration touchpoints for clients (CLI tools, SDKs, IDE plugins).
- When adding features, document configuration, encryption policies, ingestion rules, and any new test-matrix coverage required before implementation.

## Continuous Updates
- Revisit this workflow after every feature planning cycle and during the monthly governance review cadence.
- Record updates for evolving requirements (e.g., new encryption algorithms, broader platform coverage, updated ingestion processes, expanded test matrices) to maintain alignment across contributors.
- Capture a governance-log entry (see `docs/process/governance-log.md`) whenever this guidance changes so the whole team acknowledges the new expectations before coding begins.
- When documentation reviews identify process changes, ensure the outcomes are reflected in `docs/process/doc-review.md` and linked from related design, security, or integration artifacts.

## Implementation Log
- **2025-10-28 — Phase 1 (transport→router→ledger spine)**: Added unit coverage for `OfflineReplayBuffer` capacity/expiry semantics and HTTP adapter session/token paths, recording auth-failure telemetry before returning errors. All tests pass (`cargo test`), establishing the first plan slice of the 12-step roadmap.
- **2025-10-29 — Phase 2 (ledger replay + manifest backpressure)**: Landed storage-ledger replay ordering tests (partial flush requeue and concurrent pushes) plus ingestion-manifest backpressure/resume coverage. Updated buffers to track max sequence for requeued entries; suites green (`cargo test -p storage-ledger --lib`, `cargo test -p ingestion-manifest`).
- **2025-10-29 — Phase 3 (routing matrix + STDIO parity)**: Added routing matrix loader with shortest-path helpers validated against `multi-repo-matrix.json`/`multi-repo-latency.transcript`, and extended STDIO adapter tests for session telemetry plus expired token rejection. Targeted suites: `cargo test -p runtime-router`, `cargo test -p runtime-transport-stdio`.
