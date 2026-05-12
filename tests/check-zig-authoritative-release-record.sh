#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

zig_root="$tmpdir/zig-root"
evidence_path="$tmpdir/zig-authoritative-ci-evidence.json"
mockbin="$tmpdir/mockbin"
archive_root="$tmpdir/archive-root"
expected_sha="1111111111111111111111111111111111111111"
sha_dir="$archive_root/zig-authoritative/$expected_sha"
archive_dir="$sha_dir/run-123456789"
signoff_dir="$sha_dir/signoff-run-123456789"

mkdir -p "$mockbin"
bash scripts/setup-authoritative-zig-fixture.sh "$zig_root"

GITHUB_JOB="zig-release-authoritative" \
GITHUB_REPOSITORY="mjgil/chimerair" \
GITHUB_RUN_ID="123456789" \
GITHUB_SHA="$expected_sha" \
GITHUB_SERVER_URL="https://github.com" \
GITHUB_WORKFLOW="Chimera CI" \
CHIMERA_ZIG_GIT_URL="https://github.com/mjgil/zigmera-zig.git" \
CHIMERA_ZIG_GIT_REF="zigmera/snapshot-v1" \
CHIMERA_ZIG_ROOT="$zig_root" \
CHIMERA_ZIG_BIN="$zig_root/build/stage3/bin/zig" \
  python3 scripts/write-zig-authoritative-ci-evidence.py "$evidence_path"

cat >"$mockbin/gh" <<EOF
#!/usr/bin/env bash
set -euo pipefail
dest=""
while [[ \$# -gt 0 ]]; do
  case "\$1" in
    -D)
      dest="\$2"
      shift 2
      ;;
    *)
      shift
      ;;
  esac
done
cp "$evidence_path" "\$dest/zig-authoritative-ci-evidence.json"
EOF
chmod +x "$mockbin/gh"

PATH="$mockbin:$PATH" \
  bash scripts/archive-zig-authoritative-release-evidence.sh \
    123456789 \
    --expected-zig-ref "zigmera/snapshot-v1" \
    --expected-sha "$expected_sha" \
    --base-dir "$archive_root"

bash scripts/package-zig-authoritative-release-evidence.sh \
  "$archive_dir" \
  --output-dir "$signoff_dir"

python3 scripts/check-zig-authoritative-release-record.py "$sha_dir" 123456789

mv "$signoff_dir" "$signoff_dir.bak"

if python3 scripts/check-zig-authoritative-release-record.py "$sha_dir" 123456789 \
  >"$tmpdir/missing-signoff.out" 2>"$tmpdir/missing-signoff.err"; then
  echo "expected missing sign-off directory validation to fail" >&2
  exit 1
fi
grep -Fq "authoritative sign-off directory validation failed" "$tmpdir/missing-signoff.err"
