//! Vector store abstractions focused on archive quota tracking.

use thiserror::Error;

#[derive(Debug, Clone, Copy, Default)]
pub struct QuotaLimits {
    pub bytes_max: Option<u64>,
    pub entries_max: Option<u64>,
    pub nesting_max: Option<u32>,
    pub latency_budget_ms: Option<u64>,
}

#[derive(Debug, Clone, Copy, Default)]
struct QuotaObservation {
    bytes: u64,
    entries: u64,
    max_nesting: u32,
    max_latency_ms: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct ArchiveSample {
    pub bytes: u64,
    pub entries: u64,
    pub nesting_depth: u32,
    pub max_latency_ms: u64,
}

#[derive(Debug, Clone)]
pub struct QuotaDiagnostics {
    pub bytes_limit: u64,
    pub entries_limit: u64,
    pub nesting_limit: u32,
    pub bytes_observed: u64,
    pub entries_observed: u64,
    pub nesting_observed: u32,
    pub latency_budget_ms: Option<u64>,
}

#[derive(Debug, Error)]
pub enum QuotaError {
    #[error("archive quota exceeded")]
    Exceeded { diagnostics: QuotaDiagnostics },
}

#[derive(Debug, Clone)]
pub struct ArchiveQuotaTracker {
    limits: QuotaLimits,
    observed: QuotaObservation,
}

impl ArchiveQuotaTracker {
    pub fn new(limits: QuotaLimits) -> Self {
        Self {
            limits,
            observed: QuotaObservation::default(),
        }
    }

    pub fn observe(&mut self, sample: &ArchiveSample) {
        self.observed.bytes = self.observed.bytes.saturating_add(sample.bytes);
        self.observed.entries = self.observed.entries.saturating_add(sample.entries);
        self.observed.max_nesting = self.observed.max_nesting.max(sample.nesting_depth);
        self.observed.max_latency_ms = self.observed.max_latency_ms.max(sample.max_latency_ms);
    }

    pub fn check(&self) -> Result<(), QuotaError> {
        let diagnostics = self.diagnostics();
        let mut exceeded = false;
        if let Some(limit) = self.limits.bytes_max {
            if diagnostics.bytes_observed > limit {
                exceeded = true;
            }
        }
        if let Some(limit) = self.limits.entries_max {
            if diagnostics.entries_observed > limit {
                exceeded = true;
            }
        }
        if let Some(limit) = self.limits.nesting_max {
            if diagnostics.nesting_observed > limit {
                exceeded = true;
            }
        }
        if let Some(limit) = self.limits.latency_budget_ms {
            if diagnostics
                .latency_budget_ms
                .map(|value| value > limit)
                .unwrap_or(false)
            {
                exceeded = true;
            }
        }
        if exceeded {
            Err(QuotaError::Exceeded { diagnostics })
        } else {
            Ok(())
        }
    }

    pub fn diagnostics(&self) -> QuotaDiagnostics {
        QuotaDiagnostics {
            bytes_limit: self.limits.bytes_max.unwrap_or(self.observed.bytes),
            entries_limit: self.limits.entries_max.unwrap_or(self.observed.entries),
            nesting_limit: self.limits.nesting_max.unwrap_or(self.observed.max_nesting),
            bytes_observed: self.observed.bytes,
            entries_observed: self.observed.entries,
            nesting_observed: self.observed.max_nesting,
            latency_budget_ms: Some(self.observed.max_latency_ms),
        }
    }

    pub fn limits(&self) -> &QuotaLimits {
        &self.limits
    }
}

impl QuotaDiagnostics {
    pub fn merge_with_limits(mut self, limits: &QuotaLimits) -> Self {
        if let Some(limit) = limits.latency_budget_ms {
            self.latency_budget_ms = Some(self.latency_budget_ms.unwrap_or(0).max(limit));
        }
        self
    }
}
