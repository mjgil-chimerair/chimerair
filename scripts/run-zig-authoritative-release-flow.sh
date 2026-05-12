#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

ref="main"
expected_zig_ref=""
base_dir=".artifacts/release-evidence"
dry_run=false
dry_run_output=""
dry_run_manifest=""
session_output=""

usage() {
  cat <<'EOF'
usage: run-zig-authoritative-release-flow.sh [--ref <git-ref>] --expected-zig-ref <ref> [--base-dir <dir>] [--dry-run] [--dry-run-output <path>] [--dry-run-manifest <path>] [--session-output <path>]

Dispatches the Chimera CI workflow, resolves the latest run ID for the chosen
ref, waits for completion, finalizes authoritative Zig release evidence, and
packages plus validates the retained release record. With --dry-run, only
operator readiness and release-SHA resolution are checked.
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
    --dry-run)
      dry_run=true
      shift
      ;;
    --dry-run-output)
      dry_run_output="${2:-}"
      if [[ -z "$dry_run_output" ]]; then
        echo "ERROR: --dry-run-output requires a value" >&2
        exit 1
      fi
      shift 2
      ;;
    --dry-run-manifest)
      dry_run_manifest="${2:-}"
      if [[ -z "$dry_run_manifest" ]]; then
        echo "ERROR: --dry-run-manifest requires a value" >&2
        exit 1
      fi
      shift 2
      ;;
    --session-output)
      session_output="${2:-}"
      if [[ -z "$session_output" ]]; then
        echo "ERROR: --session-output requires a value" >&2
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

if [[ "$dry_run" != "true" && -n "$dry_run_output" ]]; then
  echo "ERROR: --dry-run-output requires --dry-run" >&2
  exit 1
fi

if [[ "$dry_run" == "true" && -n "$dry_run_manifest" ]]; then
  echo "ERROR: --dry-run-manifest cannot be used with --dry-run" >&2
  exit 1
fi

if [[ "$dry_run" == "true" && -n "$session_output" ]]; then
  echo "ERROR: --session-output cannot be used with --dry-run" >&2
  exit 1
fi

if [[ -z "$expected_zig_ref" ]]; then
  echo "ERROR: --expected-zig-ref is required" >&2
  usage >&2
  exit 1
fi

if ! command -v gh >/dev/null 2>&1; then
  echo "ERROR: GitHub CLI 'gh' is required" >&2
  exit 1
fi

expected_sha="$(git rev-parse "$ref")"
bash scripts/check-zig-authoritative-operator-readiness.sh

if [[ "$dry_run" == "true" ]]; then
  if [[ -n "$dry_run_output" ]]; then
    mkdir -p "$(dirname "$dry_run_output")"
    python3 - "$dry_run_output" "$ref" "$expected_sha" "$expected_zig_ref" "$base_dir" <<'PY'
import json
import sys
from pathlib import Path

output_path = Path(sys.argv[1])
data = {
    "mode": "dry-run",
    "ref": sys.argv[2],
    "expected_sha": sys.argv[3],
    "expected_zig_ref": sys.argv[4],
    "base_dir": sys.argv[5],
}
output_path.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n")
PY
  fi
  echo "Authoritative Zig flow dry run passed."
  echo "Ref: $ref"
  echo "Expected SHA: $expected_sha"
  echo "Expected Zig Ref: $expected_zig_ref"
  echo "Base dir: $base_dir"
  exit 0
fi

bash scripts/dispatch-zig-authoritative-ci.sh --ref "$ref"

run_id="$(gh run list --workflow .github/workflows/ci.yml --branch "$ref" --limit 1 --json databaseId | python3 -c 'import json, sys; data=json.load(sys.stdin); print(data[0]["databaseId"])')"
if [[ -z "$run_id" ]]; then
  echo "ERROR: failed to resolve a workflow run ID for ref '$ref'" >&2
  exit 1
fi

gh run watch "$run_id"
python3 scripts/check-zig-authoritative-run-status.py "$run_id"
bash scripts/finalize-zig-authoritative-release-evidence.sh "$run_id" --expected-zig-ref "$expected_zig_ref" --expected-sha "$expected_sha" --base-dir "$base_dir"

archive_dir="$base_dir/zig-authoritative/$expected_sha/run-$run_id"
signoff_dir="$base_dir/zig-authoritative/$expected_sha/signoff-run-$run_id"
bash scripts/package-zig-authoritative-release-evidence.sh "$archive_dir" --output-dir "$signoff_dir"
python3 scripts/check-zig-authoritative-release-record.py "$base_dir/zig-authoritative/$expected_sha" "$run_id"
if [[ -n "$dry_run_manifest" ]]; then
  python3 scripts/check-zig-authoritative-plan-match.py "$dry_run_manifest" "$base_dir/zig-authoritative/$expected_sha" "$run_id"
fi
if [[ -n "$session_output" ]]; then
  mkdir -p "$(dirname "$session_output")"
  python3 - "$session_output" "$ref" "$expected_sha" "$expected_zig_ref" "$base_dir" "$run_id" "$archive_dir" "$signoff_dir" "$dry_run_manifest" <<'PY'
import json
import sys
from pathlib import Path

output_path = Path(sys.argv[1])
data = {
    "mode": "authoritative-release",
    "ref": sys.argv[2],
    "expected_sha": sys.argv[3],
    "expected_zig_ref": sys.argv[4],
    "base_dir": sys.argv[5],
    "run_id": sys.argv[6],
    "archive_dir": sys.argv[7],
    "signoff_dir": sys.argv[8],
    "dry_run_manifest": sys.argv[9],
}
output_path.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n")
PY
  echo "Wrote authoritative Zig session manifest to $session_output"
fi
