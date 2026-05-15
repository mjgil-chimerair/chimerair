# Chimera Compiler Core Libraries
#
# Library split as follows:
# - ChimeraDialect   : Core MLIR dialect (types, ops, dialect registration)
# - ChimeraPasses    : MLIR passes (canonicalization, verification, lowering)
# - ChimeraInterfaces: MLIR interfaces (CallableOpInterface, SymbolInterface, etc.)
# - ChimeraLowering  : Lowering to LLVM dialect
# - ChimeraTranslation: Translation to other formats
# - ChimeraCAPI      : C API for compiler-core
# - ChimeraUtils     : Common utilities

# ChimeraDialect is defined in lib/Dialect/CMakeLists.txt

# Chimera Passes library
add_library(ChimeraPasses STATIC
    Passes/Passes.cpp
    Passes/Verification.cpp
)

target_include_directories(ChimeraPasses PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}/include
    ${MLIR_INCLUDE_DIRS}
)

target_link_libraries(ChimeraPasses PUBLIC
    ChimeraDialect
    MLIRIR
    MLIRPass
    MLIRTransforms
)

# Chimera Interfaces library
add_library(ChimeraInterfaces STATIC)

target_include_directories(ChimeraInterfaces PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}/include
    ${MLIR_INCLUDE_DIRS}
)

target_link_libraries(ChimeraInterfaces PUBLIC
    MLIRIR
)

# Chimera Lowering library
add_library(ChimeraLowering STATIC
    Lowering/LLVMConversion.cpp
)

target_include_directories(ChimeraLowering PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}/include
    ${MLIR_INCLUDE_DIRS}
    ${LLVM_INCLUDE_DIRS}
)

target_link_libraries(ChimeraLowering PUBLIC
    ChimeraDialect
    ChimeraPasses
    MLIRLLVMIR
    MLIRArithToLLVM
    MLIRFuncToLLVM
)

# Chimera Proof Bridge library
add_library(ChimeraProofBridge STATIC
    Proof/ProofBridge.cpp
)

target_include_directories(ChimeraProofBridge PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}/include
    ${MLIR_INCLUDE_DIRS}
)

target_link_libraries(ChimeraProofBridge PUBLIC
    ChimeraDialect
    ChimeraPasses
    MLIRIR
    MLIRPass
)

# Chimera Translation library
add_library(ChimeraTranslation STATIC)

target_include_directories(ChimeraTranslation PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}/include
    ${MLIR_INCLUDE_DIRS}
)

target_link_libraries(ChimeraTranslation PUBLIC
    ChimeraLowering
    MLIRTranslateLib
)

# Chimera C API library
add_library(ChimeraCAPI STATIC
    CCAPI.cpp
)

target_include_directories(ChimeraCAPI PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}/include
    ${MLIR_INCLUDE_DIRS}
)

target_link_libraries(ChimeraCAPI PUBLIC
    ChimeraDialect
    ChimeraPasses
    MLIRIR
    MLIRParser
    MLIRSupport
)

# Chimera Utils library
add_library(ChimeraUtils STATIC
    Diagnostics.cpp
)

target_include_directories(ChimeraUtils PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}/include
    ${MLIR_INCLUDE_DIRS}
)

target_link_libraries(ChimeraUtils PUBLIC
    MLIRIR
    MLIRSupport
)

# Chimera Benchmark library
add_library(ChimeraBenchmark STATIC
    Benchmark/Benchmark.cpp
)

target_include_directories(ChimeraBenchmark PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}/include
    ${MLIR_INCLUDE_DIRS}
)

target_link_libraries(ChimeraBenchmark PUBLIC
    ChimeraDialect
    ChimeraPasses
    ChimeraLowering
    MLIRIR
    MLIRPass
    MLIRSupport
)
