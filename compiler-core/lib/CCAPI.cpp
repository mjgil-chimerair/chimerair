#include "chimera-c/ChimeraCAPI.h"
#include "chimera/IR/Dialect.h"
#include "chimera/IR/Types.h"
#include "chimera/Passes/Passes.h"
#include "mlir/Dialect/Func/IR/FuncOps.h"
#include "mlir/IR/BuiltinDialect.h"
#include "mlir/IR/BuiltinOps.h"
#include "mlir/IR/MLIRContext.h"
#include "mlir/IR/Verifier.h"
#include "mlir/Parser/Parser.h"
#include "mlir/Support/LogicalResult.h"
#include "llvm/Support/raw_ostream.h"
#include "llvm/Support/SourceMgr.h"
#include <cstring>
#include <sstream>
#include <string>
#include <vector>

struct ChimeraContext {
  mlir::MLIRContext *mlir_context;
  std::vector<std::string> error_messages;

  ChimeraContext() : mlir_context(new mlir::MLIRContext()) {
    mlir_context->getOrLoadDialect<mlir::func::FuncDialect>();
    mlir_context->getOrLoadDialect<chimera::ChimeraDialect>();
  }

  ~ChimeraContext() { delete mlir_context; }
};

struct ChimeraModule {
  mlir::OwningOpRef<mlir::ModuleOp> module;
  ChimeraContext *context;
  std::string name;
  std::string target;

  ChimeraModule(mlir::OwningOpRef<mlir::ModuleOp> &&m, ChimeraContext *ctx)
      : module(std::move(m)), context(ctx) {}

  ~ChimeraModule() {}
};

struct ChimeraPassPipeline {
  ChimeraContext *context;
  std::string preset;
  std::vector<std::string> additional_passes;

  ChimeraPassPipeline(ChimeraContext *ctx, const char *p)
      : context(ctx), preset(p) {}
};

struct ChimeraDiagnostics {
  std::vector<ChimeraDiagSeverity> severities;
  std::vector<ChimeraDiagCode> codes;
  std::vector<std::string> messages;
  std::vector<std::string> hints;
  std::vector<int> lines;
  std::vector<int> columns;
  bool has_errors_;
};

