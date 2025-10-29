#![cfg(feature = "encryption")]
use std::sync::Arc;

use storage_vector::encryption::aes_gcm::AesGcmEncrypter;
use storage_vector::kms::InMemoryKeyManager;
use storage_vector::store::{fs as vs_fs, Store, VectorStore};
use tempfile::tempdir;

#[test]
fn invalid_envelope_is_error_when_encryption_enabled() {
    let tmp = tempdir().unwrap();
    let root = tmp.path().join("vs");
    let repo = "repo-a";
    let key = "k";
    let payload = b"hello world";

    let store = VectorStore::builder()
        .with_fs_root(&root)
        .with_encrypter(Arc::new(AesGcmEncrypter::new()))
        .with_key_manager(Arc::new(InMemoryKeyManager::new_with_secret(
            "k1", [7u8; 32],
        )))
        .build();

    // Write an encrypted record
    let _ = store.upsert(repo, key, payload).expect("upsert ok");

    // Tamper the magic so peek_key_id() no longer recognizes the envelope
    let mut raw = vs_fs::read_bytes(&root, repo, key).expect("raw present");
    assert!(!raw.is_empty());
    raw[0] ^= 0xFF; // flip first byte of magic 'EVG1'
    vs_fs::atomic_write_bytes(&root, repo, key, &raw).expect("write tampered");

    // Now a get() must return an encryption error rather than raw bytes
    let err = store.get(repo, key).unwrap_err();
    match err {
        storage_vector::StoreError::Encryption(_) => {}
        other => panic!("expected encryption error, got {other:?}"),
    }
}
