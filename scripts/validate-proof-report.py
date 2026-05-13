#!/usr/bin/env python3
"""Validate Chimera proof-report artifacts used by the release gate."""

from __future__ import annotations

import json
import sys
from pathlib import Path


def fail(message: str) -> None:
    print(f"ERROR: {message}", file=sys.stderr)
    raise SystemExit(1)


def expect(condition: bool, message: str) -> None:
    if not condition:
        fail(message)


def expect_nonempty_string(value: object, field: str, path: Path) -> None:
    expect(isinstance(value, str) and value.strip(), f"{path}: missing or empty `{field}`")


def validate_bridge_fixture(data: object, path: Path) -> None:
    expect(isinstance(data, dict), f"{path}: expected top-level object")
    expect_nonempty_string(data.get("module_name"), "module_name", path)
    expect_nonempty_string(data.get("target_triple"), "target_triple", path)
    expect(isinstance(data.get("verified"), bool), f"{path}: missing boolean `verified`")
    obligations = data.get("obligations")
    expect(isinstance(obligations, list) and obligations, f"{path}: `obligations` must be a non-empty list")
    for index, obligation in enumerate(obligations):
        expect(isinstance(obligation, dict), f"{path}: obligation {index} must be an object")
        expect_nonempty_string(obligation.get("id"), f"obligations[{index}].id", path)
        expect_nonempty_string(obligation.get("kind"), f"obligations[{index}].kind", path)
        expect_nonempty_string(obligation.get("description"), f"obligations[{index}].description", path)
        expect_nonempty_string(obligation.get("status"), f"obligations[{index}].status", path)
    assumptions = data.get("assumptions", [])
    expect(isinstance(assumptions, list), f"{path}: `assumptions` must be a list")


def validate_chproof_sidecar(data: object, path: Path) -> None:
    expect(isinstance(data, dict), f"{path}: expected top-level object")
    expect_nonempty_string(data.get("build_id"), "build_id", path)
    expect_nonempty_string(data.get("target_triple"), "target_triple", path)
    ptr_width = data.get("target_ptr_width")
    expect(isinstance(ptr_width, int) and ptr_width > 0, f"{path}: invalid `target_ptr_width`")
    expect(data.get("target_endian") in {"little", "big"}, f"{path}: invalid `target_endian`")
    obligations = data.get("obligations")
    expect(isinstance(obligations, list) and obligations, f"{path}: `obligations` must be a non-empty list")
    for index, obligation in enumerate(obligations):
        expect(isinstance(obligation, dict), f"{path}: obligation {index} must be an object")
        expect_nonempty_string(obligation.get("id"), f"obligations[{index}].id", path)
        expect_nonempty_string(obligation.get("kind"), f"obligations[{index}].kind", path)
        expect_nonempty_string(obligation.get("description"), f"obligations[{index}].description", path)
        expect_nonempty_string(obligation.get("target"), f"obligations[{index}].target", path)
        assumptions = obligation.get("assumptions")
        expect(isinstance(assumptions, list), f"{path}: obligation {index} assumptions must be a list")
    trust_assumptions = data.get("trust_assumptions", [])
    expect(isinstance(trust_assumptions, list), f"{path}: `trust_assumptions` must be a list")


def validate_file(path_str: str) -> None:
    path = Path(path_str)
    expect(path.is_file(), f"{path}: file not found")
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        fail(f"{path}: invalid JSON: {exc}")

    if isinstance(data, dict) and "module_name" in data:
        validate_bridge_fixture(data, path)
    elif isinstance(data, dict) and "build_id" in data:
        validate_chproof_sidecar(data, path)
    else:
        fail(f"{path}: unsupported proof-report shape")


def main(argv: list[str]) -> None:
    if len(argv) < 2:
        fail("usage: validate-proof-report.py <proof-report.json> [<proof-report.json> ...]")
    for path_str in argv[1:]:
        validate_file(path_str)
    print(f"Validated {len(argv) - 1} proof-report artifact(s).")


if __name__ == "__main__":
    main(sys.argv)
