#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: trace_capture.sh --profile <name> [--output <file>] [--output-dir <dir>] [--proxy-host <host>] [--proxy-port <port>]

Profiles:
  tls-handshake        Generate a deterministic TLS 1.3 handshake trace.
  tls-fuzz             Emit a fuzzing session log covering downgrade attempts.
  tls-negotiation      Capture a multi-step negotiation transcript including ALPN.
  encryption-toggle    Produce an encryption toggle state machine trace.
  encryption-latency   Emit latency samples for encryption toggles as JSON.
  tls-config-matrix    Emit supported TLS configuration matrix as YAML.
  perf-window          Write rolling performance window statistics into a directory.
  perf-baseline        Output TLS performance baseline samples as JSONL.
  dpapi-audit          Copy the latest DPAPI audit export into the requested path.
  wsl-transport        Generate a WSL transport bridge negotiation transcript.
USAGE
}

PROFILE=""
OUTPUT=""
OUTPUT_DIR=""
PROXY_HOST="localhost"
PROXY_PORT="5173"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --profile)
      PROFILE="$2"
      shift 2
      ;;
    --output)
      OUTPUT="$2"
      shift 2
      ;;
    --output-dir)
      OUTPUT_DIR="$2"
      shift 2
      ;;
    --proxy-host)
      PROXY_HOST="$2"
      shift 2
      ;;
    --proxy-port)
      PROXY_PORT="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

if [[ -z "$PROFILE" ]]; then
  echo "--profile is required" >&2
  usage >&2
  exit 1
fi

if [[ -z "$OUTPUT" && -n "${TRACE_OUT:-}" ]]; then
  OUTPUT="$TRACE_OUT"
fi

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
fixture_root="$(cd "$script_dir/.." && pwd)"
dpapi_fixture_dir="$fixture_root/tests/fixtures/security/dpapi-recovery"

ensure_output_file() {
  local path="$1"
  local parent
  parent="$(dirname "$path")"
  mkdir -p "$parent"
}

write_file() {
  local path="$1"
  ensure_output_file "$path"
  cat >"$path"
}

case "$PROFILE" in
  tls-handshake)
    if [[ -z "$OUTPUT" ]]; then
      echo "--output or TRACE_OUT is required for profile tls-handshake" >&2
      exit 1
    fi
    write_file "$OUTPUT" <<'TRACE'
# TLS Handshake Trace
capture_timestamp: 2024-03-18T20:20:00Z
client:
  random: 5a2dd3f520f844529fe902ddf4616c50a238f9d43c628d5088c42beef74a11f0
  session_id: 000000000000000000000000000000000000
  sni: api.zaevrynth.test
  cipher_suites:
    - TLS_AES_128_GCM_SHA256
    - TLS_AES_256_GCM_SHA384
    - TLS_CHACHA20_POLY1305_SHA256
  alpn:
    - h2
    - http/1.1
server:
  certificate_subject: CN=api.zaevrynth.test
  protocol: TLS1.3
  cipher_suite: TLS_AES_128_GCM_SHA256
  alpn: h2
session_resumed: false
notes:
  - handshake_completed
  - session_ticket_issued
TRACE
    ;;
  tls-fuzz)
    if [[ -z "$OUTPUT" ]]; then
      echo "--output or TRACE_OUT is required for profile tls-fuzz" >&2
      exit 1
    fi
    write_file "$OUTPUT" <<'LOG'
[2024-03-18T20:20:01Z] INFO Starting TLS fuzzing session id=zaevrynth-ci
[2024-03-18T20:20:02Z] WARN Downgrade attempt rejected: unsupported_record_version=TLS1.0
[2024-03-18T20:20:02Z] INFO Verified GREASE extension propagation
[2024-03-18T20:20:03Z] INFO Completed 256 handshake mutations without server failure
LOG
    ;;
  encryption-toggle)
    if [[ -z "$OUTPUT" ]]; then
      echo "--output or TRACE_OUT is required for profile encryption-toggle" >&2
      exit 1
    fi
    write_file "$OUTPUT" <<'TRACE'
# Encryption Toggle Trace
state: enabled
sequence:
  - timestamp: 2024-03-18T20:19:55Z
    actor: control-plane
    action: enable
    reason: wsl_bridge_handshake
  - timestamp: 2024-03-18T20:20:10Z
    actor: policy-engine
    action: verify
    result: success
  - timestamp: 2024-03-18T20:21:02Z
    actor: control-plane
    action: rotate-key
    master_key_guid: 7c9e6679-7425-40de-944b-e07fc1f90ae7
