use std::{fs, path::PathBuf};

use ingestion_planning::{ChunkPlanner, PlannerConfig, PlanningError};
use ingestion_workspace::{
    ArchiveDescriptor, IgnoreRule, IgnoreSource, RepoType, WorkspaceDescriptor, WorkspaceFile,
};
use serde::Deserialize;
use toml::Value;

#[derive(Debug, Deserialize)]
struct Scenario {
    name: String,
    bytes: u64,
    entries: u64,
    nesting_depth: u32,
    latency_budget_ms: u64,
}

fn load_quota_fixture() -> (ProfileSettings, ArchiveDescriptor) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("fixtures")
        .join("archives")
        .join("quota-latency.toml");
    let content = fs::read_to_string(path).expect("missing quota latency fixture");
    let parsed: Value = content.parse().expect("invalid quota latency fixture");
    let profile = parsed
        .get("profile")
        .and_then(|profile| profile.get("default"))
        .expect("missing default profile");
    let archives = parsed
        .get("archives")
        .and_then(|value| value.as_array())
        .expect("missing archives array");
    let archive = archives.first().expect("expected one archive case");
    (
        ProfileSettings {
            bytes_max: profile
                .get("bytes_max")
                .and_then(Value::as_integer)
                .unwrap() as u64,
            entries_max: profile
                .get("entries_max")
                .and_then(Value::as_integer)
                .unwrap() as u64,
            nesting_max: profile
                .get("nesting_max")
                .and_then(Value::as_integer)
                .unwrap() as u32,
            latency_budget_ms: profile
                .get("latency_budget_ms")
                .and_then(Value::as_integer)
                .unwrap() as u64,
        },
        ArchiveDescriptor {
            name: archive
                .get("name")
                .and_then(Value::as_str)
                .unwrap()
                .to_string(),
            bytes: archive.get("bytes").and_then(Value::as_integer).unwrap() as u64,
            entries: archive.get("entries").and_then(Value::as_integer).unwrap() as u64,
            nesting_depth: archive
                .get("nesting_depth")
                .and_then(Value::as_integer)
                .unwrap() as u32,
            expected_status: archive
                .get("expected_status")
                .and_then(Value::as_str)
                .unwrap()
                .to_string(),
            max_latency_ms: archive
                .get("max_latency_ms")
                .and_then(Value::as_integer)
                .unwrap() as u64,
            scenario: archive
                .get("scenario")
                .and_then(Value::as_str)
                .unwrap()
                .to_string(),
        },
    )
}

fn load_scenarios() -> Vec<Scenario> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("fixtures")
        .join("archives")
        .join("quota-scenarios.toml");
    let content = fs::read_to_string(path).expect("missing quota scenarios fixture");
    let parsed: Value = content.parse().expect("invalid quota scenarios fixture");
    parsed
        .get("scenarios")
        .and_then(Value::as_array)
        .expect("scenarios array")
        .iter()
        .map(|value| Scenario {
            name: value
                .get("name")
                .and_then(Value::as_str)
                .unwrap()
                .to_string(),
            bytes: value.get("bytes").and_then(Value::as_integer).unwrap() as u64,
            entries: value.get("entries").and_then(Value::as_integer).unwrap() as u64,
            nesting_depth: value
                .get("nesting_depth")
                .and_then(Value::as_integer)
                .unwrap() as u32,
            latency_budget_ms: value
                .get("latency_budget_ms")
                .and_then(Value::as_integer)
                .unwrap() as u64,
        })
        .collect()
}

#[derive(Debug, Clone, Copy)]
struct ProfileSettings {
    bytes_max: u64,
    entries_max: u64,
    nesting_max: u32,
    latency_budget_ms: u64,
}

#[test]
fn archive_quota_violation_matches_fixture() {
    let (profile, archive) = load_quota_fixture();
    let descriptor = WorkspaceDescriptor {
        repo_id: "repo-alpha".into(),
        root_path: PathBuf::from("/tmp/repo-alpha"),
        repo_type: RepoType::Archive,
        manifest_cursor: None,
        ignore_stack: vec![IgnoreRule::new(IgnoreSource::Global, "node_modules")],
        archives: vec![archive.clone()],
        latency_windows: vec![],
        files: vec![WorkspaceFile::new("archive.tar", "placeholder")],
    };

    let planner = ChunkPlanner::new(PlannerConfig {
        target_chunk_bytes: 1024,
        max_chunks_per_batch: 4,
        quota_bytes_max: Some(profile.bytes_max),
        quota_entries_max: Some(profile.entries_max),
        quota_nesting_max: Some(profile.nesting_max),
        quota_latency_budget_ms: Some(profile.latency_budget_ms),
    });
    match planner.plan(&descriptor) {
        Err(PlanningError::QuotaExceeded { diagnostics }) => {
            assert_eq!(diagnostics.bytes_limit, profile.bytes_max);
            assert_eq!(diagnostics.entries_limit, profile.entries_max);
            assert_eq!(diagnostics.nesting_limit, profile.nesting_max);
            assert_eq!(diagnostics.bytes_observed, 5632);
            assert_eq!(diagnostics.entries_observed, 22);
            assert!(diagnostics
                .latency_budget_ms
                .map(|value| value >= profile.latency_budget_ms)
                .unwrap_or(false));
        }
        other => panic!("unexpected planner result: {:?}", other),
    }
}

#[test]
fn deterministic_chunk_batches_use_sorted_filenames() {
    let scenarios = load_scenarios();
    let baseline = scenarios
        .iter()
        .find(|scenario| scenario.name == "baseline")
        .expect("missing baseline scenario");
    let descriptor = WorkspaceDescriptor {
        repo_id: "repo-delta".into(),
        root_path: PathBuf::from("/tmp/repo-delta"),
        repo_type: RepoType::Git,
        manifest_cursor: Some("cursor-12".into()),
        ignore_stack: vec![],
        archives: vec![],
        latency_windows: vec![],
        files: vec![
            WorkspaceFile::new("src/bin.rs", "fn main() {}"),
            WorkspaceFile::new("src/lib.rs", "pub fn add(a: i32, b: i32) -> i32 { a + b }"),
            WorkspaceFile::new("README.md", "# sample"),
        ],
    };
    let planner = ChunkPlanner::new(PlannerConfig {
        target_chunk_bytes: baseline.bytes as usize / 2,
        max_chunks_per_batch: 3,
        quota_bytes_max: None,
        quota_entries_max: None,
        quota_nesting_max: None,
        quota_latency_budget_ms: None,
    });

    let plans = planner.plan(&descriptor).expect("planning should succeed");
    let plan_ids: Vec<_> = plans.iter().map(|plan| plan.plan_id.clone()).collect();
    assert_eq!(
        plan_ids,
        vec![
            "repo-delta::README.md::0",
            "repo-delta::src/bin.rs::1",
            "repo-delta::src/lib.rs::2",
        ]
    );
}
