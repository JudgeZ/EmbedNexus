# Python MCP Client (Planned)

> **Status:** Implementation pending. Follow the implementation roadmap in
> [`docs/integration/client-plan.md`](../../docs/integration/client-plan.md) for the
> authoritative sequence of work and keep this README synchronized as the plan
> evolves.

## Planned script and responsibilities
- `clients/python/client.py` will expose a `main()` entry point that wires
  argument parsing, transport selection, and transcript recording exactly as
  described in the client implementation roadmap.
- A `TransportFactory` will resolve `stdio`, `http` (SSE), and `tls` adapters so
  the CLI can connect to any supported MCP server transport without code
  duplication.
- A `TranscriptRecorder` helper will persist request/response envelopes to
  `artifacts/python/<transport>.json` for regression validation.

## Transport coverage
- **`stdio`**: Launches the MCP server as a subprocess and streams JSON-RPC 2.0
  envelopes over standard I/O pipes.
- **`http` / SSE**: Exercises the asynchronous SSE channel with mock HTTP
  servers to ensure deterministic framing and retry behavior.
- **`tls`**: Wraps the HTTP transport with TLS, validating certificate handling
  via repository-provided dev CA bundles.

All transport details, fixture paths, and negotiation notes must remain aligned
with the shared plan to avoid drift across languages.

## Test expectations
- Author failing `pytest` cases (parametrized per transport) before
  implementation, invoking the CLI via subprocess to assert exit codes and
  transcript parity with the golden fixtures enumerated in the roadmap.
- Diff captured transcripts against the golden files under
  `tests/fixtures/python/` and gate updates behind an explicit
  `--update-transcripts` workflow.
- Run `ruff`, unit tests, and transport matrix scenarios that match the CI
  expectations.

## Traceability and CI alignment
- Cross-reference the architecture traceability map at
  [`docs/design/traceability.md`](../../docs/design/traceability.md) to document
  how transport adapters, fixtures, and security checks satisfy the broader
  platform commitments.
- The CI workflow requirements live in
  [`docs/testing/ci-coverage.md`](../../docs/testing/ci-coverage.md); mirror its
  matrix, fixture prerequisites, and artifact expectations when landing the
  implementation.