TRACE
    ;;
  encryption-latency)
    if [[ -z "$OUTPUT" ]]; then
      echo "--output is required for profile encryption-latency" >&2
      exit 1
    fi
    write_file "$OUTPUT" <<'JSON'
{
  "capture_id": "enc-latency-zaevrynth-ci",
  "generated_at": "2024-03-18T20:20:30Z",
  "samples": [
    {"feature": "search", "encryption": true, "latency_ms": 31.2},
    {"feature": "search", "encryption": false, "latency_ms": 18.9},
    {"feature": "ingest", "encryption": true, "latency_ms": 44.5},
    {"feature": "ingest", "encryption": false, "latency_ms": 29.8}
  ],
  "summary": {
    "encrypted_p95_ms": 44.5,
    "unencrypted_p95_ms": 29.8,
    "delta_ms": 14.7
  }
}
JSON
    ;;
  tls-config-matrix)
    if [[ -z "$OUTPUT" ]]; then
      echo "--output is required for profile tls-config-matrix" >&2
      exit 1
    fi
    write_file "$OUTPUT" <<'YAML'
---
service: zaevrynth-control-plane
date: 2024-03-18
supported_versions:
  - TLS1.3
  - TLS1.2
default_cipher_suites:
  - TLS_AES_128_GCM_SHA256
  - TLS_AES_256_GCM_SHA384
  - TLS_CHACHA20_POLY1305_SHA256
fallback_policies:
  - name: legacy-clients
    sni: legacy.zaevrynth.test
    ciphers:
      - TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384
      - TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256
YAML
    ;;
  perf-window)
    if [[ -z "$OUTPUT_DIR" ]]; then
      echo "--output-dir is required for profile perf-window" >&2
      exit 1
    fi
    mkdir -p "$OUTPUT_DIR"
    write_file "$OUTPUT_DIR/rolling-window.json" <<'JSON'
{
  "window_ms": 60000,
  "samples": [
    {"timestamp": "2024-03-18T20:19:00Z", "handshakes": 42, "failures": 0},
    {"timestamp": "2024-03-18T20:19:30Z", "handshakes": 47, "failures": 1},
    {"timestamp": "2024-03-18T20:20:00Z", "handshakes": 53, "failures": 0}
  ],
  "summary": {
    "max_handshakes": 53,
    "failure_rate": 0.006
  }
}
JSON
    write_file "$OUTPUT_DIR/notes.txt" <<'TEXT'
Rolling TLS performance window generated by trace_capture.sh --profile perf-window.
TEXT
    ;;
  perf-baseline)
    if [[ -z "$OUTPUT" ]]; then
      echo "--output is required for profile perf-baseline" >&2
      exit 1
    fi
    write_file "$OUTPUT" <<'JSONL'
{"timestamp":"2024-03-18T20:20:03Z","stage":"connect","duration_ms":15.3}
{"timestamp":"2024-03-18T20:20:03Z","stage":"handshake","duration_ms":21.7}
{"timestamp":"2024-03-18T20:20:04Z","stage":"application_data","duration_ms":8.9,"retransmits":0}
JSONL
    ;;
  tls-negotiation)
    if [[ -z "$OUTPUT" ]]; then
      echo "--output or TRACE_OUT is required for profile tls-negotiation" >&2
      exit 1
    fi
    write_file "$OUTPUT" <<TRACE
# TLS Negotiation Transcript
client_hello: api.zaevrynth.test
proxy: ${PROXY_HOST}:${PROXY_PORT}
negotiated_version: TLS1.3
selected_cipher: TLS_AES_256_GCM_SHA384
alpn_result: h2
resumption_ticket: issued
ocsp_stapling: true
TRACE
    ;;
  dpapi-audit)
    if [[ -z "$OUTPUT" ]]; then
      echo "--output is required for profile dpapi-audit" >&2
      exit 1
    fi
    source_path="$dpapi_fixture_dir/recovery-events.jsonl"
    if [[ ! -f "$source_path" ]]; then
      echo "DPAPI recovery events not found at $source_path. Run collect_dpapi.ps1 first." >&2
      exit 1
    fi
    ensure_output_file "$OUTPUT"
    cp "$source_path" "$OUTPUT"
    ;;
  wsl-transport)
    if [[ -z "$OUTPUT" ]]; then
      echo "--output or TRACE_OUT is required for profile wsl-transport" >&2
      exit 1
    fi
    write_file "$OUTPUT" <<TRACE
# WSL Transport Bridge Trace
proxy_endpoint: ${PROXY_HOST}:${PROXY_PORT}
bridge:
  version: 1
  mode: passthrough
windows_client:
  computer: WIN-CI
  session: wsl-trace
wsl_peer:
  distro: Ubuntu-22.04
  kernel: 5.15.133.1-microsoft-standard-WSL2
handshake:
  tls_version: TLS1.3
  cipher: TLS_AES_128_GCM_SHA256
  negotiated_alpn: h2
  dpapi_session_key: 7c9e6679-7425-40de-944b-e07fc1f90ae7
TRACE
    ;;
  *)
    echo "Unknown profile: $PROFILE" >&2
    usage >&2
    exit 1
    ;;
esac

echo "Generated ${PROFILE} capture" >&2
