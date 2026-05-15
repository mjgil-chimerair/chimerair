add_library(ChimeraPasses STATIC
    Passes.cpp
    Verification.cpp
    Ownership.cpp
    ABILayout.cpp
    ResultLowering.cpp
    PanicBoundary.cpp
    EffectTracking.cpp
)

target_include_directories(ChimeraPasses PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}/include
    ${MLIR_INCLUDE_DIRS}
)

target_link_libraries(ChimeraPasses PUBLIC
    ChimeraDialect
    MLIRPass
    MLIRIR
)
