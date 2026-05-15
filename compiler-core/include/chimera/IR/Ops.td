#ifndef CHIMERA_IR_OPS_H
#define CHIMERA_IR_OPS_H

#include "mlir/IR/OpBase.td"

def Chimera_Dialect : mlir::Dialect {
  let name = "chimera";
  let cppNamespace = "::chimera";
  let hasConstantMaterializer = 0;
  let hasCanonicalizer = 0;
  let hasOperationPrintInvalid = 1;
}

class Chimera_Op<string mnemonic, list<Trait> traits = []>
    : mlir::Op<Chimera_Dialect, mnemonic, traits> {
  let assemblyFormat = "";
}

def FuncOp : Chimera_Op<"func", [mlir::FunctionOpInterface::Trait]> {
  let summary = "Function definition operation";

  let arguments = (ins mlir::SymbolNameAttr:$sym_name,
                       mlir::TypeAttr:$function_type,
                       Optional<mlir::DictArrayAttr>:$arg_attrs,
                       Optional<mlir::DictArrayAttr>:$res_attrs);

  let regions = (region sized<1>:$body);

  let assemblyFormat = [{
    $sym_name `(` `)` `:` $function_type ` `{`
      $body
    `}` `:` qualified($arg_attrs) `->` qualified($res_attrs)
  }];
}

def ExportOp : Chimera_Op<"export", []> {
  let summary = "Export function marker operation";

  let arguments = (ins mlir::FlatSymbolRefAttr:$target,
                       mlir::StrAttr:$cconv);

  let assemblyFormat = [{ $target `,` $cconv }];
}

def ImportOp : Chimera_Op<"import", []> {
  let summary = "Import function declaration operation";

  let arguments = (ins mlir::FlatSymbolRefAttr:$target,
                       mlir::StrAttr:$cconv,
                       mlir::StrAttr:$dylib);

  let assemblyFormat = [{ $target `,` $cconv `,` $dylib }];
}

def ModuleOp : Chimera_Op<"module", [mlir::OpTrait::IsolatedFromAbove]> {
  let summary = "Top-level module operation";

  let arguments = (ins Optional<mlir::StrAttr>:$name,
                       Optional<mlir::StrAttr>:$target,
                       Optional<mlir::StrAttr>:$source_lang);

  let regions = (region SizedRegion<1>:$body);

  let assemblyFormat = [{
    `module` (`name` `=` $name^)?
    (`target` `=` $target^)?
    (`source_lang` `=` $source_lang^)?
    `{`
      $body
    `}`
  }];
}

def BorrowOp : Chimera_Op<"borrow", []> {
  let summary = "Borrow operation";

  let arguments = (ins mlir::Value:$source);

  let results = (outs mlir::Type:$result);

  let assemblyFormat = [{ $source `:` type($source) `->` type($result) }];
}

def LendOp : Chimera_Op<"lend", []> {
  let summary = "Lend operation";

  let arguments = (ins mlir::Value:$source);

  let results = (outs mlir::Type:$result);

  let assemblyFormat = [{ $source `:` type($source) `->` type($result) }];
}

def MoveOp : Chimera_Op<"move", []> {
  let summary = "Move ownership operation";

  let arguments = (ins mlir::Value:$source);

  let results = (outs mlir::Type:$result);

  let assemblyFormat = [{ $source `:` type($source) `->` type($result) }];
}

def DropOp : Chimera_Op<"drop", []> {
  let summary = "Drop ownership operation";

  let arguments = (ins mlir::Value:$target);

  let assemblyFormat = [{ $target }];
}

def AllocOp : Chimera_Op<"alloc", []> {
  let summary = "Allocation operation";

  let arguments = (ins mlir::Type:$element_type,
                       Optional<mlir::Value>:$size);

  let results = (outs mlir::Type:$result);

  let assemblyFormat = [{ type($element_type) (`(` $size^ `:` type($size) `)`)? `->` type($result) }];
}

def FreeOp : Chimera_Op<"free", []> {
  let summary = "Free operation";

  let arguments = (ins mlir::Value:$ptr);

  let assemblyFormat = [{ $ptr }];
}

def ResultOkOp : Chimera_Op<"result.ok", []> {
  let summary = "Wrap value in Ok Result";

  let arguments = (ins mlir::Value:$value);

  let results = (outs mlir::Type:$result);

  let assemblyFormat = [{ $value `:` type($value) `->` type($result) }];
}

def ResultErrOp : Chimera_Op<"result.err", []> {
  let summary = "Wrap error in Err Result";

  let arguments = (ins mlir::Value:$error);

  let results = (outs mlir::Type:$result);

  let assemblyFormat = [{ $error `:` type($error) `->` type($result) }];
}

def ResultIsOkOp : Chimera_Op<"result.is_ok", []> {
  let summary = "Check if Result is Ok";

  let arguments = (ins mlir::Value:$result);

  let results = (outs mlir::Type:$result_type);

  let assemblyFormat = [{ $result `:` type($result) `->` type($result_type) }];
}

def PanicOp : Chimera_Op<"panic", []> {
  let summary = "Panic operation";

  let arguments = (ins mlir::Value:$message);

  let assemblyFormat = [{ $message }];
}

def CallOp : Chimera_Op<"call", []> {
  let summary = "Function call operation";

  let arguments = (ins mlir::FlatSymbolRefAttr:$callee,
                       mlir::Variadic<mlir::Value>:$operands);

  let results = (outs mlir::Variadic<mlir::Type>:$results);

  let assemblyFormat = [{
    $callee `(` $operands `:` type($operands) `)` `:` type($results)
  }];
}

#endif // CHIMERA_IR_OPS_H