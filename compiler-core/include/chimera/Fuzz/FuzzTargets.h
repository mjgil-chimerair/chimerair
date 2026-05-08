#ifndef CHIMERA_FUZZ_FUZZ_TARGETS_H
#define CHIMERA_FUZZ_FUZZ_TARGETS_H

#include <cstddef>
#include <cstdint>

namespace chimera {
namespace fuzz {

/// Fuzz target for MLIR parser input.
int runParserFuzzInput(const uint8_t* data, size_t size);

/// Fuzz target for metadata emission and loader-adjacent parsing.
int runMetadataFuzzInput(const uint8_t* data, size_t size);

/// Fuzz target for C API input handling.
int runCapiFuzzInput(const uint8_t* data, size_t size);

/// Initialize fuzzing infrastructure
void initializeFuzzing();

/// Shutdown fuzzing infrastructure
void shutdownFuzzing();

} // namespace fuzz
} // namespace chimera

#endif // CHIMERA_FUZZ_FUZZ_TARGETS_H
