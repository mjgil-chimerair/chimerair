#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

echo "=== Lean ZigAdapter Naming Check ==="

if command -v rg >/dev/null 2>&1; then
  search_cmd=(
    rg -n
    --glob '!scripts/check-lean-zigadapter-naming.sh'
    "ZigAdapater|Adapater"
    ChimeraProof docs scripts .github tools
  )
else
  search_cmd=(
    grep -RInE
    --exclude=check-lean-zigadapter-naming.sh
    "ZigAdapater|Adapater"
    ChimeraProof docs scripts .github tools
  )
fi

if "${search_cmd[@]}"; then
  echo
  echo "FAILED: Found legacy ZigAdapater typo references."
  exit 1
fi

echo "PASSED: Canonical ZigAdapter naming is consistent."
