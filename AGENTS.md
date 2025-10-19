# Mandatory Contributor Workflows

All contributors **must** review and follow this plan before beginning any coding task. Whenever repository requirements evolve (e.g., encryption enhancements, expanded platform support, new ingestion rules), update this document immediately to keep the workflow authoritative.

## Plan-Before-Code & TDD Discipline
- Produce a written implementation plan and obtain agreement before modifying source files. Cross-reference the latest design specs in `docs/design/` and update them when planning introduces new workflows, encryption models, or client touchpoints.
- Follow test-driven development (TDD): write or update failing tests that capture the desired behavior prior to implementing code changes, referencing the required coverage in `docs/testing/test-matrix.md`.
- Preserve evidence of TDD in PRs (e.g., failing test logs, CI run links) so reviewers can confirm red-green cycles.

## Rust Engineering Standards
- Conform to idiomatic Rust style; run `rustfmt` and `cargo clippy --all-targets --all-features` before submitting changes.
- Treat security as a first-class concern. Perform threat modeling and document mitigation notes in PR discussions for any change that could affect data handling, encryption, authentication, IDE/CLI integrations, or platform interfaces.
- Consult `docs/security/threat-model.md` before and during implementation. Complete the applicable security review checklists (input validation, encryption, sandboxing) and link the checklist outcomes in the PR description.
- When a change introduces new encryption paths or expands platform coverage, update the design docs and call out the impact in the PR summary.

## Documentation Requirements (4C Framework)
- Documentation must be **Clear, Concise, Complete, and Correct** (4C).
- Include Mermaid diagrams when visualizing flows, data pipelines, or architectural relationships, and keep them synchronized with the latest design updates.
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
