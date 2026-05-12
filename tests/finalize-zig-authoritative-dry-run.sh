#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

mockbin="$tmpdir/mockbin"
gh_log="$tmpdir/gh.log"
flow_log="$tmpdir/flow.log"
dry_run_json="$tmpdir/dry-run.json"
expected_sha="$(git rev-parse HEAD)"

mkdir -p "$mockbin"

cat >"$mockbin/gh" <<EOF
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "\$*" >>"$gh_log"
if [[ "\$1" == "auth" && "\$2" == "status" ]]; then
  exit 0
fi
if [[ "\$1" == "variable" && "\$2" == "list" ]]; then
  printf '[{"name":"CHIMERA_ZIG_GIT_URL"},{"name":"CHIMERA_ZIG_GIT_REF"}]\n'
  exit 0
fi
if [[ "\$1" == "secret" && "\$2" == "list" ]]; then
  printf '[{"name":"CHIMERA_ZIG_GIT_TOKEN"}]\n'
  exit 0
fi
echo "unexpected gh invocation during finalized dry run: \$*" >&2
exit 1
EOF
chmod +x "$mockbin/gh"

PATH="$mockbin:$PATH" \
  bash scripts/finalize-zig-authoritative-dry-run.sh \
    --ref main \
    --expected-zig-ref "zigmera/snapshot-v1" \
    --base-dir "$tmpdir/archive-root" \
    --output-path "$dry_run_json" \
    >"$flow_log"

grep -Fq "Finalized authoritative Zig dry-run manifest at $dry_run_json" "$flow_log"
python3 scripts/check-zig-authoritative-dry-run-manifest.py "$dry_run_json"
grep -Fq "auth status" "$gh_log"
grep -Fq "variable list --json name" "$gh_log"
grep -Fq "secret list --json name" "$gh_log"
if grep -Fq "workflow run" "$gh_log"; then
  echo "finalized dry run should not dispatch the workflow" >&2
  exit 1
fi
grep -Fq "\"expected_sha\": \"$expected_sha\"" "$dry_run_json"
