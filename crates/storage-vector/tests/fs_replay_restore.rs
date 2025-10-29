use storage_vector::store::{Store, VectorStore};

#[test]
fn fs_replay_restores_payloads_from_disk_and_advances_sequence() {
    let tmpdir = tempfile::tempdir().expect("tmpdir");
    let root = tmpdir.path().join("vs");
    let store = VectorStore::with_fs_root(&root);

    let repo = "repo-restore";
    let p1 = b"alpha".to_vec();
    let p2 = b"beta".to_vec();

    let e1 = store.upsert(repo, "a", &p1).unwrap();
    let e2 = store.upsert(repo, "b", &p2).unwrap();

    // New instance with same root (simulates fresh process)
    let store2 = VectorStore::with_fs_root(&root);
    // Replay to restore sequence floor
    let stats = store2.replay(vec![e2.clone(), e1.clone()]).unwrap();
    assert_eq!(stats.max_sequence, Some(e2.sequence));

    // Payloads should already be readable from disk
    assert_eq!(store2.get(repo, "a").unwrap().unwrap(), p1);
    assert_eq!(store2.get(repo, "b").unwrap().unwrap(), p2);

    // Next write should continue at max+1
    let e3 = store2.upsert(repo, "c", b"gamma").unwrap();
    assert_eq!(e3.sequence, e2.sequence + 1);
}

