#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

mockbin="$tmpdir/mockbin"
mkdir -p "$mockbin"

run_with_mock() {
  local auth_mode="$1"
  local vars_json="$2"
  local secrets_json="$3"

  cat >"$mockbin/gh" <<EOF
#!/usr/bin/env bash
set -euo pipefail
if [[ "\$1" == "auth" && "\$2" == "status" ]]; then
  if [[ "$auth_mode" == "fail" ]]; then
    echo "not logged in" >&2
    exit 1
  fi
  exit 0
fi
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

  PATH="$mockbin:$PATH" bash scripts/check-zig-authoritative-operator-readiness.sh
}

run_with_mock \
  ok \
  '[{"name":"CHIMERA_ZIG_GIT_URL"},{"name":"CHIMERA_ZIG_GIT_REF"}]' \
  '[{"name":"CHIMERA_ZIG_GIT_TOKEN"}]'

if run_with_mock \
  fail \
  '[{"name":"CHIMERA_ZIG_GIT_URL"},{"name":"CHIMERA_ZIG_GIT_REF"}]' \
  '[{"name":"CHIMERA_ZIG_GIT_TOKEN"}]' \
  >"$tmpdir/auth-fail.out" 2>"$tmpdir/auth-fail.err"; then
  echo "expected auth readiness check to fail" >&2
  exit 1
fi
grep -Fq "not logged in" "$tmpdir/auth-fail.err"
