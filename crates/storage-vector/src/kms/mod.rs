//! Feature-gated key management interfaces for the vector store.

use crate::config::RotationPolicy;
use crate::encryption::KeyHandle;
use rand::{rngs::OsRng, RngCore};
use std::collections::HashMap;
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
    keys: Mutex<HashMap<String, zeroize::Zeroizing<[u8; 32]>>>,
}

impl InMemoryKeyManager {
    pub fn new_with_secret(current_id: impl Into<String>, key: [u8; 32]) -> Self {
        let id = current_id.into();
        let mut map = HashMap::new();
        map.insert(id.clone(), zeroize::Zeroizing::new(key));
        Self {
            current_id: Mutex::new(id),
            keys: Mutex::new(map),
        }
    }

    pub fn new_random(current_id: impl Into<String>) -> Self {
        let id = current_id.into();
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        Self::new_with_secret(id, key)
    }

    pub fn set_current(&self, id: impl Into<String>, key: [u8; 32]) {
        let id = id.into();
        {
            let mut keys = self.keys.lock().expect("key map mutex");
            keys.insert(id.clone(), zeroize::Zeroizing::new(key));
        }
        *self.current_id.lock().expect("key mutex") = id;
    }

    // Back-compat for tests: rotate id with a new random key
    pub fn set_current_id(&self, id: impl Into<String>) {
        let id = id.into();
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        self.set_current(id, key);
    }
}

impl KeyManager for InMemoryKeyManager {
    fn current(&self, _scope: &KeyScope) -> Result<KeyHandle, String> {
        let id = self.current_id.lock().map_err(|e| e.to_string())?.clone();
        let keys = self.keys.lock().map_err(|e| e.to_string())?;
        let key = keys
            .get(&id)
            .ok_or_else(|| "missing key bytes for current id".to_string())?
            .clone();
        Ok(KeyHandle {
            key_id: id,
            key_bytes: key,
        })
    }

    fn get(&self, key_id: &str) -> Result<KeyHandle, String> {
        let keys = self.keys.lock().map_err(|e| e.to_string())?;
        let key = keys
            .get(key_id)
            .ok_or_else(|| "unknown key id".to_string())?
            .clone();
        Ok(KeyHandle {
            key_id: key_id.to_string(),
            key_bytes: key,
        })
    }
}
