#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

bash -lc 'cd ChimeraProof && lake build Chimera.ZigAdapter.ResultLoweringSoundness'
