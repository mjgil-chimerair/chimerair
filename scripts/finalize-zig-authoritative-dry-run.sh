#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

ref="main"
expected_zig_ref=""
base_dir=".artifacts/release-evidence"
output_path=".artifacts/zig-authoritative-dry-run.json"

usage() {
  cat <<'EOF'
usage: finalize-zig-authoritative-dry-run.sh [--ref <git-ref>] --expected-zig-ref <ref> [--base-dir <dir>] [--output-path <path>]

Runs the authoritative Zig operator dry-run, writes a machine-readable
manifest, and validates that retained dry-run manifest.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --ref)
      ref="${2:-}"
      if [[ -z "$ref" ]]; then
        echo "ERROR: --ref requires a value" >&2
        exit 1
      fi
      shift 2
      ;;
    --expected-zig-ref)
      expected_zig_ref="${2:-}"
      if [[ -z "$expected_zig_ref" ]]; then
        echo "ERROR: --expected-zig-ref requires a value" >&2
        exit 1
      fi
      shift 2
      ;;
    --base-dir)
      base_dir="${2:-}"
      if [[ -z "$base_dir" ]]; then
        echo "ERROR: --base-dir requires a value" >&2
        exit 1
      fi
      shift 2
      ;;
    --output-path)
      output_path="${2:-}"
      if [[ -z "$output_path" ]]; then
        echo "ERROR: --output-path requires a value" >&2
        exit 1
      fi
      shift 2
      ;;
    *)
      echo "ERROR: unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

if [[ -z "$expected_zig_ref" ]]; then
  echo "ERROR: --expected-zig-ref is required" >&2
  usage >&2
  exit 1
fi

bash scripts/run-zig-authoritative-release-flow.sh \
  --ref "$ref" \
  --expected-zig-ref "$expected_zig_ref" \
  --base-dir "$base_dir" \
  --dry-run-output "$output_path" \
  --dry-run

python3 scripts/check-zig-authoritative-dry-run-manifest.py "$output_path"

echo "Finalized authoritative Zig dry-run manifest at $output_path"
