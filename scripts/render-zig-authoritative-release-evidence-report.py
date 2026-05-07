#!/usr/bin/env python3
"""Render a compact Markdown report for archived authoritative Zig release evidence."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Render a Markdown report from an archived authoritative Zig release-evidence directory."
    )
    parser.add_argument("archive_dir", help="Path to .../zig-authoritative/<sha>/run-<id>")
    args = parser.parse_args()

    archive_dir = Path(args.archive_dir).resolve()
    repo_root = Path(__file__).resolve().parent.parent
    check_script = repo_root / "scripts" / "check-archived-zig-authoritative-release-evidence.py"

    subprocess.run([sys.executable, str(check_script), str(archive_dir)], check=True)

    manifest = json.loads((archive_dir / "archive-manifest.json").read_text())
    evidence = json.loads(Path(manifest["artifact_path"]).read_text())

    print("# Zig Authoritative Release Evidence")
    print()
    print(f"- Run ID: `{manifest['run_id']}`")
    print(f"- Release SHA: `{manifest['git_sha']}`")
    print(f"- Expected Zig Ref: `{manifest['expected_zig_ref']}`")
    print(f"- Artifact Path: `{manifest['artifact_path']}`")
    print(f"- Archived At: `{manifest['archived_at_utc']}`")
    print(f"- CI Workflow: `{evidence['workflow']}`")
    print(f"- CI Job: `{evidence['job']}`")
    print(f"- Run URL: {evidence['run_url']}")
    print(f"- Zig Repo URL: `{evidence['zig_repo_url']}`")
    print(f"- Zig Repo Ref: `{evidence.get('zig_repo_ref', '')}`")
    print(f"- Zig Version: `{evidence['zig_version']}`")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
