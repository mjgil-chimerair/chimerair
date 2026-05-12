#!/usr/bin/env bash
# Validate patched-Zig release-gate prerequisites and smoke integration hooks.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MODE="${1:-auto}"

usage() {
  cat <<'EOF'
Usage: run-zig-release-integration.sh [auto|allow-missing|require-authoritative]

Modes:
  auto                 Allow placeholders in local/dev contract checks.
  allow-missing        Same as auto, but explicit.
  require-authoritative
                       Fail unless a usable patched Zig checkout and binary exist.
EOF
}

resolve_root() {
  local candidate
  for candidate in \
    "${CHIMERA_ZIG_ROOT:-}" \
    "$ROOT_DIR/third_party/zig" \
    "$ROOT_DIR/zigmera-zig"
  do
    if [[ -n "$candidate" && -d "$candidate" ]]; then
      printf '%s\n' "$candidate"
      return 0
    fi
  done
  return 1
}

is_placeholder_root() {
  local root="$1"
  [[ -f "$root/PLACEHOLDER.txt" ]]
}

resolve_bin() {
  local root="${1:-}"
  local candidate
  for candidate in \
    "${CHIMERA_ZIG_BIN:-}" \
    "$root/build/stage3/bin/zig" \
    "$root/build/bin/zig" \
    "$root/zig"
  do
    if [[ -n "$candidate" && -x "$candidate" ]]; then
      printf '%s\n' "$candidate"
      return 0
    fi
  done
  return 1
}

has_authoritative_checkout_shape() {
  local root="$1"
  [[ -d "$root/.git" ]] || return 1
  [[ -f "$root/CMakeLists.txt" || -f "$root/build.zig" || -f "$root/src/Compilation.zig" || -x "$root/scripts/test-zigmera.sh" ]]
}

integration_script_path() {
  local root="$1"
  local candidate
  for candidate in \
    "${CHIMERA_ZIG_TEST_SCRIPT:-}" \
    "$root/scripts/test-zigmera.sh"
  do
    if [[ -n "$candidate" && -x "$candidate" ]]; then
      printf '%s\n' "$candidate"
      return 0
    fi
  done
  return 1
}

require_help_flag() {
  local help_text="$1"
  local flag="$2"
  if [[ "$help_text" != *"$flag"* ]]; then
    echo "ERROR: patched Zig help text is missing required flag: $flag" >&2
    exit 1
  fi
}

case "$MODE" in
  auto|allow-missing|require-authoritative) ;;
  -h|--help)
    usage
    exit 0
    ;;
  *)
    echo "ERROR: unknown mode: $MODE" >&2
    usage >&2
    exit 1
    ;;
esac

root=""
if root="$(resolve_root)"; then
  :
fi

if [[ -z "$root" || "$(is_placeholder_root "${root:-/missing}" && echo yes || echo no)" == "yes" ]]; then
  if [[ "$MODE" == "require-authoritative" ]]; then
    echo "ERROR: authoritative patched Zig checkout is unavailable. See docs/zig-integration.md." >&2
    exit 1
  fi
  echo "Skipping patched Zig integration gate: authoritative checkout unavailable."
  exit 0
fi

if ! has_authoritative_checkout_shape "$root"; then
  if [[ "$MODE" == "require-authoritative" ]]; then
    echo "ERROR: patched Zig root does not look like a real source checkout: $root" >&2
    exit 1
  fi
  echo "Skipping patched Zig integration gate: authoritative source checkout shape unavailable."
  exit 0
fi

bin=""
if ! bin="$(resolve_bin "$root")"; then
  if [[ "$MODE" == "require-authoritative" ]]; then
    echo "ERROR: patched Zig binary not found under $root. See docs/zig-integration.md." >&2
    exit 1
  fi
  echo "Skipping patched Zig integration gate: patched Zig binary unavailable."
  exit 0
fi

version_output="$("$bin" version 2>/dev/null || true)"
help_output="$("$bin" --help 2>&1 || true)"

if [[ -z "$help_output" ]]; then
  echo "ERROR: failed to query patched Zig help output from $bin" >&2
  exit 1
fi

require_help_flag "$help_output" "emit-zigmera-snapshot"
require_help_flag "$help_output" "emit-zairpack"
require_help_flag "$help_output" "emit-zdep"

integration_script=""
if integration_script="$(integration_script_path "$root")"; then
  echo "==> bash $integration_script"
  CHIMERA_ZIG_BIN="$bin" bash "$integration_script"
elif [[ "$MODE" == "require-authoritative" ]]; then
  echo "ERROR: authoritative patched Zig checkout is missing scripts/test-zigmera.sh" >&2
  exit 1
else
  echo "Patched Zig smoke validated via CLI flags only."
fi

if [[ -n "$version_output" ]]; then
  echo "Patched Zig version: $version_output"
fi
