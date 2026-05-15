cmake_minimum_required(VERSION 3.25)

# Pre-set cache variables to disable problematic checks BEFORE project()
set(HAVE_FFI_CALL 0 CACHE BOOL "" FORCE)
set(LibEdit_FOUND 0 CACHE BOOL "" FORCE)

project(ChimeraCompilerCore VERSION 0.1.0 LANGUAGES C CXX)

# Set CMake policies
cmake_policy(SET CMP0042 NEW)

# Find MLIR and LLVM
find_package(MLIR REQUIRED CONFIG)
include(CTest)

list(APPEND CMAKE_MODULE_PATH "${LLVM_CMAKE_DIR}")
include(AddLLVM)

enable_testing()

# Set C++ standard
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

# Output directories
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/bin)
set(CMAKE_LIBRARY_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib)
set(CMAKE_ARCHIVE_OUTPUT_DIRECTORY ${CMAKE_BINARY_DIR}/lib)

# Include directories
include_directories(${CMAKE_CURRENT_SOURCE_DIR}/include)
include_directories(${MLIR_INCLUDE_DIRS})

# Build ChimeraDialect (base dialect) - defined in lib/Dialect
add_subdirectory(lib/Dialect)

# Build ChimeraPasses (depends on Dialect) - defined in lib/Passes
add_subdirectory(lib/Passes)

# Build ChimeraLowering (LLVM dialect lowering pipeline)
add_subdirectory(lib/Lowering)

# Build ChimeraBenchmark (parse/verify/passes/lowering/emission benchmarks)
add_subdirectory(lib/Benchmark)

# Build ChimeraProofBridge (proof obligation export)
add_subdirectory(lib/Proof)

# Build ChimeraUtils (diagnostics utilities)
add_library(ChimeraUtils STATIC
    lib/Dialect/Diagnostics.cpp
)

target_include_directories(ChimeraUtils PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}/include
    ${MLIR_INCLUDE_DIRS}
)

target_link_libraries(ChimeraUtils PUBLIC
    MLIRIR
    MLIRSupport
)

# Build ChimeraCAPI (C API support used by fuzz smoke and external consumers)
add_library(ChimeraCAPI STATIC
    lib/CCAPI.cpp
)

target_include_directories(ChimeraCAPI PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}/include
    ${MLIR_INCLUDE_DIRS}
)

target_link_libraries(ChimeraCAPI PUBLIC
    ChimeraDialect
    ChimeraPasses
    MLIRFuncDialect
    MLIRIR
    MLIRParser
    MLIRSupport
)

# Build chimerac driver
add_subdirectory(tools/driver)

# Build compiler-core fuzz smoke targets
add_subdirectory(tools/fuzz)

# Build compiler-core pass smoke target
add_subdirectory(tools/pass-smoke)

# Build compiler-core benchmark smoke target
add_subdirectory(tools/benchmark)

# Register compiler lit/ctest suites
add_subdirectory(test)
