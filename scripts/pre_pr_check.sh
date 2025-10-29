#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
cd "$ROOT_DIR"

log() { printf "\033[1;34m[pre-pr]\033[0m %s\n" "$*"; }
err() { printf "\033[1;31m[pre-pr]\033[0m %s\n" "$*" 1>&2; }

ensure_tool() {
  local cmd="$1"; shift || true
  if command -v "$cmd" >/dev/null 2>&1; then
    return 0
  fi
  return 1
}

ensure_cargo_crate() {
  local bin_name="$1"; shift
  local install_cmd=("cargo" "+stable" "install" "$@")
  if ! command -v "$bin_name" >/dev/null 2>&1; then
    log "Installing $bin_name (${install_cmd[*]})"
    "${install_cmd[@]}"
  fi
}

log "Repository root: $ROOT_DIR"

# Rustup/rustc/cargo info
if ensure_tool rustup; then
  log "rustup: $(rustup --version)"
  rustup show active-toolchain || true
else
  err "rustup not found; install rustup for best results (https://rustup.rs)."
fi
log "rustc:  $(rustc --version 2>/dev/null || echo 'not found')"
log "cargo:  $(cargo --version 2>/dev/null || echo 'not found')"

# cargo-deny (policy checks)
ensure_cargo_crate cargo-deny cargo-deny --locked
log "Running cargo-deny checks (advisories, bans, sources, licenses)"
cargo +stable deny check advisories
cargo +stable deny check bans
cargo +stable deny check sources
cargo +stable deny check licenses

# gitleaks (secret scanning)
if ensure_tool gitleaks; then
  log "Running gitleaks scan (redacted, no VCS history)"
  gitleaks detect --source . --no-git --redact --config .gitleaks.toml
else
  err "gitleaks not found. Install: https://github.com/gitleaks/gitleaks#installation"
  err "Tip: on macOS: brew install gitleaks; on Linux: download release tarball."
  exit 1
fi

# SBOM (CycloneDX)
ensure_cargo_crate cargo-cyclonedx cargo-cyclonedx --locked --version 0.5.7
log "Generating per-crate CycloneDX JSONs"
cargo +stable cyclonedx --all-features -f json -a

out_dir="artifacts/local/sbom"
mkdir -p "$out_dir"
out_file="$out_dir/zaevrynth.cdx.json"
log "Merging SBOMs to $out_file"
python3 scripts/sbom_merge.py --out "$out_file"

log "All pre-PR checks passed. SBOM at: $out_file"

