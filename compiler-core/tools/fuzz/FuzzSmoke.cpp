#include "chimera/Fuzz/FuzzTargets.h"
#include <cstdint>
#include <cstdio>
#include <cstring>
#include <vector>

int main() {
  chimera::fuzz::initializeFuzzing();

  const char parserSample[] = "module { func.func @f() { return } }";
  const char metadataSample[] = "module { func.func @meta() { return } }";
  const char capiSample[] = "module { func.func @capi() { return } }";

  std::vector<uint8_t> parserBytes(parserSample, parserSample + std::strlen(parserSample));
  std::vector<uint8_t> metadataBytes(metadataSample, metadataSample + std::strlen(metadataSample));
  std::vector<uint8_t> capiBytes = {0, 0, 0, 0};
  capiBytes.insert(capiBytes.end(), capiSample, capiSample + std::strlen(capiSample));

  if (chimera::fuzz::runParserFuzzInput(parserBytes.data(), parserBytes.size()) != 0) {
    std::fprintf(stderr, "parser fuzz smoke failed\n");
    return 1;
  }

  if (chimera::fuzz::runMetadataFuzzInput(metadataBytes.data(), metadataBytes.size()) != 0) {
    std::fprintf(stderr, "metadata fuzz smoke failed\n");
    return 1;
  }

  if (chimera::fuzz::runCapiFuzzInput(capiBytes.data(), capiBytes.size()) != 0) {
    std::fprintf(stderr, "capi fuzz smoke failed\n");
    return 1;
  }

  chimera::fuzz::shutdownFuzzing();
  std::puts("Fuzz smoke: OK");
  return 0;
}
