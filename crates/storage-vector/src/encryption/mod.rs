//! Feature-gated encryption interfaces for the vector store.
//! Provides trait definitions and small helpers for encoding/decoding
//! minimal envelopes used by AES‑GCM.

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
    /// Seal plaintext to an encoded envelope (includes key id, nonce, tag, ct).
    fn seal(&self, key: &KeyHandle, plaintext: &[u8], aad: &[u8]) -> Result<Vec<u8>, String>;
    /// Open an encoded envelope (bytes) and return plaintext.
    fn open(&self, key: &KeyHandle, envelope_bytes: &[u8], aad: &[u8]) -> Result<Vec<u8>, String>;
}

// Simple envelope layout helpers for AES‑GCM encoded payloads.
// Format (little): b"EVG1" | u16 key_id_len | key_id | 12-byte nonce | 16-byte tag | ct...

pub const ENVELOPE_MAGIC: &[u8; 4] = b"EVG1";

pub fn encode_envelope(key_id: &str, nonce: &[u8; 12], tag: &[u8; 16], ct: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(4 + 2 + key_id.len() + 12 + 16 + ct.len());
    out.extend_from_slice(ENVELOPE_MAGIC);
    let len: u16 = key_id.len() as u16;
    out.extend_from_slice(&len.to_be_bytes());
    out.extend_from_slice(key_id.as_bytes());
    out.extend_from_slice(nonce);
    out.extend_from_slice(tag);
    out.extend_from_slice(ct);
    out
}

#[allow(clippy::type_complexity)]
pub fn decode_envelope(bytes: &[u8]) -> Result<(String, [u8; 12], [u8; 16], Vec<u8>), String> {
    if bytes.len() < 4 + 2 + 12 + 16 {
        return Err("envelope too short".into());
    }
    if &bytes[0..4] != ENVELOPE_MAGIC {
        return Err("bad envelope magic".into());
    }
    let key_len = u16::from_be_bytes([bytes[4], bytes[5]]) as usize;
    let min = 4 + 2 + key_len + 12 + 16;
    if bytes.len() < min {
        return Err("envelope truncated".into());
    }
    let key_start = 6;
    let key_end = key_start + key_len;
    let key_id = std::str::from_utf8(&bytes[key_start..key_end])
        .map_err(|_| "key id not utf8")?
        .to_string();
    let mut nonce = [0u8; 12];
    nonce.copy_from_slice(&bytes[key_end..key_end + 12]);
    let mut tag = [0u8; 16];
    tag.copy_from_slice(&bytes[key_end + 12..key_end + 12 + 16]);
    let ct = bytes[key_end + 12 + 16..].to_vec();
    Ok((key_id, nonce, tag, ct))
}

pub fn peek_key_id(bytes: &[u8]) -> Option<String> {
    if bytes.len() < 6 {
        return None;
    }
    if &bytes[0..4] != ENVELOPE_MAGIC {
        return None;
    }
    let key_len = u16::from_be_bytes([bytes[4], bytes[5]]) as usize;
    if bytes.len() < 6 + key_len {
        return None;
    }
    std::str::from_utf8(&bytes[6..6 + key_len])
        .ok()
        .map(|s| s.to_string())
}
