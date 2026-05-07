#!/usr/bin/env python3
"""Validate a retained authoritative Zig sign-off directory."""

from __future__ import annotations

import argparse
import io
import json
import sys
import tarfile
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Validate a retained authoritative Zig sign-off directory."
    )
    parser.add_argument(
        "signoff_dir",
        help="Path to .../zig-authoritative/<sha>/signoff-run-<id>",
    )
    args = parser.parse_args()

    signoff_dir = Path(args.signoff_dir).resolve()
    report_path = signoff_dir / "zig-authoritative-release-evidence-report.md"
    bundle_path = signoff_dir / "zig-authoritative-release-evidence-bundle.tar.gz"

    if not report_path.is_file():
        print(f"ERROR: missing sign-off report: {report_path}", file=sys.stderr)
        return 1
    if not bundle_path.is_file():
        print(f"ERROR: missing sign-off bundle: {bundle_path}", file=sys.stderr)
        return 1

    expected_sha = signoff_dir.parent.name
    if len(expected_sha) != 40:
        print(
            f"ERROR: sign-off parent directory does not look like a git SHA: {expected_sha}",
            file=sys.stderr,
        )
        return 1

    report_text = report_path.read_text()
    if "# Zig Authoritative Release Evidence" not in report_text:
        print("ERROR: sign-off report is missing the expected title", file=sys.stderr)
        return 1
    if f"- Release SHA: `{expected_sha}`" not in report_text:
        print("ERROR: sign-off report release SHA does not match parent directory", file=sys.stderr)
        return 1

    with tarfile.open(bundle_path, "r:gz") as archive:
        names = set(archive.getnames())
        manifest_name = next(
            (name for name in names if name.endswith("/archive-manifest.json")),
            None,
        )
        artifact_name = next(
            (name for name in names if name.endswith("/zig-authoritative-ci-evidence.json")),
            None,
        )

        if "zig-authoritative-release-evidence-report.md" not in names:
            print("ERROR: sign-off bundle is missing the rendered report", file=sys.stderr)
            return 1
        if manifest_name is None:
            print("ERROR: sign-off bundle is missing archive-manifest.json", file=sys.stderr)
            return 1
        if artifact_name is None:
            print(
                "ERROR: sign-off bundle is missing zig-authoritative-ci-evidence.json",
                file=sys.stderr,
            )
            return 1

        manifest_member = archive.extractfile(manifest_name)
        if manifest_member is None:
            print("ERROR: failed to read archive-manifest.json from bundle", file=sys.stderr)
            return 1
        manifest = json.load(io.TextIOWrapper(manifest_member, encoding="utf-8"))

    if manifest.get("git_sha") != expected_sha:
        print(
            "ERROR: sign-off bundle manifest SHA does not match parent directory",
            file=sys.stderr,
        )
        return 1

    expected_run_dir = f"run-{manifest.get('run_id', '')}"
    if manifest_name.split("/", 1)[0] != expected_run_dir:
        print(
            "ERROR: sign-off bundle run directory does not match manifest run_id",
            file=sys.stderr,
        )
        return 1

    print("Authoritative Zig sign-off directory is valid.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
