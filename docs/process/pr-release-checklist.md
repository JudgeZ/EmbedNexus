# PR Release Checklist

Use this checklist before opening or merging any pull request that will advance toward a release. Attach the filled-out version in the PR description so reviewers can verify every item.

## 1. Planning Approval
- [ ] Implementation plan documented and reviewed prior to coding.
- [ ] Stakeholder sign-off recorded (include meeting notes or approval comment links).
- [ ] Scope confirms release impact and identifies required release artifacts.

## 2. Test-Driven Development Evidence
- [ ] Tests were authored or updated before implementation to capture desired behavior.
- [ ] Test suite demonstrates failures prior to code changes and passes afterward (attach logs or CI run references).
- [ ] Regression or integration coverage verified for affected modules.

## 3. Formatting and Linting Verification
- [ ] `cargo fmt --all` executed with no diff.
- [ ] `cargo clippy --all-targets --all-features -- -D warnings` executed and reported zero warnings.
- [ ] Any non-Rust tooling (e.g., TypeScript, Python) was executed and logged where applicable.

## 4. Security Review
- [ ] Threat model consulted per `docs/security/threat-model.md` and relevant scenarios documented.
- [ ] Security checklist completed (input validation, encryption, sandboxing as applicable) with findings noted.
- [ ] New or modified dependencies assessed for vulnerabilities and licensing compatibility.

## 5. Release Tagging and Governance
- [ ] Tagging strategy decided (e.g., `vX.Y.Z`) and noted in PR description.
- [ ] Changelog entries drafted or updated to reflect the change.
- [ ] Release notes outline rollback strategy and validation steps.

## 6. Release Artifact Expectations
Produce and attach the following artifacts for each release build:
- [ ] Container image published to the agreed registry with signed digest.
- [ ] Platform binaries compiled (Linux, macOS, Windows as required) and archived.
- [ ] Cryptographic checksums (SHA256 or stronger) generated for every binary and image manifest.
- [ ] Verification instructions provided for consumers to validate checksums and signatures.

## 7. Final Review and Sign-Off
- [ ] Peer reviewer(s) confirm all checklist items with explicit approval.
- [ ] Release manager or delegate validates readiness for tagging.
- [ ] Links to evidence (CI runs, security review docs, artifact repositories) captured for traceability.
