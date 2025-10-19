# Go MCP Client (Planned)

> **Status:** Implementation pending. Treat
> [`docs/integration/client-plan.md`](../../docs/integration/client-plan.md) as the
> source of truth for sequencing, transport behavior, and fixture ownership, and
> update this README alongside any roadmap change.

## Planned script and responsibilities
- `clients/go/main.go` will call into a `client` package that exposes `Run(opts
  Options)` to coordinate CLI arguments, transport selection, and transcript
  capture per the roadmap.
- Interfaces such as `Transport` and `Recorder` will isolate implementation
  details for `stdio`, `http` (SSE), and `tls` so new transports can plug in
  without altering the CLI surface.
- Captured transcripts will be written to `artifacts/go/<transport>.json` for
  regression checks and CI artifact uploads.

## Transport coverage
- **`stdio`**: Uses `exec.Command` pipes to exchange JSON-RPC envelopes with a
  mock MCP server.
- **`http` / SSE**: Leverages `net/http` helpers to process streaming responses
  and enforce retry timing.
- **`tls`**: Configures `tls.Config` with repository-provided certificate bundles
  to validate handshake behavior, pinning, and error handling.

Keep adapter responsibilities and fixture paths synchronized with the shared plan
so Go behavior mirrors Python and Node implementations.

## Test expectations
- Begin with failing `go test` table-driven cases that invoke the CLI binary and
  diff transcripts against the golden fixtures enumerated in the roadmap.
- Ensure captured transcripts under `tests/fixtures/go/` are compared against
  golden files and only refreshed through the explicit update workflow.
- Run `golangci-lint`, unit tests, and integration scenarios matching the CI
  transport matrix requirements.

## Traceability and CI alignment
- Reference [`docs/design/traceability.md`](../../docs/design/traceability.md) to
  document how Go transports and fixtures honor architecture commitments.
- Implement the CI behaviors defined in
  [`docs/testing/ci-coverage.md`](../../docs/testing/ci-coverage.md), including
  matrix job names, artifact handling, and bridge coverage.
