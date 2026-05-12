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
artifact_path="$target_dir/zig-authoritative-ci-evidence.json"
signoff_dir="$archive_root/zig-authoritative/$expected_sha/signoff-run-123456789"
bundle_path="$signoff_dir/zig-authoritative-release-evidence-bundle.tar.gz"
dry_run_manifest="$tmpdir/dry-run.json"
session_json="$tmpdir/session.json"
gh_log="$tmpdir/gh.log"
session_log="$tmpdir/session.log"

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
printf '%s\n' "\$*" >>"$gh_log"
if [[ "\$1" == "workflow" && "\$2" == "run" ]]; then
  exit 0
fi
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
if [[ "\$1" == "run" && "\$2" == "list" ]]; then
  printf '[{"databaseId":123456789}]\n'
  exit 0
fi
if [[ "\$1" == "run" && "\$2" == "watch" ]]; then
  exit 0
fi
if [[ "\$1" == "run" && "\$2" == "view" ]]; then
  printf '{"conclusion":"success","url":"https://github.com/mjgil/chimerair/actions/runs/123456789","jobs":[{"name":"zig-release-authoritative","conclusion":"success"}]}\n'
  exit 0
fi
if [[ "\$1" == "run" && "\$2" == "download" ]]; then
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
  exit 0
fi
echo "unexpected gh invocation: \$*" >&2
exit 1
EOF
chmod +x "$mockbin/gh"

PATH="$mockbin:$PATH" \
  bash scripts/run-zig-authoritative-release-session.sh \
    --ref main \
    --expected-zig-ref "zigmera/snapshot-v1" \
    --base-dir "$archive_root" \
    --dry-run-output "$dry_run_manifest" \
    --session-output "$session_json" \
    >"$session_log"

test -f "$dry_run_manifest"
test -f "$artifact_path"
test -f "$bundle_path"
python3 scripts/check-zig-authoritative-session-manifest.py "$session_json"
grep -Fq "Finalized authoritative Zig dry-run manifest at $dry_run_manifest" "$session_log"
grep -Fq "Authoritative Zig dry-run plan matches release record for run 123456789." "$session_log"
grep -Fq "Authoritative Zig session record is valid for run 123456789." "$session_log"
grep -Fq "workflow run .github/workflows/ci.yml --ref main" "$gh_log"
