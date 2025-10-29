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

# Ensure clippy/rustfmt components are available (log-only)
if ensure_tool rustup; then
  for comp in rustfmt clippy; do
    if rustup component list --installed | grep -q "^${comp} "; then
      log "rustup component present: ${comp}"
    else
      err "rustup component missing: ${comp}. Hint: rustup component add ${comp}"
    fi
  done
fi

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

# SBOM (CycloneDX) with Python fallback
ensure_cargo_crate cargo-cyclonedx cargo-cyclonedx --locked --version 0.5.7
out_dir="artifacts/local/sbom"
mkdir -p "$out_dir"
out_file="$out_dir/zaevrynth.cdx.json"

# Prefer single aggregated workspace BOM if supported
log "Attempting aggregated workspace SBOM to: $out_file"
if cargo +stable cyclonedx --workspace --all-features -f json -o "$out_file" 2>/dev/null; then
  log "Workspace SBOM generated: $out_file"
else
  log "Workspace aggregation not available; generating per-crate SBOMs (-a)"
  cargo +stable cyclonedx --all-features -f json -a
  # Merge with Python if available
  if ensure_tool python3; then
    log "Merging SBOMs via scripts/sbom_merge.py"
    python3 scripts/sbom_merge.py --out "$out_file"
  elif ensure_tool jq; then
    log "Merging SBOMs via jq (components union)"
    mapfile -t SBOMS < <(find . -type f -name "*.cdx.json" | sort)
    if ((${#SBOMS[@]}==0)); then
      err "No per-crate CycloneDX files found to merge."
      exit 1
    fi
    jq -s '{
      bomFormat: "CycloneDX",
      specVersion: "1.5",
      version: 1,
      components: ( map(.components // []) | add )
    }' "${SBOMS[@]}" > "$out_file"
  else
    err "Neither python3 nor jq available to merge SBOMs. Copying first SBOM as fallback."
    first=$(find . -type f -name "*.cdx.json" | head -n1)
    cp "$first" "$out_file"
  fi
fi

log "All pre-PR checks passed. SBOM at: $out_file"
