#!/usr/bin/env python3
"""Validate the paired archive and sign-off directories for one authoritative run."""

from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Validate an authoritative Zig release record under .../zig-authoritative/<sha>."
    )
    parser.add_argument(
        "sha_dir",
        help="Path to .../zig-authoritative/<sha>",
    )
    parser.add_argument(
        "run_id",
        help="Run ID to validate",
    )
    args = parser.parse_args()

    sha_dir = Path(args.sha_dir).resolve()
    run_id = args.run_id
    archive_dir = sha_dir / f"run-{run_id}"
    signoff_dir = sha_dir / f"signoff-run-{run_id}"

    repo_root = Path(__file__).resolve().parent.parent
    archive_check = repo_root / "scripts" / "check-archived-zig-authoritative-release-evidence.py"
    signoff_check = repo_root / "scripts" / "check-zig-authoritative-signoff-dir.py"

    try:
        subprocess.run([sys.executable, str(archive_check), str(archive_dir)], check=True)
    except subprocess.CalledProcessError:
        print("ERROR: authoritative archive directory validation failed", file=sys.stderr)
        return 1

    try:
        subprocess.run([sys.executable, str(signoff_check), str(signoff_dir)], check=True)
    except subprocess.CalledProcessError:
        print("ERROR: authoritative sign-off directory validation failed", file=sys.stderr)
        return 1

    print(f"Authoritative Zig release record is valid for run {run_id}.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
