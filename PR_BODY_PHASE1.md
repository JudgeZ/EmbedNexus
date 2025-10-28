## Summary
Phase 1 establishes the transport→router→ledger spine: storage‑ledger eviction/expiry tests, HTTP adapter token lifecycle + auth‑failure telemetry, updated docs, and a Rust CI workflow.

## Changes
- Tests
  - crates/storage-ledger/src/lib.rs: FIFO eviction, max‑age purge, requeue expiry.
  - crates/runtime-transport-http/src/lib.rs: token issuance telemetry, disallowed principals, expired‑token rejection with telemetry, capability propagation.
- Code
  - HTTP adapter now decodes principal hints on verify failure and emits `http.auth.failure` telemetry before returning `Unauthorized`.
- Docs
  - AGENTS.md Phase 1 log entry; docs/testing/test-matrix.md coverage notes.
- CI
  - New .github/workflows/ci.yml installing native deps and running `cargo fmt`, `cargo clippy -D warnings`, and `cargo test`.

## Risk Assessment
- Auth telemetry noise: principal hint extraction avoids exposing raw tokens; schema unchanged elsewhere.
- Ledger eviction regressions: FIFO + expiry tests cover capacity/timing edges.
- CI flakiness: deterministic unit suites only; no external network I/O.

## Security Notes
- Tokens never logged; only sanitized principal hints.
- Authentication and input validation items align with threat‑model docs.

## Test Evidence
```
cargo test
# All targets pass locally (pre‑existing warnings only)
```

## Follow-ups (Phase 2)
1) UDS transport parity tests (negotiate same-uid concurrency; reject unnegotiated peer).
2) Router capability checks (capability requirements + routing matrix coverage).
3) Ledger replay ordering under intermittent failures.
4) Manifest emitter backpressure and resume coverage.
