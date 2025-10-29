use storage_ledger::ReplayEntry;
use storage_vector::store::{Store, VectorStore};

#[test]
fn replay_applies_in_order_and_advances_sequence() {
    let store = VectorStore::new();
    let repo = "repo-theta";

    // Write a couple entries to bump sequence
    let _ = store.upsert(repo, "k1", b"a").unwrap();
    let _ = store.upsert(repo, "k2", b"b").unwrap();

    // Construct unordered replay entries with sequences higher than current
    let entries = vec![
        ReplayEntry {
            sequence: 10,
            repo_id: repo.into(),
            delayed_ms: 0,
            payload_checksum_before: "x".into(),
            payload_checksum_after: "x".into(),
            status: "emitted".into(),
        },
        ReplayEntry {
            sequence: 8,
            repo_id: repo.into(),
            delayed_ms: 0,
            payload_checksum_before: "y".into(),
            payload_checksum_after: "y".into(),
            status: "emitted".into(),
        },
        ReplayEntry {
            sequence: 9,
            repo_id: repo.into(),
            delayed_ms: 0,
            payload_checksum_before: "z".into(),
            payload_checksum_after: "z".into(),
            status: "emitted".into(),
        },
    ];

    let stats = store.replay(entries).expect("replay ok");
    assert_eq!(stats.applied, 3);
    assert_eq!(stats.max_sequence, Some(10));

    // Subsequent upsert should pick up at 11
    let entry = store.upsert(repo, "k3", b"c").unwrap();
    assert_eq!(entry.sequence, 11);
}
