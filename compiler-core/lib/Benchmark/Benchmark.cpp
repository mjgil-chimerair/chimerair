#include "chimera/Benchmark/Benchmark.h"
#include "chimera/IR/Dialect.h"
#include "chimera/Lowering/LLVMLowering.h"
#include "chimera/Passes/Passes.h"
#include "mlir/Dialect/Arith/IR/Arith.h"
#include "mlir/Dialect/ControlFlow/IR/ControlFlow.h"
#include "mlir/Dialect/Func/IR/FuncOps.h"
#include "mlir/Dialect/LLVMIR/LLVMDialect.h"
#include "mlir/IR/BuiltinDialect.h"
#include "mlir/IR/Verifier.h"
#include "mlir/Parser/Parser.h"
#include "mlir/Pass/PassManager.h"
#include "mlir/Support/LogicalResult.h"
#include "llvm/ADT/StringRef.h"
#include "llvm/Support/raw_ostream.h"
#include <utility>

namespace chimera::benchmark {

BenchmarkSuite::BenchmarkSuite(const BenchmarkConfig &config)
    : config(config) {
  mlir::DialectRegistry registry;
  chimera::lowering::registerLLVMDialects(registry);
  registry.insert<mlir::BuiltinDialect, chimera::ChimeraDialect>();
  context = std::make_unique<mlir::MLIRContext>(registry);
  context->loadDialect<mlir::BuiltinDialect>();
  context->loadDialect<mlir::arith::ArithDialect>();
  context->loadDialect<mlir::cf::ControlFlowDialect>();
  context->loadDialect<mlir::func::FuncDialect>();
  context->loadDialect<chimera::ChimeraDialect>();
  context->loadDialect<mlir::LLVM::LLVMDialect>();
}

mlir::OwningOpRef<mlir::ModuleOp>
BenchmarkSuite::parseModule(const std::string &source) const {
  mlir::ParserConfig parserConfig(context.get());
  return mlir::parseSourceString<mlir::ModuleOp>(llvm::StringRef(source),
                                                 parserConfig);
}

template <typename Func>
void BenchmarkSuite::warmup(Func &&func, int iterations) {
  for (int i = 0; i < iterations; ++i) {
    (void)func();
  }
}

template <typename Func>
BenchmarkResult BenchmarkSuite::timeOperation(const std::string &name,
                                              Func &&func) {
  BenchmarkResult result;
  result.name = name;

  warmup(func, config.warmup_iterations);

  auto start = std::chrono::high_resolution_clock::now();
  bool ok = true;
  for (int i = 0; i < config.iterations; ++i) {
    ok = func() && ok;
  }
  auto end = std::chrono::high_resolution_clock::now();

  auto duration =
      std::chrono::duration_cast<std::chrono::microseconds>(end - start);
  result.time_us =
      static_cast<double>(duration.count()) / std::max(config.iterations, 1);
  result.success = ok;
  return result;
}

BenchmarkResult BenchmarkSuite::benchmarkParse(const std::string &source) {
  return timeOperation("parse", [&]() { return static_cast<bool>(parseModule(source)); });
}

BenchmarkResult BenchmarkSuite::benchmarkVerify(const std::string &source) {
  return timeOperation("verify", [&]() {
    auto module = parseModule(source);
    return module && mlir::succeeded(mlir::verify(*module));
  });
}

BenchmarkResult BenchmarkSuite::benchmarkPasses(const std::string &source,
                                                const std::string &pipeline) {
  return timeOperation("passes_" + pipeline, [&]() {
    auto module = parseModule(source);
    if (!module) {
      return false;
    }

    mlir::PassManager pm(context.get());
    pm.addPass(chimera::createCanonicalizationPass());
    return mlir::succeeded(pm.run(*module));
  });
}

BenchmarkResult BenchmarkSuite::benchmarkLower(const std::string &source) {
  return timeOperation("lower_llvm", [&]() {
    auto module = parseModule(source);
    return module && mlir::succeeded(chimera::lowering::lowerModuleToLLVM(*module));
  });
}

BenchmarkResult BenchmarkSuite::benchmarkEmit(const std::string &source) {
  return timeOperation("emit_textual", [&]() {
    auto module = parseModule(source);
    if (!module) {
      return false;
    }

    std::string output;
    llvm::raw_string_ostream os(output);
    module->print(os);
    os.flush();
    return !output.empty();
  });
}

std::vector<BenchmarkResult> BenchmarkSuite::runAll(const std::string &source) {
  std::vector<BenchmarkResult> results;
  results.push_back(benchmarkParse(source));
  results.push_back(benchmarkVerify(source));
  results.push_back(benchmarkPasses(source, "canonicalization"));
  results.push_back(benchmarkLower(source));
  results.push_back(benchmarkEmit(source));
  return results;
}

void BenchmarkSuite::printResults(const std::vector<BenchmarkResult> &results,
                                  llvm::raw_ostream &os) {
  if (config.output_format == "json") {
    os << "{\n  \"benchmarks\": [\n";
    for (size_t i = 0; i < results.size(); ++i) {
      const auto &result = results[i];
      os << "    {\"name\":\"" << result.name << "\",\"time_us\":"
         << result.time_us << ",\"memory_bytes\":" << result.memory_bytes
         << ",\"success\":" << (result.success ? "true" : "false") << "}";
      if (i + 1 != results.size()) {
        os << ",";
      }
      os << "\n";
    }
    os << "  ]\n}\n";
    return;
  }

  os << "Benchmark Results\n";
  for (const auto &result : results) {
    os << result.name << " " << result.time_us << "us "
       << (result.success ? "PASS" : "FAIL") << "\n";
  }
}

namespace {

struct BenchmarkPass
    : public mlir::PassWrapper<BenchmarkPass,
                               mlir::OperationPass<mlir::ModuleOp>> {
  BenchmarkPass() = default;
  explicit BenchmarkPass(const BenchmarkConfig &cfg) : config(cfg) {}

  void runOnOperation() override {
    std::string source;
    llvm::raw_string_ostream os(source);
    getOperation().print(os);
    os.flush();

    BenchmarkSuite suite(config);
    auto results = suite.runAll(source);
    for (const auto &result : results) {
      if (!result.success) {
        signalPassFailure();
        return;
      }
    }

    std::string rendered;
    llvm::raw_string_ostream renderedStream(rendered);
    suite.printResults(results, renderedStream);
    renderedStream.flush();
    getOperation()->setAttr("chimera.benchmark_results",
                            mlir::StringAttr::get(&getContext(), rendered));
  }

  llvm::StringRef getArgument() const override { return "chimera-benchmark"; }
  llvm::StringRef getDescription() const override {
    return "Run Chimera compiler-core benchmark suite";
  }

  BenchmarkConfig config;
};

} // namespace

void registerBenchmarks() { mlir::PassRegistration<BenchmarkPass>(); }

std::unique_ptr<mlir::OperationPass<mlir::ModuleOp>>
createBenchmarkPass(const BenchmarkConfig &config) {
  return std::make_unique<BenchmarkPass>(config);
}

} // namespace chimera::benchmark
