#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

required_vars=(
  "CHIMERA_ZIG_GIT_URL"
  "CHIMERA_ZIG_GIT_REF"
)

required_secrets=(
  "CHIMERA_ZIG_GIT_TOKEN"
)

if ! command -v gh >/dev/null 2>&1; then
  echo "ERROR: GitHub CLI 'gh' is required" >&2
  exit 1
fi

vars_json="$(gh variable list --json name)"
secrets_json="$(gh secret list --json name)"

python3 - "$vars_json" "$secrets_json" "${required_vars[*]}" "${required_secrets[*]}" <<'PY'
import json
import sys

vars_data = json.loads(sys.argv[1])
secrets_data = json.loads(sys.argv[2])
required_vars = [name for name in sys.argv[3].split() if name]
required_secrets = [name for name in sys.argv[4].split() if name]

present_vars = {item["name"] for item in vars_data}
present_secrets = {item["name"] for item in secrets_data}

missing_vars = [name for name in required_vars if name not in present_vars]
missing_secrets = [name for name in required_secrets if name not in present_secrets]

if missing_vars or missing_secrets:
    if missing_vars:
        print(
            "ERROR: missing required GitHub Actions variables: "
            + ", ".join(missing_vars),
            file=sys.stderr,
        )
    if missing_secrets:
        print(
            "ERROR: missing required GitHub Actions secrets: "
            + ", ".join(missing_secrets),
            file=sys.stderr,
        )
    sys.exit(1)

print("GitHub Actions authoritative Zig configuration is present.")
PY
