#![allow(clippy::format_collect)]
//! Archive scenario builder binary.
//!
//! The binary intentionally keeps the data model small and deterministic so that
//! the fixture regeneration workflows can rely on stable outputs.  The emitted
//! artifacts are synthetic but mimic the structure expected by the downstream
//! sanitiser and checksum tooling.

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use serde::Serialize;
use tar::Builder as TarBuilder;

const FIXED_MTIME: u64 = 1_704_889_600; // 2024-02-10T00:00:00Z for deterministic archives.

#[derive(Debug, Clone, ValueEnum)]
enum Scenario {
    Quota,
    QuotaLatency,
    QuotaThroughput,
    Overflow,
    OverflowLatency,
    Bulk,
    Fuzz,
}

#[derive(Debug, Parser)]
#[command(author, version, about = "Deterministic archive fixture builder")]
struct Args {
    /// Scenario to emit
    #[arg(long)]
    scenario: Scenario,

    /// File output path for scenarios that persist a single artifact.
    #[arg(long)]
    output: Option<PathBuf>,

    /// Directory output path for scenarios emitting a corpus of files.
    #[arg(long, value_name = "DIR")]
    output_dir: Option<PathBuf>,
}

fn run(args: Args) -> Result<()> {
    match args.scenario {
        Scenario::Quota => {
            let output = args
                .output
                .context("--output must be provided for the quota scenario")?;
            write_quota_manifest(&output, "quota")
        }
        Scenario::QuotaLatency => {
            let output = args
                .output
                .context("--output must be provided for the quota-latency scenario")?;
            write_latency_manifest(&output, "quota-latency")
        }
        Scenario::QuotaThroughput => emit_throughput_stream(io::stdout().lock()),
        Scenario::Overflow => {
            let output = args
                .output
                .context("--output must be provided for the overflow scenario")?;
            write_overflow_archive(&output, OverflowProfile::Capacity)
        }
        Scenario::OverflowLatency => {
            let output = args
                .output
                .context("--output must be provided for the overflow-latency scenario")?;
            write_overflow_archive(&output, OverflowProfile::Latency)
        }
        Scenario::Bulk => {
            let output_dir = args
                .output_dir
                .context("--output-dir must be provided for the bulk scenario")?;
            write_bulk_corpus(&output_dir)
        }
        Scenario::Fuzz => emit_fuzz_stream(io::stdout().lock()),
    }
}

fn write_quota_manifest(path: &Path, scenario_name: &str) -> Result<()> {
    let manifest = QuotaManifest {
        version: 1,
        scenario: scenario_name.to_string(),
        generated_at: fixed_timestamp_string(),
        buckets: vec![
            QuotaBucket {
                tenant: "alpha-corp".to_string(),
                window_seconds: 3600,
                allowed_requests: 12_000,
                used_requests: 11_420,
                burst_capacity: 800,
            },
            QuotaBucket {
                tenant: "beta-labs".to_string(),
                window_seconds: 3600,
                allowed_requests: 9_600,
                used_requests: 7_488,
                burst_capacity: 600,
            },
            QuotaBucket {
                tenant: "gamma-studios".to_string(),
                window_seconds: 3600,
                allowed_requests: 15_000,
                used_requests: 14_200,
                burst_capacity: 1_200,
            },
        ],
    };

    ensure_parent(path)?;
    let encoded = toml::to_string_pretty(&manifest).context("serialising quota manifest")?;
    fs::write(path, encoded)
        .with_context(|| format!("writing quota manifest to {}", path.display()))
}

fn write_latency_manifest(path: &Path, scenario_name: &str) -> Result<()> {
    let manifest = LatencyManifest {
        version: 1,
        scenario: scenario_name.to_string(),
        generated_at: fixed_timestamp_string(),
        windows: vec![
            LatencyWindow {
                window_label: "peak_ingest".to_string(),
                percentile: 95,
                duration_ms: 2400,
            },
            LatencyWindow {
                window_label: "steady_state".to_string(),
                percentile: 75,
                duration_ms: 820,
            },
            LatencyWindow {
                window_label: "recovery".to_string(),
                percentile: 99,
                duration_ms: 5100,
            },
        ],
    };

    ensure_parent(path)?;
    let encoded = toml::to_string_pretty(&manifest).context("serialising latency manifest")?;
    fs::write(path, encoded)
        .with_context(|| format!("writing latency manifest to {}", path.display()))
}

