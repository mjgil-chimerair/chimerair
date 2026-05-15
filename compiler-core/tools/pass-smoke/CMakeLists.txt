add_executable(chimera-pass-smoke
    PassSmoke.cpp
)

target_include_directories(chimera-pass-smoke PRIVATE
    ${CMAKE_SOURCE_DIR}/include
    ${MLIR_INCLUDE_DIRS}
)

target_link_libraries(chimera-pass-smoke PRIVATE
    ChimeraDialect
    ChimeraPasses
    ChimeraLowering
    MLIRArithDialect
    MLIRControlFlowDialect
    MLIRFuncDialect
    MLIRIR
    MLIRParser
    MLIRPass
    MLIRSupport
)
