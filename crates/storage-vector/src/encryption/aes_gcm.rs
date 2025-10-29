//! AES-GCM encrypter placeholder. Implementation will land in subsequent M3 steps.

use super::{CipherSuite, Encrypter, KeyHandle};

pub struct AesGcmEncrypter;

impl AesGcmEncrypter {
    pub fn new() -> Self { Self }
}

impl Encrypter for AesGcmEncrypter {
    fn algorithm(&self) -> CipherSuite { CipherSuite::AesGcm256 }

    fn seal(&self, _key: &KeyHandle, _plaintext: &[u8], _aad: &[u8]) -> Result<Vec<u8>, String> {
        // Skeleton: return plaintext for now; integrity plumbing will be added.
        Ok(Vec::new())
    }

    fn open(&self, _key: &KeyHandle, _ciphertext: &[u8], _aad: &[u8]) -> Result<Vec<u8>, String> {
        Ok(Vec::new())
    }
}

