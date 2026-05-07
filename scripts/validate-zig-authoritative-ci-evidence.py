#!/usr/bin/env python3
"""Validate the authoritative Zig CI evidence record."""

from __future__ import annotations

import json
import sys
from pathlib import Path


REQUIRED_FIELDS = (
    "schema_version",
    "artifact_name",
    "ci_provider",
    "checkout_mode",
    "generated_at_utc",
    "git_sha",
    "job",
    "repository",
    "run_url",
    "workflow",
    "integration_script",
    "zig_repo_url",
    "zig_root",
    "zig_bin",
    "zig_version",
)

EXPECTED_SCHEMA_VERSION = 2
EXPECTED_ARTIFACT_NAME = "zig-authoritative-ci-evidence"
EXPECTED_CI_PROVIDER = "github-actions"
EXPECTED_CHECKOUT_MODE = "authoritative_external"
EXPECTED_JOB = "zig-release-authoritative"
EXPECTED_INTEGRATION_SCRIPT = "scripts/test-zigmera.sh"


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: validate-zig-authoritative-ci-evidence.py <path>", file=sys.stderr)
        return 1

    evidence_path = Path(sys.argv[1])
    data = json.loads(evidence_path.read_text())

    for field in REQUIRED_FIELDS:
        value = data.get(field)
        if value in ("", None):
            print(f"ERROR: missing or empty field: {field}", file=sys.stderr)
            return 1

    if data["schema_version"] != EXPECTED_SCHEMA_VERSION:
        print(f"ERROR: unsupported schema_version: {data['schema_version']}", file=sys.stderr)
        return 1

    if "/actions/runs/" not in data["run_url"]:
        print("ERROR: run_url must point to a GitHub Actions run", file=sys.stderr)
        return 1

    if data["artifact_name"] != EXPECTED_ARTIFACT_NAME:
        print(f"ERROR: unexpected artifact_name: {data['artifact_name']}", file=sys.stderr)
        return 1

    if data["ci_provider"] != EXPECTED_CI_PROVIDER:
        print(f"ERROR: unexpected ci_provider: {data['ci_provider']}", file=sys.stderr)
        return 1

    if data["checkout_mode"] != EXPECTED_CHECKOUT_MODE:
        print(f"ERROR: unexpected checkout_mode: {data['checkout_mode']}", file=sys.stderr)
        return 1

    if data["job"] != EXPECTED_JOB:
        print(f"ERROR: unexpected job: {data['job']}", file=sys.stderr)
        return 1

    if len(data["git_sha"]) != 40:
        print("ERROR: git_sha must be a full 40-character commit SHA", file=sys.stderr)
        return 1

    if data["integration_script"] != EXPECTED_INTEGRATION_SCRIPT:
        print(
            f"ERROR: unexpected integration_script: {data['integration_script']}",
            file=sys.stderr,
        )
        return 1

    if data["zig_repo_url"].startswith(("/", ".", "file://")):
        print("ERROR: zig_repo_url must be an external repository URL", file=sys.stderr)
        return 1

    print("Authoritative Zig CI evidence validated.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