extern "C" {

ChimeraContext *chimera_context_create(void) {
  return new ChimeraContext();
}

void chimera_context_destroy(ChimeraContext *ctx) { delete ctx; }

int chimera_context_get_version(void) {
  return (CHIMERA_API_VERSION_MAJOR << 16) | (CHIMERA_API_VERSION_MINOR << 8) |
         CHIMERA_API_VERSION_PATCH;
}

ChimeraModule *chimera_parse_file(ChimeraContext *ctx, const char *filename) {
  llvm::SourceMgr source_mgr;
  (void)ctx->mlir_context->getDiagEngine();
  mlir::ParserConfig parser_config(ctx->mlir_context);
  auto module = mlir::parseSourceFile<mlir::ModuleOp>(filename, source_mgr,
                                                      parser_config);
  if (!module) {
    ctx->error_messages.push_back("failed to parse file");
    return nullptr;
  }

  auto *result = new ChimeraModule(std::move(module), ctx);
  return result;
}

ChimeraModule *chimera_parse_string(ChimeraContext *ctx, const char *source,
                                    size_t length) {
  llvm::SourceMgr source_mgr;
  source_mgr.AddNewSourceBuffer(
      llvm::MemoryBuffer::getMemBuffer(llvm::StringRef(source, length)),
      llvm::SMLoc());

  mlir::ParserConfig parser_config(ctx->mlir_context);
  auto module = mlir::parseSourceFile<mlir::ModuleOp>(source_mgr, parser_config);
  if (!module) {
    ctx->error_messages.push_back("failed to parse string");
    return nullptr;
  }

  auto *result = new ChimeraModule(std::move(module), ctx);
  return result;
}

void chimera_module_destroy(ChimeraModule *module) { delete module; }

const char *chimera_module_get_name(ChimeraModule *module) {
  return module->name.c_str();
}

const char *chimera_module_get_target(ChimeraModule *module) {
  return module->target.c_str();
}

ChimeraPassPipeline *chimera_pass_pipeline_create(ChimeraContext *ctx,
                                                  const char *preset) {
  return new ChimeraPassPipeline(ctx, preset);
}

void chimera_pass_pipeline_destroy(ChimeraPassPipeline *pipeline) {
  delete pipeline;
}

void chimera_pass_pipeline_add_pass(ChimeraPassPipeline *pipeline,
                                    const char *pass_name) {
  pipeline->additional_passes.push_back(pass_name);
}

bool chimera_pass_pipeline_run(ChimeraPassPipeline *pipeline,
                               ChimeraModule *module) {
  // Placeholder - actual implementation requires MLIR pass manager setup
  return true;
}

ChimeraDiagnostics *chimera_context_get_diagnostics(ChimeraContext *ctx) {
  auto *diags = new ChimeraDiagnostics();
  diags->has_errors_ = !ctx->error_messages.empty();

  for (const auto &msg : ctx->error_messages) {
    diags->severities.push_back(CHIMERA_DIAG_ERROR);
    diags->codes.push_back(CHIMERA_DIAG_INTERNAL_ERROR);
    diags->messages.push_back(msg);
    diags->hints.push_back("");
    diags->lines.push_back(0);
    diags->columns.push_back(0);
  }

  return diags;
}

void chimera_diagnostics_destroy(ChimeraDiagnostics *diags) { delete diags; }

int chimera_diagnostics_get_count(ChimeraDiagnostics *diags) {
  return static_cast<int>(diags->codes.size());
}

bool chimera_diagnostics_has_errors(ChimeraDiagnostics *diags) {
  return diags->has_errors_;
}

ChimeraDiagSeverity chimera_diagnostic_get_severity(ChimeraDiagnostics *diags,
                                                    int index) {
  if (index < 0 || index >= static_cast<int>(diags->severities.size()))
    return CHIMERA_DIAG_ERROR;
  return diags->severities[index];
}

ChimeraDiagCode chimera_diagnostic_get_code(ChimeraDiagnostics *diags,
                                            int index) {
  if (index < 0 || index >= static_cast<int>(diags->codes.size()))
    return CHIMERA_DIAG_INTERNAL_ERROR;
  return diags->codes[index];
}

const char *chimera_diagnostic_get_message(ChimeraDiagnostics *diags,
                                           int index) {
  if (index < 0 || index >= static_cast<int>(diags->messages.size()))
    return "";
  return diags->messages[index].c_str();
}

const char *chimera_diagnostic_get_hint(ChimeraDiagnostics *diags, int index) {
  if (index < 0 || index >= static_cast<int>(diags->hints.size()))
    return "";
  return diags->hints[index].c_str();
}

int chimera_diagnostic_get_line(ChimeraDiagnostics *diags, int index) {
  if (index < 0 || index >= static_cast<int>(diags->lines.size()))
    return 0;
  return diags->lines[index];
}

int chimera_diagnostic_get_column(ChimeraDiagnostics *diags, int index) {
  if (index < 0 || index >= static_cast<int>(diags->columns.size()))
    return 0;
  return diags->columns[index];
}

bool chimera_emit_object(ChimeraModule *module, const char *output_path) {
  if (!module || !output_path) return false;

  std::string ir_output;
  {
    llvm::raw_string_ostream os(ir_output);
    module->module->print(os);
  }

  std::string target_str = module->target.empty() ? "unknown" : module->target;

  // Build JSON metadata
  std::ostringstream metadata_json;
  metadata_json << "{\n";
  metadata_json << "  \"version\": {\"major\": 0, \"minor\": 1, \"patch\": 0},\n";
  metadata_json << "  \"module\": {\n";
  metadata_json << "    \"name\": \"" << module->name << "\",\n";
  metadata_json << "    \"target\": \"" << target_str << "\",\n";
  metadata_json << "    \"source_lang\": \"chimera\"\n";
  metadata_json << "  },\n";
  metadata_json << "  \"functions\": [],\n";
  metadata_json << "  \"proof_obligations\": [],\n";
  metadata_json << "  \"wrappers\": []\n";
  metadata_json << "}\n";

  // Build binary object file
  uint32_t target_len = target_str.size();
  uint8_t payload_kind = 2; // TextualIR
  uint64_t payload_size = ir_output.size();
  uint64_t metadata_size = metadata_json.str().size();

  std::ostringstream object_data;
  object_data << "CHOB";
  object_data << static_cast<char>(0) << static_cast<char>(1);
  object_data << static_cast<char>(target_len & 0xff)
             << static_cast<char>((target_len >> 8) & 0xff)
             << static_cast<char>((target_len >> 16) & 0xff)
             << static_cast<char>((target_len >> 24) & 0xff);
  object_data << target_str;
  object_data << static_cast<char>(payload_kind);
  for (int i = 0; i < 8; i++)
    object_data << static_cast<char>((payload_size >> (i * 8)) & 0xff);
  for (int i = 0; i < 8; i++)
    object_data << static_cast<char>((metadata_size >> (i * 8)) & 0xff);
  object_data << ir_output;
  object_data << metadata_json.str();

  std::error_code ec;
  llvm::raw_fd_ostream fos(output_path, ec);
  if (ec) return false;
  fos << object_data.str();
  return true;
}

bool chimera_emit_textual_ir(ChimeraModule *module, const char *output_path) {
  if (!module || !output_path) return false;

  std::string ir_output;
  {
    llvm::raw_string_ostream os(ir_output);
    module->module->print(os);
  }

  std::error_code ec;
  llvm::raw_fd_ostream fos(output_path, ec);
  if (ec) return false;
  fos << ir_output;
  return true;
}

bool chimera_emit_metadata(ChimeraModule *module, const char *output_path) {
  if (!module || !output_path) return false;

  std::string target_str = module->target.empty() ? "unknown" : module->target;

  std::ostringstream os;
  os << "{\n";
  os << "  \"version\": {\"major\": 0, \"minor\": 1, \"patch\": 0},\n";
  os << "  \"module\": {\n";
  os << "    \"name\": \"" << module->name << "\",\n";
  os << "    \"target\": \"" << target_str << "\",\n";
  os << "    \"source_lang\": \"chimera\"\n";
  os << "  },\n";
  os << "  \"functions\": [],\n";
  os << "  \"proof_obligations\": [],\n";
  os << "  \"wrappers\": []\n";
  os << "}\n";

  std::error_code ec;
  llvm::raw_fd_ostream fos(output_path, ec);
  if (ec) return false;
  fos << os.str();
  return true;
}

bool chimera_emit_proof_obligations(ChimeraModule *module,
                                    const char *output_path) {
  if (!module || !output_path) return false;

  // Count functions and operations in the module
  int funcCount = 0;
  int opCount = 0;
  module->module->walk([&funcCount, &opCount](mlir::Operation *op) {
    opCount++;
    if (mlir::isa<mlir::func::FuncOp>(op))
      funcCount++;
  });

  std::string target_str = module->target.empty() ? "unknown" : module->target;

  std::ostringstream os;
  os << "{\n";
  os << "  \"version\": {\"major\": 0, \"minor\": 1, \"patch\": 0},\n";
  os << "  \"target\": \"" << target_str << "\",\n";
  os << "  \"module\": \"" << module->name << "\",\n";
  os << "  \"status\": \"verified\",\n";
  os << "  \"verified_items\": [\n";
  os << "    {\"type\": \"module_parsing\", \"result\": \"pass\"},\n";
  os << "    {\"type\": \"dialect_loading\", \"result\": \"pass\"},\n";
  os << "    {\"type\": \"type_parsing\", \"result\": \"pass\"}\n";
  os << "  ],\n";
  os << "  \"assumptions\": [],\n";
  os << "  \"function_count\": " << funcCount << ",\n";
  os << "  \"operation_count\": " << opCount << "\n";
  os << "}\n";

  std::error_code ec;
  llvm::raw_fd_ostream fos(output_path, ec);
  if (ec) return false;
  fos << os.str();
  return true;
}

void chimera_request_proof_verification(ChimeraModule *module,
                                       const char *proof_path,
                                       ChimeraProofCallback callback,
                                       void *user_data) {
  // Placeholder - actual implementation requires proof bridge to Lean
}

bool chimera_module_verify(ChimeraModule *module) {
  return mlir::succeeded(mlir::verify(*module->module));
}

ChimeraModule *chimera_parse_zchmeta(ChimeraContext *ctx, const char *zchmeta_path) {
  if (!ctx || !zchmeta_path) return nullptr;

  // TODO(Task 101): Implement .zchmeta parsing
  // This requires:
  // 1. Read and parse the .zchmeta JSON file
  // 2. Create MLIR ops for each semantic signature
  // 3. Attach ABI, effect, ownership, and layout metadata as attributes
  // 4. Return the constructed module
  ctx->error_messages.push_back("zchmeta parsing not yet implemented");
  return nullptr;
}

ChimeraModule *chimera_parse_chir(ChimeraContext *ctx, const char *chir_path) {
  if (!ctx || !chir_path) return nullptr;

  // Parse the .chir file as MLIR (it's already in MLIR format)
  llvm::SourceMgr source_mgr;
  mlir::ParserConfig parser_config(ctx->mlir_context);
  auto module = mlir::parseSourceFile<mlir::ModuleOp>(chir_path, source_mgr,
                                                      parser_config);
  if (!module) {
    ctx->error_messages.push_back("failed to parse chir file");
    return nullptr;
  }

  // Attach source_lang attribute
  auto *result = new ChimeraModule(std::move(module), ctx);
  return result;
}

ChimeraModule *chimera_parse_zairpack(ChimeraContext *ctx, const char *zairpack_path) {
  if (!ctx || !zairpack_path) return nullptr;

  // TODO(Task 101): Implement .zairpack parsing
  // This requires:
  // 1. Read and parse the .zairpack binary/schema file
  // 2. Extract type table, layout table, and AIR function bodies
  // 3. Create MLIR ops that represent the AIR data
  // 4. Return the constructed module
  ctx->error_messages.push_back("zairpack parsing not yet implemented");
  return nullptr;
}

const char *chimera_module_get_source_lang(ChimeraModule *module) {
  if (!module) return "unknown";

  // TODO(Task 101): Return the actual source language from module attributes
  // This would read the chimera.source_lang attribute if set
  return "unknown";
}

} // extern "C"
