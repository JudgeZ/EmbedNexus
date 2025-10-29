//! AES‑GCM encrypter for the storage‑vector crate.

use aes_gcm::{Aes256Gcm, KeyInit, aead::{Aead, Payload}};
use rand::{rngs::OsRng, RngCore};
use sha2::{Digest, Sha256};

use super::{CipherSuite, Encrypter, KeyHandle, encode_envelope, decode_envelope};

pub struct AesGcmEncrypter;

impl AesGcmEncrypter { pub fn new() -> Self { Self } }

fn derive_key(key_id: &str) -> [u8; 32] {
    // Deterministic 32‑byte key from key_id (test‑only derivation for M3).
    let mut h = Sha256::new();
    h.update(key_id.as_bytes());
    let out = h.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&out);
    key
}

impl Encrypter for AesGcmEncrypter {
    fn algorithm(&self) -> CipherSuite { CipherSuite::AesGcm256 }

    fn seal(&self, key: &KeyHandle, plaintext: &[u8], aad: &[u8]) -> Result<Vec<u8>, String> {
        let key_bytes = derive_key(&key.key_id);
        let cipher = Aes256Gcm::new_from_slice(&key_bytes).map_err(|e| e.to_string())?;
        let mut nonce = [0u8; 12];
        OsRng.fill_bytes(&mut nonce);
        let ct = cipher
            .encrypt(aes_gcm::Nonce::from_slice(&nonce), Payload { msg: plaintext, aad })
            .map_err(|e| e.to_string())?;
        // Last 16 bytes are tag in AES‑GCM's output? In aes-gcm crate, tag is appended implicitly?
        // The encrypt() output is ciphertext without explicit tag separation; tag is authenticated inside.
        // We store entire ct as envelope payload and set a zero tag placeholder to keep envelope structure simple.
        // For clarity in M3, we don't split tag; aes-gcm manages it internally.
        let tag = [0u8; 16];
        Ok(encode_envelope(&key.key_id, &nonce, &tag, &ct))
    }

    fn open(&self, key: &KeyHandle, envelope_bytes: &[u8], aad: &[u8]) -> Result<Vec<u8>, String> {
        let (kid, nonce, _tag, ct) = decode_envelope(envelope_bytes)?;
        if kid != key.key_id { return Err("key id mismatch".into()); }
        let key_bytes = derive_key(&key.key_id);
        let cipher = Aes256Gcm::new_from_slice(&key_bytes).map_err(|e| e.to_string())?;
        cipher
            .decrypt(aes_gcm::Nonce::from_slice(&nonce), Payload { msg: &ct, aad })
            .map_err(|e| e.to_string())
    }
}
