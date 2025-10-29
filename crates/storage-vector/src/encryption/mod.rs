//! Feature-gated encryption interfaces for the vector store.
//! This module provides trait definitions and minimal stubs so tests can
//! compile. Concrete algorithms land incrementally during M3.

#[cfg(feature = "encryption")]
pub mod aes_gcm;

#[derive(Debug, Clone, Copy)]
pub enum CipherSuite {
    AesGcm256,
    #[allow(dead_code)]
    ChaCha20Poly1305,
}

#[derive(Debug, Clone)]
pub struct KeyHandle {
    pub key_id: String,
}

pub trait Encrypter: Send + Sync {
    fn algorithm(&self) -> CipherSuite;
    fn seal(&self, _key: &KeyHandle, _plaintext: &[u8], _aad: &[u8]) -> Result<Vec<u8>, String>;
    fn open(&self, _key: &KeyHandle, _ciphertext: &[u8], _aad: &[u8]) -> Result<Vec<u8>, String>;
}

