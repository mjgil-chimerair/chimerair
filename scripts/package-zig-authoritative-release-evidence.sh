#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

usage() {
  cat <<'EOF'
usage: package-zig-authoritative-release-evidence.sh <archive-dir> [--output-dir <dir>]

Validates an archived authoritative Zig release-evidence directory, renders a
Markdown sign-off report beside it, and packages the archive plus report into a
tar.gz bundle for release retention.
EOF
}

if [[ $# -lt 1 ]]; then
  usage >&2
  exit 1
fi

archive_dir="$1"
shift

output_dir=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-dir)
      output_dir="${2:-}"
      if [[ -z "$output_dir" ]]; then
        echo "ERROR: --output-dir requires a value" >&2
        exit 1
      fi
      shift 2
      ;;
    *)
      echo "ERROR: unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

archive_dir="$(python3 -c 'import pathlib,sys; print(pathlib.Path(sys.argv[1]).resolve())' "$archive_dir")"
archive_name="$(basename "$archive_dir")"
output_dir="${output_dir:-$archive_dir}"
output_dir="$(python3 -c 'import pathlib,sys; print(pathlib.Path(sys.argv[1]).resolve())' "$output_dir")"

mkdir -p "$output_dir"

python3 scripts/check-archived-zig-authoritative-release-evidence.py "$archive_dir"

report_path="$output_dir/zig-authoritative-release-evidence-report.md"
bundle_path="$output_dir/zig-authoritative-release-evidence-bundle.tar.gz"

python3 scripts/render-zig-authoritative-release-evidence-report.py "$archive_dir" >"$report_path"

tar_args=(
  -czf "$bundle_path"
  -C "$(dirname "$archive_dir")"
)

if [[ "$output_dir" == "$archive_dir" ]]; then
  tar_args+=(
    --exclude "$archive_name/$(basename "$report_path")"
    --exclude "$archive_name/$(basename "$bundle_path")"
  )
fi

tar_args+=(
  "$archive_name"
  -C "$output_dir"
  "$(basename "$report_path")"
)

tar "${tar_args[@]}"

echo "Packaged authoritative Zig release evidence at $bundle_path"
