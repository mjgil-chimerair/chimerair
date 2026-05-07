#!/usr/bin/env python3
"""Verify that a GitHub Actions run completed successfully and that the
zig-release-authoritative job itself succeeded."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Check that a GitHub Actions run contains a successful zig-release-authoritative job."
    )
    parser.add_argument("run_id", help="GitHub Actions run ID")
    args = parser.parse_args()

    result = subprocess.run(
        [
            "gh",
            "run",
            "view",
            args.run_id,
            "--json",
            "conclusion,jobs,url",
        ],
        check=True,
        capture_output=True,
        text=True,
    )
    data = json.loads(result.stdout)

    if data.get("conclusion") != "success":
        print(
            f"ERROR: workflow run {args.run_id} did not succeed: conclusion={data.get('conclusion')!r}",
            file=sys.stderr,
        )
        return 1

    target_job = None
    for job in data.get("jobs", []):
        if job.get("name") == "zig-release-authoritative":
            target_job = job
            break

    if target_job is None:
        print(
            f"ERROR: workflow run {args.run_id} does not contain a zig-release-authoritative job",
            file=sys.stderr,
        )
        return 1

    if target_job.get("conclusion") != "success":
        print(
            "ERROR: zig-release-authoritative job did not succeed: "
            f"conclusion={target_job.get('conclusion')!r}",
            file=sys.stderr,
        )
        return 1

    print(f"Authoritative Zig run {args.run_id} completed successfully: {data.get('url', '')}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
