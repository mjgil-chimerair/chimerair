#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

workflow=".github/workflows/ci.yml"

python3 - <<'PY'
from pathlib import Path
import sys
import yaml

workflow = Path(".github/workflows/ci.yml")
data = yaml.safe_load(workflow.read_text())
jobs = data.get("jobs", {})
on_section = data.get("on", data.get(True, {}))

if "workflow_dispatch" not in on_section:
    raise SystemExit("ci workflow is not manually dispatchable")

for job in ("zig-release-smoke", "zig-release-authoritative", "release-gate-clean-checkout"):
    if job not in jobs:
        raise SystemExit(f"missing workflow job: {job}")

zig_smoke = jobs["zig-release-smoke"]["steps"]
zig_authoritative = jobs["zig-release-authoritative"]["steps"]
release_gate = jobs["release-gate-clean-checkout"]["steps"]
root_smoke = jobs["root-test-smoke"]["steps"]

def step_contains(steps, text):
    return any(text in step.get("run", "") for step in steps if isinstance(step, dict))

if not step_contains(zig_smoke, "bash scripts/setup-authoritative-zig-fixture.sh /tmp/zigmera-zig"):
    raise SystemExit("zig-release-smoke does not create the authoritative Zig fixture")
if not step_contains(zig_smoke, "bash scripts/run-zig-release-integration.sh require-authoritative"):
    raise SystemExit("zig-release-smoke does not run the authoritative Zig integration gate")
if not step_contains(zig_authoritative, "bash scripts/prepare-zig-authoritative-checkout.sh /tmp/zigmera-zig-real"):
    raise SystemExit("zig-release-authoritative does not prepare the real authoritative Zig checkout")
if not step_contains(zig_authoritative, "bash scripts/check-zig-authoritative-ci-config.sh"):
    raise SystemExit("zig-release-authoritative does not validate authoritative CI config")
if not step_contains(zig_authoritative, "bash scripts/run-zig-release-integration.sh require-authoritative"):
    raise SystemExit("zig-release-authoritative does not run the authoritative Zig integration gate")
if not step_contains(zig_authoritative, "python3 scripts/write-zig-authoritative-ci-evidence.py"):
    raise SystemExit("zig-release-authoritative does not write authoritative CI evidence")
if not step_contains(zig_authoritative, "python3 scripts/validate-zig-authoritative-ci-evidence.py"):
    raise SystemExit("zig-release-authoritative does not validate authoritative CI evidence")
if not any(step.get("uses") == "actions/upload-artifact@v4" for step in zig_authoritative if isinstance(step, dict)):
    raise SystemExit("zig-release-authoritative does not upload authoritative CI evidence")
if not any(
    step.get("uses") == "actions/upload-artifact@v4"
    and step.get("with", {}).get("name") == "zig-authoritative-ci-evidence"
    for step in zig_authoritative
    if isinstance(step, dict)
):
    raise SystemExit("zig-release-authoritative does not upload the expected authoritative CI artifact name")
if not step_contains(release_gate, "bash tests/release-gate-clean-checkout.sh"):
    raise SystemExit("release-gate-clean-checkout does not run the clean-checkout gate test")
if not step_contains(root_smoke, "bash scripts/release-gate.sh --contracts-only"):
    raise SystemExit("root-test-smoke does not run the contracts-only release gate")
PY

echo "CI release jobs verified."
