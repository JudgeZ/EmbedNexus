# Contributing Guide

Thank you for helping improve Cursor Local Embedding MCP. Before writing any code, review the mandatory workflows described in [AGENTS.md](AGENTS.md) and make sure your plan is approved by the team.

## Pull Request Expectations
1. Draft and agree on an implementation plan prior to coding.
2. Use test-driven development—author or update failing tests before implementing changes.
3. Run `cargo fmt --all` and `cargo clippy --all-targets --all-features -- -D warnings` to ensure style and lint compliance.
4. Complete the [PR release checklist](docs/process/pr-release-checklist.md) and attach it to your pull request description.
5. Perform a security review referencing `docs/security/threat-model.md`, documenting findings in the PR.
6. Provide links to release artifacts (container images, binaries, checksums) when preparing tagged releases.

For substantial or security-sensitive changes, involve the release manager early to schedule reviews and coordinate artifact publication.
