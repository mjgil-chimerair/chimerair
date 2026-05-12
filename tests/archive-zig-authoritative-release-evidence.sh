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
target_dir="$archive_root/zig-authoritative/$expected_sha/run-123456789"
artifact_path="$target_dir/zig-authoritative-ci-evidence.json"
manifest_path="$target_dir/archive-manifest.json"

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

test -f "$artifact_path"
test -f "$manifest_path"
cmp -s "$evidence_path" "$artifact_path"

python3 - "$manifest_path" "$expected_sha" <<'PY'
import json
import sys
from pathlib import Path

manifest_path = Path(sys.argv[1])
expected_sha = sys.argv[2]
data = json.loads(manifest_path.read_text())

assert data["schema_version"] == 1
assert data["run_id"] == "123456789"
assert data["git_sha"] == expected_sha
assert data["expected_zig_ref"] == "zigmera/snapshot-v1"
assert data["artifact_path"].endswith("zig-authoritative-ci-evidence.json")
PY
