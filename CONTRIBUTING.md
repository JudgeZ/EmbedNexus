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
