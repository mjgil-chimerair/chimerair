#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

zig_root="$tmpdir/zig-root"
evidence_path="$tmpdir/zig-authoritative-ci-evidence.json"
mockbin="$tmpdir/mockbin"
download_dir="$tmpdir/download"
retained_path="$tmpdir/retained/zig-authoritative-ci-evidence.json"
expected_sha="1111111111111111111111111111111111111111"

mkdir -p "$mockbin" "$download_dir"
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
if [[ "\$1" != "run" || "\$2" != "download" ]]; then
  echo "unexpected gh invocation: \$*" >&2
  exit 1
fi
run_id="\$3"
if [[ "\$run_id" != "123456789" ]]; then
  echo "unexpected run id: \$run_id" >&2
  exit 1
fi
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
  bash scripts/fetch-zig-authoritative-ci-artifact.sh \
    123456789 \
    --expected-zig-ref "zigmera/snapshot-v1" \
    --expected-sha "$expected_sha" \
    --output-path "$retained_path"

test -f "$retained_path"
cmp -s "$evidence_path" "$retained_path"

if PATH="$mockbin:$PATH" \
  bash scripts/fetch-zig-authoritative-ci-artifact.sh 123456789 --expected-zig-ref "zigmera/wrong-ref" --expected-sha "$expected_sha"
then
  echo "expected fetched artifact verification to fail for wrong Zig ref" >&2
  exit 1
fi
