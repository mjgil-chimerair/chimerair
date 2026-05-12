#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

zig_root="$tmpdir/zig-root"
mockbin="$tmpdir/mockbin"
archive_root="$tmpdir/archive-root"
expected_sha="0123456789abcdef0123456789abcdef01234567"
run_id="123456789"
sha_dir="$archive_root/zig-authoritative/$expected_sha"
archive_dir="$sha_dir/run-$run_id"
signoff_dir="$sha_dir/signoff-run-$run_id"
evidence_path="$tmpdir/zig-authoritative-ci-evidence.json"
dry_run_manifest="$tmpdir/dry-run.json"
session_manifest="$tmpdir/session.json"

mkdir -p "$mockbin"
bash scripts/setup-authoritative-zig-fixture.sh "$zig_root"

cat >"$dry_run_manifest" <<EOF
{
  "base_dir": "$archive_root",
  "expected_sha": "$expected_sha",
  "expected_zig_ref": "zigmera/snapshot-v1",
  "mode": "dry-run",
  "ref": "main"
}
EOF

GITHUB_JOB="zig-release-authoritative" \
GITHUB_REPOSITORY="mjgil/chimerair" \
GITHUB_RUN_ID="$run_id" \
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
    "$run_id" \
    --expected-zig-ref "zigmera/snapshot-v1" \
    --expected-sha "$expected_sha" \
    --base-dir "$archive_root" >/dev/null

bash scripts/package-zig-authoritative-release-evidence.sh "$archive_dir" --output-dir "$signoff_dir" >/dev/null

cat >"$session_manifest" <<EOF
{
  "archive_dir": "$archive_dir",
  "base_dir": "$archive_root",
  "dry_run_manifest": "$dry_run_manifest",
  "expected_sha": "$expected_sha",
  "expected_zig_ref": "zigmera/snapshot-v1",
  "mode": "authoritative-release",
  "ref": "main",
  "run_id": "$run_id",
  "signoff_dir": "$signoff_dir"
}
EOF

python3 scripts/check-zig-authoritative-session-record.py "$session_manifest" >"$tmpdir/ok.log"
grep -Fq "Authoritative Zig session record is valid for run $run_id." "$tmpdir/ok.log"

python3 - "$session_manifest" <<'PY'
import json
import sys
from pathlib import Path

path = Path(sys.argv[1])
data = json.loads(path.read_text())
data["signoff_dir"] = str(Path(data["signoff_dir"]).parent / "signoff-run-999999999")
path.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n")
PY

if python3 scripts/check-zig-authoritative-session-record.py "$session_manifest" >"$tmpdir/bad.log" 2>&1; then
  echo "expected invalid session record to fail" >&2
  exit 1
fi
grep -Fq "authoritative session-manifest validation failed" "$tmpdir/bad.log"
