file(MAKE_DIRECTORY "${OUTDIR}")

set(METADATA "${OUTDIR}/driver_smoke.chmeta")
set(PROOF "${OUTDIR}/driver_smoke.chproof")
set(OBJECT "${OUTDIR}/driver_smoke.cho")

execute_process(
  COMMAND "${CHIMERAC}"
          --emit-metadata
          --metadata-output "${METADATA}"
          --emit-proof
          --proof-output "${PROOF}"
          --emit-object
          --object-output "${OBJECT}"
          --target x86_64-unknown-linux-gnu
          "${INPUT}"
  RESULT_VARIABLE CHIMERAC_STATUS
  OUTPUT_VARIABLE CHIMERAC_STDOUT
  ERROR_VARIABLE CHIMERAC_STDERR
)

if(NOT CHIMERAC_STATUS EQUAL 0)
  message(FATAL_ERROR
    "chimerac sidecar emission failed.\nstdout:\n${CHIMERAC_STDOUT}\nstderr:\n${CHIMERAC_STDERR}")
endif()

foreach(artifact IN ITEMS "${METADATA}" "${PROOF}" "${OBJECT}")
  if(NOT EXISTS "${artifact}")
    message(FATAL_ERROR "Expected sidecar artifact missing: ${artifact}")
  endif()
endforeach()

file(READ "${METADATA}" METADATA_CONTENTS)
file(READ "${PROOF}" PROOF_CONTENTS)
file(READ "${OBJECT}" OBJECT_CONTENTS HEX)

if(NOT METADATA_CONTENTS MATCHES "\"source_lang\": \"chimera\"")
  message(FATAL_ERROR "Metadata sidecar is missing the expected source_lang field.")
endif()

if(NOT PROOF_CONTENTS MATCHES "\"build_id\": \"chimera-export\"")
  message(FATAL_ERROR "Proof sidecar is missing the expected proof export build_id.")
endif()

if(NOT PROOF_CONTENTS MATCHES "\"kind\": \"signature\"")
  message(FATAL_ERROR "Proof sidecar is missing signature obligations.")
endif()

if(NOT PROOF_CONTENTS MATCHES "\"kind\": \"trusted_foreign_abi\"")
  message(FATAL_ERROR "Proof sidecar is missing expected trust assumptions for external functions.")
endif()

if(NOT OBJECT_CONTENTS MATCHES "^43484f42")
  message(FATAL_ERROR "Object sidecar is missing the CHOB magic header.")
endif()
