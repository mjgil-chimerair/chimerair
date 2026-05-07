#ifndef CHIMERA_BENCHMARK_BENCHMARK_H
#define CHIMERA_BENCHMARK_BENCHMARK_H

#include "mlir/IR/BuiltinOps.h"
#include "mlir/IR/MLIRContext.h"
#include "mlir/Pass/Pass.h"
#include "mlir/Pass/PassManager.h"
#include "llvm/Support/raw_ostream.h"
#include <memory>
#include <string>
#include <chrono>
#include <vector>

namespace chimera::benchmark {

struct BenchmarkResult {
  std::string name;
  double time_us;
  size_t memory_bytes;
  bool success;

  BenchmarkResult() : time_us(0.0), memory_bytes(0), success(false) {}
};

struct BenchmarkConfig {
  int iterations;
  int warmup_iterations;
  bool track_memory;
  std::string output_format;

  BenchmarkConfig()
      : iterations(100), warmup_iterations(10), track_memory(true),
        output_format("text") {}
};

class BenchmarkSuite {
public:
  explicit BenchmarkSuite(const BenchmarkConfig &config);

  BenchmarkResult benchmarkParse(const std::string &source);
  BenchmarkResult benchmarkVerify(const std::string &source);
  BenchmarkResult benchmarkPasses(const std::string &source,
                                  const std::string &pipeline);
  BenchmarkResult benchmarkLower(const std::string &source);
  BenchmarkResult benchmarkEmit(const std::string &source);
  std::vector<BenchmarkResult> runAll(const std::string &source);
  void printResults(const std::vector<BenchmarkResult> &results,
                    llvm::raw_ostream &os);

  const BenchmarkConfig &getConfig() const { return config; }

private:
  BenchmarkConfig config;
  std::unique_ptr<mlir::MLIRContext> context;

  mlir::OwningOpRef<mlir::ModuleOp> parseModule(const std::string &source) const;

  template <typename Func>
  BenchmarkResult timeOperation(const std::string &name, Func &&func);

  template <typename Func>
  void warmup(Func &&func, int iterations);
};

void registerBenchmarks();
std::unique_ptr<mlir::OperationPass<mlir::ModuleOp>>
createBenchmarkPass(const BenchmarkConfig &config = BenchmarkConfig{});

} // namespace chimera::benchmark

#endif // CHIMERA_BENCHMARK_BENCHMARK_H
