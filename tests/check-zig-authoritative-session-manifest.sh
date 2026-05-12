#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

base_dir="$tmpdir/archive-root"
expected_sha="0123456789abcdef0123456789abcdef01234567"
run_id="123456789"
sha_dir="$base_dir/zig-authoritative/$expected_sha"
archive_dir="$sha_dir/run-$run_id"
signoff_dir="$sha_dir/signoff-run-$run_id"
dry_run_manifest="$tmpdir/dry-run.json"
session_manifest="$tmpdir/session.json"

mkdir -p "$archive_dir" "$signoff_dir"
cat >"$dry_run_manifest" <<EOF
{
  "base_dir": "$base_dir",
  "expected_sha": "$expected_sha",
  "expected_zig_ref": "zigmera/snapshot-v1",
  "mode": "dry-run",
  "ref": "main"
}
EOF

cat >"$session_manifest" <<EOF
{
  "archive_dir": "$archive_dir",
  "base_dir": "$base_dir",
  "dry_run_manifest": "$dry_run_manifest",
  "expected_sha": "$expected_sha",
  "expected_zig_ref": "zigmera/snapshot-v1",
  "mode": "authoritative-release",
  "ref": "main",
  "run_id": "$run_id",
  "signoff_dir": "$signoff_dir"
}
EOF

python3 scripts/check-zig-authoritative-session-manifest.py "$session_manifest" >"$tmpdir/ok.log"
grep -Fq "Authoritative Zig session manifest is valid." "$tmpdir/ok.log"

python3 - "$session_manifest" <<'PY'
import json
import sys
from pathlib import Path

path = Path(sys.argv[1])
data = json.loads(path.read_text())
data["run_id"] = "run-123456789"
path.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n")
PY

if python3 scripts/check-zig-authoritative-session-manifest.py "$session_manifest" >"$tmpdir/bad.log" 2>&1; then
  echo "expected invalid session manifest to fail" >&2
  exit 1
fi
grep -Fq "session manifest run_id must contain only decimal digits" "$tmpdir/bad.log"
