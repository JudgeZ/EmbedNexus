//! Chunk planner placeholder logic.

use blake3::Hasher;
use ingestion_workspace::{ArchiveDescriptor, WorkspaceDescriptor};
use storage_vector::{ArchiveQuotaTracker, ArchiveSample, QuotaError, QuotaLimits};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct PlannerConfig {
    pub target_chunk_bytes: usize,
    pub max_chunks_per_batch: usize,
    pub quota_bytes_max: Option<u64>,
    pub quota_entries_max: Option<u64>,
    pub quota_nesting_max: Option<u32>,
    pub quota_latency_budget_ms: Option<u64>,
}

impl PlannerConfig {
    #[must_use]
    pub const fn new(target_chunk_bytes: usize, max_chunks_per_batch: usize) -> Self {
        Self {
            target_chunk_bytes,
            max_chunks_per_batch,
            quota_bytes_max: None,
            quota_entries_max: None,
            quota_nesting_max: None,
            quota_latency_budget_ms: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub backoff_ms: u64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            backoff_ms: 1_000,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChunkPlan {
    pub plan_id: String,
    pub repo_id: String,
    pub chunker_config: String,
    pub source_span: String,
    pub hash: String,
    pub retry_policy: RetryPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedChunk {
    plan: ChunkPlan,
    payload: String,
}

impl PlannedChunk {
    pub fn new(plan: ChunkPlan, payload: impl Into<String>) -> Self {
        Self {
            plan,
            payload: payload.into(),
        }
    }

    #[must_use]
    pub const fn plan(&self) -> &ChunkPlan {
        &self.plan
    }

    #[must_use]
    pub fn payload(&self) -> &str {
        &self.payload
    }
}

#[derive(Debug, Error)]
pub enum PlanningError {
    #[error("planning not implemented")]
    NotImplemented,
    #[error("quota exceeded")]
    QuotaExceeded {
        diagnostics: storage_vector::QuotaDiagnostics,
    },
}

#[derive(Debug, Clone)]
pub struct ChunkPlanner {
    config: PlannerConfig,
}

impl ChunkPlanner {
    #[must_use]
    pub const fn new(config: PlannerConfig) -> Self {
        Self { config }
    }

    pub fn plan(&self, workspace: &WorkspaceDescriptor) -> Result<Vec<ChunkPlan>, PlanningError> {
        self.check_archive_quotas(workspace)?;
        let chunk_size = self.config.target_chunk_bytes.max(1);
        let mut files = workspace.files.clone();
        files.sort_by(|a, b| a.path.cmp(&b.path));
        let mut plans = Vec::new();
        let mut global_index: usize = 0;
        for file in &files {
            let bytes = file.content.as_bytes();
            if bytes.is_empty() {
                let plan_id = format!("{}::{}::{}", workspace.repo_id, file.path, global_index);
                plans.push(ChunkPlan {
                    plan_id,
                    repo_id: workspace.repo_id.clone(),
                    chunker_config: format!(
                        "bytes={chunk_size};max={}",
                        self.config.max_chunks_per_batch
                    ),
                    source_span: format!("{}:0-0", file.path),
                    hash: blake3::hash(&[]).to_hex().to_string(),
                    retry_policy: RetryPolicy::default(),
                });
                global_index += 1;
                continue;
            }
            let mut offset = 0usize;
            while offset < bytes.len() {
                let end = (offset + chunk_size).min(bytes.len());
                let slice = &bytes[offset..end];
                let mut hasher = Hasher::new();
                hasher.update(slice);
                let hash = hasher.finalize().to_hex().to_string();
                let plan_id = format!("{}::{}::{}", workspace.repo_id, file.path, global_index);
                plans.push(ChunkPlan {
                    plan_id,
                    repo_id: workspace.repo_id.clone(),
                    chunker_config: format!(
                        "bytes={chunk_size};max={}",
                        self.config.max_chunks_per_batch
                    ),
                    source_span: format!("{}:{}-{}", file.path, offset, end),
                    hash,
                    retry_policy: RetryPolicy::default(),
                });
                offset = end;
                global_index += 1;
            }
        }
        if plans.len() > self.config.max_chunks_per_batch {
            plans.truncate(self.config.max_chunks_per_batch);
        }
        Ok(plans)
    }

    fn check_archive_quotas(&self, workspace: &WorkspaceDescriptor) -> Result<(), PlanningError> {
        if workspace.archives.is_empty()
            && self.config.quota_bytes_max.is_none()
            && self.config.quota_entries_max.is_none()
            && self.config.quota_nesting_max.is_none()
        {
            return Ok(());
        }
        let mut tracker = ArchiveQuotaTracker::new(QuotaLimits {
            bytes_max: self.config.quota_bytes_max,
            entries_max: self.config.quota_entries_max,
            nesting_max: self.config.quota_nesting_max,
            latency_budget_ms: self.config.quota_latency_budget_ms,
        });
        for archive in &workspace.archives {
            tracker.observe(&self.sample_for_archive(archive));
        }
        match tracker.check() {
            Ok(()) => Ok(()),
            Err(QuotaError::Exceeded { diagnostics }) => Err(PlanningError::QuotaExceeded {
                diagnostics: diagnostics.merge_with_limits(tracker.limits()),
            }),
        }
    }

    const fn sample_for_archive(&self, archive: &ArchiveDescriptor) -> ArchiveSample {
        ArchiveSample {
            bytes: archive.bytes,
            entries: archive.entries,
            nesting_depth: archive.nesting_depth,
            max_latency_ms: archive.max_latency_ms,
        }
    }
}
