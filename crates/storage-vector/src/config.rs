//! Minimal configuration placeholders for the vector store skeleton.

#[derive(Debug, Clone, Default)]
pub struct StoreConfig {
    pub repo_scope: Option<String>,
}

/// Minimal rotation policy placeholder for feature-gated encryption flows.
#[derive(Debug, Clone, Default)]
pub struct RotationPolicy {
    pub max_uses: Option<u64>,
    pub rotate_after_seconds: Option<u64>,
}
