use std::{
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, SystemTime},
};

use ingestion_embedding::{EmbeddingConfig, EmbeddingGenerator};
use ingestion_manifest::{ManifestDiff, ManifestEmitter, ManifestEmitterConfig, ManifestQueue};
use ingestion_planning::{ChunkPlan, PlannedChunk, RetryPolicy};
use ingestion_sanitization::{SanitizationConfig, Sanitizer};
use storage_ledger::{OfflineReplayBuffer, ReplayEntry};

#[derive(Default)]
struct TestQueue {
    inner: Mutex<Vec<ReplayEntry>>,
    fail: Mutex<bool>,
}

impl TestQueue {
    fn push(&self, entry: ReplayEntry) -> anyhow::Result<()> {
        if *self.fail.lock().unwrap() {
            anyhow::bail!("queue offline");
        }
        self.inner.lock().unwrap().push(entry);
        Ok(())
    }

    fn collected(&self) -> Vec<ReplayEntry> {
        self.inner.lock().unwrap().clone()
    }
}

impl ManifestQueue for TestQueue {
    fn send(&self, entry: ReplayEntry) -> anyhow::Result<()> {
        self.push(entry)
    }
}

fn load_delayed_entries() -> Vec<ReplayEntry> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("fixtures")
        .join("ingestion")
        .join("delayed-ledger")
        .join("placeholder.jsonl");
    fs::read_to_string(path)
        .expect("missing delayed-ledger fixture")
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str::<ReplayEntry>(line).expect("invalid replay entry"))
        .collect()
}

fn load_manifest_golden() -> (u64, String, String) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("golden")
        .join("ingestion")
        .join("manifest-replay.log");
    let content = fs::read_to_string(path).expect("missing manifest replay golden");
    let mut sequence = 0;
    let mut checksum_before = String::new();
    let mut checksum_after = String::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("sequence:") {
            sequence = rest.trim().parse().unwrap();
        } else if let Some(rest) = trimmed.strip_prefix("checksum_before:") {
            checksum_before = rest.trim().to_string();
        } else if let Some(rest) = trimmed.strip_prefix("checksum_after:") {
            checksum_after = rest.trim().to_string();
        }
    }
    (sequence, checksum_before, checksum_after)
}

fn sanitized_payload() -> ingestion_sanitization::SanitizedChunk {
    let plan = ChunkPlan {
        plan_id: "repo-theta::docs/spec.md::0".into(),
        repo_id: "repo-theta".into(),
        chunker_config: "size=512".into(),
        source_span: "docs/spec.md:1-200".into(),
        hash: "aa55aa".into(),
        retry_policy: RetryPolicy::default(),
    };
    let planned = PlannedChunk::new(plan, "# Spec\nSECRET token");
    let sanitizer = Sanitizer::new(SanitizationConfig::default());
    sanitizer
        .apply(&planned)
        .expect("sanitization should succeed")
}

#[test]
fn emitter_flushes_offline_buffer_against_golden() {
    let (expected_sequence, checksum_before, checksum_after) = load_manifest_golden();
    let replay_entries = load_delayed_entries();
    let buffer = OfflineReplayBuffer::new(128, Duration::from_millis(120_000));
    for entry in replay_entries {
        buffer.push(entry).unwrap();
    }
    let queue = Arc::new(TestQueue::default());
    *queue.fail.lock().unwrap() = true;

    let config = ManifestEmitterConfig {
        sequence_start: expected_sequence - 2,
        encryption_key: "test-key".into(),
        retention_max_entries: 128,
        retention_max_age: Duration::from_millis(120_000),
    };
    let generator = EmbeddingGenerator::new(EmbeddingConfig::new("encoder-z".into(), 6));
    let sanitized = sanitized_payload();
    let batch = generator
        .encode(&[sanitized])
        .expect("encoding should succeed");
    let mut emitter = ManifestEmitter::new(config, buffer.clone(), queue.clone());

    let diff = ManifestDiff {
        repo_id: "repo-theta".into(),
        applied_at: SystemTime::now(),
        added_chunks: batch
            .vectors
            .iter()
            .enumerate()
            .map(|(idx, _)| format!("chunk-{idx}"))
            .collect(),
        removed_chunks: vec![],
        checksum_before: checksum_before.clone(),
        checksum_after: checksum_after.clone(),
    };

    // Emit while queue is offline, forcing a buffer write.
    emitter
        .emit(diff.clone(), batch.clone())
        .expect_err("queue offline should error");

    // Bring the queue back online and flush.
    *queue.fail.lock().unwrap() = false;
    emitter.flush_offline().expect("flush should succeed");

    let collected = queue.collected();
    assert!(!collected.is_empty());
    let golden_entry = collected
        .iter()
        .find(|entry| entry.sequence == expected_sequence)
        .expect("expected golden replay entry");
    assert_eq!(golden_entry.payload_checksum_before, checksum_before);
    assert_eq!(golden_entry.payload_checksum_after, checksum_after);
    assert!(collected
        .iter()
        .any(|entry| entry.sequence > expected_sequence));
}

