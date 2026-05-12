#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

source_root="$tmpdir/source-zig"
clone_root="$tmpdir/cloned-zig"

bash scripts/setup-authoritative-zig-fixture.sh "$source_root"
git -C "$source_root" init >/dev/null
git -C "$source_root" config user.name "Chimera Test"
git -C "$source_root" config user.email "test@example.com"
git -C "$source_root" add .
git -C "$source_root" commit -m "fixture" >/dev/null

CHIMERA_ZIG_GIT_URL="$source_root" \
CHIMERA_ZIG_GIT_REF="HEAD" \
  bash scripts/prepare-zig-authoritative-checkout.sh "$clone_root"

test -d "$clone_root/.git"
test -f "$clone_root/CMakeLists.txt"
test -x "$clone_root/build/stage3/bin/zig"
test -x "$clone_root/scripts/test-zigmera.sh"

CHIMERA_ZIG_ROOT="$clone_root" \
  bash scripts/run-zig-release-integration.sh require-authoritative

if bash scripts/prepare-zig-authoritative-checkout.sh "$tmpdir/should-fail"; then
  echo "expected prepare-zig-authoritative-checkout.sh to fail without CHIMERA_ZIG_GIT_URL" >&2
  exit 1
fi
