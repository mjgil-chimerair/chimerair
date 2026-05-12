#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

manifest_path="$tmpdir/dry-run.json"

cat >"$manifest_path" <<EOF
{
  "base_dir": "/tmp/chimerair-release-evidence",
  "expected_sha": "1111111111111111111111111111111111111111",
  "expected_zig_ref": "zigmera/snapshot-v1",
  "mode": "dry-run",
  "ref": "main"
}
EOF

python3 scripts/check-zig-authoritative-dry-run-manifest.py "$manifest_path"

cat >"$manifest_path" <<EOF
{
  "base_dir": "/tmp/chimerair-release-evidence",
  "expected_sha": "short",
  "expected_zig_ref": "zigmera/snapshot-v1",
  "mode": "dry-run",
  "ref": "main"
}
EOF

if python3 scripts/check-zig-authoritative-dry-run-manifest.py "$manifest_path" \
  >"$tmpdir/invalid.out" 2>"$tmpdir/invalid.err"; then
  echo "expected invalid dry-run manifest to fail" >&2
  exit 1
fi
grep -Fq "expected_sha is not a 40-character git SHA" "$tmpdir/invalid.err"
