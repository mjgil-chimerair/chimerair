#include "chimera/Benchmark/Benchmark.h"
#include "llvm/ADT/StringRef.h"
#include "llvm/Support/raw_ostream.h"
#include <array>
#include <string>

int main() {
  const std::string source = R"mlir(
module {
  func.func @bench(%lhs: i32, %rhs: i32) -> i32 {
    %sum = arith.addi %lhs, %rhs : i32
    return %sum : i32
  }
}
)mlir";

  chimera::benchmark::BenchmarkConfig config;
  config.iterations = 1;
  config.warmup_iterations = 0;
  chimera::benchmark::BenchmarkSuite suite(config);
  auto results = suite.runAll(source);

  if (results.size() != 5) {
    llvm::errs() << "unexpected benchmark result count\n";
    return 1;
  }

  constexpr std::array<llvm::StringRef, 5> expectedNames = {
      "parse", "verify", "passes_canonicalization", "lower_llvm",
      "emit_textual"};

  for (size_t i = 0; i < results.size(); ++i) {
    if (results[i].name != expectedNames[i] || !results[i].success) {
      llvm::errs() << "benchmark smoke failed for " << expectedNames[i] << "\n";
      return 1;
    }
  }

  suite.printResults(results, llvm::outs());
  llvm::outs() << "Benchmark smoke: OK\n";
  return 0;
}
