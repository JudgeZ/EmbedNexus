# Cursor Local Embedding MCP

## Mission
- Deliver a local-first embedding management platform that accelerates retrieval-augmented workflows without relying on external services.
- Provide deterministic ingestion across mixed repositories while preserving governance, auditability, and the 4C documentation standard.
- Empower developers with tooling that integrates seamlessly into existing CLI, IDE, and automation pipelines.

## Architecture at a Glance
```mermaid
flowchart LR
    subgraph Workspace[Local Workspace]
        Sources[Repositories & Documents]
        Policies[Ignore & Sanitization Rules]
    end
    subgraph Runtime[Local Runtime]
        Transport[Transport Adapters]
        Pipeline[Ingestion & Validation Pipeline]
        Vault[Encrypted Persistence]
    end
    subgraph Clients
        CLI[CLI & Client Scripts]
        IDE[IDE Extensions]
        API[Local HTTP Consumers]
    end

    Sources --> Pipeline
    Policies --> Pipeline
    Clients --> Transport --> Pipeline --> Vault
    Vault --> Transport
```
- Local transports (HTTP, stdio, and Unix domain sockets) keep traffic on-device while supporting diverse client scenarios.
- The ingestion pipeline enforces ignore policies, sanitizes inputs, and streams embeddings into encrypted storage.
- Encrypted persistence and manifest auditing provide reproducible state across multi-repository environments.

## Security Posture
- Encrypted stores are backed by per-repository keys and audited manifest chains to prevent tampering and rollback attacks.
- Transport adapters enforce authentication, input validation, and zero-trust defaults for CLI, IDE, and automation clients.
- Threat modeling, mitigation tracking, and checklist completion are mandated for every change; reference [`docs/security/threat-model.md`](docs/security/threat-model.md).

## Contributor Workflow Essentials
- **Plan before code:** Draft and circulate an implementation plan before touching source files, per [`AGENTS.md`](AGENTS.md).
- **Test-driven development (TDD):** Capture expected behavior in failing tests ahead of implementation and keep regression suites green.
- **Documentation discipline:** Maintain 4C compliance—ensure updates are clear, concise, complete, and correct, and synchronize README and related docs with feature changes.
- **Release readiness:** Run `rustfmt`, `cargo clippy --all-targets --all-features`, and complete the [PR release checklist](docs/process/pr-release-checklist.md) before requesting review.

## Documentation Map
- **Design Overview:** Architectural context, ingestion pipeline details, and transport adapters in [`docs/design/overview.md`](docs/design/overview.md).
- **Testing Strategy:** Platform and scenario coverage in [`docs/testing/test-matrix.md`](docs/testing/test-matrix.md).
- **Security Guidance:** Threat modeling reference and controls in [`docs/security/threat-model.md`](docs/security/threat-model.md).
- **Process & Governance:** Contribution governance, decision logs, and release workflow in [`docs/process/`](docs/process/).
- **Fixture Tooling:** Multi-language script stubs, runtime requirements, and linting expectations in [`scripts/README.md`](scripts/README.md).

## Fixture Generation Tooling

The [`scripts/`](scripts/) directory now hosts descriptive stubs for the tooling
referenced throughout [`docs/testing/fixtures-plan.md`](docs/testing/fixtures-plan.md)
and [`docs/testing/ci-coverage.md`](docs/testing/ci-coverage.md). Each placeholder
captures its intended responsibilities plus the runtimes and third-party
dependencies required to regenerate fixtures across Windows, macOS, Linux, and
WSL environments.

When implementing these utilities:

- Follow the linting and formatting expectations documented in
  [`scripts/README.md`](scripts/README.md) (`ruff`/`black` for Python, `rustfmt`/
  `clippy` for Rust, `shfmt`/`shellcheck` for shell, and `PSScriptAnalyzer` for
  PowerShell).
- Document new configuration switches or host prerequisites directly in the
  script docstrings and associated fixture READMEs to keep regeneration guidance
  complete.
- Update CI workflows to invoke the appropriate linters once the scripts move
  beyond placeholder status.

## Upcoming Client & IDE Resources
- Client integration patterns and validated scripts will land in [`docs/integration/clients.md`](docs/integration/clients.md); watch for connector templates and signing guidance.
- IDE and editor onboarding lives in [`docs/integration/ide-overview.md`](docs/integration/ide-overview.md), which will link to language-specific SDKs and Marketplace packages as they are published.
- A starter toolkit (CLI wrappers, VS Code tasks, and JetBrains run configurations) will ship alongside the integration docs so new contributors can bootstrap local testing quickly.

## Getting Started
1. Review the design, security, testing, and process documentation.
2. Draft your implementation plan and secure consensus before writing code.
3. Implement features with TDD, run the required linters, and document changes following the 4C standard.
