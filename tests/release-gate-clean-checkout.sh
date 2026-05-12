#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

original_home="${HOME:-}"
original_elan_home="${ELAN_HOME:-${original_home}/.elan}"
original_rustup_home="${RUSTUP_HOME:-${original_home}/.rustup}"

tmpdir="$(mktemp -d "$ROOT_DIR/.tmp-release-gate.XXXXXX")"
worktree_dir="$tmpdir/checkout"
fake_zig_root="$tmpdir/fake-zig"
cargo_home="$tmpdir/cargo-home"
cargo_target_dir="$tmpdir/cargo-target"
home_dir="$tmpdir/home"

cleanup() {
  git worktree remove --force "$worktree_dir" >/dev/null 2>&1 || true
  rm -rf "$tmpdir"
}
trap cleanup EXIT

git worktree add --detach "$worktree_dir" HEAD >/dev/null

bash "$ROOT_DIR/scripts/setup-authoritative-zig-fixture.sh" "$fake_zig_root"
mkdir -p "$cargo_home" "$cargo_target_dir" "$home_dir"

(
  cd "$worktree_dir"
  HOME="$home_dir" \
  ELAN_HOME="$original_elan_home" \
  RUSTUP_HOME="$original_rustup_home" \
  CARGO_HOME="$cargo_home" \
  CARGO_TARGET_DIR="$cargo_target_dir" \
  CARGO_INCREMENTAL=0 \
  CARGO_BUILD_JOBS=1 \
  CHIMERA_ZIG_ROOT="$fake_zig_root" \
    bash scripts/release-gate.sh
)
