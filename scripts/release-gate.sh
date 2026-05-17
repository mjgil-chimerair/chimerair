#!/usr/bin/env bash
# Enforce the Chimera Zig proof/release gate for the current workspace.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MODE="${1:-full}"

run() {
  echo "==> $*"
  "$@"
}

cd "$ROOT_DIR"

run python3 scripts/validate-version-manifest.py
run python3 scripts/validate-completion-ledger.py
run python3 scripts/validate-proof-report.py \
  tools/crates/chimera-proof-bridge/fixtures/proof_report.json \
  tests/fixtures/proof-sidecar.chproof.json
run bash scripts/check-placeholders.sh
run bash scripts/check-docs-links.sh

if [[ "$MODE" == "--contracts-only" || "$MODE" == "contracts-only" ]]; then
  echo "Release gate contract checks passed."
  exit 0
fi

run bash scripts/run-zig-release-integration.sh allow-missing
run bash tests/proof-report-validation.sh
run bash tests/zig-release-integration.sh
run bash tests/lean-zigadapter-naming.sh
run bash tests/lean-zigadapter-cache.sh
run bash tests/lean-zigadapter-invalidation.sh
run bash tests/lean-zigadapter-layout.sh
run bash tests/lean-zigadapter-result.sh
run bash tests/lean-zigadapter-defer.sh
run cargo test --manifest-path tools/crates/zigmera-schema/Cargo.toml --quiet
run cargo test --manifest-path tools/crates/zigmera-paths/Cargo.toml --quiet
run cargo test --manifest-path tools/crates/zigmera-hash/Cargo.toml --quiet
run cargo test --manifest-path tools/crates/chimera-adapter-zig/Cargo.toml --quiet
run cargo test --manifest-path tools/crates/chimera-cli/Cargo.toml --quiet test_cli_explain_cache
run bash -lc 'cd ChimeraProof && lake build Chimera.ZigAdapter.ProofInput'

run bash scripts/run-zig-release-integration.sh require-authoritative
run env CHIMERA_SKIP_NESTED_CARGO_TESTS=1 cargo test --workspace --quiet --manifest-path tools/Cargo.toml

if [[ -d ChimeraProof ]]; then
  run bash ChimeraProof/test.sh
fi

if [[ -d compiler-core/build ]]; then
  run cmake --build compiler-core/build
  run ctest --output-on-failure --test-dir compiler-core/build
fi

if [[ -f runtime/test_conformance.sh ]]; then
  run bash runtime/test_conformance.sh
fi

if [[ -f runtime/test_sanitizers.sh ]]; then
  run bash runtime/test_sanitizers.sh
fi

echo "Release gate passed."
