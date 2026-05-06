#!/usr/bin/env python3
"""Verify a downloaded authoritative Zig CI evidence artifact for release use."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import tempfile
import zipfile
from pathlib import Path


def git_head_sha(repo_root: Path) -> str:
    result = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=repo_root,
        check=True,
        capture_output=True,
        text=True,
    )
    return result.stdout.strip()


def resolve_artifact_json(artifact_path: Path) -> Path:
    if artifact_path.suffix != ".zip":
        return artifact_path

    with zipfile.ZipFile(artifact_path) as archive:
        candidates = [
            name for name in archive.namelist() if name.endswith("zig-authoritative-ci-evidence.json")
        ]
        if not candidates:
            json_candidates = [name for name in archive.namelist() if name.endswith(".json")]
            if len(json_candidates) == 1:
                candidates = json_candidates

        if len(candidates) != 1:
            raise SystemExit(
                "ERROR: expected exactly one authoritative CI evidence JSON file inside the artifact zip"
            )

        extracted_dir = Path(tempfile.mkdtemp(prefix="zig-authoritative-ci-artifact-"))
        extracted_path = extracted_dir / Path(candidates[0]).name
        extracted_path.write_bytes(archive.read(candidates[0]))
        return extracted_path


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Check that an authoritative Zig CI evidence artifact matches the expected release revision."
    )
    parser.add_argument(
        "artifact_path",
        help="Path to zig-authoritative-ci-evidence.json or the downloaded artifact zip",
    )
    parser.add_argument(
        "--expected-repository",
        default="mjgil/chimerair",
        help="Expected GitHub repository slug",
    )
    parser.add_argument(
        "--expected-job",
        default="zig-release-authoritative",
        help="Expected GitHub Actions job name",
    )
    parser.add_argument(
        "--expected-sha",
        help="Expected commit SHA. Defaults to git rev-parse HEAD in the current repo.",
    )
    parser.add_argument(
        "--expected-zig-ref",
        help="Expected external patched-Zig ref, if one must be enforced.",
    )
    args = parser.parse_args()

    artifact_path = Path(args.artifact_path).resolve()
    repo_root = Path(__file__).resolve().parent.parent
    artifact_json_path = resolve_artifact_json(artifact_path)

    validate_script = repo_root / "scripts" / "validate-zig-authoritative-ci-evidence.py"
    subprocess.run(
        [sys.executable, str(validate_script), str(artifact_json_path)],
        check=True,
    )

    data = json.loads(artifact_json_path.read_text())
    expected_sha = args.expected_sha or git_head_sha(repo_root)

    if data["repository"] != args.expected_repository:
        print(
            f"ERROR: repository mismatch: expected {args.expected_repository}, got {data['repository']}",
            file=sys.stderr,
        )
        return 1

    if data["job"] != args.expected_job:
        print(
            f"ERROR: job mismatch: expected {args.expected_job}, got {data['job']}",
            file=sys.stderr,
        )
        return 1

    if data["git_sha"] != expected_sha:
        print(
            f"ERROR: git_sha mismatch: expected {expected_sha}, got {data['git_sha']}",
            file=sys.stderr,
        )
        return 1

    if args.expected_zig_ref is not None and data.get("zig_repo_ref", "") != args.expected_zig_ref:
        print(
            f"ERROR: zig_repo_ref mismatch: expected {args.expected_zig_ref}, got {data.get('zig_repo_ref', '')}",
            file=sys.stderr,
        )
        return 1

    print("Authoritative Zig CI artifact matches the expected release revision.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
