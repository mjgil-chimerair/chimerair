// Production lowering entrypoint for the current compiler-core surface.
// This integrates the builtin func/arith/control-flow lowering path that the
// driver can exercise end to end today.

#include "chimera/Lowering/LLVMLowering.h"
#include "mlir/Conversion/ArithToLLVM/ArithToLLVM.h"
#include "mlir/Conversion/ControlFlowToLLVM/ControlFlowToLLVM.h"
#include "mlir/Conversion/FuncToLLVM/ConvertFuncToLLVM.h"
#include "mlir/Conversion/Passes.h"
#include "mlir/Conversion/ReconcileUnrealizedCasts/ReconcileUnrealizedCasts.h"
#include "mlir/Dialect/Arith/IR/Arith.h"
#include "mlir/Dialect/ControlFlow/IR/ControlFlow.h"
#include "mlir/Dialect/Func/IR/FuncOps.h"
#include "mlir/Dialect/LLVMIR/LLVMDialect.h"
#include "mlir/IR/BuiltinOps.h"
#include "mlir/IR/DialectRegistry.h"
#include "mlir/Pass/PassManager.h"

namespace chimera::lowering {

void registerLLVMDialects(mlir::DialectRegistry &registry) {
  registry.insert<mlir::arith::ArithDialect,
                  mlir::cf::ControlFlowDialect,
                  mlir::func::FuncDialect,
                  mlir::LLVM::LLVMDialect>();
}

void populateLLVMConversionPipeline(mlir::PassManager &pm) {
  pm.addPass(mlir::createConvertControlFlowToLLVMPass());
  pm.addPass(mlir::createArithToLLVMConversionPass());
  pm.addPass(mlir::createConvertFuncToLLVMPass());
  pm.addPass(mlir::createReconcileUnrealizedCastsPass());
}

mlir::LogicalResult lowerModuleToLLVM(mlir::ModuleOp module) {
  mlir::PassManager pm(module.getContext());
  populateLLVMConversionPipeline(pm);
  return pm.run(module);
}

} // namespace chimera::lowering
