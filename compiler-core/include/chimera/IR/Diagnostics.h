#ifndef CHIMERA_DIAGNOSTICS_H
#define CHIMERA_DIAGNOSTICS_H

#include "mlir/IR/Location.h"
#include "mlir/IR/Operation.h"
#include "llvm/ADT/SmallVector.h"
#include "llvm/Support/SourceMgr.h"
#include <string>

namespace chimera {

enum class DiagSeverity { Note, Warning, Error, Fatal };

enum class DiagCode {
  // Parser diagnostics (1000-1999)
  PARSE_UNKNOWN_TYPE,
  PARSE_MALFORMED_FUNCTION_TYPE,
  PARSE_INVALID_LIFETIME,

  // Type diagnostics (2000-2999)
  TYPE_MISMATCH,
  TYPE_NOT_BORROWABLE,
  TYPE_NOT_OWNED,
  TYPE_INVALID_RESULT,
  TYPE_INVALID_SLICE,

  // Ownership diagnostics (3000-3999)
  OWNERSHIP_DOUBLE_BORROW,
  OWNERSHIP_USE_AFTER_MOVE,
  OWNERSHIP_ILLEGAL_ESCAPE,
  OWNERSHIP_DANGLING_REFERENCE,
  OWNERSHIP_BORROW_EXCLUSIVITY,

  // Memory diagnostics (4000-4999)
  MEMORY_INVALID_ALLOC,
  MEMORY_INVALID_FREE,
  MEMORY_LEAK,
  MEMORY_DOUBLE_FREE,

  // Result diagnostics (5000-5999)
  RESULT_INVALID_OK,
  RESULT_INVALID_ERR,
  RESULT_UNWRAP_WITHOUT_CHECK,

  // Panic diagnostics (6000-6999)
  PANIC_INVALID_MESSAGE,
  PANIC_UNWIND_MISMATCH,

  // Link diagnostics (7000-7999)
  LINK_DUPLICATE_SYMBOL,
  LINK_UNRESOLVED_IMPORT,
  LINK_TARGET_MISMATCH,
  LINK_ABI_MISMATCH,

  // Internal diagnostics (9000-9999)
  INTERNAL_ERROR,
  VERIFIER_FAILED
};

struct Diagnostic {
  DiagSeverity severity;
  DiagCode code;
  mlir::Location location{nullptr};
  std::string message;
  std::string hint;
};

class DiagnosticEngine {
public:
  DiagnosticEngine();

  void emit(mlir::Operation *op, DiagCode code, const std::string &msg,
           DiagSeverity severity = DiagSeverity::Error);

  void emitNote(mlir::Operation *op, const std::string &msg);
  void emitWarning(mlir::Operation *op, DiagCode code, const std::string &msg);
  void emitError(mlir::Operation *op, DiagCode code, const std::string &msg);
  void emitFatal(mlir::Operation *op, DiagCode code, const std::string &msg);

  bool hasErrors() const;
  const char *getDiagName(DiagCode code);
  const char *getDiagMessage(DiagCode code);

  const llvm::SmallVector<Diagnostic> &getDiagnostics() const { return diagnostics_; }

private:
  llvm::SmallVector<Diagnostic> diagnostics_;
};

} // namespace chimera

#endif // CHIMERA_DIAGNOSTICS_H