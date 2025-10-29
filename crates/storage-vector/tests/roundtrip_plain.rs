use storage_vector::store::{Store, VectorStore};

#[test]
fn roundtrip_plaintext_upsert_get() {
    let store = VectorStore::new();
    let repo = "repo-alpha";
    let key = "vec-1";
    let payload = vec![1_u8, 2, 3, 4];

    let entry = store.upsert(repo, key, &payload).expect("upsert ok");
    assert_eq!(entry.repo_id, repo);
    assert!(entry.sequence > 0);

    let got = store.get(repo, key).expect("get ok").expect("present");
    assert_eq!(got, payload);
}
