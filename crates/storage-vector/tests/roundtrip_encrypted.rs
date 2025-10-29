#![cfg(feature = "encryption")]

use std::sync::Arc;
use storage_vector::encryption::aes_gcm::AesGcmEncrypter;
use storage_vector::kms::InMemoryKeyManager;
use storage_vector::store::{Store, VectorStore};

#[test]
fn roundtrip_encrypted_upsert_get() {
    let store = VectorStore::builder()
        .with_encrypter(Arc::new(AesGcmEncrypter::new()))
        .with_key_manager(Arc::new(InMemoryKeyManager::new_with_secret(
            "k1", [7u8; 32],
        )))
        .build();

    let repo = "repo-alpha";
    let key = "vec-enc-1";
    let payload = vec![9_u8, 8, 7, 6, 5];

    let entry = store.upsert(repo, key, &payload).expect("upsert ok");
    assert!(entry.sequence > 0);

    let got = store.get(repo, key).expect("get ok").expect("present");
    assert_eq!(got, payload);
}
