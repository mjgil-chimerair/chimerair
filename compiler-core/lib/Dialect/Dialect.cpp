#include "chimera/IR/Dialect.h"
#include "chimera/IR/Types.h"
#include "mlir/IR/MLIRContext.h"
#include "mlir/IR/DialectRegistry.h"
#include "mlir/IR/DialectImplementation.h"
#include "mlir/IR/Types.h"

namespace chimera {

ChimeraDialect::ChimeraDialect(mlir::MLIRContext *context)
    : mlir::Dialect(getDialectNamespace(), context,
                    mlir::TypeID::get<ChimeraDialect>()) {
  // Chimera types are semantic wrappers around MLIR builtin types (i32, i64).
  // No custom type registration needed - we use type checking functions
  // in Types.h to identify semantic meaning.
}

mlir::Type ChimeraDialect::parseType(mlir::DialectAsmParser &parser) const {
  mlir::MLIRContext *context = parser.getContext();
  llvm::StringRef keyword;

  if (parser.parseKeyword(&keyword))
    return {};

  if (keyword == "status")
    return StatusType::get(context);
  if (keyword == "error")
    return ErrorType::get(context);
  if (keyword == "borrow" || keyword == "borrow_mut" || keyword == "owned")
    return OwnedType::get(context);
  if (keyword == "result" || keyword == "slice" || keyword == "string")
    return OwnedType::get(context);
  if (keyword == "opaque")
    return OpaqueType::get(context);
  if (keyword == "ptr")
    return OwnedType::get(context);
  if (keyword == "handle")
    return OwnedType::get(context);

  return {};
}

void ChimeraDialect::printType(mlir::Type type,
                               mlir::DialectAsmPrinter &printer) const {
  // Use type checking functions from Types.h to identify Chimera types
  if (isStatusType(type)) {
    printer.getStream() << "status";
    return;
  }
  if (isErrorType(type)) {
    printer.getStream() << "error";
    return;
  }
  // For pointer types (i64), we just print "ptr" since the semantic
  // meaning (borrow, owned, handle, etc.) is determined by context
  if (isPointerType(type)) {
    printer.getStream() << "ptr";
    return;
  }
  // Note: ResultType, SliceType also use i64 physical representation
  // For now, just print generic representation
  printer.getStream() << "unknown";
}

} // namespace chimera