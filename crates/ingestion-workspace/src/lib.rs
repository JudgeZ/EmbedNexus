//! Workspace enumeration and preparation stubs.

use std::collections::HashSet;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Default)]
pub struct EnumeratorConfig {
    pub global_ignores: Vec<IgnoreRule>,
    pub sandbox_ignores: Vec<IgnoreRule>,
}

#[derive(Debug, Clone, Default)]
pub struct RegistrySnapshot {
    pub workspaces: Vec<WorkspaceRecord>,
}

impl RegistrySnapshot {
    pub fn new(workspaces: Vec<WorkspaceRecord>) -> Self {
        Self { workspaces }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepoType {
    Git,
    Archive,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IgnoreSource {
    Git,
    Editor,
    Sandbox,
    Global,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IgnoreRule {
    pub source: IgnoreSource,
    pub pattern: String,
}

impl IgnoreRule {
    pub fn new(source: IgnoreSource, pattern: impl Into<String>) -> Self {
        Self {
            source,
            pattern: pattern.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LatencyEvent {
    pub path: String,
    pub action: String,
    pub latency_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LatencyWindow {
    pub window_ms: u64,
    pub debounce_ms: u64,
    pub queue_depth: u32,
    pub events: Vec<LatencyEvent>,
    pub events_observed: u32,
    pub max_latency_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchiveDescriptor {
    pub name: String,
    pub bytes: u64,
    pub entries: u64,
    pub nesting_depth: u32,
    pub expected_status: String,
    pub max_latency_ms: u64,
    pub scenario: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceFile {
    pub path: String,
    pub content: String,
}

impl WorkspaceFile {
    pub fn new(path: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            content: content.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceRecord {
    pub repo_id: String,
    pub root_path: PathBuf,
    pub repo_type: RepoType,
    pub manifest_cursor: Option<String>,
    pub ignore_rules: Vec<IgnoreRule>,
    pub archives: Vec<ArchiveDescriptor>,
    pub latency_windows: Vec<LatencyWindow>,
    pub files: Vec<WorkspaceFile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceDescriptor {
    pub repo_id: String,
    pub root_path: PathBuf,
    pub repo_type: RepoType,
    pub manifest_cursor: Option<String>,
    pub ignore_stack: Vec<IgnoreRule>,
    pub archives: Vec<ArchiveDescriptor>,
    pub latency_windows: Vec<LatencyWindow>,
    pub files: Vec<WorkspaceFile>,
}

impl WorkspaceDescriptor {
    pub fn latency_windows(&self) -> &[LatencyWindow] {
        &self.latency_windows
    }

    pub fn ignore_stack(&self) -> &[IgnoreRule] {
        &self.ignore_stack
    }

    pub fn files(&self) -> &[WorkspaceFile] {
        &self.files
    }
}

#[derive(Debug, Error)]
pub enum WorkspaceError {
    #[error("workspace enumeration failed: {0}")]
    Enumeration(String),
}

#[derive(Debug, Clone)]
pub struct WorkspaceEnumerator {
    config: EnumeratorConfig,
}

impl WorkspaceEnumerator {
    pub fn new(config: EnumeratorConfig) -> Self {
        Self { config }
    }

    pub fn scan(
        &self,
        snapshot: &RegistrySnapshot,
    ) -> Result<Vec<WorkspaceDescriptor>, WorkspaceError> {
        let mut descriptors = Vec::with_capacity(snapshot.workspaces.len());
        for record in snapshot.workspaces.iter() {
            let ignore_stack = self.merge_ignore_stack(&record.ignore_rules);
            let latency_windows = record
                .latency_windows
                .iter()
                .map(Self::normalize_window)
                .collect();
            descriptors.push(WorkspaceDescriptor {
                repo_id: record.repo_id.clone(),
                root_path: record.root_path.clone(),
                repo_type: record.repo_type.clone(),
                manifest_cursor: record.manifest_cursor.clone(),
                ignore_stack,
                archives: record.archives.clone(),
                latency_windows,
                files: record.files.clone(),
            });
        }
        Ok(descriptors)
    }

    fn merge_ignore_stack(&self, repo_rules: &[IgnoreRule]) -> Vec<IgnoreRule> {
        let mut seen = HashSet::new();
        let mut stack = Vec::new();
        for rule in &self.config.global_ignores {
            Self::push_rule(&mut stack, &mut seen, rule.clone());
        }
        for rule in &self.config.sandbox_ignores {
            Self::push_rule(&mut stack, &mut seen, rule.clone());
        }
        for rule in repo_rules {
            if seen.remove(&rule.pattern) {
                stack.retain(|existing| existing.pattern != rule.pattern);
            }
            Self::push_rule(&mut stack, &mut seen, rule.clone());
        }
        stack
    }

    fn normalize_window(window: &LatencyWindow) -> LatencyWindow {
        let mut normalized = window.clone();
        if normalized.events_observed == 0 {
            normalized.events_observed = normalized.events.len() as u32;
        }
        if normalized.max_latency_ms == 0 {
            normalized.max_latency_ms = normalized
                .events
                .iter()
                .map(|event| event.latency_ms)
                .max()
                .unwrap_or_default();
        }
        normalized
    }

    fn push_rule(stack: &mut Vec<IgnoreRule>, seen: &mut HashSet<String>, rule: IgnoreRule) {
        if seen.insert(rule.pattern.clone()) {
            stack.push(rule);
        }
    }
}
