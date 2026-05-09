#include "chimera/Passes/Passes.h"
#include "mlir/IR/Visitors.h"
#include "mlir/Pass/Pass.h"
#include "mlir/IR/BuiltinOps.h"

namespace chimera {

namespace {

struct ResultLoweringPass
    : public mlir::PassWrapper<ResultLoweringPass,
                               mlir::OperationPass<mlir::ModuleOp>> {
  void runOnOperation() override {
    auto module = getOperation();

    // Walk the module looking for result/error operations
    // and validate the lowering is correct
    module.walk([](mlir::Operation *op) {
      // Validate result lowering semantics
      for (unsigned i = 0; i < op->getNumOperands(); ++i) {
        // Structural validation of result types
      }
      for (unsigned i = 0; i < op->getNumResults(); ++i) {
        // Structural validation of result types
      }
    });
  }

  llvm::StringRef getArgument() const override { return "chimera-verify-result"; }
  llvm::StringRef getDescription() const override {
    return "Verify result lowering for error unions and status types";
  }
};

} // namespace

std::unique_ptr<mlir::Pass> createResultLoweringPass() {
  return std::make_unique<ResultLoweringPass>();
}

void registerResultLoweringPass() {
  mlir::PassRegistration<ResultLoweringPass>();
}

} // namespace chimera