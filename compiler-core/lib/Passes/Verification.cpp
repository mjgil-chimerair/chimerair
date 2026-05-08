#include "chimera/Passes/Verification.h"
#include "mlir/IR/BuiltinOps.h"
#include "mlir/IR/Verifier.h"
#include "mlir/Pass/Pass.h"

namespace chimera {
namespace {

template <typename Derived>
struct StructuralVerificationPassBase
    : public mlir::PassWrapper<Derived, mlir::OperationPass<mlir::ModuleOp>> {
  void runOnOperation() override {
    if (mlir::failed(mlir::verify(this->getOperation()))) {
      this->signalPassFailure();
    }
  }
};

#define CHIMERA_DEFINE_STRUCTURAL_PASS(TYPE, ARG, DESC)                        \
  struct TYPE : public StructuralVerificationPassBase<TYPE> {                  \
    llvm::StringRef getArgument() const override { return ARG; }               \
    llvm::StringRef getDescription() const override { return DESC; }           \
  };

CHIMERA_DEFINE_STRUCTURAL_PASS(OwnershipValidationPass,
                               "chimera-verify-ownership",
                               "Run structural ownership-surface verification")
CHIMERA_DEFINE_STRUCTURAL_PASS(ABILayoutMaterializationPass,
                               "chimera-verify-abi-layout",
                               "Run structural ABI layout-surface verification")
CHIMERA_DEFINE_STRUCTURAL_PASS(ResultLoweringPass,
                               "chimera-verify-result",
                               "Run structural Result lowering verification")
CHIMERA_DEFINE_STRUCTURAL_PASS(PanicBoundaryPass,
                               "chimera-verify-panic",
                               "Run structural panic-boundary verification")
CHIMERA_DEFINE_STRUCTURAL_PASS(AllocatorVerificationPass,
                               "chimera-verify-allocator",
                               "Run structural allocator verification")
CHIMERA_DEFINE_STRUCTURAL_PASS(EffectTrackingPass,
                               "chimera-verify-effects",
                               "Run structural effect verification")
CHIMERA_DEFINE_STRUCTURAL_PASS(LinkVerificationPass,
                               "chimera-verify-link",
                               "Run structural link verification")

#undef CHIMERA_DEFINE_STRUCTURAL_PASS

} // namespace

std::unique_ptr<mlir::Pass> createOwnershipValidationPass() {
  return std::make_unique<OwnershipValidationPass>();
}

void registerOwnershipValidationPass() {
  mlir::PassRegistration<OwnershipValidationPass>();
}

std::unique_ptr<mlir::Pass> createABILayoutMaterializationPass() {
  return std::make_unique<ABILayoutMaterializationPass>();
}

void registerABILayoutMaterializationPass() {
  mlir::PassRegistration<ABILayoutMaterializationPass>();
}

std::unique_ptr<mlir::Pass> createResultLoweringPass() {
  return std::make_unique<ResultLoweringPass>();
}

void registerResultLoweringPass() { mlir::PassRegistration<ResultLoweringPass>(); }

std::unique_ptr<mlir::Pass> createPanicBoundaryPass() {
  return std::make_unique<PanicBoundaryPass>();
}

void registerPanicBoundaryPass() { mlir::PassRegistration<PanicBoundaryPass>(); }

std::unique_ptr<mlir::Pass> createAllocatorVerificationPass() {
  return std::make_unique<AllocatorVerificationPass>();
}

void registerAllocatorVerificationPass() {
  mlir::PassRegistration<AllocatorVerificationPass>();
}

std::unique_ptr<mlir::Pass> createEffectTrackingPass() {
  return std::make_unique<EffectTrackingPass>();
}

void registerEffectTrackingPass() { mlir::PassRegistration<EffectTrackingPass>(); }

std::unique_ptr<mlir::Pass> createLinkVerificationPass() {
  return std::make_unique<LinkVerificationPass>();
}

void registerLinkVerificationPass() { mlir::PassRegistration<LinkVerificationPass>(); }

void populateVerificationPipeline(mlir::PassManager &pm) {
  pm.addPass(createOwnershipValidationPass());
  pm.addPass(createABILayoutMaterializationPass());
  pm.addPass(createResultLoweringPass());
  pm.addPass(createPanicBoundaryPass());
  pm.addPass(createAllocatorVerificationPass());
  pm.addPass(createEffectTrackingPass());
  pm.addPass(createLinkVerificationPass());
}

} // namespace chimera
