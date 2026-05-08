#ifndef CHIMERA_PROOF_PROOF_BRIDGE_H
#define CHIMERA_PROOF_PROOF_BRIDGE_H

#include "mlir/IR/BuiltinOps.h"
#include "mlir/IR/Operation.h"
#include "mlir/Pass/Pass.h"
#include "mlir/Support/LLVM.h"
#include <string>
#include <vector>

namespace chimera {
namespace proof {

//===----------------------------------------------------------------------===//
// Proof Obligation Kinds
//===----------------------------------------------------------------------===//

enum class ObligationKind {
  Layout,
  Signature,
  Ownership,
  Allocator,
  Result,
  Panic,
  Effects,
  Wrappers,
  Link
};

//===----------------------------------------------------------------------===//
// Proof Obligation
//===----------------------------------------------------------------------===//

struct ProofObligation {
  std::string id;
  ObligationKind kind;
  std::string target;
  std::string description;
  std::string sourceLocation;
  std::vector<std::string> assumptions;
};

//===----------------------------------------------------------------------===//
// Trust Assumption
//===----------------------------------------------------------------------===//

enum class TrustKind {
  TrustedFunction,
  TrustedAllocator,
  TrustedDrop,
  TrustedLinker,
  TrustedForeignAbi,
  ManualProof
};

struct TrustAssumption {
  TrustKind kind;
  std::string description;
  std::string sourceLocation;
  bool verified;
};

//===----------------------------------------------------------------------===//
// Proof Export
//===----------------------------------------------------------------------===//

struct ProofExportConfig {
  std::string targetTriple;
  unsigned pointerWidth;
  std::string endian;
  bool includeTrustAssumptions;
  std::string format;

  ProofExportConfig()
      : targetTriple("unknown"), pointerWidth(64), endian("little"),
        includeTrustAssumptions(true), format("json") {}
};

class ProofExporter {
public:
  explicit ProofExporter(const ProofExportConfig &config);

  std::vector<ProofObligation> extractObligations(mlir::ModuleOp module);
  std::vector<TrustAssumption> extractTrustAssumptions(mlir::ModuleOp module);
  std::string exportToJson(mlir::ModuleOp module);
  std::string exportToLean(mlir::ModuleOp module);
  const ProofExportConfig &getConfig() const { return config; }

private:
  ProofExportConfig config;

  void extractFunctionObligations(mlir::ModuleOp module,
                                   std::vector<ProofObligation> &obligations);
  void extractTypeObligations(mlir::ModuleOp module,
                               std::vector<ProofObligation> &obligations);
  void extractOperationObligations(mlir::Operation *op,
                                     std::vector<ProofObligation> &obligations);
  std::string generateObligationId(const std::string &prefix);
};

void registerProofExportPass();
std::unique_ptr<mlir::OperationPass<mlir::ModuleOp>> createProofExportPass(
    const ProofExportConfig &config = ProofExportConfig());

} // namespace proof
} // namespace chimera

#endif // CHIMERA_PROOF_PROOF_BRIDGE_H
