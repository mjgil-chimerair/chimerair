# Trusted Computing Base

The current Chimera trusted computing base is split across:

- `compiler-core/` for MLIR verification and lowering
- `tools/` for manifest parsing, orchestration, and artifact planning
- `runtime/` for ABI boundary definitions
- `ChimeraProof/` for proof-side modeling

The normative system model remains [ChimeraIR Final Design](chimerair-final-design.md).
