#ifndef CHIMERA_IR_TYPES_H
#define CHIMERA_IR_TYPES_H

#include "mlir/IR/Types.h"
#include "mlir/IR/MLIRContext.h"
#include "mlir/IR/BuiltinTypes.h"
#include "mlir/IR/BuiltinTypeInterfaces.h"
#include <string>
#include <cstdint>

namespace chimera {

// Type kind enums for semantic properties
enum class OwnershipKind { Borrow, BorrowMut, Owned, Raw };
enum class LifetimeKind { Call, Static, Owner };
enum class MutabilityKind { Const, Mut };

// ============================================================================
// Chimera Type System
//
// The Chimera dialect uses MLIR's type system to represent:
// - Primitive types (status, error) as i32
// - Reference types (borrow, owned, handle) as i64
// - Compound types (result, slice, string) as struct-typed i64
//
// All Chimera types carry semantic meaning beyond their physical
// representation. Use the type checking functions below to verify
// the semantic properties of a type.
// ============================================================================

// Status type - represents a Chimera status code
// Physical: i32, Semantic: function return status (0=ok, >0=error)
struct StatusType {
  static mlir::Type get(mlir::MLIRContext *context) {
    return mlir::IntegerType::get(context, 32);
  }
  static constexpr uint32_t kWidth = 32;
  static constexpr const char *kName = "status";
};

// Error type - represents a Chimera error domain
// Physical: i32, Semantic: error code from a specific domain
struct ErrorType {
  static mlir::Type get(mlir::MLIRContext *context) {
    return mlir::IntegerType::get(context, 32);
  }
  static constexpr uint32_t kWidth = 32;
  static constexpr const char *kName = "error";
};

// Borrow type - represents a borrowed reference with ownership semantics
// Physical: i64 (pointer), Semantic: non-owning reference with lifetime
struct BorrowType {
  static mlir::Type get(mlir::MLIRContext *context,
                        MutabilityKind mut = MutabilityKind::Const,
                        LifetimeKind lifetime = LifetimeKind::Call) {
    (void)mut;
    (void)lifetime;
    return mlir::IntegerType::get(context, 64);
  }
};

// BorrowMut type - represents a mutable borrowed reference
// Physical: i64 (pointer), Semantic: exclusive mutable reference
struct BorrowMutType {
  static mlir::Type get(mlir::MLIRContext *context, LifetimeKind lifetime = LifetimeKind::Call) {
    (void)lifetime;
    return mlir::IntegerType::get(context, 64);
  }
};

// Owned type - represents an owned value
// Physical: i64, Semantic: owned memory that requires drop
struct OwnedType {
  static mlir::Type get(mlir::MLIRContext *context) {
    return mlir::IntegerType::get(context, 64);
  }
};

// Result type - represents Chimera Result[T, E] (error union)
// Physical: i64 (two slots), Semantic: success value or error
struct ResultType {
  static mlir::Type get(mlir::MLIRContext *context, mlir::Type, mlir::Type) {
    return mlir::IntegerType::get(context, 64);
  }
};

// Slice type - represents a dynamic-sized slice (pointer + length)
// Physical: i64, Semantic: reference to contiguous memory with known length
struct SliceType {
  static mlir::Type get(mlir::MLIRContext *context, mlir::Type element_type = nullptr) {
    (void)element_type;
    return mlir::IntegerType::get(context, 64);
  }
};

// String type - represents a string (pointer + length + null terminator)
// Physical: i64, Semantic: UTF-8 string with known length
struct StringType {
  static mlir::Type get(mlir::MLIRContext *context) {
    return mlir::IntegerType::get(context, 64);
  }
};

// Opaque type - represents an opaque handle / external type
// Physical: i64, Semantic: handle to external resource
struct OpaqueType {
  static mlir::Type get(mlir::MLIRContext *context) {
    return mlir::IntegerType::get(context, 64);
  }
};

// Target pointer type - represents a target-specific pointer
// Physical: i64, Semantic: typed pointer for a specific target
struct TargetPointerType {
  static mlir::Type get(mlir::MLIRContext *context, const std::string &target = "unknown") {
    (void)target;
    return mlir::IntegerType::get(context, 64);
  }
};

// Handle type - represents a named resource handle
// Physical: i64, Semantic: reference to a named resource
struct HandleType {
  static mlir::Type get(mlir::MLIRContext *context, const std::string &name = "") {
    (void)name;
    return mlir::IntegerType::get(context, 64);
  }
};

// ============================================================================
// Type Checking Functions
//
// Use these to verify the semantic properties of a type.
// ============================================================================

/// Check if a type is a Chimera status type (i32)
inline bool isStatusType(mlir::Type t) {
  return t.isa<mlir::IntegerType>() && t.cast<mlir::IntegerType>().getWidth() == StatusType::kWidth;
}

/// Check if a type is a Chimera error type (i32)
inline bool isErrorType(mlir::Type t) {
  return t.isa<mlir::IntegerType>() && t.cast<mlir::IntegerType>().getWidth() == ErrorType::kWidth;
}

/// Check if a type is a Chimera pointer type (i64)
inline bool isPointerType(mlir::Type t) {
  return t.isa<mlir::IntegerType>() && t.cast<mlir::IntegerType>().getWidth() == 64;
}

/// Check if a type is a reference type (borrow, owned, handle)
inline bool isReferenceType(mlir::Type t) {
  return isPointerType(t);
}

/// Get the width of a Chimera type in bits
inline uint32_t getTypeWidth(mlir::Type t) {
  if (auto intType = t.dyn_cast<mlir::IntegerType>()) {
    return intType.getWidth();
  }
  return 0;
}

/// Get the MLIR type name for debugging
inline std::string getTypeName(mlir::Type t) {
  if (isStatusType(t)) return "status";
  if (isErrorType(t)) return "error";
  if (isPointerType(t)) return "ptr";
  std::string name;
  llvm::raw_string_ostream os(name);
  t.print(os);
  return name;
}

} // namespace chimera

#endif // CHIMERA_IR_TYPES_H