#include "chimera/IR/Diagnostics.h"

namespace chimera {

DiagnosticEngine::DiagnosticEngine() = default;

void DiagnosticEngine::emit(mlir::Operation *op, DiagCode code,
                           const std::string &msg, DiagSeverity severity) {
  Diagnostic diag;
  diag.severity = severity;
  diag.code = code;
  diag.location = op->getLoc();
  diag.message = msg;
  diagnostics_.push_back(diag);
}

void DiagnosticEngine::emitNote(mlir::Operation *op, const std::string &msg) {
  emit(op, DiagCode::INTERNAL_ERROR, msg, DiagSeverity::Note);
}

void DiagnosticEngine::emitWarning(mlir::Operation *op, DiagCode code,
                                   const std::string &msg) {
  emit(op, code, msg, DiagSeverity::Warning);
}

void DiagnosticEngine::emitError(mlir::Operation *op, DiagCode code,
                                 const std::string &msg) {
  emit(op, code, msg, DiagSeverity::Error);
}

void DiagnosticEngine::emitFatal(mlir::Operation *op, DiagCode code,
                                 const std::string &msg) {
  emit(op, code, msg, DiagSeverity::Fatal);
}

bool DiagnosticEngine::hasErrors() const {
  for (const auto &diag : diagnostics_) {
    if (diag.severity == DiagSeverity::Error ||
        diag.severity == DiagSeverity::Fatal) {
      return true;
    }
  }
  return false;
}

const char *DiagnosticEngine::getDiagName(DiagCode code) {
  switch (code) {
  case DiagCode::PARSE_UNKNOWN_TYPE:
    return "PARSE_UNKNOWN_TYPE";
  case DiagCode::PARSE_MALFORMED_FUNCTION_TYPE:
    return "PARSE_MALFORMED_FUNCTION_TYPE";
  case DiagCode::PARSE_INVALID_LIFETIME:
    return "PARSE_INVALID_LIFETIME";
  case DiagCode::TYPE_MISMATCH:
    return "TYPE_MISMATCH";
  case DiagCode::TYPE_NOT_BORROWABLE:
    return "TYPE_NOT_BORROWABLE";
  case DiagCode::TYPE_NOT_OWNED:
    return "TYPE_NOT_OWNED";
  case DiagCode::TYPE_INVALID_RESULT:
    return "TYPE_INVALID_RESULT";
  case DiagCode::TYPE_INVALID_SLICE:
    return "TYPE_INVALID_SLICE";
  case DiagCode::OWNERSHIP_DOUBLE_BORROW:
    return "OWNERSHIP_DOUBLE_BORROW";
  case DiagCode::OWNERSHIP_USE_AFTER_MOVE:
    return "OWNERSHIP_USE_AFTER_MOVE";
  case DiagCode::OWNERSHIP_ILLEGAL_ESCAPE:
    return "OWNERSHIP_ILLEGAL_ESCAPE";
  case DiagCode::OWNERSHIP_DANGLING_REFERENCE:
    return "OWNERSHIP_DANGLING_REFERENCE";
  case DiagCode::OWNERSHIP_BORROW_EXCLUSIVITY:
    return "OWNERSHIP_BORROW_EXCLUSIVITY";
  case DiagCode::MEMORY_INVALID_ALLOC:
    return "MEMORY_INVALID_ALLOC";
  case DiagCode::MEMORY_INVALID_FREE:
    return "MEMORY_INVALID_FREE";
  case DiagCode::MEMORY_LEAK:
    return "MEMORY_LEAK";
  case DiagCode::MEMORY_DOUBLE_FREE:
    return "MEMORY_DOUBLE_FREE";
  case DiagCode::RESULT_INVALID_OK:
    return "RESULT_INVALID_OK";
  case DiagCode::RESULT_INVALID_ERR:
    return "RESULT_INVALID_ERR";
  case DiagCode::RESULT_UNWRAP_WITHOUT_CHECK:
    return "RESULT_UNWRAP_WITHOUT_CHECK";
  case DiagCode::PANIC_INVALID_MESSAGE:
    return "PANIC_INVALID_MESSAGE";
  case DiagCode::PANIC_UNWIND_MISMATCH:
    return "PANIC_UNWIND_MISMATCH";
  case DiagCode::LINK_DUPLICATE_SYMBOL:
    return "LINK_DUPLICATE_SYMBOL";
  case DiagCode::LINK_UNRESOLVED_IMPORT:
    return "LINK_UNRESOLVED_IMPORT";
  case DiagCode::LINK_TARGET_MISMATCH:
    return "LINK_TARGET_MISMATCH";
  case DiagCode::LINK_ABI_MISMATCH:
    return "LINK_ABI_MISMATCH";
  case DiagCode::INTERNAL_ERROR:
    return "INTERNAL_ERROR";
  case DiagCode::VERIFIER_FAILED:
    return "VERIFIER_FAILED";
  default:
    return "UNKNOWN";
  }
}

const char *DiagnosticEngine::getDiagMessage(DiagCode code) {
  switch (code) {
  case DiagCode::PARSE_UNKNOWN_TYPE:
    return "unknown type";
  case DiagCode::PARSE_MALFORMED_FUNCTION_TYPE:
    return "malformed function type";
  case DiagCode::PARSE_INVALID_LIFETIME:
    return "invalid lifetime specification";
  case DiagCode::TYPE_MISMATCH:
    return "type mismatch";
  case DiagCode::TYPE_NOT_BORROWABLE:
    return "type cannot be borrowed";
  case DiagCode::TYPE_NOT_OWNED:
    return "type is not owned";
  case DiagCode::TYPE_INVALID_RESULT:
    return "invalid result type";
  case DiagCode::TYPE_INVALID_SLICE:
    return "invalid slice type";
  case DiagCode::OWNERSHIP_DOUBLE_BORROW:
    return "value already borrowed";
  case DiagCode::OWNERSHIP_USE_AFTER_MOVE:
    return "use after move";
  case DiagCode::OWNERSHIP_ILLEGAL_ESCAPE:
    return "illegal ownership escape";
  case DiagCode::OWNERSHIP_DANGLING_REFERENCE:
    return "dangling reference";
  case DiagCode::OWNERSHIP_BORROW_EXCLUSIVITY:
    return "borrow exclusivity violation";
  case DiagCode::MEMORY_INVALID_ALLOC:
    return "invalid allocation";
  case DiagCode::MEMORY_INVALID_FREE:
    return "invalid free";
  case DiagCode::MEMORY_LEAK:
    return "memory leak detected";
  case DiagCode::MEMORY_DOUBLE_FREE:
    return "double free detected";
  case DiagCode::RESULT_INVALID_OK:
    return "invalid ok value";
  case DiagCode::RESULT_INVALID_ERR:
    return "invalid err value";
  case DiagCode::RESULT_UNWRAP_WITHOUT_CHECK:
    return "unwrap without check";
  case DiagCode::PANIC_INVALID_MESSAGE:
    return "invalid panic message";
  case DiagCode::PANIC_UNWIND_MISMATCH:
    return "panic/unwind mismatch";
  case DiagCode::LINK_DUPLICATE_SYMBOL:
    return "duplicate symbol";
  case DiagCode::LINK_UNRESOLVED_IMPORT:
    return "unresolved import";
  case DiagCode::LINK_TARGET_MISMATCH:
    return "target triple mismatch";
  case DiagCode::LINK_ABI_MISMATCH:
    return "ABI mismatch";
  case DiagCode::INTERNAL_ERROR:
    return "internal error";
  case DiagCode::VERIFIER_FAILED:
    return "verification failed";
  default:
    return "unknown error";
  }
}

} // namespace chimera