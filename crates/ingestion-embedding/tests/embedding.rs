#![allow(clippy::cloned_ref_to_slice_refs)]
use ingestion_embedding::{EmbeddingConfig, EmbeddingGenerator};
use ingestion_planning::{ChunkPlan, PlannedChunk, RetryPolicy};
use ingestion_sanitization::{SanitizationConfig, Sanitizer};

fn sanitized_chunk(payload: &str) -> ingestion_sanitization::SanitizedChunk {
    let plan = ChunkPlan {
        plan_id: "repo-epsilon::src/lib.rs::0".into(),
        repo_id: "repo-epsilon".into(),
        chunker_config: "size=256".into(),
        source_span: "src/lib.rs:1-80".into(),
        hash: "ff00ff".into(),
        retry_policy: RetryPolicy::default(),
    };
    let planned = PlannedChunk::new(plan, payload);
    let sanitizer = Sanitizer::new(SanitizationConfig::default());
    sanitizer
        .apply(&planned)
        .expect("sanitization should succeed")
}

#[test]
fn embedding_is_deterministic_for_identical_chunks() {
    let config = EmbeddingConfig::new("encoder-a".into(), 4);
    let generator = EmbeddingGenerator::new(config.clone());
    let chunk = sanitized_chunk("fn add(a: i32, b: i32) -> i32 { a + b }");
    let batch1 = generator
        .encode(&[chunk.clone()])
        .expect("encoding should succeed");
    let batch2 = generator.encode(&[chunk]).expect("encoding should succeed");
    assert_eq!(batch1.vectors, batch2.vectors);
    assert_eq!(batch1.encoder_id, config.encoder_id);
}

#[test]
fn embedding_batch_contains_metadata() {
    let generator = EmbeddingGenerator::new(EmbeddingConfig::new("encoder-b".into(), 8));
    let chunks = vec![
        sanitized_chunk("first"),
        sanitized_chunk("second"),
        sanitized_chunk("third"),
    ];
    let batch = generator.encode(&chunks).expect("encoding should succeed");
    assert_eq!(batch.vectors.len(), 3);
    assert!(batch.compression_fingerprint.starts_with("comp:"));
}
