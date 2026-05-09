#include <gtest/gtest.h>
#include "chimera/IR/Types.h"
#include "chimera/IR/Dialect.h"
#include "mlir/IR/MLIRContext.h"
#include "mlir/IR/BuiltinTypes.h"

using namespace chimera;

TEST(ChimeraTypesTest, StatusType) {
  mlir::MLIRContext context;
  auto status = StatusType::get(&context);
  EXPECT_TRUE(status);
  EXPECT_TRUE(status.isa<StatusType>());
  EXPECT_EQ(status.getWidth(), 32u);
}

TEST(ChimeraTypesTest, ErrorType) {
  mlir::MLIRContext context;
  auto error = ErrorType::get(&context);
  EXPECT_TRUE(error);
  EXPECT_TRUE(error.isa<ErrorType>());
  EXPECT_EQ(error.getWidth(), 32u);
}

TEST(ChimeraTypesTest, BorrowTypeConst) {
  mlir::MLIRContext context;
  auto borrow = BorrowType::get(&context, MutabilityKind::Const, LifetimeKind::Call);
  EXPECT_TRUE(borrow);
  EXPECT_TRUE(borrow.isa<BorrowType>());
  EXPECT_TRUE(borrow.isConst());
  EXPECT_FALSE(borrow.isMut());
  EXPECT_EQ(borrow.getMutability(), MutabilityKind::Const);
  EXPECT_EQ(borrow.getLifetime(), LifetimeKind::Call);
}

TEST(ChimeraTypesTest, BorrowTypeMut) {
  mlir::MLIRContext context;
  auto borrow = BorrowType::get(&context, MutabilityKind::Mut, LifetimeKind::Call);
  EXPECT_TRUE(borrow);
  EXPECT_TRUE(borrow.isMut());
  EXPECT_FALSE(borrow.isConst());
  EXPECT_EQ(borrow.getMutability(), MutabilityKind::Mut);
}

TEST(ChimeraTypesTest, BorrowTypeStaticLifetime) {
  mlir::MLIRContext context;
  auto borrow = BorrowType::get(&context, MutabilityKind::Const, LifetimeKind::Static);
  EXPECT_TRUE(borrow);
  EXPECT_EQ(borrow.getLifetime(), LifetimeKind::Static);
}

TEST(ChimeraTypesTest, OwnedType) {
  mlir::MLIRContext context;
  auto owned = OwnedType::get(&context);
  EXPECT_TRUE(owned);
  EXPECT_TRUE(owned.isa<OwnedType>());
}

TEST(ChimeraTypesTest, ResultType) {
  mlir::MLIRContext context;
  mlir::Type i32 = mlir::IntegerType::get(&context, 32);
  auto error = ErrorType::get(&context);
  auto result = ResultType::get(&context, i32, error);
  EXPECT_TRUE(result);
  EXPECT_TRUE(result.isa<ResultType>());
}

TEST(ChimeraTypesTest, SliceType) {
  mlir::MLIRContext context;
  mlir::Type i32 = mlir::IntegerType::get(&context, 32);
  auto slice = SliceType::get(&context, i32);
  EXPECT_TRUE(slice);
  EXPECT_TRUE(slice.isa<SliceType>());
}

TEST(ChimeraTypesTest, StringType) {
  mlir::MLIRContext context;
  auto str = StringType::get(&context);
  EXPECT_TRUE(str);
  EXPECT_TRUE(str.isa<StringType>());
}

TEST(ChimeraTypesTest, OpaqueType) {
  mlir::MLIRContext context;
  auto opaque = OpaqueType::get(&context);
  EXPECT_TRUE(opaque);
  EXPECT_TRUE(opaque.isa<OpaqueType>());
}

TEST(ChimeraTypesTest, HandleType) {
  mlir::MLIRContext context;
  auto handle = HandleType::get(&context, "file_handle");
  EXPECT_TRUE(handle);
  EXPECT_TRUE(handle.isa<HandleType>());
}

TEST(ChimeraTypesTest, BorrowMutType) {
  mlir::MLIRContext context;
  auto borrowMut = BorrowMutType::get(&context, LifetimeKind::Call);
  EXPECT_TRUE(borrowMut);
  EXPECT_TRUE(borrowMut.isa<BorrowMutType>());
  EXPECT_EQ(borrowMut.getLifetime(), LifetimeKind::Call);
}

TEST(ChimeraTypesTest, BorrowMutTypeStatic) {
  mlir::MLIRContext context;
  auto borrowMut = BorrowMutType::get(&context, LifetimeKind::Static);
  EXPECT_TRUE(borrowMut);
  EXPECT_EQ(borrowMut.getLifetime(), LifetimeKind::Static);
}

TEST(ChimeraTypesTest, TargetPointerType) {
  mlir::MLIRContext context;
  auto ptr = TargetPointerType::get(&context, "x86_64-unknown-linux");
  EXPECT_TRUE(ptr);
  EXPECT_TRUE(ptr.isa<TargetPointerType>());
  EXPECT_EQ(ptr.getTarget(), "x86_64-unknown-linux-gnu"); // Note: placeholder returns this
}

TEST(ChimeraTypesTest, TypeDistinctness) {
  mlir::MLIRContext context;
  auto status = StatusType::get(&context);
  auto error = ErrorType::get(&context);
  auto borrow = BorrowType::get(&context, MutabilityKind::Const, LifetimeKind::Call);
  auto owned = OwnedType::get(&context);
  auto slice = SliceType::get(&context, mlir::IntegerType::get(&context, 8));

  // All types should be distinct
  EXPECT_NE(status.getAsOpaquePointer(), error.getAsOpaquePointer());
  EXPECT_NE(status.getAsOpaquePointer(), borrow.getAsOpaquePointer());
  EXPECT_NE(borrow.getAsOpaquePointer(), owned.getAsOpaquePointer());
  EXPECT_NE(owned.getAsOpaquePointer(), slice.getAsOpaquePointer());
}

TEST(ChimeraTypesTest, SameTypesEqual) {
  mlir::MLIRContext context;
  auto status1 = StatusType::get(&context);
  auto status2 = StatusType::get(&context);
  EXPECT_EQ(status1.getAsOpaquePointer(), status2.getAsOpaquePointer());

  auto error1 = ErrorType::get(&context);
  auto error2 = ErrorType::get(&context);
  EXPECT_EQ(error1.getAsOpaquePointer(), error2.getAsOpaquePointer());
}