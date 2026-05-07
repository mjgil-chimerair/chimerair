#!/usr/bin/env python3
"""Validate that a retained dry-run plan matches a retained release record."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Check that an authoritative dry-run manifest matches a retained release record."
    )
    parser.add_argument("dry_run_manifest", help="Path to the retained dry-run manifest JSON")
    parser.add_argument("sha_dir", help="Path to .../zig-authoritative/<sha>")
    parser.add_argument("run_id", help="Run ID for the retained release record")
    args = parser.parse_args()

    repo_root = Path(__file__).resolve().parent.parent
    dry_run_check = repo_root / "scripts" / "check-zig-authoritative-dry-run-manifest.py"
    record_check = repo_root / "scripts" / "check-zig-authoritative-release-record.py"

    try:
        subprocess.run([sys.executable, str(dry_run_check), args.dry_run_manifest], check=True)
    except subprocess.CalledProcessError:
        print("ERROR: authoritative dry-run manifest validation failed", file=sys.stderr)
        return 1

    try:
        subprocess.run([sys.executable, str(record_check), args.sha_dir, args.run_id], check=True)
    except subprocess.CalledProcessError:
        print("ERROR: authoritative release record validation failed", file=sys.stderr)
        return 1

    dry_run_manifest = Path(args.dry_run_manifest).resolve()
    dry_run_data = json.loads(dry_run_manifest.read_text())

    sha_dir = Path(args.sha_dir).resolve()
    archive_manifest_path = sha_dir / f"run-{args.run_id}" / "archive-manifest.json"
    archive_manifest = json.loads(archive_manifest_path.read_text())

    expected_sha = dry_run_data["expected_sha"]
    if sha_dir.name != expected_sha:
        print("ERROR: dry-run expected_sha does not match release-record SHA directory", file=sys.stderr)
        return 1

    expected_sha_dir = (Path(dry_run_data["base_dir"]).resolve() / "zig-authoritative" / expected_sha)
    if expected_sha_dir != sha_dir:
        print("ERROR: dry-run base_dir does not match release-record root", file=sys.stderr)
        return 1

    if dry_run_data["expected_zig_ref"] != archive_manifest["expected_zig_ref"]:
        print("ERROR: dry-run expected_zig_ref does not match release-record zig ref", file=sys.stderr)
        return 1

    print(f"Authoritative Zig dry-run plan matches release record for run {args.run_id}.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
