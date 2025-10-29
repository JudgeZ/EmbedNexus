use ingestion_planning::{ChunkPlan, PlannedChunk, RetryPolicy};
use ingestion_sanitization::{SanitizationConfig, Sanitizer};

fn build_plan() -> ChunkPlan {
    ChunkPlan {
        plan_id: "repo-gamma::src/secret.rs::0".into(),
        repo_id: "repo-gamma".into(),
        chunker_config: "size=128".into(),
        source_span: "src/secret.rs:1-40".into(),
        hash: "deadbeef".into(),
        retry_policy: RetryPolicy::default(),
    }
}

#[test]
fn sanitizer_redacts_secret_tokens() {
    let plan = build_plan();
    let chunk = PlannedChunk::new(
        plan.clone(),
        "let TOKEN = \"SECRET-123\";\nlet api_key = \"API_KEY=XYZ\";",
    );
    let sanitizer = Sanitizer::new(SanitizationConfig::default());
    let sanitized = sanitizer
        .apply(&chunk)
        .expect("sanitization should succeed");
    assert!(!sanitized.scrubbed_payload.contains("SECRET"));
    assert!(sanitized
        .redaction_log
        .iter()
        .any(|entry| entry.contains("SECRET[-_A-Z0-9]+ => count=1")));
    assert!(sanitized
        .redaction_log
        .iter()
        .any(|entry| entry.contains("API_KEY[=:][A-Za-z0-9_-]+ => count=1")));
    assert!(!sanitized
        .redaction_log
        .iter()
        .any(|entry| entry.contains("SECRET-123") || entry.contains("API_KEY=XYZ")));
    assert!(sanitized
        .redaction_log
        .iter()
        .all(|entry| entry.contains("digest=")));
    assert_eq!(sanitized.plan_id, plan.plan_id);
}

#[test]
fn sanitizer_flags_script_with_shebang() {
    let plan = build_plan();
    let chunk = PlannedChunk::new(plan, "#!/bin/bash\necho SECRET");
    let sanitizer = Sanitizer::new(SanitizationConfig::default());
    let sanitized = sanitizer
        .apply(&chunk)
        .expect("sanitization should succeed");
    assert_eq!(sanitized.validation_status, "script-reviewed");
}
