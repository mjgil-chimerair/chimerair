#!/usr/bin/env python3
"""Validate that an authoritative session manifest matches a valid release record."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


def fail(message: str) -> int:
    print(f"ERROR: {message}", file=sys.stderr)
    return 1


def main() -> int:
    if len(sys.argv) != 2:
        return fail("usage: check-zig-authoritative-session-record.py <session-manifest>")

    manifest_path = Path(sys.argv[1]).resolve()
    repo_root = Path(__file__).resolve().parent.parent
    session_manifest_check = repo_root / "scripts" / "check-zig-authoritative-session-manifest.py"
    release_record_check = repo_root / "scripts" / "check-zig-authoritative-release-record.py"

    try:
        subprocess.run([sys.executable, str(session_manifest_check), str(manifest_path)], check=True)
    except subprocess.CalledProcessError:
        return fail("authoritative session-manifest validation failed")

    try:
        data = json.loads(manifest_path.read_text())
    except json.JSONDecodeError as exc:
        return fail(f"invalid JSON in session manifest: {exc}")

    base_dir = Path(data["base_dir"]).resolve()
    expected_sha = data["expected_sha"]
    run_id = data["run_id"]
    sha_dir = base_dir / "zig-authoritative" / expected_sha

    try:
        subprocess.run([sys.executable, str(release_record_check), str(sha_dir), run_id], check=True)
    except subprocess.CalledProcessError:
        return fail("authoritative release-record validation failed for session manifest")

    print(f"Authoritative Zig session record is valid for run {run_id}.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
