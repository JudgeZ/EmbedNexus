//! AES‑GCM encrypter for the storage‑vector crate.

use aes_gcm::{
    aead::{AeadInPlace, Tag},
    Aes256Gcm, KeyInit,
};
use rand::{rngs::OsRng, RngCore};

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

impl Encrypter for AesGcmEncrypter {
    fn algorithm(&self) -> CipherSuite {
        CipherSuite::AesGcm256
    }

    fn seal(&self, key: &KeyHandle, plaintext: &[u8], aad: &[u8]) -> Result<Vec<u8>, String> {
        let cipher =
            Aes256Gcm::new_from_slice(key.key_bytes.as_ref()).map_err(|e| e.to_string())?;
        let mut nonce = [0u8; 12];
        OsRng.fill_bytes(&mut nonce);
        // Encrypt in-place to obtain a detached tag we can store in the envelope.
        let mut buf = plaintext.to_vec();
        #[allow(deprecated)]
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
        let cipher =
            Aes256Gcm::new_from_slice(key.key_bytes.as_ref()).map_err(|e| e.to_string())?;
        let mut buf = ct.clone();
        #[allow(deprecated)]
        let tag_ref = Tag::<Aes256Gcm>::from_slice(&tag);
        #[allow(deprecated)]
        cipher
            .decrypt_in_place_detached(aes_gcm::Nonce::from_slice(&nonce), aad, &mut buf, tag_ref)
            .map_err(|e| e.to_string())?;
        Ok(buf)
    }
}
