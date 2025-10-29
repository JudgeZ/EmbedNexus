# Phased Delivery Plan — Zævrynth

This plan aligns implementation to the repository’s design, security, and testing docs.
It follows the mandatory contributor workflow in AGENTS.md and documents milestones,
acceptance criteria, and key risks. Update this plan as milestones land.

## Guiding References
- Architecture and C4: docs/design/overview.md, docs/design/c4/
- Testing: docs/testing/test-matrix.md, docs/testing/ci-coverage.md
- Security: docs/security/threat-model.md (checklists), clippy.toml
- Process: docs/process/doc-review.md, docs/process/pr-release-checklist.md
- Packaging: docs/implementation/runtime-packaging-plan.md

---

## Milestone 1 — CI Hardening & Security Baseline (this PR)

Scope
- Add matrix CI for Rust across OS/toolchains without changing existing ci.yml.
- Add security workflows: cargo-audit, cargo-deny, gitleaks, and CycloneDX SBOM.
- Add CodeQL analysis for Rust.
- Add Dependabot for GitHub Actions, Cargo, and Go modules.
- Add .editorconfig for cross-language formatting consistency.
- Add .pre-commit-config.yaml with safe defaults and Rust fmt/clippy hooks.

Acceptance Criteria
- New workflows run on pull requests and via manual dispatch; ci.yml remains intact.
- Security jobs succeed or surface actionable findings; artifacts include SBOM.
- Dependabot PRs enabled for actions, Cargo, and clients/go.
- Pre-commit and EditorConfig present and documented in CONTRIBUTING.md on follow-up.

Risks
- First cargo-deny/audit runs may fail on advisory/license issues (resolve or ignore with rationale).
- gitleaks false-positives; use allowlist only with justification.

---

## Milestone 2 — Transport Spine & Offline Resilience

Scope
- Complete HTTP/STDIO/UDS adapter behaviors (loopback bind, CSRF, token issue/expiry, capability propagation).
- Implement retry buffers and offline replay per tests/runtime_transport and docs.
- Regenerate fixtures/goldens via existing workflows.

Acceptance Criteria
- All adapter unit/integration tests green across OS matrix; telemetry and auth tests passing.
- Updated design/security docs referencing checklists and telemetry.

Risks
- Cross-platform timing flakes (Windows/WSL) and race conditions in retry queues.

---

## Milestone 3 — Encrypted Storage & Ledger Replay

Scope
- Implement encrypted storage-vector and storage-ledger replay ordering.
- Key management integration (document DPAPI/WSL parity) and rotation checks.

Acceptance Criteria
- Storage/ledger tests pass; replay and rotation verified; encryption checklist satisfied.

Risks
- Performance regressions on large replays; key handling edge cases.

---

## Milestone 4 — Ingestion Pipeline & Routing Federation

Scope
- Ingestion planning/sanitization/embedding orchestration; routing matrix and fan-out.

Acceptance Criteria
- Subsystem tests in docs/testing/test-matrix.md pass; archive quotas and routing latency thresholds met.

Risks
- Fixture scale and sanitizer tuning; latency vs. throughput trade-offs.

---

## Milestone 5 — Packaging & Client Matrix

Scope
- Implement package-runtime.yml; publish signed artifacts with checksums/SBOM.
- Enable client matrix (Python/Node/Go) replaying goldens across transports/OS.

Acceptance Criteria
- Packaging artifacts published and verified; client matrix green; docs/integration updated.

Risks
- Code signing on macOS/Windows runners; transcript drift requiring coordinated updates.

