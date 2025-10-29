use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crate::error::StoreError;
use crate::ledger::build_replay_entry;
use storage_ledger::ReplayEntry;
// Aliases to reduce clippy::type_complexity noise without changing behavior
type RepoKey = (String, String);
type Blob = Vec<u8>;
pub mod fs;
/// Build AEAD associated data binding: (repo_id, key_id, record_key).
/// Encoding: u16 be repo_len | repo_bytes | u16 be key_id_len | key_id_bytes | u16 be record_key_len | record_key_bytes.
pub fn build_aad(repo_id: &str, key_id: &str, record_key: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(2 + repo_id.len() + 2 + key_id.len() + 2 + record_key.len());
    let rlen = repo_id.len() as u16;
    out.extend_from_slice(&rlen.to_be_bytes());
    out.extend_from_slice(repo_id.as_bytes());
    let klen = key_id.len() as u16;
    out.extend_from_slice(&klen.to_be_bytes());
    out.extend_from_slice(key_id.as_bytes());
    let slen = record_key.len() as u16;
    out.extend_from_slice(&slen.to_be_bytes());
    out.extend_from_slice(record_key.as_bytes());
    out
}

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
    fn replay<I: IntoIterator<Item = ReplayEntry>>(
        &self,
        entries: I,
    ) -> Result<ReplayStats, StoreError>;
}

/// An in-memory store stub to enable TDD. Future work may add FS-backed shards.
pub struct VectorStore {
    inner: Arc<Mutex<HashMap<RepoKey, Blob>>>,
    next_sequence: AtomicU64,
    fs_root: Option<PathBuf>,
    #[cfg(feature = "encryption")]
    encrypter: Option<Arc<dyn crate::encryption::Encrypter + Send + Sync>>,
    #[cfg(feature = "encryption")]
    kms: Option<Arc<dyn crate::kms::KeyManager + Send + Sync>>,
}

impl Default for VectorStore {
    fn default() -> Self {
        Self::new()
    }
}

impl VectorStore {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
            next_sequence: AtomicU64::new(1),
            fs_root: None,
            #[cfg(feature = "encryption")]
            encrypter: None,
            #[cfg(feature = "encryption")]
            kms: None,
        }
    }

    fn checksum_placeholder(bytes: &[u8]) -> String {
        // Keep cheap and deterministic to avoid pulling hashing deps in the skeleton.
        format!("len:{}", bytes.len())
    }

    /// Create a filesystem-backed store using the given root directory.
    pub fn with_fs_root(root: impl Into<PathBuf>) -> Self {
        let mut s = Self::new();
        s.fs_root = Some(root.into());
        s
    }

    #[cfg(feature = "encryption")]
    pub fn builder() -> VectorStoreBuilder {
        VectorStoreBuilder::default()
    }
}

impl Store for VectorStore {
    fn upsert(&self, repo_id: &str, key: &str, payload: &[u8]) -> Result<ReplayEntry, StoreError> {
        let before = Self::checksum_placeholder(payload);
        #[cfg(feature = "encryption")]
        if let (Some(enc), Some(kms)) = (&self.encrypter, &self.kms) {
            let scope = crate::kms::KeyScope {
                repo_id: repo_id.to_string(),
            };
            let kh = kms.current(&scope).map_err(StoreError::Key)?;
            let aad = build_aad(repo_id, &kh.key_id, key);
            let sealed = enc
                .seal(&kh, payload, &aad)
                .map_err(StoreError::Encryption)?;
            if let Some(root) = &self.fs_root {
                fs::atomic_write_bytes(root, repo_id, key, &sealed)
                    .map_err(|e| StoreError::Io(e.to_string()))?;
            } else {
                let mut guard = self
                    .inner
                    .lock()
                    .map_err(|e| StoreError::Io(e.to_string()))?;
                guard.insert((repo_id.to_string(), key.to_string()), sealed);
            }
            let after = Self::checksum_placeholder(payload);
            let seq = self.next_sequence.fetch_add(1, Ordering::SeqCst);
            return Ok(build_replay_entry(seq, repo_id, &before, &after, "emitted"));
        }
        // Plaintext path
        if let Some(root) = &self.fs_root {
            fs::atomic_write_bytes(root, repo_id, key, payload)
                .map_err(|e| StoreError::Io(e.to_string()))?;
        } else {
            let mut guard = self
                .inner
                .lock()
                .map_err(|e| StoreError::Io(e.to_string()))?;
            guard.insert((repo_id.to_string(), key.to_string()), payload.to_vec());
        }
        let after = before.clone();
        let seq = self.next_sequence.fetch_add(1, Ordering::SeqCst);
        Ok(build_replay_entry(seq, repo_id, &before, &after, "emitted"))
    }

