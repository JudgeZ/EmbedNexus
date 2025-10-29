use std::fs;
use storage_vector::store::{Store, VectorStore};

#[test]
fn fs_roundtrip_plaintext() {
    let tmpdir = tempfile::tempdir().expect("tmpdir");
    let root = tmpdir.path().join("vs");
    fs::create_dir_all(&root).unwrap();
    let store = VectorStore::with_fs_root(&root);

    let repo = "repo-fs";
    let key = "k1";
    let payload = b"fs-payload".to_vec();

    let entry = store.upsert(repo, key, &payload).unwrap();
    assert!(entry.sequence > 0);

    // File should exist
    let path = storage_vector::store::fs::make_path(&root, repo, key);
    assert!(path.exists());

    let got = store.get(repo, key).unwrap().unwrap();
    assert_eq!(got, payload);
}
