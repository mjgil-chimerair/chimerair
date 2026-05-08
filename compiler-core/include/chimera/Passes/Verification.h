#ifndef CHIMERA_PASSES_VERIFICATION_H
#define CHIMERA_PASSES_VERIFICATION_H

#include "chimera/Passes/Passes.h"

namespace chimera {

/// Compatibility verification surface for semantic checks that are not yet
/// lowered into dedicated Chimera dialect ops. The current implementation runs
/// structural MLIR verification through named pass entrypoints so the
/// compiler-core build graph, tests, and external integrations can target a
/// stable API.
std::unique_ptr<mlir::Pass> createPanicBoundaryPass();
void registerPanicBoundaryPass();

std::unique_ptr<mlir::Pass> createAllocatorVerificationPass();
void registerAllocatorVerificationPass();

std::unique_ptr<mlir::Pass> createEffectTrackingPass();
void registerEffectTrackingPass();

std::unique_ptr<mlir::Pass> createLinkVerificationPass();
void registerLinkVerificationPass();

} // namespace chimera

#endif // CHIMERA_PASSES_VERIFICATION_H
