#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

python3 scripts/validate-proof-report.py \
  tools/crates/chimera-proof-bridge/fixtures/proof_report.json \
  tests/fixtures/proof-sidecar.chproof.json

cat >"$tmpdir/bad-proof.json" <<'EOF'
{
  "build_id": "broken",
  "target_triple": "",
  "target_ptr_width": 64,
  "target_endian": "little",
  "obligations": []
}
EOF

if python3 scripts/validate-proof-report.py "$tmpdir/bad-proof.json"; then
  echo "expected invalid proof report to fail validation" >&2
  exit 1
fi
