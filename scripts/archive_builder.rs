//! Placeholder for the archive fixture builder.
//!
//! Planned responsibilities:
//! - Generate scenario-specific archives (e.g., quota, overflow, bulk sample)
//!   used by ingestion and regression suites.
//! - Emit manifest streams compatible with downstream sanitizers and golden
//!   corpus promotion.
//! - Support configurable compression formats (e.g., tar.zst) and metadata
//!   stamping for reproducibility.
//!
//! Runtime requirements:
//! - Rust toolchain (1.76+ recommended)
//! - Crates to be finalized: compression (`zstd`, `tar`), serialization (`serde`),
//!   and CLI (`clap`).
//!
//! Implementation notes:
//! - Structure the binary as a `bin` target inside a dedicated Cargo workspace
//!   member when promoted beyond this stub.
//! - Integrate with `scripts/checksums.sh` to automatically emit hash manifests
//!   alongside generated artifacts.

fn main() {
    panic!("archive_builder stub not implemented");
}
