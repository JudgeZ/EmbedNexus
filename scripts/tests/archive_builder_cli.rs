use std::fs;
use std::io::Read;
use std::path::Path;

use tempfile::tempdir;

#[allow(deprecated)]
fn cargo_bin() -> assert_cmd::Command {
    assert_cmd::Command::cargo_bin("archive_builder").expect("binary not built")
}

#[test]
fn quota_manifest_is_deterministic() {
    let dir = tempdir().unwrap();
    let output_path = dir.path().join("quota.toml");

    cargo_bin()
        .arg("--scenario")
        .arg("quota")
        .arg("--output")
        .arg(&output_path)
        .assert()
        .success();

    let contents = fs::read_to_string(&output_path).unwrap();
    assert!(contents.contains("alpha-corp"));
    assert!(contents.contains("beta-labs"));
    assert!(contents.contains("gamma-studios"));
}

#[test]
fn fuzz_stream_matches_expected_snapshot() {
    let mut cmd = cargo_bin();
    cmd.arg("--scenario").arg("fuzz");
    let assert = cmd.assert().success();
    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let expected = r#"{"scenario":"fuzz","tenant":"alpha-corp","operation":"ingest","request_count":256,"budget_bytes":4096,"overflow_expected":false}
{"scenario":"fuzz","tenant":"beta-labs","operation":"plan","request_count":293,"budget_bytes":4608,"overflow_expected":true}
{"scenario":"fuzz","tenant":"gamma-studios","operation":"commit","request_count":330,"budget_bytes":5120,"overflow_expected":false}
{"scenario":"fuzz","tenant":"delta-systems","operation":"audit","request_count":367,"budget_bytes":5632,"overflow_expected":true}
{"scenario":"fuzz","tenant":"epsilon-holdings","operation":"prune","request_count":404,"budget_bytes":6144,"overflow_expected":false}
"#;
    assert_eq!(output, expected);
}

#[test]
fn overflow_archive_contains_metadata() {
    let dir = tempdir().unwrap();
    let output_path = dir.path().join("overflow.tar.zst");

    cargo_bin()
        .arg("--scenario")
        .arg("overflow")
        .arg("--output")
        .arg(&output_path)
        .assert()
        .success();

    let mut decoder = zstd::Decoder::new(fs::File::open(&output_path).unwrap()).unwrap();
    let mut tar_bytes = Vec::new();
    decoder.read_to_end(&mut tar_bytes).unwrap();
    let mut archive = tar::Archive::new(&tar_bytes[..]);

    let mut entries: Vec<String> = archive
        .entries()
        .unwrap()
        .map(|entry| {
            let entry = entry.unwrap();
            entry.path().unwrap().to_string_lossy().into_owned()
        })
        .collect();
    entries.sort();
    assert_eq!(entries, vec!["README.txt", "metadata.json"]);
}

#[test]
fn bulk_corpus_writes_index_and_manifests() {
    let dir = tempdir().unwrap();

    cargo_bin()
        .arg("--scenario")
        .arg("bulk")
        .arg("--output-dir")
        .arg(dir.path())
        .assert()
        .success();

    let expected_files = [
        "index.toml",
        "session-001-manifest.toml",
        "session-002-manifest.toml",
        "session-003-manifest.toml",
        "README.md",
    ];

    for file in expected_files.iter() {
        let path = dir.path().join(file);
        assert!(path.exists(), "missing {file}");
    }
}

#[test]
fn quota_throughput_stream_matches_snapshot() {
    let mut cmd = cargo_bin();
    cmd.arg("--scenario").arg("quota-throughput");
    let assert = cmd.assert().success();
    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let expected = r#"{"scenario":"quota-throughput","tenant":"alpha-corp","window_seconds":300,"requests":3000,"average_latency_ms":120,"saturation_ratio":0.78}
{"scenario":"quota-throughput","tenant":"beta-labs","window_seconds":300,"requests":3450,"average_latency_ms":135,"saturation_ratio":0.84999996}
{"scenario":"quota-throughput","tenant":"gamma-studios","window_seconds":300,"requests":3900,"average_latency_ms":150,"saturation_ratio":0.91999996}
"#;
    assert_eq!(output, expected);
}

#[test]
fn overflow_latency_archive_contains_csv() {
    let dir = tempdir().unwrap();
    let output_path = dir.path().join("overflow-latency.tar.zst");

    cargo_bin()
        .arg("--scenario")
        .arg("overflow-latency")
        .arg("--output")
        .arg(&output_path)
        .assert()
        .success();

    let mut decoder = zstd::Decoder::new(fs::File::open(&output_path).unwrap()).unwrap();
    let mut tar_bytes = Vec::new();
    decoder.read_to_end(&mut tar_bytes).unwrap();
    let mut archive = tar::Archive::new(&tar_bytes[..]);

    let mut seen_latency = false;
    for entry in archive.entries().unwrap() {
        let mut entry = entry.unwrap();
        if entry.path().unwrap() == Path::new("latency.csv") {
            let mut contents = String::new();
            entry.read_to_string(&mut contents).unwrap();
            assert!(contents.contains("sample_0,1500"));
            seen_latency = true;
        }
    }
    assert!(seen_latency, "latency.csv not found");
}
