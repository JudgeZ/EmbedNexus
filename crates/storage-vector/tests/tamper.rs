#![cfg(feature = "encryption")]

use std::sync::Arc;
use storage_vector::encryption::aes_gcm::AesGcmEncrypter;
use storage_vector::kms::InMemoryKeyManager;
use storage_vector::store::fs as vs_fs;
use storage_vector::store::{Store, VectorStore};
use storage_vector::StoreError;
use tempfile::tempdir;

#[test]
fn detects_tampered_envelope() {
    let tmp = tempdir().unwrap();
    let root = tmp.path().join("vs");
    let store = VectorStore::builder()
        .with_fs_root(&root)
        .with_encrypter(Arc::new(AesGcmEncrypter::new()))
        .with_key_manager(Arc::new(InMemoryKeyManager::new("k1")))
        .build();

    let repo = "repo-tamper";
    let key = "k";
    let payload = b"secret-payload".to_vec();

    store.upsert(repo, key, &payload).expect("upsert ok");
    // Tamper the stored envelope on disk
    let path = vs_fs::make_path(&root, repo, key);
    let mut bytes = std::fs::read(&path).unwrap();
    let last = bytes.len() - 1;
    bytes[last] ^= 0xFF;
    std::fs::write(&path, &bytes).unwrap();

    let res = store.get(repo, key);
    match res {
        Err(StoreError::Encryption(_)) => {}
        other => panic!("expected encryption error, got: {:?}", other),
    }
}
