use runtime_transport_stdio::{RetryBuffer, RetryPayload};
use serde::Deserialize;
use serde_json::Value;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

#[derive(Debug, Deserialize)]
struct SnapshotRecord {
    sequence: u64,
    command: String,
    payload: Value,
    token_id: String,
    #[serde(rename = "enqueued_at")]
    _enqueued_at: i64,
}

fn load_snapshot() -> Vec<SnapshotRecord> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("fixtures")
        .join("transport")
        .join("offline-queue")
        .join("snapshot.jsonl");
    let reader = BufReader::new(File::open(path).expect("snapshot available"));
    reader
        .lines()
        .map(|line| line.expect("line"))
        .map(|line| serde_json::from_str::<SnapshotRecord>(&line).expect("valid snapshot entry"))
        .collect()
}

#[test]
fn retry_buffer_rehydrates_snapshot_fifo() {
    let snapshot = load_snapshot();
    let buffer = RetryBuffer::new(16, Duration::from_secs(86_400));

    for (idx, record) in snapshot.iter().enumerate() {
        let offset = (snapshot.len() - idx) as u64;
        buffer
            .enqueue_at(
                RetryPayload {
                    sequence: record.sequence,
                    command: record.command.clone(),
                    payload: record.payload.clone(),
                    token_id: record.token_id.clone(),
                },
                SystemTime::now() - Duration::from_secs(offset),
            )
            .unwrap();
    }

    assert_eq!(buffer.max_sequence(), Some(3));

    let drained = buffer.drain_ready();
    let sequences: Vec<u64> = drained
        .iter()
        .map(|entry| entry.payload().sequence)
        .collect();
    assert_eq!(sequences, vec![1, 2, 3]);
}

#[test]
fn retry_buffer_preserves_sequence_on_requeue_after_snapshot() {
    let snapshot = load_snapshot();
    let buffer = RetryBuffer::new(4, Duration::from_secs(86_400));

    for (idx, record) in snapshot.iter().enumerate() {
        let offset = (snapshot.len() - idx) as u64;
        buffer
            .enqueue_at(
                RetryPayload {
                    sequence: record.sequence,
                    command: record.command.clone(),
                    payload: record.payload.clone(),
                    token_id: record.token_id.clone(),
                },
                SystemTime::now() - Duration::from_secs(offset),
            )
            .unwrap();
    }

    let mut drained = buffer.drain_ready();
    assert_eq!(drained.len(), 3);
    let retry_entry = drained.remove(0);
    buffer.requeue(retry_entry).unwrap();

    buffer
        .enqueue(RetryPayload {
            sequence: 4,
            command: "ingest".into(),
            payload: serde_json::json!({ "doc": 4 }),
            token_id: "offline-4".into(),
        })
        .unwrap();

    let drained_again = buffer.drain_ready();
    let sequences: Vec<u64> = drained_again
        .iter()
        .map(|entry| entry.payload().sequence)
        .collect();
    assert_eq!(sequences, vec![1, 4]);
}
