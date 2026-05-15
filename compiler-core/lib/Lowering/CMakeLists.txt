add_library(ChimeraLowering STATIC
    LLVMConversion.cpp
)

target_include_directories(ChimeraLowering PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}/include
    ${MLIR_INCLUDE_DIRS}
    ${LLVM_INCLUDE_DIRS}
)

target_link_libraries(ChimeraLowering PUBLIC
    ChimeraDialect
    ChimeraUtils
    MLIRArithToLLVM
    MLIRControlFlowToLLVM
    MLIRFuncToLLVM
    MLIRLLVMDialect
    MLIRPass
    MLIRReconcileUnrealizedCasts
    MLIRIR
    MLIRTransforms
)
