#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

if bash scripts/check-zig-authoritative-ci-config.sh; then
  echo "expected config validation to fail without CHIMERA_ZIG_GIT_URL" >&2
  exit 1
fi

CHIMERA_ZIG_GIT_URL="$tmpdir/local-zig" \
  bash scripts/check-zig-authoritative-ci-config.sh

if CHIMERA_ZIG_GIT_URL="https://github.com/example/private-zig.git" \
    bash scripts/check-zig-authoritative-ci-config.sh; then
  echo "expected config validation to fail for GitHub HTTPS without token" >&2
  exit 1
fi

CHIMERA_ZIG_GIT_URL="https://github.com/example/private-zig.git" \
CHIMERA_ZIG_GIT_TOKEN="secret-token" \
CHIMERA_ZIG_GIT_REF="zigmera/snapshot-v1" \
  bash scripts/check-zig-authoritative-ci-config.sh

if CHIMERA_ZIG_GIT_URL="$tmpdir/local-zig" \
    CHIMERA_ZIG_GIT_REF="bad ref" \
    bash scripts/check-zig-authoritative-ci-config.sh; then
  echo "expected config validation to fail for whitespace in ref" >&2
  exit 1
fi
