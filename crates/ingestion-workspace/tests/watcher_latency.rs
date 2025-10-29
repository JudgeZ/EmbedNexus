use std::{collections::HashMap, fs, path::PathBuf};

use ingestion_workspace::{
    EnumeratorConfig, IgnoreRule, IgnoreSource, RegistrySnapshot, RepoType, WorkspaceEnumerator,
    WorkspaceRecord,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct LatencyFixture {
    #[allow(dead_code)]
    scenario: String,
    workspace: String,
    latency_windows: Vec<LatencyWindowFixture>,
}

#[derive(Debug, Deserialize)]
struct LatencyWindowFixture {
    window_ms: u64,
    debounce_ms: u64,
    queue_depth: u32,
    #[serde(default)]
    observed: Option<u32>,
    #[serde(default)]
    max_latency_ms: Option<u64>,
    events: Vec<LatencyEventFixture>,
}

#[derive(Debug, Deserialize)]
struct LatencyEventFixture {
    path: String,
    action: String,
    latency_ms: u64,
}

fn load_fixture() -> LatencyFixture {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("fixtures")
        .join("filesystem")
        .join("latency-window.yaml");
    let bytes = fs::read(path).expect("failed to load latency-window fixture");
    serde_yaml::from_slice(&bytes).expect("invalid latency-window fixture")
}

fn parse_golden() -> HashMap<u64, (usize, u64)> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("tests")
        .join("golden")
        .join("filesystem")
        .join("watch-latency-burst.log");
    let content = fs::read_to_string(path).expect("failed to load latency golden");
    let mut result = HashMap::new();
    let mut current_window: Option<u64> = None;
    let mut current_events: Option<usize> = None;
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("- window_ms:") {
            let parts: Vec<_> = line.split(':').collect();
            let window = parts[1].trim().parse::<u64>().expect("window");
            current_window = Some(window);
        } else if line.starts_with("events_observed:") {
            let parts: Vec<_> = line.split(':').collect();
            current_events = Some(parts[1].trim().parse::<usize>().expect("events"));
        } else if line.starts_with("max_latency_ms:") {
            let parts: Vec<_> = line.split(':').collect();
            let latency = parts[1].trim().parse::<u64>().expect("latency");
            if let (Some(window), Some(events)) = (current_window.take(), current_events.take()) {
                result.insert(window, (events, latency));
            }
        }
    }
    result
}

#[test]
fn watcher_latency_budget_matches_golden() {
    let fixture = load_fixture();
    let golden = parse_golden();

    let record = WorkspaceRecord {
        repo_id: fixture.workspace.clone(),
        root_path: PathBuf::from("/workspaces/repo-alpha"),
        repo_type: RepoType::Git,
        manifest_cursor: None,
        ignore_rules: vec![
            IgnoreRule::new(IgnoreSource::Git, "target"),
            IgnoreRule::new(IgnoreSource::Editor, "*.swp"),
        ],
        archives: vec![],
        latency_windows: fixture
            .latency_windows
            .iter()
            .map(LatencyWindowFixture::to_latency_window)
            .collect(),
        files: vec![],
    };
    let snapshot = RegistrySnapshot::new(vec![record]);
    let enumerator = WorkspaceEnumerator::new(EnumeratorConfig {
        global_ignores: vec![IgnoreRule::new(IgnoreSource::Global, "node_modules")],
        sandbox_ignores: vec![IgnoreRule::new(IgnoreSource::Sandbox, "tmp")],
    });

    let descriptors = enumerator
        .scan(&snapshot)
        .expect("workspace enumeration should succeed");
    assert_eq!(descriptors.len(), 1);
    let descriptor = &descriptors[0];
    for fixture_window in &fixture.latency_windows {
        let actual = descriptor
            .latency_windows()
            .iter()
            .find(|window| window.window_ms == fixture_window.window_ms)
            .expect("expected window to be present");
        let (expected_events, expected_latency) = golden
            .get(&fixture_window.window_ms)
            .expect("missing golden entry");
        assert_eq!(actual.events_observed as usize, *expected_events);
        assert_eq!(actual.max_latency_ms, *expected_latency);
        assert_eq!(actual.queue_depth, fixture_window.queue_depth);
        assert_eq!(actual.debounce_ms, fixture_window.debounce_ms);
    }
}

#[test]
fn ignore_stack_merges_deterministically() {
    let record = WorkspaceRecord {
        repo_id: "repo-beta".into(),
        root_path: PathBuf::from("/workspaces/repo-beta"),
        repo_type: RepoType::Archive,
        manifest_cursor: Some("cursor-9".into()),
        ignore_rules: vec![
            IgnoreRule::new(IgnoreSource::Git, ".git"),
            IgnoreRule::new(IgnoreSource::Custom("repo".into()), "vendor"),
        ],
        archives: vec![],
        latency_windows: vec![],
        files: vec![],
    };
    let snapshot = RegistrySnapshot::new(vec![record]);
    let enumerator = WorkspaceEnumerator::new(EnumeratorConfig {
        global_ignores: vec![
            IgnoreRule::new(IgnoreSource::Global, "node_modules"),
            IgnoreRule::new(IgnoreSource::Global, ".git"),
        ],
        sandbox_ignores: vec![IgnoreRule::new(IgnoreSource::Sandbox, "tmp")],
    });

    let descriptors = enumerator
        .scan(&snapshot)
        .expect("workspace enumeration should succeed");
    let descriptor = &descriptors[0];
    let patterns: Vec<_> = descriptor
        .ignore_stack()
        .iter()
        .map(|rule| rule.pattern.clone())
        .collect();
    assert_eq!(patterns, vec!["node_modules", "tmp", ".git", "vendor"]);
}

impl LatencyWindowFixture {
    fn to_latency_window(&self) -> ingestion_workspace::LatencyWindow {
        ingestion_workspace::LatencyWindow {
            window_ms: self.window_ms,
            debounce_ms: self.debounce_ms,
            queue_depth: self.queue_depth,
            events: self
                .events
                .iter()
                .map(|event| ingestion_workspace::LatencyEvent {
                    path: event.path.clone(),
                    action: event.action.clone(),
                    latency_ms: event.latency_ms,
                })
                .collect(),
            events_observed: self.observed.unwrap_or(self.events.len() as u32),
            max_latency_ms: self.max_latency_ms.unwrap_or_else(|| {
                self.events
                    .iter()
                    .map(|event| event.latency_ms)
                    .max()
                    .unwrap_or_default()
            }),
        }
    }
}
