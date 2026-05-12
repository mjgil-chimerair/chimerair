#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

zig_root="$tmpdir/zig-root"
evidence_path="$tmpdir/evidence.json"
zip_path="$tmpdir/zig-authoritative-ci-evidence.zip"
expected_sha="$(git rev-parse HEAD)"

bash scripts/setup-authoritative-zig-fixture.sh "$zig_root"

GITHUB_JOB="zig-release-authoritative" \
GITHUB_REPOSITORY="mjgil/chimerair" \
GITHUB_RUN_ID="123456789" \
GITHUB_SHA="$expected_sha" \
GITHUB_SERVER_URL="https://github.com" \
GITHUB_WORKFLOW="Chimera CI" \
CHIMERA_ZIG_GIT_URL="https://github.com/mjgil/zigmera-zig.git" \
CHIMERA_ZIG_GIT_REF="zigmera/snapshot-v1" \
CHIMERA_ZIG_ROOT="$zig_root" \
CHIMERA_ZIG_BIN="$zig_root/build/stage3/bin/zig" \
  python3 scripts/write-zig-authoritative-ci-evidence.py "$evidence_path"

python3 scripts/check-zig-authoritative-ci-artifact.py \
  "$evidence_path" \
  --expected-zig-ref "zigmera/snapshot-v1"

(
  cd "$tmpdir"
  zip -q "$(basename "$zip_path")" "$(basename "$evidence_path")"
)

python3 scripts/check-zig-authoritative-ci-artifact.py \
  "$zip_path" \
  --expected-zig-ref "zigmera/snapshot-v1"

if python3 scripts/check-zig-authoritative-ci-artifact.py \
  "$evidence_path" \
  --expected-sha "ffffffffffffffffffffffffffffffffffffffff"
then
  echo "expected artifact verification to fail for the wrong commit SHA" >&2
  exit 1
fi

if python3 scripts/check-zig-authoritative-ci-artifact.py \
  "$evidence_path" \
  --expected-zig-ref "zigmera/other-ref"
then
  echo "expected artifact verification to fail for the wrong Zig ref" >&2
  exit 1
fi
