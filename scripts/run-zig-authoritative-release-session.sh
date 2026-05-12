#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

ref="main"
expected_zig_ref=""
base_dir=".artifacts/release-evidence"
dry_run_output=".artifacts/zig-authoritative-dry-run.json"
session_output=""

usage() {
  cat <<'EOF'
usage: run-zig-authoritative-release-session.sh [--ref <git-ref>] --expected-zig-ref <ref> [--base-dir <dir>] [--dry-run-output <path>] [--session-output <path>]

Finalizes the retained authoritative dry-run manifest, then runs the real
authoritative release flow using that same manifest for enforced plan-to-record
matching. Optionally retains and validates a machine-readable session manifest
for the real authoritative run.
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
    --dry-run-output)
      dry_run_output="${2:-}"
      if [[ -z "$dry_run_output" ]]; then
        echo "ERROR: --dry-run-output requires a value" >&2
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

if [[ -z "$expected_zig_ref" ]]; then
  echo "ERROR: --expected-zig-ref is required" >&2
  usage >&2
  exit 1
fi

flow_args=(
  --ref "$ref"
  --expected-zig-ref "$expected_zig_ref"
  --base-dir "$base_dir"
  --dry-run-manifest "$dry_run_output"
)

if [[ -n "$session_output" ]]; then
  flow_args+=(--session-output "$session_output")
fi

bash scripts/finalize-zig-authoritative-dry-run.sh \
  --ref "$ref" \
  --expected-zig-ref "$expected_zig_ref" \
  --base-dir "$base_dir" \
  --output-path "$dry_run_output"

bash scripts/run-zig-authoritative-release-flow.sh "${flow_args[@]}"

if [[ -n "$session_output" ]]; then
  python3 scripts/check-zig-authoritative-session-record.py "$session_output"
fi
