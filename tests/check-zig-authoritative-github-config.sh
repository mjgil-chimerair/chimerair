#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

mockbin="$tmpdir/mockbin"
mkdir -p "$mockbin"

run_with_mock() {
  local vars_json="$1"
  local secrets_json="$2"

  cat >"$mockbin/gh" <<EOF
#!/usr/bin/env bash
set -euo pipefail
if [[ "\$1" == "variable" && "\$2" == "list" ]]; then
  printf '%s\n' '$vars_json'
  exit 0
fi
if [[ "\$1" == "secret" && "\$2" == "list" ]]; then
  printf '%s\n' '$secrets_json'
  exit 0
fi
echo "unexpected gh invocation: \$*" >&2
exit 1
EOF
  chmod +x "$mockbin/gh"

  PATH="$mockbin:$PATH" bash scripts/check-zig-authoritative-github-config.sh
}

run_with_mock \
  '[{"name":"CHIMERA_ZIG_GIT_URL"},{"name":"CHIMERA_ZIG_GIT_REF"}]' \
  '[{"name":"CHIMERA_ZIG_GIT_TOKEN"}]'

if run_with_mock \
  '[{"name":"CHIMERA_ZIG_GIT_URL"}]' \
  '[{"name":"CHIMERA_ZIG_GIT_TOKEN"}]' \
  >"$tmpdir/missing-var.out" 2>"$tmpdir/missing-var.err"; then
  echo "expected missing variable check to fail" >&2
  exit 1
fi
grep -Fq "CHIMERA_ZIG_GIT_REF" "$tmpdir/missing-var.err"

if run_with_mock \
  '[{"name":"CHIMERA_ZIG_GIT_URL"},{"name":"CHIMERA_ZIG_GIT_REF"}]' \
  '[]' \
  >"$tmpdir/missing-secret.out" 2>"$tmpdir/missing-secret.err"; then
  echo "expected missing secret check to fail" >&2
  exit 1
fi
grep -Fq "CHIMERA_ZIG_GIT_TOKEN" "$tmpdir/missing-secret.err"
