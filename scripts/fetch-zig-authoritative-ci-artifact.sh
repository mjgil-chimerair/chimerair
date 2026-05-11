#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

artifact_name="zig-authoritative-ci-evidence"
expected_zig_ref=""
expected_sha=""
output_path=""

usage() {
  cat <<'EOF'
usage: fetch-zig-authoritative-ci-artifact.sh <run-id> [--expected-zig-ref <ref>] [--expected-sha <sha>] [--output-path <path>]

Downloads the authoritative Zig CI evidence artifact for a GitHub Actions run
using `gh run download`, verifies it against the expected release revision, and
optionally copies the verified artifact to a caller-chosen output path.
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
    --output-path)
      output_path="${2:-}"
      if [[ -z "$output_path" ]]; then
        echo "ERROR: --output-path requires a value" >&2
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

if ! command -v gh >/dev/null 2>&1; then
  echo "ERROR: GitHub CLI 'gh' is required" >&2
  exit 1
fi

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

gh run download "$run_id" -n "$artifact_name" -D "$tmpdir"

artifact_path=""
if [[ -f "$tmpdir/${artifact_name}.json" ]]; then
  artifact_path="$tmpdir/${artifact_name}.json"
elif [[ -f "$tmpdir/${artifact_name}.zip" ]]; then
  artifact_path="$tmpdir/${artifact_name}.zip"
else
  while IFS= read -r path; do
    artifact_path="$path"
    break
  done < <(find "$tmpdir" -type f \( -name "${artifact_name}.json" -o -name "${artifact_name}.zip" \))
fi

if [[ -z "$artifact_path" ]]; then
  echo "ERROR: downloaded artifact did not contain ${artifact_name}.json or ${artifact_name}.zip" >&2
  exit 1
fi

args=(python3 scripts/check-zig-authoritative-ci-artifact.py "$artifact_path")
if [[ -n "$expected_zig_ref" ]]; then
  args+=(--expected-zig-ref "$expected_zig_ref")
fi
if [[ -n "$expected_sha" ]]; then
  args+=(--expected-sha "$expected_sha")
fi

"${args[@]}"

if [[ -n "$output_path" ]]; then
  mkdir -p "$(dirname "$output_path")"
  cp "$artifact_path" "$output_path"
fi
