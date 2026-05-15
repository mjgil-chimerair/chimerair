add_executable(chimerac
    driver.cpp
)

target_include_directories(chimerac PRIVATE
    ${CMAKE_SOURCE_DIR}/include
    ${MLIR_INCLUDE_DIRS}
)

target_link_libraries(chimerac PRIVATE
    ChimeraDialect
    ChimeraLowering
    ChimeraProofBridge
    MLIRParser
    MLIRIR
    MLIRSupport
    MLIRFuncDialect
)
