#!/usr/bin/env python3
"""Write a machine-readable evidence record for the authoritative Zig CI job."""

from __future__ import annotations

import json
import os
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path


REQUIRED_ENV = (
    "CHIMERA_ZIG_GIT_URL",
    "GITHUB_JOB",
    "GITHUB_REPOSITORY",
    "GITHUB_RUN_ID",
    "GITHUB_SHA",
    "GITHUB_SERVER_URL",
    "GITHUB_WORKFLOW",
)

ARTIFACT_NAME = "zig-authoritative-ci-evidence"
CI_PROVIDER = "github-actions"
CHECKOUT_MODE = "authoritative_external"
INTEGRATION_SCRIPT = "scripts/test-zigmera.sh"


def require_env(name: str) -> str:
    value = os.environ.get(name, "").strip()
    if not value:
        raise SystemExit(f"ERROR: required environment variable is missing: {name}")
    return value


def zig_version(zig_bin: Path) -> str:
    result = subprocess.run(
        [str(zig_bin), "version"],
        check=True,
        capture_output=True,
        text=True,
    )
    return result.stdout.strip()


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: write-zig-authoritative-ci-evidence.py <output-path>", file=sys.stderr)
        return 1

    output_path = Path(sys.argv[1])
    zig_root = Path(os.environ.get("CHIMERA_ZIG_ROOT", "")).resolve()
    zig_bin = Path(os.environ.get("CHIMERA_ZIG_BIN", "")).resolve()

    if not zig_root.exists():
      print("ERROR: CHIMERA_ZIG_ROOT must point to an existing checkout", file=sys.stderr)
      return 1
    if not zig_bin.exists():
      print("ERROR: CHIMERA_ZIG_BIN must point to an existing Zig binary", file=sys.stderr)
      return 1

    data = {
        "schema_version": 2,
        "artifact_name": ARTIFACT_NAME,
        "ci_provider": CI_PROVIDER,
        "checkout_mode": CHECKOUT_MODE,
        "generated_at_utc": datetime.now(timezone.utc).isoformat().replace("+00:00", "Z"),
        "git_sha": require_env("GITHUB_SHA"),
        "job": require_env("GITHUB_JOB"),
        "repository": require_env("GITHUB_REPOSITORY"),
        "run_url": f"{require_env('GITHUB_SERVER_URL')}/{os.environ['GITHUB_REPOSITORY']}/actions/runs/{require_env('GITHUB_RUN_ID')}",
        "workflow": require_env("GITHUB_WORKFLOW"),
        "integration_script": INTEGRATION_SCRIPT,
        "zig_repo_url": require_env("CHIMERA_ZIG_GIT_URL"),
        "zig_repo_ref": os.environ.get("CHIMERA_ZIG_GIT_REF", "").strip(),
        "zig_root": str(zig_root),
        "zig_bin": str(zig_bin),
        "zig_version": zig_version(zig_bin),
    }

    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
