#ifndef CHIMERA_PASSES_PASSES_H
#define CHIMERA_PASSES_PASSES_H

#include "mlir/Pass/PassManager.h"
#include "mlir/Transforms/Passes.h"
#include <memory>

namespace chimera {

/// Register all Chimera passes
void registerCanonicalizationPasses();
void registerOwnershipValidationPass();
void registerABILayoutMaterializationPass();
void registerResultLoweringPass();
void registerWrapperPrepPass();
void registerProofObligationEmissionPass();
void registerLLVMLoweringPass();
void registerPanicBoundaryPass();
void registerAllocatorVerificationPass();
void registerEffectTrackingPass();
void registerLinkVerificationPass();

/// Create canonicalization pass
std::unique_ptr<mlir::Pass> createCanonicalizationPass();

/// Create ownership validation pass
std::unique_ptr<mlir::Pass> createOwnershipValidationPass();

/// Create ABI layout materialization pass
std::unique_ptr<mlir::Pass> createABILayoutMaterializationPass();

/// Create result lowering pass
std::unique_ptr<mlir::Pass> createResultLoweringPass();

/// Create wrapper preparation pass
std::unique_ptr<mlir::Pass> createWrapperPrepPass();

/// Create proof obligation emission pass
std::unique_ptr<mlir::Pass> createProofObligationEmissionPass();

/// Create LLVM lowering pass
std::unique_ptr<mlir::Pass> createLLVMLoweringPass();

/// Add all Chimera passes to a pass manager
void populateChimeraPassPipeline(mlir::PassManager &pm, const std::string &level);
void populateVerificationPipeline(mlir::PassManager &pm);

} // namespace chimera

#endif // CHIMERA_PASSES_PASSES_H
