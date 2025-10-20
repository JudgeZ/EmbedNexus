#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<USAGE
Usage: $0 (--update | --verify) <paths...>

Generate or verify SHA-256 manifests alongside fixture directories and files.

Modes:
  --update    Regenerate manifests for the provided paths.
  --verify    Verify manifests for the provided paths.

Examples:
  $0 --update tests/fixtures/ tests/golden/archives/fuzzed-manifests.jsonl
  $0 --verify tests/fixtures/ tests/golden/
USAGE
}

abs_path() {
  python3 - "$1" <<'PY'
import os
import sys
print(os.path.abspath(sys.argv[1]))
PY
}

generate_manifest() {
  local target="$1"
  local manifest="$2"
  local target_abs
  target_abs=$(abs_path "$target")
  local manifest_abs
  manifest_abs=$(abs_path "$manifest")

  if [[ -d "$target_abs" ]]; then
    generate_dir_manifest "$target_abs" "$manifest_abs"
  else
    local dir
    dir=$(dirname "$target_abs")
    local base
    base=$(basename "$target_abs")
    mkdir -p "$(dirname "$manifest_abs")"
    (
      cd "$dir"
      sha256sum "$base"
    ) >"$manifest_abs"
  fi
}

generate_dir_manifest() {
  local base_abs="$1"
  local manifest_abs="$2"
  mkdir -p "$(dirname "$manifest_abs")"

  local -a files=()
  while IFS= read -r -d '' entry; do
    files+=("$entry")
  done < <(find "$base_abs" -type f ! -name '*.sha256' -print0)

  if ((${#files[@]} > 0)); then
    IFS=$'\n' files=($(printf '%s\n' "${files[@]}" | LC_ALL=C sort))
  fi

  : >"$manifest_abs"
  for file in "${files[@]}"; do
    local rel="${file#"$base_abs"/}"
    local sum
    sum=$(sha256sum "$file" | awk '{print $1}')
    printf '%s  %s\n' "$sum" "$rel" >>"$manifest_abs"
  done
}

verify_manifest() {
  local target="$1"
  local manifest="$2"
  local target_abs
  target_abs=$(abs_path "$target")
  local manifest_abs
  manifest_abs=$(abs_path "$manifest")

  if [[ ! -f "$manifest_abs" ]]; then
    echo "checksum manifest missing for $target" >&2
    return 1
  fi

  if [[ -d "$target_abs" ]]; then
    (
      cd "$target_abs"
      sha256sum --check --strict "$manifest_abs"
    )
  else
    local dir
    dir=$(dirname "$target_abs")
    (
      cd "$dir"
      sha256sum --check --strict "$manifest_abs"
    )
  fi
}

MODE=""
TARGETS=()

while (($#)); do
  case "$1" in
  --update|--verify)
    if [[ -n "$MODE" ]]; then
      echo "only one mode can be specified" >&2
      usage
      exit 1
    fi
    MODE="${1#--}"
    ;;
  --help|-h)
    usage
    exit 0
    ;;
  -*)
    echo "unknown flag: $1" >&2
    usage
    exit 1
    ;;
  *)
    TARGETS+=("$1")
    ;;
  esac
  shift
done

if [[ -z "$MODE" ]]; then
  echo "either --update or --verify must be provided" >&2
  usage
  exit 1
fi

if ((${#TARGETS[@]} == 0)); then
  echo "at least one path must be provided" >&2
  usage
  exit 1
fi

for target in "${TARGETS[@]}"; do
  base="${target%/}"
  if [[ ! -e "$base" ]]; then
    echo "target does not exist: $target" >&2
    exit 1
  fi
  manifest="${base}.sha256"
  if [[ "$MODE" == "update" ]]; then
    generate_manifest "$base" "$manifest"
    echo "updated $manifest"
  else
    verify_manifest "$base" "$manifest"
  fi
done
