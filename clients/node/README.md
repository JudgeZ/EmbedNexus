# Node.js MCP Client (Planned)

> **Status:** Implementation pending. Implementation sequencing, fixture
> ownership, and transport notes are governed by
> [`docs/integration/client-plan.md`](../../docs/integration/client-plan.md);
> update this README whenever that roadmap changes.

## Planned script and responsibilities
- `clients/node/index.mjs` will export a `runClient` routine consumed by a CLI
  wrapper, adhering to the architecture spelled out in the roadmap.
- Transport strategy classes will implement a shared `connect()` contract for
  `stdio`, `http` (SSE/WebSocket), and `tls` channels, ensuring feature parity
  with the Python and Go clients.
- A `Recorder` module will persist JSON-RPC transcripts to
  `artifacts/node/<transport>.json` for regression checks and CI artifact
  uploads.

## Transport coverage
- **`stdio`**: Spawns the MCP server via `child_process.spawn`, streaming
  envelopes through the standard I/O pipes.
- **`http` / SSE & WebSocket**: Uses native `fetch` (Node 18+) and WebSocket
  helpers to capture streaming updates while exercising retry semantics.
- **`tls`**: Relies on `https.Agent` with custom CA support to validate
  certificate pinning and mutual TLS scenarios described in the roadmap.

Keep the transport surface, adapter responsibilities, and fixture directories in
lockstep with the shared plan to preserve cross-language consistency.

## Test expectations
- Write failing `vitest`/`jest` integration tests before implementation that run
  the CLI via `execa`, asserting exit codes and transcript equality against the
  golden fixtures identified in the roadmap.
- Use snapshot diffing of captured transcripts versus the canonical files under
  `tests/fixtures/node/`, gating updates behind an explicit opt-in flag.
- Execute `eslint`, unit tests, and transport-matrix scenarios required by the
  CI workflow.

## Traceability and CI alignment
- Document how Node transport adapters and fixtures satisfy platform promises by
  referencing [`docs/design/traceability.md`](../../docs/design/traceability.md)
  in planning discussions and PRs.
- Align CI tasks with [`docs/testing/ci-coverage.md`](../../docs/testing/ci-coverage.md),
  ensuring matrix keys, artifact uploads, and bridge scenarios match the
  repository-wide expectations.
