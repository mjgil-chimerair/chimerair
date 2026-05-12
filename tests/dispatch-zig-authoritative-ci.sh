#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

mockbin="$tmpdir/mockbin"
logfile="$tmpdir/gh.log"
mkdir -p "$mockbin"

cat >"$mockbin/gh" <<EOF
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "\$*" >"$logfile"
EOF
chmod +x "$mockbin/gh"

PATH="$mockbin:$PATH" bash scripts/dispatch-zig-authoritative-ci.sh --ref release/proofs

grep -Fqx "workflow run .github/workflows/ci.yml --ref release/proofs" "$logfile"
