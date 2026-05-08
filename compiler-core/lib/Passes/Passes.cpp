#include "chimera/Passes/Passes.h"
#include "chimera/Passes/Verification.h"
#include "chimera/Lowering/LLVMLowering.h"
#include "mlir/IR/BuiltinOps.h"
#include "mlir/Pass/Pass.h"
#include "mlir/IR/PatternMatch.h"
#include "mlir/Pass/PassManager.h"
#include "mlir/Transforms/Passes.h"

namespace chimera {

void registerCanonicalizationPasses() {
  // Register canonicalization patterns
}

std::unique_ptr<mlir::Pass> createCanonicalizationPass() {
  return mlir::createCanonicalizerPass();
}

std::unique_ptr<mlir::Pass> createOwnershipValidationPass();
void registerOwnershipValidationPass();

std::unique_ptr<mlir::Pass> createABILayoutMaterializationPass();
void registerABILayoutMaterializationPass();

std::unique_ptr<mlir::Pass> createResultLoweringPass();
void registerResultLoweringPass();

std::unique_ptr<mlir::Pass> createPanicBoundaryPass();
void registerPanicBoundaryPass();

std::unique_ptr<mlir::Pass> createEffectTrackingPass();
void registerEffectTrackingPass();

std::unique_ptr<mlir::Pass> createWrapperPrepPass() {
  return nullptr;
}

std::unique_ptr<mlir::Pass> createProofObligationEmissionPass() {
  return nullptr;
}

void registerProofObligationEmissionPass() {
}

namespace {

struct LLVMLoweringPass
    : public mlir::PassWrapper<LLVMLoweringPass,
                               mlir::OperationPass<mlir::ModuleOp>> {
  void runOnOperation() override {
    if (mlir::failed(chimera::lowering::lowerModuleToLLVM(getOperation()))) {
      signalPassFailure();
    }
  }

  llvm::StringRef getArgument() const override { return "chimera-lower-llvm"; }
  llvm::StringRef getDescription() const override {
    return "Lower the current compiler-core surface to LLVM dialect";
  }
};

} // namespace

std::unique_ptr<mlir::Pass> createLLVMLoweringPass() {
  return std::make_unique<LLVMLoweringPass>();
}

void registerWrapperPrepPass() {}
void registerLLVMLoweringPass() {
  mlir::PassRegistration<LLVMLoweringPass>();
}

void populateChimeraPassPipeline(mlir::PassManager &pm, const std::string &level) {
  if (level == "check-only") {
    populateVerificationPipeline(pm);
  } else if (level == "wrapper-gen") {
    pm.addPass(createCanonicalizationPass());
  } else if (level == "object-emit") {
    pm.addPass(createCanonicalizationPass());
    populateVerificationPipeline(pm);
    pm.addPass(createLLVMLoweringPass());
  } else if (level == "proof-obligations") {
    pm.addPass(createCanonicalizationPass());
    pm.addPass(createOwnershipValidationPass());
  }
}

} // namespace chimera
