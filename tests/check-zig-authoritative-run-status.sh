#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

mockbin="$tmpdir/mockbin"
mkdir -p "$mockbin"

run_with_mock() {
  local payload="$1"

  cat >"$mockbin/gh" <<EOF
#!/usr/bin/env bash
set -euo pipefail
if [[ "\$1" == "run" && "\$2" == "view" ]]; then
  printf '%s\n' '$payload'
  exit 0
fi
echo "unexpected gh invocation: \$*" >&2
exit 1
EOF
  chmod +x "$mockbin/gh"

  PATH="$mockbin:$PATH" python3 scripts/check-zig-authoritative-run-status.py 123456789
}

run_with_mock '{"conclusion":"success","url":"https://github.com/mjgil/chimerair/actions/runs/123456789","jobs":[{"name":"zig-release-authoritative","conclusion":"success"}]}'

if run_with_mock \
  '{"conclusion":"success","url":"https://github.com/mjgil/chimerair/actions/runs/123456789","jobs":[{"name":"zig-release-authoritative","conclusion":"failure"}]}' \
  >"$tmpdir/job-failure.out" 2>"$tmpdir/job-failure.err"; then
  echo "expected failed job check to fail" >&2
  exit 1
fi
grep -Fq "zig-release-authoritative job did not succeed" "$tmpdir/job-failure.err"

if run_with_mock \
  '{"conclusion":"success","url":"https://github.com/mjgil/chimerair/actions/runs/123456789","jobs":[{"name":"tools-test","conclusion":"success"}]}' \
  >"$tmpdir/missing-job.out" 2>"$tmpdir/missing-job.err"; then
  echo "expected missing job check to fail" >&2
  exit 1
fi
grep -Fq "does not contain a zig-release-authoritative job" "$tmpdir/missing-job.err"
