#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

ref="main"

usage() {
  cat <<'EOF'
usage: dispatch-zig-authoritative-ci.sh [--ref <git-ref>]

Triggers the Chimera CI GitHub Actions workflow manually via `gh workflow run`
so the real `zig-release-authoritative` job can execute on the chosen ref.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --ref)
      ref="${2:-}"
      if [[ -z "$ref" ]]; then
        echo "ERROR: --ref requires a value" >&2
        exit 1
      fi
      shift 2
      ;;
    *)
      echo "ERROR: unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

if ! command -v gh >/dev/null 2>&1; then
  echo "ERROR: GitHub CLI 'gh' is required" >&2
  exit 1
fi

gh workflow run .github/workflows/ci.yml --ref "$ref"
echo "Triggered Chimera CI for ref '$ref'. Monitor the zig-release-authoritative job in GitHub Actions."
