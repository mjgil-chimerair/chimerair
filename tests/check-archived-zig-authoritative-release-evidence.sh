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
expected_sha="$(git rev-parse HEAD)"
target_dir="$archive_root/zig-authoritative/$expected_sha/run-123456789"
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
    --base-dir "$archive_root"

python3 scripts/check-archived-zig-authoritative-release-evidence.py "$target_dir"

python3 - "$manifest_path" <<'PY'
import json
import sys
from pathlib import Path

manifest_path = Path(sys.argv[1])
data = json.loads(manifest_path.read_text())
data["git_sha"] = "ffffffffffffffffffffffffffffffffffffffff"
manifest_path.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n")
PY

if python3 scripts/check-archived-zig-authoritative-release-evidence.py "$target_dir"; then
  echo "expected archived release evidence check to fail for wrong git_sha" >&2
  exit 1
fi