#[test]
fn emitter_respects_configured_sequence_floor_over_buffered_entries() {
    let queue = Arc::new(TestQueue::default());
    let buffer = OfflineReplayBuffer::new(8, Duration::from_secs(60));

    for sequence in [4_u64, 5_u64, 6_u64] {
        buffer
            .push(ReplayEntry {
                sequence,
                repo_id: format!("repo-theta-{sequence}"),
                delayed_ms: 0,
                payload_checksum_before: format!("before-{sequence}"),
                payload_checksum_after: format!("after-{sequence}"),
                status: "buffered".into(),
            })
            .expect("buffer push should succeed");
    }

    let config = ManifestEmitterConfig {
        sequence_start: 42,
        encryption_key: "test-key".into(),
        retention_max_entries: 8,
        retention_max_age: Duration::from_secs(60),
    };

    let generator = EmbeddingGenerator::new(EmbeddingConfig::new("encoder-z".into(), 6));
    let mut emitter = ManifestEmitter::new(config, buffer, queue.clone());

    let diff = ManifestDiff {
        repo_id: "repo-floor".into(),
        applied_at: SystemTime::now(),
        added_chunks: vec!["chunk-floor".into()],
        removed_chunks: vec![],
        checksum_before: "before-floor".into(),
        checksum_after: "after-floor".into(),
    };

    emitter
        .emit(
            diff,
            generator
                .encode(&[sanitized_payload()])
                .expect("encoding should succeed"),
        )
        .expect("emit should succeed with configured floor");

    let collected = queue.collected();
    assert_eq!(collected.len(), 1, "expected one emitted entry");
    assert_eq!(
        collected[0].sequence, 42,
        "emitter should honor configured sequence_start even when buffer has lower sequences"
    );
}

#[test]
fn emitter_records_delay_based_on_applied_at() {
    let queue = Arc::new(TestQueue::default());
    let buffer = OfflineReplayBuffer::new(16, Duration::from_secs(60));
    let config = ManifestEmitterConfig {
        sequence_start: 1,
        encryption_key: "test-key".into(),
        retention_max_entries: 16,
        retention_max_age: Duration::from_secs(60),
    };

    let generator = EmbeddingGenerator::new(EmbeddingConfig::new("encoder-z".into(), 6));
    let mut emitter = ManifestEmitter::new(config, buffer, queue.clone());

    let applied_at = SystemTime::now() - Duration::from_millis(1_500);
    let diff = ManifestDiff {
        repo_id: "repo-omega".into(),
        applied_at,
        added_chunks: vec!["chunk-0".into()],
        removed_chunks: vec![],
        checksum_before: "before".into(),
        checksum_after: "after".into(),
    };

    let before_emit = SystemTime::now();
    emitter
        .emit(
            diff,
            generator
                .encode(&[sanitized_payload()])
                .expect("encoding should succeed"),
        )
        .expect("emit should succeed");
    let after_emit = SystemTime::now();

    let collected = queue.collected();
    assert_eq!(collected.len(), 1, "expected emitted replay entry");
    let delayed_ms = collected[0].delayed_ms;

    let min_expected = before_emit
        .duration_since(applied_at)
        .expect("before emit should be after applied_at")
        .as_millis() as u64;
    let max_expected = after_emit
        .duration_since(applied_at)
        .expect("after emit should be after applied_at")
        .as_millis() as u64;
    assert!(
        delayed_ms >= min_expected && delayed_ms <= max_expected,
        "delayed_ms {delayed_ms} should fall within [{min_expected}, {max_expected}]"
    );

    let future_diff = ManifestDiff {
        repo_id: "repo-omega".into(),
        applied_at: SystemTime::now() + Duration::from_secs(5),
        added_chunks: vec!["chunk-0".into()],
        removed_chunks: vec![],
        checksum_before: "before".into(),
        checksum_after: "after".into(),
    };

    emitter
        .emit(
            future_diff,
            generator
                .encode(&[sanitized_payload()])
                .expect("encoding should succeed"),
        )
        .expect("emit should succeed for future diff");

    let collected = queue.collected();
    assert_eq!(collected.len(), 2, "expected future replay entry");
    assert_eq!(
        collected[1].delayed_ms, 0,
        "future diffs should clamp delay to zero"
    );
}

