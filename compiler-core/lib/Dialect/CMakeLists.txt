add_library(ChimeraDialect STATIC
    Diagnostics.cpp
    Dialect.cpp
    Types.cpp
)

target_include_directories(ChimeraDialect PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}/include
    ${MLIR_INCLUDE_DIRS}
)

target_link_libraries(ChimeraDialect PUBLIC
    MLIRIR
    MLIRSideEffectInterfaces
)