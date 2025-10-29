#![cfg(feature = "encryption")]

use std::sync::Arc;
use storage_vector::encryption::aes_gcm::AesGcmEncrypter;
use storage_vector::kms::InMemoryKeyManager;
use storage_vector::store::{Store, VectorStore};
use storage_vector::StoreError;

#[test]
fn detects_tampered_envelope() {
    let store = VectorStore::builder()
        .with_encrypter(Arc::new(AesGcmEncrypter::new()))
        .with_key_manager(Arc::new(InMemoryKeyManager::new("k1")))
        .build();

    let repo = "repo-tamper";
    let key = "k";
    let payload = b"secret-payload".to_vec();

    store.upsert(repo, key, &payload).expect("upsert ok");
    assert!(store.tamper_flip_last_byte(repo, key));

    let res = store.get(repo, key);
    match res {
        Err(StoreError::Encryption(_)) => {},
        other => panic!("expected encryption error, got: {:?}", other),
    }
}
