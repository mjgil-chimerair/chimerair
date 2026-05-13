#!/usr/bin/env python3
"""Validate the markdown-backed completion ledger entries."""

from __future__ import annotations

import re
import sys
from dataclasses import dataclass, field
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parent.parent
LEDGER_PATH = REPO_ROOT / "docs/completion-ledger.md"
TASK_HEADER_RE = re.compile(r"^\[task\.(\d+)\]\s*$")
STATUS_RE = re.compile(r'^status = "(Complete|Incomplete)"$')


@dataclass
class Entry:
    task_id: str
    status: str | None = None
    evidence: dict[str, list[str]] = field(
        default_factory=lambda: {"code": [], "tests": [], "docs": [], "ci_jobs": []}
    )


def parse_inline_items(line: str) -> list[str]:
    return re.findall(r'"([^"]+)"', line)


def parse_entries(lines: list[str]) -> list[Entry]:
    entries: list[Entry] = []
    current: Entry | None = None
    collecting_field: str | None = None

    for raw_line in lines:
        line = raw_line.strip()
        header_match = TASK_HEADER_RE.match(line)
        if header_match:
            current = Entry(task_id=header_match.group(1))
            entries.append(current)
            collecting_field = None
            continue

        if current is None:
            continue

        if collecting_field is not None:
            current.evidence[collecting_field].extend(parse_inline_items(line))
            if "]" in line:
                collecting_field = None
            continue

        status_match = STATUS_RE.match(line)
        if status_match:
            current.status = status_match.group(1)
            continue

        if line == "evidence = {":
            continue

        if line == "}":
            collecting_field = None
            continue

        for field_name in current.evidence:
            prefix = f"{field_name} = ["
            if line.startswith(prefix):
                current.evidence[field_name].extend(parse_inline_items(line))
                if "]" not in line:
                    collecting_field = field_name
                break

    return entries


def main() -> int:
    entries = parse_entries(LEDGER_PATH.read_text().splitlines())
    errors: list[str] = []

    complete_entries = [entry for entry in entries if entry.status == "Complete"]
    if not complete_entries:
        errors.append("no Complete entries were found in the ledger")

    for entry in complete_entries:
        for field_name, values in entry.evidence.items():
            if not values:
                errors.append(f"task.{entry.task_id} is Complete but missing evidence.{field_name}")
        for code_path in entry.evidence["code"]:
            if "/" not in code_path:
                continue
            if not (REPO_ROOT / code_path).exists():
                errors.append(f"task.{entry.task_id} references missing code path {code_path}")
        for doc_path in entry.evidence["docs"]:
            if "/" not in doc_path:
                continue
            if not (REPO_ROOT / doc_path).exists():
                errors.append(f"task.{entry.task_id} references missing doc path {doc_path}")

    if errors:
        for error in errors:
            print(f"ERROR: {error}", file=sys.stderr)
        return 1

    print(f"Completion ledger validated successfully ({len(complete_entries)} complete entries checked).")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
