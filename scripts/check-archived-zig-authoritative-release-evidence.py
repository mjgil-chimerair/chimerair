#!/usr/bin/env python3
"""Validate an archived authoritative Zig release-evidence directory."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Validate an archived authoritative Zig release-evidence directory."
    )
    parser.add_argument("archive_dir", help="Path to .../zig-authoritative/<sha>/run-<id>")
    args = parser.parse_args()

    archive_dir = Path(args.archive_dir).resolve()
    manifest_path = archive_dir / "archive-manifest.json"

    if not manifest_path.is_file():
        print(f"ERROR: missing archive manifest: {manifest_path}", file=sys.stderr)
        return 1

    data = json.loads(manifest_path.read_text())

    required = (
        "schema_version",
        "archived_at_utc",
        "run_id",
        "git_sha",
        "artifact_path",
        "expected_zig_ref",
    )
    for field in required:
        if data.get(field, "") in ("", None):
            print(f"ERROR: missing or empty archive field: {field}", file=sys.stderr)
            return 1

    if data["schema_version"] != 1:
        print(f"ERROR: unsupported archive schema_version: {data['schema_version']}", file=sys.stderr)
        return 1

    expected_run_dir = f"run-{data['run_id']}"
    if archive_dir.name != expected_run_dir:
        print(
            f"ERROR: archive directory name mismatch: expected {expected_run_dir}, got {archive_dir.name}",
            file=sys.stderr,
        )
        return 1

    artifact_path = Path(data["artifact_path"])
    if not artifact_path.is_file():
        print(f"ERROR: archived artifact path does not exist: {artifact_path}", file=sys.stderr)
        return 1

    repo_root = Path(__file__).resolve().parent.parent
    check_script = repo_root / "scripts" / "check-zig-authoritative-ci-artifact.py"
    cmd = [
        sys.executable,
        str(check_script),
        str(artifact_path),
        "--expected-sha",
        data["git_sha"],
    ]
    if data.get("expected_zig_ref", ""):
        cmd.extend(["--expected-zig-ref", data["expected_zig_ref"]])

    try:
        subprocess.run(cmd, check=True)
    except subprocess.CalledProcessError:
        print("ERROR: archived authoritative Zig artifact verification failed", file=sys.stderr)
        return 1

    print("Archived authoritative Zig release evidence is valid.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
