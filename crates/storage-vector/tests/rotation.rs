#![cfg(feature = "encryption")]

use std::sync::Arc;
use storage_vector::encryption::aes_gcm::AesGcmEncrypter;
use storage_vector::kms::InMemoryKeyManager;
use storage_vector::store::{Store, VectorStore};

#[test]
fn rotates_keys_and_reads_old_records() {
    let kms = Arc::new(InMemoryKeyManager::new_with_secret("k1", [1u8; 32]));
    let store = VectorStore::builder()
        .with_encrypter(Arc::new(AesGcmEncrypter::new()))
        .with_key_manager(kms.clone())
        .build();

    let repo = "repo-rot";

    // Write under K1
    let p1 = b"alpha".to_vec();
    store.upsert(repo, "a", &p1).expect("upsert a");

    // Rotate to K2
    kms.set_current("k2", [2u8; 32]);

    // Write under K2
    let p2 = b"beta".to_vec();
    store.upsert(repo, "b", &p2).expect("upsert b");

    // Both records should be readable
    assert_eq!(store.get(repo, "a").unwrap().unwrap(), p1);
    assert_eq!(store.get(repo, "b").unwrap().unwrap(), p2);
}
