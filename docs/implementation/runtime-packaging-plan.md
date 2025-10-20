# Runtime Packaging & Distribution Plan

This plan defines the sequencing required to ship a cross-platform runtime bundle that
includes transport adapters, runtime services, and client-script validation policies.
It extends the architecture overview and phase 1 runtime milestone by mapping them to
specific packaging deliverables and CI automation. All artifacts, including validation
fixtures and golden transcripts, are regenerated exclusively by GitHub Actions workflows
as mandated by the repository contribution standards.

## Packaging Objectives

1. **Unified bundle composition** – Ship a deterministic package layout that contains
   the runtime binaries, transport adapter assets, configuration templates, and policy
   manifests required for first-run initialization on Linux, macOS, and Windows/WSL.
2. **Security-first distribution** – Embed policy validation data, checksum manifests,
   and signed metadata that align with the security checklists in
   [`docs/security/threat-model.md`](../security/threat-model.md).
3. **Automation alignment** – Extend the packaging GitHub Actions workflow so fixtures,
   goldens, and package archives are regenerated together and uploaded as reviewable
   artifacts.

## Target Deliverables

| Platform | Installer / Archive | Notes |
| --- | --- | --- |
| Linux (x86_64, aarch64) | `.tar.zst` runtime payload plus `.deb` meta-package | Includes systemd service template, shell completion, and UDS socket permissions manifest. |
| macOS (universal) | Signed `.pkg` installer | Provides launchd plist, notarization checklist, and quarantine-release script. |
| Windows / WSL bridge | `.zip` payload and `.msi` bootstrapper | Ships DPAPI key escrow helper, PowerShell bridge scripts, and WSL integration shim. |

Each bundle must include:
- Runtime binaries built with the feature matrix from `Cargo.toml` (HTTP, stdio, UDS
  transports; encrypted storage; governance policy engine).
- Client-script validation policy cache (`policies/runtime/*.json`).
- Default configuration (`config/runtime.toml`) matching the architecture overview.
- Checksums (`SHA256SUMS`, `manifest.json`) produced during the packaging workflow.

## Sequenced Workstreams

1. **Bundle Definition & Metadata**
   - Create `packaging/runtime/manifest.yaml` describing binary targets, asset staging
     paths, and platform-specific post-install hooks.
   - Document signing requirements and checksum expectations in
     [`docs/design/overview.md`](../design/overview.md#platform-support-and-operational-guidance).
   - Align feature flags with `runtime-phase-1` crate scaffolding so packaging picks up
     the correct binaries.

2. **Policy & Configuration Embedding**
   - Introduce a packaging helper crate (`crates/runtime-packaging`) that copies policy
     bundles and configuration templates into the staging directory while verifying
     signatures.
   - Reference the [Input Validation](../security/threat-model.md#input-validation-checklist)
     and [Sandboxing](../security/threat-model.md#sandboxing-checklist) checklists in the
     crate README to maintain traceability.

3. **Installer Generation Scripts**
   - Add scripts under `scripts/packaging/` (`build_deb.sh`, `build_pkg.sh`,
     `build_msi.ps1`) that transform the staged bundle into platform-specific installers.
   - Scripts must surface checksum outputs that the GitHub Actions workflow collects.
   - Windows scripts coordinate with the existing WSL bridge automation so `.msi`
     installers include bridge prerequisites.

4. **GitHub Actions Workflow Updates**
   - Extend the packaging workflow (new file `package-runtime.yml`) to execute the
     staging helper, call each installer script, and upload the resulting archives plus
     checksum manifests as artifacts.
   - Chain the workflow after `Regenerate Fixture Corpus` and `Regenerate Golden
     Artifacts` so freshly generated fixtures/goldens are included in the bundle.
   - Store artifact metadata (commit SHA, feature flags, security checklist references)
     in `artifacts/package-runtime/metadata.json` for governance audits.

5. **Verification & Release Gates**
   - Document verification steps in `docs/process/pr-release-checklist.md`, including
     installer smoke tests (`--version`, configuration path checks) and checksum
     validation.
   - Require governance log entries referencing the workflow artifact URL before
     promoting a package to release status.

## Documentation & Cross-References

- Update [`docs/design/overview.md`](../design/overview.md#next-steps) to reference this
  plan and clarify packaging expectations.
- Link CI coverage requirements in [`docs/testing/ci-coverage.md`](../testing/ci-coverage.md)
  once the packaging workflow publishes artifacts consumed by transport regression jobs.
- Capture packaging review outcomes in the governance log per
  [`docs/process/governance-log.md`](../process/governance-log.md).

## Pending Follow-Up Items

- Draft threat-model deltas capturing installer delivery vectors and artifact signing.
- Define rollback procedures for package revocation and document them alongside the
  packaging scripts.
- Coordinate with the regression test expansion plan so packaged artifacts feed the
  WSL multi-repository test harnesses without manual fixture copying (the harnesses must
  consume workflow artifacts generated by GitHub Actions).

This plan satisfies the repository mandate to plan before coding and provides the
traceable steps required to enable cross-platform runtime distribution.
