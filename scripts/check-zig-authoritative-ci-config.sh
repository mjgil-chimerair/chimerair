#!/usr/bin/env bash

set -euo pipefail

mode="${1:-require-configured}"
repo_url="${CHIMERA_ZIG_GIT_URL:-}"
repo_ref="${CHIMERA_ZIG_GIT_REF:-}"
repo_token="${CHIMERA_ZIG_GIT_TOKEN:-}"

case "$mode" in
  require-configured) ;;
  -h|--help)
    cat <<'EOF'
Usage: check-zig-authoritative-ci-config.sh [require-configured]
EOF
    exit 0
    ;;
  *)
    echo "ERROR: unknown mode: $mode" >&2
    exit 1
    ;;
esac

if [[ -z "$repo_url" ]]; then
  echo "ERROR: CHIMERA_ZIG_GIT_URL is required for the authoritative Zig CI job" >&2
  exit 1
fi

if [[ "$repo_url" =~ [[:space:]] ]]; then
  echo "ERROR: CHIMERA_ZIG_GIT_URL must not contain whitespace" >&2
  exit 1
fi

if [[ -n "$repo_ref" && "$repo_ref" =~ [[:space:]] ]]; then
  echo "ERROR: CHIMERA_ZIG_GIT_REF must not contain whitespace" >&2
  exit 1
fi

if [[ "$repo_url" =~ ^https://github\.com/ && -z "$repo_token" ]]; then
  echo "ERROR: CHIMERA_ZIG_GIT_TOKEN is required for GitHub HTTPS authoritative Zig checkouts" >&2
  exit 1
fi

echo "Authoritative Zig CI config validated."
