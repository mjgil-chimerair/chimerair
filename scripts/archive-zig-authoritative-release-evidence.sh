#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

base_dir=".artifacts/release-evidence"
expected_zig_ref=""
expected_sha="$(git rev-parse HEAD)"

usage() {
  cat <<'EOF'
usage: archive-zig-authoritative-release-evidence.sh <run-id> [--expected-zig-ref <ref>] [--expected-sha <sha>] [--base-dir <dir>]

Downloads and verifies the authoritative Zig CI artifact for a GitHub Actions
run, then archives the verified artifact under:

  <base-dir>/zig-authoritative/<git-sha>/run-<run-id>/
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

target_dir="$base_dir/zig-authoritative/$expected_sha/run-$run_id"
artifact_path="$target_dir/zig-authoritative-ci-evidence.json"
manifest_path="$target_dir/archive-manifest.json"

mkdir -p "$target_dir"

args=(bash scripts/fetch-zig-authoritative-ci-artifact.sh "$run_id" --output-path "$artifact_path")
if [[ -n "$expected_zig_ref" ]]; then
  args+=(--expected-zig-ref "$expected_zig_ref")
fi
args+=(--expected-sha "$expected_sha")

"${args[@]}"

python3 - "$manifest_path" "$run_id" "$expected_sha" "$artifact_path" "$expected_zig_ref" <<'PY'
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

manifest_path = Path(sys.argv[1])
run_id = sys.argv[2]
expected_sha = sys.argv[3]
artifact_path = sys.argv[4]
expected_zig_ref = sys.argv[5]

data = {
    "schema_version": 1,
    "archived_at_utc": datetime.now(timezone.utc).isoformat().replace("+00:00", "Z"),
    "run_id": run_id,
    "git_sha": expected_sha,
    "artifact_path": artifact_path,
    "expected_zig_ref": expected_zig_ref,
}

manifest_path.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n")
PY

echo "Archived authoritative Zig release evidence to $target_dir"
