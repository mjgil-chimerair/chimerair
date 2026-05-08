// Implementation of proof bridge export for compiler-core
// Emits Lean/Rust-checkable proof facts from the MLIR IR

#include "chimera/Proof/ProofBridge.h"
#include "mlir/IR/BuiltinOps.h"
#include "mlir/IR/Operation.h"
#include "mlir/IR/BuiltinAttributes.h"
#include "mlir/IR/SymbolTable.h"
#include "mlir/IR/Visitors.h"
#include "mlir/Dialect/Func/IR/FuncOps.h"
#include "mlir/Pass/Pass.h"
#include "llvm/ADT/StringRef.h"
#include "llvm/Support/raw_ostream.h"
#include <ctime>
#include <sstream>

namespace chimera {
namespace proof {

namespace {

std::string stringifyType(mlir::Type type) {
  std::string rendered;
  llvm::raw_string_ostream os(rendered);
  type.print(os);
  return rendered;
}

const char *obligationKindToString(ObligationKind kind) {
  switch (kind) {
  case ObligationKind::Layout:
    return "layout";
  case ObligationKind::Signature:
    return "signature";
  case ObligationKind::Ownership:
    return "ownership";
  case ObligationKind::Allocator:
    return "allocator";
  case ObligationKind::Result:
    return "result";
  case ObligationKind::Panic:
    return "panic";
  case ObligationKind::Effects:
    return "effects";
  case ObligationKind::Wrappers:
    return "wrappers";
  case ObligationKind::Link:
    return "link";
  }
  return "unknown";
}

const char *trustKindToString(TrustKind kind) {
  switch (kind) {
  case TrustKind::TrustedFunction:
    return "trusted_function";
  case TrustKind::TrustedAllocator:
    return "trusted_allocator";
  case TrustKind::TrustedDrop:
    return "trusted_drop";
  case TrustKind::TrustedLinker:
    return "trusted_linker";
  case TrustKind::TrustedForeignAbi:
    return "trusted_foreign_abi";
  case TrustKind::ManualProof:
    return "manual_proof";
  }
  return "unknown";
}

} // namespace

//===----------------------------------------------------------------------===//
// Proof Export Implementation
//===----------------------------------------------------------------------===//

ProofExporter::ProofExporter(const ProofExportConfig &config) : config(config) {}

std::string ProofExporter::generateObligationId(const std::string &prefix) {
  static int counter = 0;
  std::string id = prefix + "_" + std::to_string(counter++);
  return id;
}

void ProofExporter::extractFunctionObligations(mlir::ModuleOp module,
                                                std::vector<ProofObligation> &obligations) {
  module.walk([&](mlir::func::FuncOp func) {
    // Extract signature obligations
    {
      ProofObligation obl;
      obl.id = generateObligationId("sig");
      obl.kind = ObligationKind::Signature;
      obl.target = func.getName().str();
      obl.description = "Function signature check for " + func.getName().str();
      obl.sourceLocation = "";

      // Add assumptions about parameters
      auto funcType = func.getFunctionType();
      for (unsigned i = 0; i < funcType.getNumInputs(); ++i) {
        std::string paramAssumption =
            "param_" + std::to_string(i) + " : " +
            stringifyType(funcType.getInput(i));
        obl.assumptions.push_back(paramAssumption);
      }

      obligations.push_back(obl);
    }

    // Extract ownership obligations for functions with body
    if (func.isPrivate() && func.getBody().empty() == false) {
      ProofObligation obl;
      obl.id = generateObligationId("owner");
      obl.kind = ObligationKind::Ownership;
      obl.target = func.getName().str();
      obl.description = "Ownership check for " + func.getName().str();
      obl.sourceLocation = "";
      obl.assumptions.push_back("No aliased mutable references");
      obligations.push_back(obl);
    }

    // Extract result obligations
    {
      ProofObligation obl;
      obl.id = generateObligationId("result");
      obl.kind = ObligationKind::Result;
      obl.target = func.getName().str();
      obl.description = "Result/panic policy check for " + func.getName().str();
      obl.sourceLocation = "";
      obligations.push_back(obl);
    }
  });
}

void ProofExporter::extractTypeObligations(mlir::ModuleOp module,
                                             std::vector<ProofObligation> &obligations) {
  module.walk([&](mlir::func::FuncOp func) {
    auto funcType = func.getFunctionType();

    // Check input types
    for (unsigned i = 0; i < funcType.getNumInputs(); ++i) {
      ProofObligation obl;
      obl.id = generateObligationId("layout");
      obl.kind = ObligationKind::Layout;
      obl.target = "param_" + std::to_string(i);
      obl.description = "Layout check for parameter type " +
                       stringifyType(funcType.getInput(i));
      obl.sourceLocation = "";
      obligations.push_back(obl);
    }

    // Check result types
    for (unsigned i = 0; i < funcType.getNumResults(); ++i) {
      ProofObligation obl;
      obl.id = generateObligationId("layout");
      obl.kind = ObligationKind::Layout;
      obl.target = "result_" + std::to_string(i);
      obl.description = "Layout check for result type " +
                       stringifyType(funcType.getResult(i));
      obl.sourceLocation = "";
      obligations.push_back(obl);
    }
  });
}

void ProofExporter::extractOperationObligations(mlir::Operation *op,
                                                  std::vector<ProofObligation> &obligations) {
  // Check for FFI/call operations that need link obligations
  if (mlir::isa<mlir::func::CallOp>(op)) {
    ProofObligation obl;
    obl.id = generateObligationId("link");
    obl.kind = ObligationKind::Link;
    obl.target = mlir::cast<mlir::func::CallOp>(op).getCallee().str();
    obl.description = "Link/ABI check for call to " +
                     mlir::cast<mlir::func::CallOp>(op).getCallee().str();
    obl.sourceLocation = "";
    obligations.push_back(obl);
  }
}

std::vector<ProofObligation> ProofExporter::extractObligations(mlir::ModuleOp module) {
  std::vector<ProofObligation> obligations;

  extractFunctionObligations(module, obligations);
  extractTypeObligations(module, obligations);

  // Walk all operations for additional obligations
  module.walk([&](mlir::Operation *op) {
    extractOperationObligations(op, obligations);
  });

  return obligations;
}

std::vector<TrustAssumption> ProofExporter::extractTrustAssumptions(mlir::ModuleOp module) {
  std::vector<TrustAssumption> assumptions;

  module.walk([&](mlir::func::FuncOp func) {
    // Check for external functions that need trust assumptions
    if (func.isExternal()) {
      TrustAssumption ta;
      ta.kind = TrustKind::TrustedForeignAbi;
      ta.description = "External function " + func.getName().str() +
                      " is ABI-compliant";
      ta.sourceLocation = "";
      ta.verified = false;
      assumptions.push_back(ta);
    }
  });

  return assumptions;
}

std::string ProofExporter::exportToJson(mlir::ModuleOp module) {
  std::ostringstream oss;

  auto obligations = extractObligations(module);
  auto trustAssumptions = extractTrustAssumptions(module);

  oss << "{\n";
  oss << "  \"build_id\": \"chimera-export\",\n";
  oss << "  \"timestamp\": " << std::time(nullptr) << ",\n";
  oss << "  \"target_triple\": \"" << config.targetTriple << "\",\n";
  oss << "  \"target_ptr_width\": " << config.pointerWidth << ",\n";
  oss << "  \"target_endian\": \"" << config.endian << "\",\n";
  oss << "  \"obligations\": [\n";

  for (size_t i = 0; i < obligations.size(); ++i) {
    const auto &obl = obligations[i];
    oss << "    {\n";
    oss << "      \"id\": \"" << obl.id << "\",\n";
    oss << "      \"kind\": \"" << obligationKindToString(obl.kind) << "\",\n";
    oss << "      \"target\": \"" << obl.target << "\",\n";
    oss << "      \"description\": \"" << obl.description << "\",\n";
    oss << "      \"assumptions\": [";

    for (size_t j = 0; j < obl.assumptions.size(); ++j) {
      oss << "\"" << obl.assumptions[j] << "\"";
      if (j < obl.assumptions.size() - 1) oss << ", ";
    }

    oss << "]\n";
    oss << "    }";
    if (i < obligations.size() - 1) oss << ",";
    oss << "\n";
  }

  oss << "  ],\n";
  oss << "  \"trust_assumptions\": [\n";

  for (size_t i = 0; i < trustAssumptions.size(); ++i) {
    const auto &ta = trustAssumptions[i];
    oss << "    {\n";
    oss << "      \"kind\": \"" << trustKindToString(ta.kind) << "\",\n";
    oss << "      \"description\": \"" << ta.description << "\",\n";
    oss << "      \"verified\": " << (ta.verified ? "true" : "false") << "\n";
    oss << "    }";
    if (i < trustAssumptions.size() - 1) oss << ",";
    oss << "\n";
  }

  oss << "  ]\n";
  oss << "}\n";

  return oss.str();
}

std::string ProofExporter::exportToLean(mlir::ModuleOp module) {
  std::ostringstream oss;

  auto obligations = extractObligations(module);

  oss << "-- Auto-generated proof obligations from Chimera compiler-core\n";
  oss << "-- Target: " << config.targetTriple << "\n";
  oss << "-- Pointer width: " << config.pointerWidth << "\n\n";

  oss << "namespace ChimeraProof.Exported\n\n";

  // Generate Lean declarations for each obligation
  for (const auto &obl : obligations) {
    oss << "-- " << obl.description << "\n";
    oss << "structure Obligation_" << obl.id << " where\n";
    oss << "  id : String := \"" << obl.id << "\"\n";
    oss << "  kind : String := \"" << obligationKindToString(obl.kind) << "\"\n";
    oss << "  target : String := \"" << obl.target << "\"\n";
    oss << "\n";
  }

  oss << "end ChimeraProof.Exported\n";

  return oss.str();
}

//===----------------------------------------------------------------------===//
// Proof Export Pass
//===----------------------------------------------------------------------===//

namespace {

struct ProofExportPass
    : public mlir::PassWrapper<ProofExportPass, mlir::OperationPass<mlir::ModuleOp>> {
  ProofExportPass() = default;
  ProofExportPass(const ProofExportConfig &config) : exportConfig(config) {}

  void runOnOperation() override {
    auto module = getOperation();
    ProofExporter exporter(exportConfig);

    // Extract and emit obligations
    auto jsonOutput = exporter.exportToJson(module);

    // Attach the JSON as an attribute on the module for downstream consumption
    module->setAttr("chimera.proof_export",
                    mlir::StringAttr::get(&getContext(), jsonOutput));
  }

  llvm::StringRef getArgument() const override { return "chimera-export-proofs"; }
  llvm::StringRef getDescription() const override { return "Export proof obligations"; }

  ProofExportConfig exportConfig;
};

} // namespace

void registerProofExportPass() {
  mlir::PassRegistration<ProofExportPass>();
}

std::unique_ptr<mlir::OperationPass<mlir::ModuleOp>> createProofExportPass(
    const ProofExportConfig &config) {
  return std::make_unique<ProofExportPass>(config);
}

} // namespace proof
} // namespace chimera
