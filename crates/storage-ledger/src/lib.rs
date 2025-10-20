//! Ledger persistence and offline replay buffer utilities.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReplayEntry {
    pub sequence: u64,
    pub repo_id: String,
    pub delayed_ms: u64,
    pub payload_checksum_before: String,
    pub payload_checksum_after: String,
    pub status: String,
}

#[derive(Debug, Clone)]
struct ReplayEnvelope {
    entry: ReplayEntry,
    inserted_at: SystemTime,
}

#[derive(Debug, Clone)]
pub struct ReadyReplayEntry {
    pub entry: ReplayEntry,
    pub inserted_at: SystemTime,
}

#[derive(Debug, Clone)]
pub struct OfflineReplayBuffer {
    max_entries: usize,
    max_age: Duration,
    inner: Arc<Mutex<VecDeque<ReplayEnvelope>>>,
}

impl OfflineReplayBuffer {
    pub fn new(max_entries: usize, max_age: Duration) -> Self {
        Self {
            max_entries,
            max_age,
            inner: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn push(&self, entry: ReplayEntry) -> Result<(), ReplayError> {
        let now = SystemTime::now();
        self.push_envelope(entry, now)
    }

    pub fn requeue(&self, ready: ReadyReplayEntry) -> Result<(), ReplayError> {
        self.push_envelope(ready.entry, ready.inserted_at)
    }

    pub fn drain_ready(&self) -> Vec<ReadyReplayEntry> {
        let mut guard = self.inner.lock().expect("buffer mutex poisoned");
        let now = SystemTime::now();
        self.purge_locked(&mut guard, now);
        guard
            .drain(..)
            .map(|env| ReadyReplayEntry {
                entry: env.entry,
                inserted_at: env.inserted_at,
            })
            .collect()
    }

    pub fn is_empty(&self) -> bool {
        self.inner
            .lock()
            .map(|guard| guard.is_empty())
            .unwrap_or(true)
    }

    pub fn max_sequence(&self) -> Option<u64> {
        self.inner
            .lock()
            .ok()
            .and_then(|guard| guard.iter().map(|env| env.entry.sequence).max())
    }

    fn purge_locked(&self, guard: &mut VecDeque<ReplayEnvelope>, now: SystemTime) {
        guard.retain(|envelope| match now.duration_since(envelope.inserted_at) {
            Ok(age) => age <= self.max_age,
            Err(_) => true,
        });
    }

    fn push_envelope(
        &self,
        entry: ReplayEntry,
        inserted_at: SystemTime,
    ) -> Result<(), ReplayError> {
        if self.max_entries == 0 {
            return Err(ReplayError::Misconfigured(
                "max_entries cannot be zero".into(),
            ));
        }
        let mut guard = self.inner.lock().expect("buffer mutex poisoned");
        let now = SystemTime::now();
        guard.push_back(ReplayEnvelope { entry, inserted_at });
        self.purge_locked(&mut guard, now);
        while guard.len() > self.max_entries {
            guard.pop_front();
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum ReplayError {
    #[error("offline replay buffer misconfigured: {0}")]
    Misconfigured(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn requeue_preserves_original_age_for_expiration() {
        let buffer = OfflineReplayBuffer::new(16, Duration::from_millis(100));
        let entry = ReplayEntry {
            sequence: 1,
            repo_id: "repo-alpha".into(),
            delayed_ms: 0,
            payload_checksum_before: "before".into(),
            payload_checksum_after: "after".into(),
            status: "buffered".into(),
        };

        buffer.push(entry).unwrap();
        thread::sleep(Duration::from_millis(40));

        let mut ready = buffer.drain_ready();
        assert_eq!(ready.len(), 1);
        let ready_entry = ready.pop().unwrap();

        buffer.requeue(ready_entry).unwrap();
        assert!(!buffer.is_empty());

        thread::sleep(Duration::from_millis(80));
        let drained_after_wait = buffer.drain_ready();
        assert!(drained_after_wait.is_empty());
        assert!(buffer.is_empty());
    }
}