fn emit_throughput_stream(mut writer: impl Write) -> Result<()> {
    for (idx, tenant) in ["alpha-corp", "beta-labs", "gamma-studios"]
        .iter()
        .enumerate()
    {
        let record = ThroughputRecord {
            scenario: "quota-throughput",
            tenant,
            window_seconds: 300,
            requests: 3_000 + (idx as u32) * 450,
            average_latency_ms: 120 + (idx as u32) * 15,
            saturation_ratio: (idx as f32).mul_add(0.07, 0.78),
        };
        serde_json::to_writer(&mut writer, &record).context("serialising throughput record")?;
        writer
            .write_all(b"\n")
            .context("writing throughput newline")?;
    }
    Ok(())
}

fn emit_fuzz_stream(mut writer: impl Write) -> Result<()> {
    let operations = ["ingest", "plan", "commit", "audit", "prune"];
    for (idx, tenant) in [
        "alpha-corp",
        "beta-labs",
        "gamma-studios",
        "delta-systems",
        "epsilon-holdings",
    ]
    .iter()
    .enumerate()
    {
        let record = FuzzRecord {
            scenario: "fuzz",
            tenant,
            operation: operations[idx % operations.len()],
            request_count: 256 + (idx as u32) * 37,
            budget_bytes: 4096 + (idx as u32) * 512,
            overflow_expected: idx % 2 == 1,
        };
        serde_json::to_writer(&mut writer, &record).context("serialising fuzz record")?;
        writer.write_all(b"\n").context("writing fuzz newline")?;
    }
    Ok(())
}

fn write_overflow_archive(path: &Path, profile: OverflowProfile) -> Result<()> {
    ensure_parent(path)?;
    let file = File::create(path).with_context(|| format!("creating {}", path.display()))?;
    let encoder =
        zstd::stream::write::Encoder::new(file, 0).context("initialising zstd encoder")?;
    let mut builder = TarBuilder::new(encoder.auto_finish());

    match profile {
        OverflowProfile::Capacity => {
            add_tar_entry(&mut builder, "README.txt", b"Overflow capacity scenario\n")?;
            let metadata = OverflowMetadata {
                scenario: "overflow",
                generated_at: fixed_timestamp_string(),
                window_seconds: 900,
                queued_chunks: 42,
                dropped_chunks: 5,
            };
            let json =
                serde_json::to_vec_pretty(&metadata).context("serialising overflow metadata")?;
            add_tar_entry(&mut builder, "metadata.json", &json)?;
        }
        OverflowProfile::Latency => {
            add_tar_entry(
                &mut builder,
                "README.txt",
                b"Overflow latency probe scenario\n",
            )?;
            let samples = LatencySamples {
                scenario: "overflow-latency",
                generated_at: fixed_timestamp_string(),
                samples_ms: vec![1500, 2100, 1980, 2750, 3100],
            };
            let csv = samples
                .samples_ms
                .iter()
                .enumerate()
                .map(|(idx, value)| format!("sample_{idx},{value}\n"))
                .collect::<String>();
            add_tar_entry(&mut builder, "latency.csv", csv.as_bytes())?;
            let json =
                serde_json::to_vec_pretty(&samples).context("serialising latency samples")?;
            add_tar_entry(&mut builder, "metadata.json", &json)?;
        }
    }

    builder.finish().context("finishing tar archive")?;
    Ok(())
}

