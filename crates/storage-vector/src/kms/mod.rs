//! Feature-gated key management interfaces for the vector store.

use crate::config::RotationPolicy;
use crate::encryption::KeyHandle;
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub struct KeyScope {
    pub repo_id: String,
}

pub trait KeyManager: Send + Sync {
    fn current(&self, scope: &KeyScope) -> Result<KeyHandle, String>;
    fn rotate_if_needed(
        &self,
        _scope: &KeyScope,
        _policy: &RotationPolicy,
    ) -> Result<Option<KeyHandle>, String> {
        Ok(None)
    }
    fn get(&self, key_id: &str) -> Result<KeyHandle, String>;
}

/// Minimal in-memory key manager for tests.
pub struct InMemoryKeyManager {
    current_id: Mutex<String>,
}

impl InMemoryKeyManager {
    pub fn new(current_id: impl Into<String>) -> Self {
        Self {
            current_id: Mutex::new(current_id.into()),
        }
    }
    pub fn set_current_id(&self, id: impl Into<String>) {
        *self.current_id.lock().expect("key mutex") = id.into();
    }
}

impl KeyManager for InMemoryKeyManager {
    fn current(&self, _scope: &KeyScope) -> Result<KeyHandle, String> {
        Ok(KeyHandle {
            key_id: self.current_id.lock().map_err(|e| e.to_string())?.clone(),
        })
    }

    fn get(&self, key_id: &str) -> Result<KeyHandle, String> {
        Ok(KeyHandle {
            key_id: key_id.to_string(),
        })
    }
}
