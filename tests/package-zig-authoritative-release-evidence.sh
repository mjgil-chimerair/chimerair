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
package_root="$tmpdir/package-root"
expected_sha="$(git rev-parse HEAD)"
target_dir="$archive_root/zig-authoritative/$expected_sha/run-123456789"
report_path="$package_root/zig-authoritative-release-evidence-report.md"
bundle_path="$package_root/zig-authoritative-release-evidence-bundle.tar.gz"

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

bash scripts/package-zig-authoritative-release-evidence.sh \
  "$target_dir" \
  --output-dir "$package_root"

test -f "$report_path"
test -f "$bundle_path"
grep -Fqx "# Zig Authoritative Release Evidence" "$report_path"
grep -Fq -- "- Run ID: \`123456789\`" "$report_path"

tar -tzf "$bundle_path" | grep -Fqx "run-123456789/"
tar -tzf "$bundle_path" | grep -Fqx "run-123456789/archive-manifest.json"
tar -tzf "$bundle_path" | grep -Fqx "run-123456789/zig-authoritative-ci-evidence.json"
tar -tzf "$bundle_path" | grep -Fqx "zig-authoritative-release-evidence-report.md"
