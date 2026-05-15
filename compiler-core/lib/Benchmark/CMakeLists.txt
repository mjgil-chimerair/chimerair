add_library(ChimeraBenchmark STATIC
    Benchmark.cpp
)

target_include_directories(ChimeraBenchmark PUBLIC
    ${CMAKE_SOURCE_DIR}/include
    ${MLIR_INCLUDE_DIRS}
)

target_link_libraries(ChimeraBenchmark PUBLIC
    ChimeraDialect
    ChimeraLowering
    ChimeraPasses
    MLIRArithDialect
    MLIRControlFlowDialect
    MLIRFuncDialect
    MLIRIR
    MLIRParser
    MLIRPass
    MLIRSupport
    MLIRTransforms
)
