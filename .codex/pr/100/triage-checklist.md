# Triage Checklist — Milestone 1 (CI Hardening & Security Baseline)

## Scope
- [ ] Add matrix CI for Rust across OS/toolchains without changing existing ci.yml.
- [ ] Add security workflows: cargo-audit, cargo-deny, gitleaks, and CycloneDX SBOM.
- [ ] Add CodeQL analysis for Rust.
- [ ] Add Dependabot for GitHub Actions, Cargo, and Go modules.
- [ ] Add .editorconfig for cross-language formatting consistency.
- [ ] Add .pre-commit-config.yaml with safe defaults and Rust fmt/clippy hooks.

## Acceptance Criteria
- [ ] New workflows run on pull requests and via manual dispatch; ci.yml remains intact.
- [ ] Security jobs succeed or surface actionable findings; artifacts include SBOM.
- [ ] Dependabot PRs enabled for actions, Cargo, and clients/go.
- [ ] Pre-commit and EditorConfig present and documented in CONTRIBUTING.md on follow-up.
