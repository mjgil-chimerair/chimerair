#!/usr/bin/env python3
"""Validate a machine-readable authoritative dry-run manifest."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Validate an authoritative Zig dry-run manifest."
    )
    parser.add_argument("manifest_path", help="Path to the dry-run manifest JSON")
    args = parser.parse_args()

    manifest_path = Path(args.manifest_path).resolve()
    if not manifest_path.is_file():
        print(f"ERROR: dry-run manifest does not exist: {manifest_path}", file=sys.stderr)
        return 1

    data = json.loads(manifest_path.read_text())

    required = ("mode", "ref", "expected_sha", "expected_zig_ref", "base_dir")
    for field in required:
        if data.get(field, "") in ("", None):
            print(f"ERROR: missing or empty dry-run field: {field}", file=sys.stderr)
            return 1

    if data["mode"] != "dry-run":
        print(f"ERROR: unexpected dry-run mode: {data['mode']}", file=sys.stderr)
        return 1

    if len(data["expected_sha"]) != 40:
        print("ERROR: expected_sha is not a 40-character git SHA", file=sys.stderr)
        return 1

    print("Authoritative Zig dry-run manifest is valid.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
