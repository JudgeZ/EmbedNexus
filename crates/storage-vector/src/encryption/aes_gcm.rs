//! AES‑GCM encrypter for the storage‑vector crate.

use aes_gcm::{
    aead::{AeadInPlace, Tag},
    Aes256Gcm, KeyInit,
};
use rand::{rngs::OsRng, RngCore};
use sha2::{Digest, Sha256};

use super::{decode_envelope, encode_envelope, CipherSuite, Encrypter, KeyHandle};

pub struct AesGcmEncrypter;

impl AesGcmEncrypter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AesGcmEncrypter {
    fn default() -> Self {
        Self::new()
    }
}

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
    fn algorithm(&self) -> CipherSuite {
        CipherSuite::AesGcm256
    }

    fn seal(&self, key: &KeyHandle, plaintext: &[u8], aad: &[u8]) -> Result<Vec<u8>, String> {
        let key_bytes = derive_key(&key.key_id);
        let cipher = Aes256Gcm::new_from_slice(&key_bytes).map_err(|e| e.to_string())?;
        let mut nonce = [0u8; 12];
        OsRng.fill_bytes(&mut nonce);
        // Encrypt in-place to obtain a detached tag we can store in the envelope.
        let mut buf = plaintext.to_vec();
        let tag = cipher
            .encrypt_in_place_detached(aes_gcm::Nonce::from_slice(&nonce), aad, &mut buf)
            .map_err(|e| e.to_string())?;
        let mut tag_bytes = [0u8; 16];
        tag_bytes.copy_from_slice(<Tag<Aes256Gcm> as AsRef<[u8; 16]>>::as_ref(&tag));
        Ok(encode_envelope(&key.key_id, &nonce, &tag_bytes, &buf))
    }

    fn open(&self, key: &KeyHandle, envelope_bytes: &[u8], aad: &[u8]) -> Result<Vec<u8>, String> {
        let (kid, nonce, tag, ct) = decode_envelope(envelope_bytes)?;
        if kid != key.key_id {
            return Err("key id mismatch".into());
        }
        let key_bytes = derive_key(&key.key_id);
        let cipher = Aes256Gcm::new_from_slice(&key_bytes).map_err(|e| e.to_string())?;
        let mut buf = ct.clone();
        let tag_ref = Tag::<Aes256Gcm>::from_slice(&tag);
        cipher
            .decrypt_in_place_detached(aes_gcm::Nonce::from_slice(&nonce), aad, &mut buf, tag_ref)
            .map_err(|e| e.to_string())?;
        Ok(buf)
    }
}
