# Roadmap — Phase 2 Test Plan

Phase 2 extends the transport→router→ledger spine established in Phase 1. The tests below keep
the existing crate-local harness approach (unit tests under each crate’s `tests/` module plus
integration harnesses when needed) and rely on current fixtures. No new infrastructure is
required at this stage; these are planning notes only.

## Planned Coverage

### 1. Runtime Transport UDS Parity (Auth & Dispatch)
- **File:** `crates/runtime-transport-uds/src/lib.rs` (`#[cfg(test)] mod tests`)
  - **Test:** `negotiate_peer_concurrent_connections_same_uid`
    - **Given:** UDS adapter bound with `allowed_principals` and `allowed_uids = {1000}`; two
      simulated peers share UID 1000.
    - **When:** Both peers negotiate concurrently via `tokio::join!`.
    - **Then:** Each negotiates successfully, telemetry records two `uds.peer.accepted` events,
      and the router sees two authenticated dispatches.
  - **Test:** `dispatch_rejects_unnegotiated_peer`
    - **Given:** Adapter with no negotiated peers and a request carrying a valid token.
    - **When:** `dispatch` executes without a prior handshake.
    - **Then:** Returns `TransportError::Unauthorized`, emits `uds.peer.rejected`, and the router
      mock sees zero commands.

### 2. Runtime Router Capability Enforcement
- **File:** `crates/runtime-router/src/lib.rs` (new `#[cfg(test)] mod tests`)
  - **Test:** `dispatch_enforces_capability_requirements`
    - **Given:** Router wrapper exposing `dispatch_with_capability_guard` and a script that
      requires `"admin"` capability.
    - **When:** A session context with capabilities `["read"]` sends `"admin.reset"`.
    - **Then:** Router returns `RouterError::InsufficientCapabilities`, records a downgrade
      telemetry event, and does not execute the command handler.
  - **Test:** `dispatch_forwards_authorized_command`
    - **Given:** Same guard, but the session carries `["admin"]`.
    - **When:** Dispatching `"admin.reset"`.
    - **Then:** Command executes, the mock handler records the call, and telemetry shows
      `router.dispatch.ok`.

### 3. Storage Ledger Replay Ordering
- **File:** `crates/storage-ledger/src/lib.rs` (`#[cfg(test)] mod tests`)
  - **Test:** `drain_ready_concurrent_push_preserves_order`
    - **Given:** Buffer containing sequences `[1, 2, 3]` plus a concurrent producer pushing
      `[4, 5]`.
    - **When:** `tokio::join!` drains the buffer while pushes occur.
    - **Then:** First drain yields `[1, 2, 3]`, second drain yields `[4, 5]`, and max sequence
      remains 5.
  - **Test:** `requeue_after_partial_flush_maintains_sequence`
    - **Given:** Drain returns `[1, 2, 3]` but commit of sequence 2 fails and is requeued.
    - **When:** `requeue` is applied followed by another drain.
    - **Then:** Requeued entry surfaces before higher sequences, and retries do not duplicate
      committed entries.

### 4. Manifest Emitter Backpressure & Resume
- **File:** `crates/ingestion-manifest/tests/manifest_emitter.rs`
  - **Test:** `flush_offline_handles_queue_failure_mid_flush`
    - **Given:** TestQueue configured to fail after N sends while buffer holds >N entries.
    - **When:** `flush_offline` executes.
    - **Then:** First N entries leave the buffer, remaining entries stay queued, and telemetry
      records `manifest.flush.partial_failure`.
  - **Test:** `emit_during_active_flush_queues_follow_up`
    - **Given:** Flush in progress with outstanding entries and a concurrent `emit` call.
    - **When:** Flush completes.
    - **Then:** New emission remains queued, subsequent `flush_offline` drains it in FIFO order,
      and backpressure metrics reset.

## Next Steps
- Land these failing tests (or equivalent harnesses) prior to implementing code changes.
- Capture red/green evidence for each area to maintain the TDD discipline documented in
  `AGENTS.md`.
