set(CHIMERA_TEST_INPUT "${CMAKE_CURRENT_SOURCE_DIR}/driver_smoke.mlir")
set(CHIMERA_TEST_TMP_DIR "${CMAKE_CURRENT_BINARY_DIR}/tmp")

file(MAKE_DIRECTORY "${CHIMERA_TEST_TMP_DIR}")

add_test(
  NAME chimerac-help
  COMMAND $<TARGET_FILE:chimerac> --help
)
set_tests_properties(chimerac-help PROPERTIES
  PASS_REGULAR_EXPRESSION "Chimera compiler driver"
)

add_test(
  NAME chimerac-parse-smoke
  COMMAND $<TARGET_FILE:chimerac> "${CHIMERA_TEST_INPUT}"
)
set_tests_properties(chimerac-parse-smoke PROPERTIES
  PASS_REGULAR_EXPRESSION "func.func @test"
)

add_test(
  NAME chimerac-sidecar-smoke
  COMMAND ${CMAKE_COMMAND}
          -DCHIMERAC=$<TARGET_FILE:chimerac>
          -DINPUT=${CHIMERA_TEST_INPUT}
          -DOUTDIR=${CHIMERA_TEST_TMP_DIR}
          -P ${CMAKE_CURRENT_SOURCE_DIR}/verify_sidecars.cmake
)

add_test(
  NAME chimerac-lower-llvm-smoke
  COMMAND ${CMAKE_COMMAND}
          -DCHIMERAC=$<TARGET_FILE:chimerac>
          -DINPUT=${CMAKE_CURRENT_SOURCE_DIR}/Dialect/Lowering/llvm-lowering.mlir
          -P ${CMAKE_CURRENT_SOURCE_DIR}/verify_lowering.cmake
)

add_test(
  NAME chimera-fuzz-smoke
  COMMAND $<TARGET_FILE:chimera-fuzz-smoke>
)
set_tests_properties(chimera-fuzz-smoke PROPERTIES
  PASS_REGULAR_EXPRESSION "Fuzz smoke: OK"
)

add_test(
  NAME chimera-pass-smoke
  COMMAND $<TARGET_FILE:chimera-pass-smoke>
)
set_tests_properties(chimera-pass-smoke PROPERTIES
  PASS_REGULAR_EXPRESSION "Pass smoke: OK"
)

add_test(
  NAME chimera-benchmark-smoke
  COMMAND $<TARGET_FILE:chimera-benchmark-smoke>
)
set_tests_properties(chimera-benchmark-smoke PROPERTIES
  PASS_REGULAR_EXPRESSION "Benchmark smoke: OK"
)

# Task 110: End-to-end compiler-core driver test for Zig
# Feeds Zig-generated MLIR through Chimera verifier and proof exporter
add_test(
  NAME chimerac-zig-ffi-safe
  COMMAND $<TARGET_FILE:chimerac>
          --target x86_64-unknown-linux-gnu
          --emit-metadata
          --emit-proof
          --metadata-output "${CHIMERA_TEST_TMP_DIR}/zig_ffi_safe.chmeta"
          --proof-output "${CHIMERA_TEST_TMP_DIR}/zig_ffi_safe.chproof"
          "${CMAKE_CURRENT_SOURCE_DIR}/Dialect/Chimera/zig-ffi-safe.mlir"
)
set_tests_properties(chimerac-zig-ffi-safe PROPERTIES
  PASS_REGULAR_EXPRESSION "zig_ffi_demo|chimera.source_lang"
)
