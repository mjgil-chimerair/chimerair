#include "chimera/IR/Dialect.h"
#include "chimera/Lowering/LLVMLowering.h"
#include "chimera/Passes/Passes.h"
#include "mlir/Dialect/Arith/IR/Arith.h"
#include "mlir/Dialect/ControlFlow/IR/ControlFlow.h"
#include "mlir/Dialect/Func/IR/FuncOps.h"
#include "mlir/Dialect/LLVMIR/LLVMDialect.h"
#include "mlir/IR/BuiltinDialect.h"
#include "mlir/IR/DialectRegistry.h"
#include "mlir/IR/MLIRContext.h"
#include "mlir/Parser/Parser.h"
#include "mlir/Pass/PassManager.h"
#include "llvm/ADT/StringRef.h"
#include "llvm/Support/raw_ostream.h"
#include <array>
#include <string>

namespace {

mlir::OwningOpRef<mlir::ModuleOp> parseModule(mlir::MLIRContext &context,
                                              llvm::StringRef source) {
  mlir::ParserConfig parserConfig(&context);
  return mlir::parseSourceString<mlir::ModuleOp>(source, parserConfig);
}

bool runPipeline(mlir::MLIRContext &context, llvm::StringRef source,
                 const std::string &level) {
  auto module = parseModule(context, source);
  if (!module) {
    return false;
  }

  mlir::PassManager pm(&context);
  chimera::populateChimeraPassPipeline(pm, level);
  return mlir::succeeded(pm.run(*module));
}

bool runVerification(mlir::MLIRContext &context, llvm::StringRef source) {
  auto module = parseModule(context, source);
  if (!module) {
    return false;
  }

  mlir::PassManager pm(&context);
  chimera::populateVerificationPipeline(pm);
  return mlir::succeeded(pm.run(*module));
}

} // namespace

int main() {
  mlir::DialectRegistry registry;
  chimera::lowering::registerLLVMDialects(registry);
  registry.insert<mlir::BuiltinDialect, chimera::ChimeraDialect>();
  mlir::MLIRContext context(registry);
  context.loadDialect<mlir::BuiltinDialect>();
  context.loadDialect<mlir::arith::ArithDialect>();
  context.loadDialect<mlir::cf::ControlFlowDialect>();
  context.loadDialect<mlir::func::FuncDialect>();
  context.loadDialect<chimera::ChimeraDialect>();
  context.loadDialect<mlir::LLVM::LLVMDialect>();

  const std::string source = R"mlir(
module {
  func.func @pass_smoke(%lhs: i32, %rhs: i32) -> i32 {
    %sum = arith.addi %lhs, %rhs : i32
    return %sum : i32
  }
}
)mlir";

  if (!runVerification(context, source)) {
    llvm::errs() << "verification pipeline failed\n";
    return 1;
  }

  constexpr std::array<const char *, 4> levels = {
      "check-only", "wrapper-gen", "object-emit", "proof-obligations"};
  for (const char *level : levels) {
    if (!runPipeline(context, source, level)) {
      llvm::errs() << "pipeline failed: " << level << "\n";
      return 1;
    }
  }

  llvm::outs() << "Pass smoke: OK\n";
  return 0;
}
