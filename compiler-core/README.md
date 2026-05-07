# Chimera Compiler Core

C++/MLIR implementation of the ChimeraIR compiler.

## Status

**Section B in progress** (Tasks 9-18): Bootstrap and MLIR substrate

## Directory Structure

```
compiler-core/
├── CMakeLists.txt           # Main build configuration
├── include/
│   └── chimera/IR/          # Dialect, types, ops headers
├── lib/
│   └── Dialect/             # Dialect implementation
│       └── Chimera/         # Chimera-specific dialect
├── tools/
│   └── driver/              # chimerac compiler driver
├── test/
│   └── Dialect/             # lit tests for Chimera dialect
└── unittests/               # gtest unit tests
```

## Requirements

- CMake 3.25+
- Ninja build system
- LLVM 17 with MLIR
- C++17 compiler

## Building

```bash
mkdir build && cd build
cmake .. -G Ninja \
    -DLLVM_DIR=/path/to/llvm/lib/cmake/llvm \
    -DMLIR_DIR=/path/to/mlir/lib/cmake/mlir
ninja
```

## Running Tests

```bash
cd build
ctest --output-on-failure
```

## Components

### Library Graph

The compiler-core is split into these libraries (in build order):

```
ChimeraDialect       - Core MLIR dialect (types, ops, dialect registration)
ChimeraPasses        - MLIR passes (canonicalization, verification, lowering)
ChimeraInterfaces    - MLIR interfaces (CallableOpInterface, SymbolInterface)
ChimeraLowering      - Lowering to LLVM dialect
ChimeraTranslation   - Translation to other formats
ChimeraCAPI          - C API for compiler-core
ChimeraUtils         - Common utilities (diagnostics)
```

Dependencies:
- ChimeraDialect: MLIRIR, MLIRSideEffectInterfaces
- ChimeraPasses: ChimeraDialect, MLIRPass, MLIRTransforms
- ChimeraInterfaces: MLIRIR
- ChimeraLowering: ChimeraDialect, ChimeraPasses, MLIRLLVMIR, MLIRArithToLLVM, MLIRFuncToLLVM
- ChimeraTranslation: ChimeraLowering, MLIRTranslateLib
- ChimeraCAPI: ChimeraDialect, ChimeraPasses, MLIRIR, MLIRParser, MLIRSupport
- ChimeraUtils: MLIRIR, MLIRSupport

### Dialect (lib/Dialect/Chimera/)

The Chimera dialect provides the core IR types and operations:

- **Dialect.cpp** - Dialect registration and verification
- **Types.cpp** - Type system implementation (Status, Error, Borrow, Owned, Result, etc.)

### Types (include/chimera/IR/Types.h)

ChimeraIR semantic types:

- `StatusType` - Status code wrapper
- `ErrorType` - Error domain wrapper
- `BorrowType` / `BorrowMutType` - Borrowed reference types
- `OwnedType` - Owned value type
- `ResultType` - Result[T, E] type
- `SliceType` - Dynamic-sized slice
- `StringType` - String type
- `OpaqueType` - Opaque handle type
- `HandleType` - Named resource handle

### Operations (include/chimera/IR/Ops.td)

Core operations:

- `module` - Top-level module
- `func` - Function definition
- `import` / `export` - Cross-language boundaries
- `borrow` / `lend` / `move` / `drop` - Ownership operations
- `alloc` / `free` - Memory operations
- `result.ok` / `result.err` / `result.is_ok` - Result handling
- `panic` - Panic boundary
- `call` - Function call

## MLIR Integration

The compiler uses MLIR's dialect infrastructure for:

- Type parsing and printing
- Operation parsing and printing
- Verification and canonicalization
- Pass pipeline integration
- Lowering to LLVM dialect