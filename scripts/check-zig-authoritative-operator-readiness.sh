#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

if ! command -v gh >/dev/null 2>&1; then
  echo "ERROR: GitHub CLI 'gh' is required" >&2
  exit 1
fi

gh auth status >/dev/null
bash scripts/check-zig-authoritative-github-config.sh

echo "Authoritative Zig operator readiness checks passed."
