#ifndef CHIMERA_LOWERING_LLVM_LOWERING_H
#define CHIMERA_LOWERING_LLVM_LOWERING_H

#include "mlir/IR/BuiltinOps.h"
#include "mlir/IR/DialectRegistry.h"
#include "mlir/Support/LogicalResult.h"
#include "mlir/Pass/Pass.h"

namespace chimera::lowering {

/// Registers the dialects required for the currently supported LLVM lowering
/// pipeline.
void registerLLVMDialects(mlir::DialectRegistry &registry);

/// Populates the current compiler-core LLVM lowering pipeline.
void populateLLVMConversionPipeline(mlir::PassManager &pm);

/// Runs the LLVM lowering pipeline on a parsed module.
mlir::LogicalResult lowerModuleToLLVM(mlir::ModuleOp module);

} // namespace chimera::lowering

#endif // CHIMERA_LOWERING_LLVM_LOWERING_H
