#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

bash scripts/check-lean-zigadapter-naming.sh
bash -lc 'cd ChimeraProof && lake build Chimera.ZigAdapter.ComptimeCache'
bash -lc 'cd ChimeraProof && lake build Chimera.ZigAdapter.ProofInput'