#[test]
fn flush_offline_handles_queue_failure_mid_flush() {
    let queue = Arc::new(TestQueue::default());
    let buffer = OfflineReplayBuffer::new(8, Duration::from_secs(60));

    for sequence in 40..=42 {
        buffer
            .push(ReplayEntry {
                sequence,
                repo_id: "repo-mid-flush".into(),
                delayed_ms: 0,
                payload_checksum_before: format!("before-{sequence}"),
                payload_checksum_after: format!("after-{sequence}"),
                status: "buffered".into(),
            })
            .expect("buffer push should succeed");
    }

    *queue.fail.lock().unwrap() = true;

    let config = ManifestEmitterConfig {
        sequence_start: 50,
        encryption_key: "test-key".into(),
        retention_max_entries: 8,
        retention_max_age: Duration::from_secs(60),
    };

    let generator = EmbeddingGenerator::new(EmbeddingConfig::new("encoder-z".into(), 6));
    let mut emitter = ManifestEmitter::new(config, buffer.clone(), queue.clone());

    let diff = ManifestDiff {
        repo_id: "repo-mid-flush".into(),
        applied_at: SystemTime::now(),
        added_chunks: vec!["chunk-mid".into()],
        removed_chunks: vec![],
        checksum_before: "before-mid".into(),
        checksum_after: "after-mid".into(),
    };

    emitter
        .emit(
            diff,
            generator
                .encode(&[sanitized_payload()])
                .expect("encoding should succeed"),
        )
        .expect_err("emit should fail when queue is offline");

    *queue.fail.lock().unwrap() = false;
    emitter.flush_offline().expect("flush should succeed");

    let collected = queue.collected();
    let sequences: Vec<u64> = collected.iter().map(|entry| entry.sequence).collect();
    assert_eq!(sequences, vec![40, 41, 42, 50]);
}

#[test]
fn emit_during_active_flush_queues_follow_up() {
    let queue = Arc::new(TestQueue::default());
    let buffer = OfflineReplayBuffer::new(8, Duration::from_secs(60));

    for sequence in 60..=61 {
        buffer
            .push(ReplayEntry {
                sequence,
                repo_id: "repo-follow-up".into(),
                delayed_ms: 0,
                payload_checksum_before: format!("before-{sequence}"),
                payload_checksum_after: format!("after-{sequence}"),
                status: "buffered".into(),
            })
            .expect("buffer push should succeed");
    }

    let config = ManifestEmitterConfig {
        sequence_start: 70,
        encryption_key: "test-key".into(),
        retention_max_entries: 8,
        retention_max_age: Duration::from_secs(60),
    };

    let generator = EmbeddingGenerator::new(EmbeddingConfig::new("encoder-z".into(), 6));
    let mut emitter = ManifestEmitter::new(config, buffer.clone(), queue.clone());

    emitter.flush_offline().expect("initial flush should succeed");
    assert_eq!(queue.collected().len(), 2);

    let diff = ManifestDiff {
        repo_id: "repo-follow-up".into(),
        applied_at: SystemTime::now(),
        added_chunks: vec!["chunk-new".into()],
        removed_chunks: vec![],
        checksum_before: "before-new".into(),
        checksum_after: "after-new".into(),
    };
    emitter
        .emit(
            diff,
            generator
                .encode(&[sanitized_payload()])
                .expect("encoding should succeed"),
        )
        .expect("emit should succeed while queue online");

    emitter.flush_offline().expect("follow-up flush should succeed");

    let collected = queue.collected();
    let sequences: Vec<u64> = collected.iter().map(|entry| entry.sequence).collect();
    assert_eq!(sequences, vec![60, 61, 70]);
}
