add_executable(chimera-benchmark-smoke
    BenchmarkSmoke.cpp
)

target_include_directories(chimera-benchmark-smoke PRIVATE
    ${CMAKE_SOURCE_DIR}/include
    ${MLIR_INCLUDE_DIRS}
)

target_link_libraries(chimera-benchmark-smoke PRIVATE
    ChimeraBenchmark
    MLIRSupport
)
