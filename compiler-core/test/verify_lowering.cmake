execute_process(
  COMMAND "${CHIMERAC}" --lower-llvm "${INPUT}"
  RESULT_VARIABLE lower_result
  OUTPUT_VARIABLE lower_stdout
  ERROR_VARIABLE lower_stderr
)

if(NOT lower_result EQUAL 0)
  message(FATAL_ERROR "LLVM lowering failed:\n${lower_stderr}")
endif()

string(FIND "${lower_stdout}" "llvm.func @test_identity_i32" has_llvm_func)
if(has_llvm_func EQUAL -1)
  message(FATAL_ERROR "Lowered output did not contain expected llvm.func:\n${lower_stdout}")
endif()

string(FIND "${lower_stdout}" "llvm.add" has_llvm_add)
if(has_llvm_add EQUAL -1)
  message(FATAL_ERROR "Lowered output did not contain expected llvm.add:\n${lower_stdout}")
endif()
