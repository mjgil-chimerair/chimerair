add_library(ChimeraFuzzTargets STATIC
    FuzzTargets.cpp
)

target_include_directories(ChimeraFuzzTargets PRIVATE
    ${CMAKE_SOURCE_DIR}/include
    ${MLIR_INCLUDE_DIRS}
)

target_link_libraries(ChimeraFuzzTargets PRIVATE
    ChimeraCAPI
    ChimeraDialect
    MLIRFuncDialect
    MLIRIR
    MLIRParser
    MLIRSupport
)

add_executable(chimera-fuzz-smoke
    FuzzSmoke.cpp
)

target_include_directories(chimera-fuzz-smoke PRIVATE
    ${CMAKE_SOURCE_DIR}/include
)

target_link_libraries(chimera-fuzz-smoke PRIVATE
    ChimeraFuzzTargets
)
