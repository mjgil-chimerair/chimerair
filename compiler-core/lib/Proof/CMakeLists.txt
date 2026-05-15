add_library(ChimeraProofBridge STATIC
    ProofBridge.cpp
)

target_include_directories(ChimeraProofBridge PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}/include
    ${MLIR_INCLUDE_DIRS}
)

target_link_libraries(ChimeraProofBridge PUBLIC
    ChimeraDialect
    MLIRFuncDialect
    MLIRIR
    MLIRPass
    MLIRSupport
)
