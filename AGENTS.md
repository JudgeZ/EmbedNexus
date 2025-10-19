# Mandatory Contributor Workflows

All contributors **must** review and follow this plan before beginning any coding task. Whenever repository requirements evolve (e.g., encryption enhancements, expanded platform support, new ingestion rules), update this document immediately to keep the workflow authoritative.

## Plan-Before-Code & TDD Discipline
- Produce a written implementation plan and obtain agreement before modifying source files.
- Follow test-driven development (TDD): write or update failing tests that capture the desired behavior prior to implementing code changes.

## Rust Engineering Standards
- Conform to idiomatic Rust style; run `rustfmt` and `cargo clippy --all-targets --all-features` before submitting changes.
- Treat security as a first-class concern. Perform threat modeling and document mitigation notes in PR discussions for any change that could affect data handling, encryption, authentication, or platform integration.
- Consult `docs/security/threat-model.md` before and during implementation. Complete the applicable security review checklists (input validation, encryption, sandboxing) and link the checklist outcomes in the PR description.

## Documentation Requirements (4C Framework)
- Documentation must be **Clear, Concise, Complete, and Correct** (4C).
- Include Mermaid diagrams when visualizing flows, data pipelines, or architectural relationships.
- Reference or update client integration scripts and IDE compatibility work where relevant to the change.

## Pull Request Expectations
- Each PR must include:
  - A checklist covering plan review, TDD adherence, `rustfmt`, `clippy`, security assessment, documentation updates, and confirmation that relevant items from `docs/security/threat-model.md` were evaluated.
  - Evidence of peer review before merge; at least one reviewer must sign off on security considerations.
  - Release tagging notes when the change affects public interfaces or deployment artifacts.

## Documentation Conventions
- Keep README and supplementary docs synchronized with implemented features.
- Link relevant Mermaid diagrams and outline integration touchpoints for clients (CLI tools, SDKs, IDE plugins).
- When adding features, document configuration, encryption policies, and ingestion rules.

## Continuous Updates
- Revisit this workflow after every feature planning cycle.
- Record updates for evolving requirements (e.g., new encryption algorithms, broader platform coverage, updated ingestion processes) to maintain alignment across contributors.
