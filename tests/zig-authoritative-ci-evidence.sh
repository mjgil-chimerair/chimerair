#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

zig_root="$tmpdir/zig-root"
evidence_path="$tmpdir/evidence.json"

bash scripts/setup-authoritative-zig-fixture.sh "$zig_root"

GITHUB_JOB="zig-release-authoritative" \
GITHUB_REPOSITORY="mjgil/chimerair" \
GITHUB_RUN_ID="123456789" \
GITHUB_SHA="0123456789abcdef0123456789abcdef01234567" \
GITHUB_SERVER_URL="https://github.com" \
GITHUB_WORKFLOW="Chimera CI" \
CHIMERA_ZIG_GIT_URL="https://github.com/mjgil/zigmera-zig.git" \
CHIMERA_ZIG_GIT_REF="zigmera/snapshot-v1" \
CHIMERA_ZIG_ROOT="$zig_root" \
CHIMERA_ZIG_BIN="$zig_root/build/stage3/bin/zig" \
  python3 scripts/write-zig-authoritative-ci-evidence.py "$evidence_path"

python3 scripts/validate-zig-authoritative-ci-evidence.py "$evidence_path"

bad_path="$tmpdir/bad-evidence.json"
cat >"$bad_path" <<'EOF'
{"schema_version": 2, "job": "", "repository": "mjgil/chimerair"}
EOF

if python3 scripts/validate-zig-authoritative-ci-evidence.py "$bad_path"; then
  echo "expected evidence validation to fail for missing fields" >&2
  exit 1
fi

local_path="$tmpdir/local-evidence.json"
cat >"$local_path" <<'EOF'
{
  "schema_version": 2,
  "artifact_name": "zig-authoritative-ci-evidence",
  "ci_provider": "github-actions",
  "checkout_mode": "authoritative_external",
  "generated_at_utc": "2026-05-07T00:00:00Z",
  "git_sha": "0123456789abcdef0123456789abcdef01234567",
  "job": "zig-release-authoritative",
  "repository": "mjgil/chimerair",
  "run_url": "https://github.com/mjgil/chimerair/actions/runs/123456789",
  "workflow": "Chimera CI",
  "integration_script": "scripts/test-zigmera.sh",
  "zig_repo_url": "/tmp/local-zig",
  "zig_root": "/tmp/zig-root",
  "zig_bin": "/tmp/zig-root/build/stage3/bin/zig",
  "zig_version": "0.13.0-zigmera"
}
EOF

if python3 scripts/validate-zig-authoritative-ci-evidence.py "$local_path"; then
  echo "expected evidence validation to fail for local zig_repo_url" >&2
  exit 1
fi
