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
    max_sequence_seen: Arc<Mutex<Option<u64>>>,
}

impl OfflineReplayBuffer {
    #[must_use]
    pub fn new(max_entries: usize, max_age: Duration) -> Self {
        Self {
            max_entries,
            max_age,
            inner: Arc::new(Mutex::new(VecDeque::new())),
            max_sequence_seen: Arc::new(Mutex::new(None)),
        }
    }

    pub fn push(&self, entry: ReplayEntry) -> Result<(), ReplayError> {
        let now = SystemTime::now();
        self.push_envelope(entry, now)
    }

    pub fn requeue(&self, ready: ReadyReplayEntry) -> Result<(), ReplayError> {
        self.push_envelope(ready.entry, ready.inserted_at)
    }

    #[must_use]
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

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner
            .lock()
            .map(|guard| guard.is_empty())
            .unwrap_or(true)
    }

    #[must_use]
    pub fn max_sequence(&self) -> Option<u64> {
        *self.max_sequence_seen.lock().unwrap()
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
        {
            let mut max_seen = self.max_sequence_seen.lock().unwrap();
            match *max_seen {
                Some(existing) if existing >= entry.sequence => {}
                _ => *max_seen = Some(entry.sequence),
            }
        }
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

    fn entry_with_sequence(sequence: u64) -> ReplayEntry {
        ReplayEntry {
            sequence,
            repo_id: format!("repo-{sequence}"),
            delayed_ms: 0,
            payload_checksum_before: format!("before-{sequence}"),
            payload_checksum_after: format!("after-{sequence}"),
            status: "buffered".into(),
        }
    }

    #[test]
    fn requeue_preserves_original_age_for_expiration() {
        let buffer = OfflineReplayBuffer::new(16, Duration::from_millis(100));
        buffer.push(entry_with_sequence(1)).unwrap();
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

    #[test]
    fn capacity_eviction_removes_oldest_entry_fifo() {
        let buffer = OfflineReplayBuffer::new(3, Duration::from_secs(60));
        for sequence in 1..=4 {
            buffer.push(entry_with_sequence(sequence)).unwrap();
        }

        assert_eq!(buffer.max_sequence(), Some(4));

        let drained = buffer.drain_ready();
        let sequences: Vec<u64> = drained.iter().map(|ready| ready.entry.sequence).collect();
        assert_eq!(sequences, vec![2, 3, 4], "oldest entry should be evicted");
        assert!(buffer.is_empty());
    }

    #[test]
    fn purges_entries_exceeding_max_age() {
        let buffer = OfflineReplayBuffer::new(4, Duration::from_millis(50));
        buffer.push(entry_with_sequence(10)).unwrap();

        thread::sleep(Duration::from_millis(65));

        let drained = buffer.drain_ready();
        assert!(drained.is_empty(), "expired entry should be purged");
        assert!(buffer.is_empty());
    }

    #[test]
    fn requeue_after_partial_flush_maintains_sequence() {
        let buffer = OfflineReplayBuffer::new(8, Duration::from_secs(60));
        for seq in 1..=3 {
            buffer.push(entry_with_sequence(seq)).unwrap();
        }

        // First drain simulates a partial flush (sequence 2 fails downstream)
        let mut drained = buffer.drain_ready();
        assert_eq!(drained.len(), 3);
        let failed_entry = drained.remove(1);

        buffer.requeue(failed_entry).unwrap();
        assert_eq!(buffer.max_sequence(), Some(3));

        // New pushes arrive while the failed entry is waiting to reflush
        for seq in 4..=5 {
            buffer.push(entry_with_sequence(seq)).unwrap();
        }

        let drained_after_requeue = buffer.drain_ready();
        let sequences: Vec<u64> = drained_after_requeue
            .iter()
            .map(|ready| ready.entry.sequence)
            .collect();
        assert_eq!(sequences, vec![2, 4, 5]);
        assert!(buffer.is_empty());
    }

    #[test]
    fn drain_ready_concurrent_push_preserves_order() {
        let buffer = OfflineReplayBuffer::new(8, Duration::from_secs(60));
        for seq in 10..=12 {
            buffer.push(entry_with_sequence(seq)).unwrap();
        }

        // Drain existing entries while new ones arrive
        let drained = buffer.drain_ready();
        let drained_sequences: Vec<u64> =
            drained.iter().map(|ready| ready.entry.sequence).collect();
        assert_eq!(drained_sequences, vec![10, 11, 12]);

        for seq in 13..=15 {
            buffer.push(entry_with_sequence(seq)).unwrap();
        }

        let drained_after_push = buffer.drain_ready();
        let sequences_after_push: Vec<u64> = drained_after_push
            .iter()
            .map(|ready| ready.entry.sequence)
            .collect();
        assert_eq!(sequences_after_push, vec![13, 14, 15]);
        assert!(buffer.is_empty());
    }
}
