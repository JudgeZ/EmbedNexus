//! Ledger helpers for replay integration.

use storage_ledger::ReplayEntry;

/// Build a minimal replay entry for a write. Checksums are placeholders; the
/// integration tests assert ordering rather than specific checksum values.
pub fn build_replay_entry(
    sequence: u64,
    repo_id: &str,
    checksum_before: &str,
    checksum_after: &str,
    status: &str,
) -> ReplayEntry {
    ReplayEntry {
        sequence,
        repo_id: repo_id.to_string(),
        delayed_ms: 0,
        payload_checksum_before: checksum_before.to_string(),
        payload_checksum_after: checksum_after.to_string(),
        status: status.to_string(),
    }
}