fn write_bulk_corpus(output_dir: &Path) -> Result<()> {
    fs::create_dir_all(output_dir).with_context(|| format!("creating {}", output_dir.display()))?;

    let index = BulkIndex {
        version: 1,
        generated_at: fixed_timestamp_string(),
        sessions: vec![
            BulkSession {
                id: "session-001".into(),
                tenant: "alpha-corp".into(),
                chunk_count: 12,
            },
            BulkSession {
                id: "session-002".into(),
                tenant: "beta-labs".into(),
                chunk_count: 8,
            },
            BulkSession {
                id: "session-003".into(),
                tenant: "gamma-studios".into(),
                chunk_count: 15,
            },
        ],
    };

    let index_path = output_dir.join("index.toml");
    fs::write(
        &index_path,
        toml::to_string_pretty(&index).context("serialising bulk index")?,
    )
    .with_context(|| format!("writing {}", index_path.display()))?;

    for (idx, session) in index.sessions.iter().enumerate() {
        let manifest = BulkManifest {
            session_id: session.id.clone(),
            tenant: session.tenant.clone(),
            sequence_start: 1,
            sequence_end: session.chunk_count,
            notes: format!("deterministic bulk sample {}", idx + 1),
        };
        let manifest_path = output_dir.join(format!("{}-manifest.toml", session.id));
        fs::write(
            &manifest_path,
            toml::to_string_pretty(&manifest).context("serialising bulk manifest")?,
        )
        .with_context(|| format!("writing {}", manifest_path.display()))?;
    }

    let readme_path = output_dir.join("README.md");
    fs::write(
        &readme_path,
        "# Bulk sample corpus\n\nGenerated by archive_builder for deterministic fixture coverage.\n",
    )
    .with_context(|| format!("writing {}", readme_path.display()))?;
    Ok(())
}

fn ensure_parent(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
        }
    }
    Ok(())
}

fn fixed_timestamp_string() -> String {
    let ts = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(FIXED_MTIME);
    humantime::format_rfc3339(ts).to_string()
}

fn add_tar_entry<W>(builder: &mut TarBuilder<W>, path: &str, contents: &[u8]) -> Result<()>
where
    W: Write,
{
    let mut header = tar::Header::new_gnu();
    header.set_path(path).context("setting tar path")?;
    header.set_mode(0o644);
    header.set_uid(0);
    header.set_gid(0);
    header.set_size(contents.len() as u64);
    header.set_mtime(FIXED_MTIME);
    header.set_cksum();
    builder
        .append(&header, contents)
        .with_context(|| format!("writing tar entry {path}"))
}

#[derive(Serialize)]
struct QuotaManifest {
    version: u32,
    scenario: String,
    generated_at: String,
    buckets: Vec<QuotaBucket>,
}

#[derive(Serialize)]
struct QuotaBucket {
    tenant: String,
    window_seconds: u32,
    allowed_requests: u32,
    used_requests: u32,
    burst_capacity: u32,
}

#[derive(Serialize)]
struct LatencyManifest {
    version: u32,
    scenario: String,
    generated_at: String,
    windows: Vec<LatencyWindow>,
}

#[derive(Serialize)]
struct LatencyWindow {
    window_label: String,
    percentile: u8,
    duration_ms: u32,
}

#[derive(Serialize)]
struct ThroughputRecord<'a> {
    scenario: &'a str,
    tenant: &'a str,
    window_seconds: u32,
    requests: u32,
    average_latency_ms: u32,
    saturation_ratio: f32,
}

#[derive(Serialize)]
struct FuzzRecord<'a> {
    scenario: &'a str,
    tenant: &'a str,
    operation: &'a str,
    request_count: u32,
    budget_bytes: u32,
    overflow_expected: bool,
}

#[derive(Serialize)]
struct OverflowMetadata {
    scenario: &'static str,
    generated_at: String,
    window_seconds: u32,
    queued_chunks: u32,
    dropped_chunks: u32,
}

#[derive(Serialize)]
struct LatencySamples {
    scenario: &'static str,
    generated_at: String,
    samples_ms: Vec<u32>,
}

#[derive(Serialize)]
struct BulkIndex {
    version: u32,
    generated_at: String,
    sessions: Vec<BulkSession>,
}

#[derive(Serialize)]
struct BulkSession {
    id: String,
    tenant: String,
    chunk_count: u32,
}

#[derive(Serialize)]
struct BulkManifest {
    session_id: String,
    tenant: String,
    sequence_start: u32,
    sequence_end: u32,
    notes: String,
}

enum OverflowProfile {
    Capacity,
    Latency,
}

fn main() {
    if let Err(err) = run(Args::parse()) {
        eprintln!("error: {err:#}");
        std::process::exit(1);
    }
}
