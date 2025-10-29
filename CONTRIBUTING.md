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
- EditorConfig
  - The repo includes `.editorconfig` to keep indentation, line endings, and trailing whitespace consistent across editors.
  - Most IDEs auto-detect it; if not, install the EditorConfig plugin for your editor. CI enforces consistent formatting via `cargo fmt`.
- CI security scans
  - CI runs `cargo-audit`, `cargo-deny` (advisories, bans, sources, licenses), `gitleaks` (with allowlist in `.gitleaks.toml` scoped to fixtures/goldens), and publishes a CycloneDX SBOM.
  - To run locally: `cargo audit`, `cargo deny check advisories bans sources licenses`, and `gitleaks detect --source . --config-path .gitleaks.toml --redact`.
- Dependency updates
  - Dependabot is enabled for GitHub Actions, Cargo (workspace), and Go modules in `clients/go`.

## Pre-PR Verification

Run the local verification suite before opening a PR to surface policy and secret-scan issues early and to generate an SBOM for review:

- Manual run (recommended): `pre-commit run pre-pr-verify -a`
- Direct script: `bash scripts/pre_pr_check.sh`

What it does:
- Confirms Rust toolchain context and versions (toolchain, rustfmt, clippy).
- Runs `cargo deny` checks (advisories, bans, sources, licenses).
- Runs `gitleaks` with redaction against `.gitleaks.toml`.
- Generates a CycloneDX SBOM at `artifacts/local/sbom/zaevrynth.cdx.json` (untracked).

Enable hooks:
- `pre-commit install` (commit-time hooks)
- `pre-commit install --hook-type pre-push` (to run on `git push`; the `pre-pr-verify` hook is configured for the `push` stage and can also be run manually)
- Notes
  - Keep any new fake/test secrets clearly marked (e.g., `FAKE_...`, `NOT_A_SECRET`) so they remain covered by the gitleaks allowlist.
  - If a scan requires an exception, document rationale in the PR and scope the allowlist narrowly.
