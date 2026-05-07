#ifndef CHIMERA_IR_DIALECT_H
#define CHIMERA_IR_DIALECT_H

#include "mlir/IR/Dialect.h"
#include "mlir/IR/Types.h"

namespace chimera {

class ChimeraDialect : public mlir::Dialect {
public:
  ChimeraDialect(mlir::MLIRContext *context);

  static constexpr const char *getDialectNamespace() { return "chimera"; }

  mlir::Type parseType(mlir::DialectAsmParser &parser) const override;
  void printType(mlir::Type type, mlir::DialectAsmPrinter &printer) const override;
};

} // namespace chimera

#endif // CHIMERA_IR_DIALECT_H