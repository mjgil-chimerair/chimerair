#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

placeholder_root="$tmpdir/placeholder-zig"
mkdir -p "$placeholder_root"
touch "$placeholder_root/PLACEHOLDER.txt"

CHIMERA_ZIG_ROOT="$placeholder_root" \
  bash scripts/run-zig-release-integration.sh allow-missing

if CHIMERA_ZIG_ROOT="$placeholder_root" \
    bash scripts/run-zig-release-integration.sh require-authoritative; then
  echo "expected authoritative mode to fail for placeholder Zig checkout" >&2
  exit 1
fi

real_root="$tmpdir/real-zig"
bash scripts/setup-authoritative-zig-fixture.sh "$real_root"

CHIMERA_ZIG_ROOT="$real_root" \
  bash scripts/run-zig-release-integration.sh require-authoritative

scriptless_root="$tmpdir/scriptless-zig"
mkdir -p "$scriptless_root/build/stage3/bin" "$scriptless_root/.git"
touch "$scriptless_root/CMakeLists.txt"
cp "$real_root/build/stage3/bin/zig" "$scriptless_root/build/stage3/bin/zig"
if CHIMERA_ZIG_ROOT="$scriptless_root" \
    bash scripts/run-zig-release-integration.sh require-authoritative; then
  echo "expected authoritative mode to fail without integration script" >&2
  exit 1
fi
