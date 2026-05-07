#!/usr/bin/env python3

import json
import re
import sys
from pathlib import Path


SHA_RE = re.compile(r"^[0-9a-f]{40}$")
RUN_ID_RE = re.compile(r"^[0-9]+$")


def fail(message: str) -> None:
    print(f"ERROR: {message}", file=sys.stderr)
    raise SystemExit(1)


def main() -> None:
    if len(sys.argv) != 2:
        fail("usage: check-zig-authoritative-session-manifest.py <manifest>")

    manifest_path = Path(sys.argv[1])
    if not manifest_path.is_file():
        fail(f"session manifest not found: {manifest_path}")

    try:
        data = json.loads(manifest_path.read_text())
    except json.JSONDecodeError as exc:
        fail(f"invalid JSON in session manifest: {exc}")

    required_fields = {
        "mode",
        "ref",
        "expected_sha",
        "expected_zig_ref",
        "base_dir",
        "run_id",
        "archive_dir",
        "signoff_dir",
        "dry_run_manifest",
    }
    missing = sorted(required_fields - data.keys())
    if missing:
        fail(f"session manifest missing required fields: {', '.join(missing)}")

    if data["mode"] != "authoritative-release":
        fail("session manifest mode must be 'authoritative-release'")

    expected_sha = data["expected_sha"]
    if not isinstance(expected_sha, str) or not SHA_RE.fullmatch(expected_sha):
        fail("session manifest expected_sha must be a 40-character lowercase git SHA")

    run_id = data["run_id"]
    if not isinstance(run_id, str) or not RUN_ID_RE.fullmatch(run_id):
        fail("session manifest run_id must contain only decimal digits")

    base_dir = Path(data["base_dir"]).resolve()
    archive_dir = Path(data["archive_dir"]).resolve()
    signoff_dir = Path(data["signoff_dir"]).resolve()
    dry_run_manifest = Path(data["dry_run_manifest"]).resolve()

    expected_archive_dir = (base_dir / "zig-authoritative" / expected_sha / f"run-{run_id}").resolve()
    expected_signoff_dir = (base_dir / "zig-authoritative" / expected_sha / f"signoff-run-{run_id}").resolve()

    if archive_dir != expected_archive_dir:
        fail("session manifest archive_dir does not match base_dir/expected_sha/run_id layout")
    if signoff_dir != expected_signoff_dir:
        fail("session manifest signoff_dir does not match base_dir/expected_sha/run_id layout")

    if not archive_dir.is_dir():
        fail(f"session manifest archive_dir not found: {archive_dir}")
    if not signoff_dir.is_dir():
        fail(f"session manifest signoff_dir not found: {signoff_dir}")
    if not dry_run_manifest.is_file():
        fail(f"session manifest dry_run_manifest not found: {dry_run_manifest}")

    print("Authoritative Zig session manifest is valid.")


if __name__ == "__main__":
    main()
