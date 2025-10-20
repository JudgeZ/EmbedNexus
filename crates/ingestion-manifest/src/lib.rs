//! Manifest emitter and replay scaffolding.

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::SystemTime;

use ingestion_embedding::EmbeddingBatch;
use storage_ledger::{OfflineReplayBuffer, ReplayEntry};
use thiserror::Error;

pub trait ManifestQueue: Send + Sync {
    fn send(&self, entry: ReplayEntry) -> anyhow::Result<()>;
}

#[derive(Debug, Clone)]
pub struct ManifestEmitterConfig {
    pub sequence_start: u64,
    pub encryption_key: String,
    pub retention_max_entries: usize,
    pub retention_max_age: std::time::Duration,
}

#[derive(Debug, Clone)]
pub struct ManifestDiff {
    pub repo_id: String,
    pub applied_at: SystemTime,
    pub added_chunks: Vec<String>,
    pub removed_chunks: Vec<String>,
    pub checksum_before: String,
    pub checksum_after: String,
}

#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("manifest queue offline: {0}")]
    QueueOffline(String),
    #[error("offline buffer error: {0}")]
    Buffer(String),
}

#[derive(Debug)]
pub struct ManifestEmitter<Q: ManifestQueue + ?Sized> {
    config: ManifestEmitterConfig,
    buffer: OfflineReplayBuffer,
    queue: Arc<Q>,
    next_sequence: u64,
}

impl<Q> ManifestEmitter<Q>
where
    Q: ManifestQueue + ?Sized,
{
    pub fn new(config: ManifestEmitterConfig, buffer: OfflineReplayBuffer, queue: Arc<Q>) -> Self {
        let next_sequence = buffer
            .max_sequence()
            .map(|seq| seq + 1)
            .unwrap_or(config.sequence_start);
        Self {
            config,
            buffer,
            queue,
            next_sequence,
        }
    }

    pub fn emit(
        &mut self,
        diff: ManifestDiff,
        _batch: EmbeddingBatch,
    ) -> Result<(), ManifestError> {
        let sequence = self.next_sequence;
        let mut entry = self.build_entry(&diff, sequence);
        let mut send_entry = entry.clone();
        send_entry.status = "emitted".into();
        match self.queue.send(send_entry) {
            Ok(()) => {
                self.next_sequence = sequence + 1;
                Ok(())
            }
            Err(err) => {
                entry.status = "buffered".into();
                self.buffer
                    .push(entry)
                    .map_err(|error| ManifestError::Buffer(error.to_string()))?;
                self.next_sequence = sequence + 1;
                Err(ManifestError::QueueOffline(err.to_string()))
            }
        }
    }

    pub fn flush_offline(&mut self) -> Result<(), ManifestError> {
        let mut drained: VecDeque<_> = self.buffer.drain_ready().into();
        while let Some(mut ready) = drained.pop_front() {
            let sequence = ready.entry.sequence;
            ready.entry.status = "emitted".into();
            if let Err(err) = self.queue.send(ready.entry.clone()) {
                // push back into buffer to retry later and preserve ordering
                ready.entry.status = "buffered".into();
                self.buffer
                    .requeue(ready)
                    .map_err(|error| ManifestError::Buffer(error.to_string()))?;

                while let Some(mut remaining) = drained.pop_front() {
                    remaining.entry.status = "buffered".into();
                    self.buffer
                        .requeue(remaining)
                        .map_err(|error| ManifestError::Buffer(error.to_string()))?;
                }

                return Err(ManifestError::QueueOffline(err.to_string()));
            }
            self.next_sequence = self.next_sequence.max(sequence + 1);
        }
        Ok(())
    }

    fn build_entry(&self, diff: &ManifestDiff, sequence: u64) -> ReplayEntry {
        ReplayEntry {
            sequence,
            repo_id: diff.repo_id.clone(),
            delayed_ms: self.config.retention_max_age.as_millis() as u64,
            payload_checksum_before: diff.checksum_before.clone(),
            payload_checksum_after: diff.checksum_after.clone(),
            status: String::from("pending"),
        }
    }
}
