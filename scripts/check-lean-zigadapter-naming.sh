#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

echo "=== Lean ZigAdapter Naming Check ==="

if rg -n \
  --glob '!scripts/check-lean-zigadapter-naming.sh' \
  "ZigAdapater|Adapater" \
  ChimeraProof docs scripts .github tools; then
  echo
  echo "FAILED: Found legacy ZigAdapater typo references."
  exit 1
fi

echo "PASSED: Canonical ZigAdapter naming is consistent."
