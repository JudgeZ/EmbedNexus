use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::error::StoreError;
use crate::ledger::build_replay_entry;
use storage_ledger::ReplayEntry;

#[derive(Debug, Default, Clone)]
pub struct ReplayStats {
    pub applied: usize,
    pub skipped: usize,
    pub max_sequence: Option<u64>,
}

/// Minimal store abstraction for Milestone 3.
pub trait Store: Send + Sync {
    /// Insert or update a payload and return a replay entry describing the write.
    fn upsert(&self, repo_id: &str, key: &str, payload: &[u8]) -> Result<ReplayEntry, StoreError>;
    /// Fetch the latest payload for a key.
    fn get(&self, repo_id: &str, key: &str) -> Result<Option<Vec<u8>>, StoreError>;
    /// Apply replay entries; ordering/idempotency semantics are implementation-defined.
    fn replay<I: IntoIterator<Item = ReplayEntry>>(&self, entries: I) -> Result<ReplayStats, StoreError>;
}

/// An in-memory store stub to enable TDD. Future work may add FS-backed shards.
pub struct VectorStore {
    inner: Arc<Mutex<HashMap<(String, String), Vec<u8>>>>,
    next_sequence: AtomicU64,
}

impl Default for VectorStore {
    fn default() -> Self {
        Self::new()
    }
}

impl VectorStore {
    pub fn new() -> Self {
        Self { inner: Arc::new(Mutex::new(HashMap::new())), next_sequence: AtomicU64::new(1) }
    }

    fn checksum_placeholder(bytes: &[u8]) -> String {
        // Keep cheap and deterministic to avoid pulling hashing deps in the skeleton.
        format!("len:{}", bytes.len())
    }
}

impl Store for VectorStore {
    fn upsert(&self, repo_id: &str, key: &str, payload: &[u8]) -> Result<ReplayEntry, StoreError> {
        let before = Self::checksum_placeholder(payload);
        // Store plaintext for now; encryption gates will hook here in future commits.
        {
            let mut guard = self.inner.lock().map_err(|e| StoreError::Io(e.to_string()))?;
            guard.insert((repo_id.to_string(), key.to_string()), payload.to_vec());
        }
        let after = before.clone();
        let seq = self.next_sequence.fetch_add(1, Ordering::SeqCst);
        Ok(build_replay_entry(seq, repo_id, &before, &after, "emitted"))
    }

    fn get(&self, repo_id: &str, key: &str) -> Result<Option<Vec<u8>>, StoreError> {
        let guard = self.inner.lock().map_err(|e| StoreError::Io(e.to_string()))?;
        Ok(guard.get(&(repo_id.to_string(), key.to_string())).cloned())
    }

    fn replay<I: IntoIterator<Item = ReplayEntry>>(&self, entries: I) -> Result<ReplayStats, StoreError> {
        // Minimal semantics: update max_sequence and count entries; payload restoration to be added later.
        let mut stats = ReplayStats::default();
        let mut max_seq: Option<u64> = None;
        for entry in entries {
            stats.applied += 1;
            max_seq = Some(max_seq.map(|m| m.max(entry.sequence)).unwrap_or(entry.sequence));
        }
        if let Some(m) = max_seq {
            // Advance floor if replay moved sequence forward
            let mut current = self.next_sequence.load(Ordering::SeqCst);
            while m + 1 > current {
                match self.next_sequence.compare_exchange(current, m + 1, Ordering::SeqCst, Ordering::SeqCst) {
                    Ok(_) => break,
                    Err(now) => current = now,
                }
            }
        }
        stats.max_sequence = max_seq;
        Ok(stats)
    }
}