    fn get(&self, repo_id: &str, key: &str) -> Result<Option<Vec<u8>>, StoreError> {
        // Prefer filesystem when configured, otherwise in-memory map.
        let bytes = if let Some(root) = &self.fs_root {
            match fs::read_bytes(root, repo_id, key) {
                Ok(b) => b,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    // fallback to memory if present
                    let guard = self
                        .inner
                        .lock()
                        .map_err(|e| StoreError::Io(e.to_string()))?;
                    match guard.get(&(repo_id.to_string(), key.to_string())) {
                        Some(b) => b.clone(),
                        None => return Ok(None),
                    }
                }
                Err(e) => return Err(StoreError::Io(e.to_string())),
            }
        } else {
            let guard = self
                .inner
                .lock()
                .map_err(|e| StoreError::Io(e.to_string()))?;
            match guard.get(&(repo_id.to_string(), key.to_string())) {
                Some(b) => b.clone(),
                None => return Ok(None),
            }
        };
        #[cfg(feature = "encryption")]
        if let (Some(enc), Some(kms)) = (&self.encrypter, &self.kms) {
            if let Some(kid) = crate::encryption::peek_key_id(&bytes) {
                let kh = kms.get(&kid).map_err(StoreError::Key)?;
                let aad = build_aad(repo_id, &kh.key_id, key);
                let pt = enc
                    .open(&kh, &bytes, &aad)
                    .map_err(StoreError::Encryption)?;
                return Ok(Some(pt));
            } else {
                // Encryption is configured but the stored bytes are not a valid envelope.
                // Treat as corruption/tampering rather than passing plaintext through.
                return Err(StoreError::Encryption(
                    "missing or invalid envelope".to_string(),
                ));
            }
        }
        Ok(Some(bytes))
    }

    fn replay<I: IntoIterator<Item = ReplayEntry>>(
        &self,
        entries: I,
    ) -> Result<ReplayStats, StoreError> {
        // Minimal semantics: update max_sequence and count entries; payload restoration to be added later.
        let mut stats = ReplayStats::default();
        let mut max_seq: Option<u64> = None;
        for entry in entries {
            stats.applied += 1;
            max_seq = Some(
                max_seq
                    .map(|m| m.max(entry.sequence))
                    .unwrap_or(entry.sequence),
            );
        }
        if let Some(m) = max_seq {
            // Advance floor if replay moved sequence forward
            let mut current = self.next_sequence.load(Ordering::SeqCst);
            while m + 1 > current {
                match self.next_sequence.compare_exchange(
                    current,
                    m + 1,
                    Ordering::SeqCst,
                    Ordering::SeqCst,
                ) {
                    Ok(_) => break,
                    Err(now) => current = now,
                }
            }
        }
        stats.max_sequence = max_seq;
        Ok(stats)
    }
}

#[cfg(test)]
impl VectorStore {
    /// Test-only helper: flip the last byte of the stored value for (repo_id, key).
    /// Returns true on success.
    pub fn tamper_flip_last_byte(&self, repo_id: &str, key: &str) -> bool {
        if let Ok(mut guard) = self.inner.lock() {
            if let Some(v) = guard.get_mut(&(repo_id.to_string(), key.to_string())) {
                if let Some(last) = v.last_mut() {
                    *last ^= 0xFF;
                    return true;
                }
            }
        }
        false
    }

    /// Test-only helper: return raw stored bytes for (repo_id, key), from FS or memory.
    pub fn test_dump_raw(&self, repo_id: &str, key: &str) -> Option<Vec<u8>> {
        if let Some(root) = &self.fs_root {
            if let Ok(b) = fs::read_bytes(root, repo_id, key) {
                return Some(b);
            }
        }
        if let Ok(guard) = self.inner.lock() {
            return guard.get(&(repo_id.to_string(), key.to_string())).cloned();
        }
        None
    }
}

#[cfg(feature = "encryption")]
#[derive(Default)]
pub struct VectorStoreBuilder {
    encrypter: Option<Arc<dyn crate::encryption::Encrypter + Send + Sync>>,
    kms: Option<Arc<dyn crate::kms::KeyManager + Send + Sync>>,
    fs_root: Option<PathBuf>,
}

#[cfg(feature = "encryption")]
impl VectorStoreBuilder {
    pub fn with_encrypter(
        mut self,
        e: Arc<dyn crate::encryption::Encrypter + Send + Sync>,
    ) -> Self {
        self.encrypter = Some(e);
        self
    }
    pub fn with_key_manager(mut self, k: Arc<dyn crate::kms::KeyManager + Send + Sync>) -> Self {
        self.kms = Some(k);
        self
    }
    pub fn with_fs_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.fs_root = Some(root.into());
        self
    }
    pub fn build(self) -> VectorStore {
        VectorStore {
            inner: Arc::new(Mutex::new(HashMap::new())),
            next_sequence: AtomicU64::new(1),
            fs_root: self.fs_root,
            encrypter: self.encrypter,
            kms: self.kms,
        }
    }
}
