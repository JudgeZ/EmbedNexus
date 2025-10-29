#![cfg(feature = "encryption")]

use std::sync::Arc;
use storage_vector::encryption::{aes_gcm::AesGcmEncrypter, KeyHandle, Encrypter, peek_key_id};
use storage_vector::kms::InMemoryKeyManager;
use storage_vector::store::{Store, VectorStore};

#[test]
fn aad_mismatch_fails_decrypt() {
    let kms = Arc::new(InMemoryKeyManager::new("k1"));
    let store = VectorStore::builder()
        .with_encrypter(Arc::new(AesGcmEncrypter::new()))
        .with_key_manager(kms.clone())
        .build();

    let repo = "repo-a";
    let key = "k";
    let payload = b"hello".to_vec();
    store.upsert(repo, key, &payload).expect("upsert");

    // Dump raw envelope bytes and attempt open() with mismatched AAD
    let raw = store.test_dump_raw(repo, key).expect("raw present");
    let kid = peek_key_id(&raw).expect("kid");
    let kh = KeyHandle { key_id: kid };
    let enc = AesGcmEncrypter::new();
    // Wrong repo id in AAD should fail
    let aad = b"repo-b:k1".to_vec();
    let res = enc.open(&kh, &raw, &aad);
    assert!(res.is_err());
}

