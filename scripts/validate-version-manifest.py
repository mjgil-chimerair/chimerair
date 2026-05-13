#!/usr/bin/env python3
"""Validate the shared Zig integration version manifest against repo constants."""

from __future__ import annotations

import re
import sys
import tomllib
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parent.parent
MANIFEST_PATH = REPO_ROOT / "docs/version-definitions.toml"

ZIGMERA_CRATES = [
    "zigmera-schema",
    "zigmera-paths",
    "zigmera-hash",
    "zigmera-io",
    "zigmera-target",
    "zigmera-diagnostics",
]

SCHEMA_FILES = {
    "zsnap_schema_version": "tools/crates/zigmera-schema/src/zsnap.rs",
    "zdep_schema_version": "tools/crates/zigmera-schema/src/zdep.rs",
    "zairpack_schema_version": "tools/crates/zigmera-schema/src/zairpack.rs",
    "zchmeta_schema_version": "tools/crates/zigmera-schema/src/zchmeta.rs",
    "zchproof_schema_version": "tools/crates/zigmera-schema/src/zchproof.rs",
}


def main() -> int:
    manifest = tomllib.loads(MANIFEST_PATH.read_text())
    errors: list[str] = []

    abi_version = manifest["abi"]["version"]
    for crate_name in ZIGMERA_CRATES:
        cargo_toml = REPO_ROOT / "tools/crates" / crate_name / "Cargo.toml"
        cargo = tomllib.loads(cargo_toml.read_text())
        crate_version = cargo["package"]["version"]
        if crate_version != abi_version:
            errors.append(
                f"{cargo_toml.relative_to(REPO_ROOT)} version {crate_version} != abi.version {abi_version}"
            )

    version_rs = REPO_ROOT / "tools/crates/zigmera-schema/src/version.rs"
    version_source = version_rs.read_text()
    match = re.search(r"Self::new\((\d+), (\d+), (\d+)\)", version_source)
    if not match:
        errors.append(f"could not parse SchemaVersion::current() from {version_rs.relative_to(REPO_ROOT)}")
    else:
        schema_version = ".".join(match.groups())
        if schema_version != abi_version:
            errors.append(
                f"{version_rs.relative_to(REPO_ROOT)} current schema version {schema_version} != abi.version {abi_version}"
            )

    for manifest_key, relative_path in SCHEMA_FILES.items():
        source = (REPO_ROOT / relative_path).read_text()
        match = re.search(r"pub const SCHEMA_VERSION: u32 = (\d+);", source)
        if not match:
            errors.append(f"could not parse SCHEMA_VERSION from {relative_path}")
            continue
        expected = manifest["artifacts"][manifest_key]
        actual = f"{match.group(1)}.0"
        if actual != expected:
            errors.append(f"{relative_path} schema version {actual} != artifacts.{manifest_key} {expected}")

    if errors:
        for error in errors:
            print(f"ERROR: {error}", file=sys.stderr)
        return 1

    print("Version manifest validated successfully.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
