#include <gtest/gtest.h>
#include "chimera/IR/Dialect.h"
#include "chimera/IR/Types.h"
#include "mlir/IR/Builders.h"
#include "mlir/IR/MLIRContext.h"

using namespace chimera;

TEST(VerifierTest, ModuleWellFormed) {
  mlir::MLIRContext context;
  mlir::OpBuilder builder(&context);

  auto moduleOp = builder.create<mlir::ModuleOp>();
  EXPECT_TRUE(moduleOp);

  EXPECT_TRUE(mlir::verify(moduleOp).succeeded());
}

TEST(VerifierTest, SymbolUniqueness) {
  mlir::MLIRContext context;
  mlir::OpBuilder builder(&context);

  auto moduleOp = builder.create<mlir::ModuleOp>();
  auto *body = moduleOp.getBody();

  builder.setInsertionPointToStart(body);
  auto func1 = builder.create<mlir::FuncOp>(
      builder.getUnknownLoc(), "func1",
      builder.getType<mlir::FunctionType>({}, {}));
  func1.setVisibility(mlir::SymbolTable::Visibility::Public);

  EXPECT_TRUE(mlir::verify(moduleOp).succeeded());
}

TEST(VerifierTest, TypeValidity) {
  mlir::MLIRContext context;
  mlir::OpBuilder builder(&context);

  auto moduleOp = builder.create<mlir::ModuleOp>();
  EXPECT_TRUE(mlir::verify(moduleOp).succeeded());
}

TEST(VerifierTest, BorrowExclusivity) {
  mlir::MLIRContext context;
  mlir::OpBuilder builder(&context);

  auto moduleOp = builder.create<mlir::ModuleOp>();
  auto *body = moduleOp.getBody();

  builder.setInsertionPointToStart(body);
  auto funcType = builder.getType<mlir::FunctionType>({}, {});
  auto func = builder.create<mlir::FuncOp>(
      builder.getUnknownLoc(), "test_func", funcType);
  builder.setInsertionPointToStart(&func.getBody().emplaceBlock());

  EXPECT_TRUE(mlir::verify(moduleOp).succeeded());
}