#![cfg(feature = "encryption")]

use storage_vector::store::{Store, VectorStore};

#[test]
#[ignore = "M3 encryption implementation pending"]
fn roundtrip_encrypted_upsert_get() {
    // TODO: initialize store with test key manager and encrypter, then assert
    // that ciphertext is produced and can be read back.
    let store = VectorStore::new();
    let repo = "repo-alpha";
    let key = "vec-enc-1";
    let payload = vec![9_u8, 8, 7];
    let _entry = store.upsert(repo, key, &payload).expect("upsert ok");
    let _got = store.get(repo, key).expect("get ok").expect("present");
    todo!("seal/open to be implemented");
}

