add_executable(chimera_core_tests
    TypesTest.cpp
    VerifierTest.cpp
)

target_include_directories(chimera_core_tests PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR}/include
    ${MLIR_INCLUDE_DIRS}
)

target_link_libraries(chimera_core_tests PRIVATE
    ChimeraDialect
    MLIRIR
    gtest
    gtest_main
)