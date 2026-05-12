#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

base_dir=".artifacts/release-evidence"
expected_zig_ref=""
expected_sha="$(git rev-parse HEAD)"

usage() {
  cat <<'EOF'
usage: finalize-zig-authoritative-release-evidence.sh <run-id> [--expected-zig-ref <ref>] [--expected-sha <sha>] [--base-dir <dir>]

Downloads, verifies, archives, and re-validates authoritative Zig CI release
evidence for a GitHub Actions run.
EOF
}

if [[ $# -lt 1 ]]; then
  usage >&2
  exit 1
fi

run_id="$1"
shift

while [[ $# -gt 0 ]]; do
  case "$1" in
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
    --expected-sha)
      expected_sha="${2:-}"
      if [[ -z "$expected_sha" ]]; then
        echo "ERROR: --expected-sha requires a value" >&2
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

args=(bash scripts/archive-zig-authoritative-release-evidence.sh "$run_id" --base-dir "$base_dir")
if [[ -n "$expected_zig_ref" ]]; then
  args+=(--expected-zig-ref "$expected_zig_ref")
fi
args+=(--expected-sha "$expected_sha")
"${args[@]}"

archive_dir="$base_dir/zig-authoritative/$expected_sha/run-$run_id"
python3 scripts/check-archived-zig-authoritative-release-evidence.py "$archive_dir"

echo "Finalized authoritative Zig release evidence at $archive_dir"
