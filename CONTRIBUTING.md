# Contributing Guide

Thank you for helping improve Cursor Local Embedding MCP. Before writing any code, review the mandatory workflows described in [AGENTS.md](AGENTS.md) and make sure your plan is approved by the team. Incorporate the latest design guidance in `docs/design/`, the active test requirements in `docs/testing/test-matrix.md`, and any security considerations highlighted in `docs/security/threat-model.md` while preparing your plan.

## Pull Request Expectations
1. Draft and agree on an implementation plan prior to coding, calling out any required design doc updates (Mermaid diagrams, encryption flows, client/IDE integration notes).
2. Use test-driven development—author or update failing tests before implementing changes, and link evidence (e.g., CI runs showing the red/green progression) in the PR description.
3. Run `cargo fmt --all` and `cargo clippy --all-targets --all-features -- -D warnings` to ensure style and lint compliance. Execute and document any additional language/toolchain linters touched by the change.
4. Complete the [PR release checklist](docs/process/pr-release-checklist.md) and attach it to your pull request description, including references to updated design docs and test-matrix entries.
5. Perform a security review referencing `docs/security/threat-model.md`, documenting findings in the PR and explaining any encryption or sandboxing deviations.
6. Update the relevant [C4 architecture assets](docs/design/c4/) alongside other design artefacts (Mermaid diagrams, test matrices, security narratives). Pull requests will be reviewed for current C4 narratives and diagrams that accurately reflect the proposed change.
7. Provide links to release artifacts (container images, binaries, checksums) when preparing tagged releases, along with updated delivery instructions for CLI packages, SDKs, or IDE extensions impacted by the release.

For substantial or security-sensitive changes, involve the release manager early to schedule reviews and coordinate artifact publication.

## Pre-commit & Security Checks

- Pre-commit hooks
  - Install and run once locally: `pipx install pre-commit && pre-commit install && pre-commit run -a`.
  - Hooks include whitespace/YAML hygiene, ShellCheck, and Rust `rustfmt`. Clippy is available via the `manual` stage: `pre-commit run clippy -a`.
- CI security scans
  - CI runs `cargo-audit`, `cargo-deny` (advisories, bans, sources, licenses), `gitleaks` (with allowlist in `.gitleaks.toml` scoped to fixtures/goldens), and publishes a CycloneDX SBOM.
  - To run locally: `cargo audit`, `cargo deny check advisories bans sources licenses`, and `gitleaks detect --source . --config-path .gitleaks.toml --redact`.
- Dependency updates
  - Dependabot is enabled for GitHub Actions, Cargo (workspace), and Go modules in `clients/go`.
- Notes
  - Keep any new fake/test secrets clearly marked (e.g., `FAKE_...`, `NOT_A_SECRET`) so they remain covered by the gitleaks allowlist.
  - If a scan requires an exception, document rationale in the PR and scope the allowlist narrowly.
