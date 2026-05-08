#ifndef CHIMERA_C_API_H
#define CHIMERA_C_API_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// Version
#define CHIMERA_API_VERSION_MAJOR 0
#define CHIMERA_API_VERSION_MINOR 1
#define CHIMERA_API_VERSION_PATCH 0

// Opaque types
typedef struct ChimeraContext ChimeraContext;
typedef struct ChimeraModule ChimeraModule;
typedef struct ChimeraPassPipeline ChimeraPassPipeline;
typedef struct ChimeraDiagnostics ChimeraDiagnostics;

// Diagnostic codes
typedef enum {
  CHIMERA_DIAG_PARSE_UNKNOWN_TYPE = 1000,
  CHIMERA_DIAG_PARSE_MALFORMED_FUNCTION_TYPE = 1001,
  CHIMERA_DIAG_PARSE_INVALID_LIFETIME = 1002,
  CHIMERA_DIAG_TYPE_MISMATCH = 2000,
  CHIMERA_DIAG_TYPE_NOT_BORROWABLE = 2001,
  CHIMERA_DIAG_TYPE_NOT_OWNED = 2002,
  CHIMERA_DIAG_TYPE_INVALID_RESULT = 2003,
  CHIMERA_DIAG_TYPE_INVALID_SLICE = 2004,
  CHIMERA_DIAG_OWNERSHIP_DOUBLE_BORROW = 3000,
  CHIMERA_DIAG_OWNERSHIP_USE_AFTER_MOVE = 3001,
  CHIMERA_DIAG_OWNERSHIP_ILLEGAL_ESCAPE = 3002,
  CHIMERA_DIAG_OWNERSHIP_DANGLING_REFERENCE = 3003,
  CHIMERA_DIAG_OWNERSHIP_BORROW_EXCLUSIVITY = 3004,
  CHIMERA_DIAG_MEMORY_INVALID_ALLOC = 4000,
  CHIMERA_DIAG_MEMORY_INVALID_FREE = 4001,
  CHIMERA_DIAG_MEMORY_LEAK = 4002,
  CHIMERA_DIAG_MEMORY_DOUBLE_FREE = 4003,
  CHIMERA_DIAG_RESULT_INVALID_OK = 5000,
  CHIMERA_DIAG_RESULT_INVALID_ERR = 5001,
  CHIMERA_DIAG_RESULT_UNWRAP_WITHOUT_CHECK = 5002,
  CHIMERA_DIAG_PANIC_INVALID_MESSAGE = 6000,
  CHIMERA_DIAG_PANIC_UNWIND_MISMATCH = 6001,
  CHIMERA_DIAG_LINK_DUPLICATE_SYMBOL = 7000,
  CHIMERA_DIAG_LINK_UNRESOLVED_IMPORT = 7001,
  CHIMERA_DIAG_LINK_TARGET_MISMATCH = 7002,
  CHIMERA_DIAG_LINK_ABI_MISMATCH = 7003,
  CHIMERA_DIAG_INTERNAL_ERROR = 9000,
  CHIMERA_DIAG_VERIFIER_FAILED = 9001
} ChimeraDiagCode;

// Diagnostic severity
typedef enum {
  CHIMERA_DIAG_NOTE = 0,
  CHIMERA_DIAG_WARNING = 1,
  CHIMERA_DIAG_ERROR = 2,
  CHIMERA_DIAG_FATAL = 3
} ChimeraDiagSeverity;

// Proof obligation types
typedef enum {
  CHIMERA_PROOF_LAYOUT = 0,
  CHIMERA_PROOF_OWNERSHIP = 1,
  CHIMERA_PROOF_PANIC = 2,
  CHIMERA_PROOF_CONTRACT = 3
} ChimeraProofObligationType;

// Context management

ChimeraContext *chimera_context_create(void);
void chimera_context_destroy(ChimeraContext *ctx);
int chimera_context_get_version(void);

// Module loading and parsing

ChimeraModule *chimera_parse_file(ChimeraContext *ctx, const char *filename);
ChimeraModule *chimera_parse_string(ChimeraContext *ctx, const char *source,
                                     size_t length);
void chimera_module_destroy(ChimeraModule *module);
const char *chimera_module_get_name(ChimeraModule *module);
const char *chimera_module_get_target(ChimeraModule *module);

// Pass execution

ChimeraPassPipeline *chimera_pass_pipeline_create(ChimeraContext *ctx,
                                                   const char *preset);
void chimera_pass_pipeline_destroy(ChimeraPassPipeline *pipeline);
void chimera_pass_pipeline_add_pass(ChimeraPassPipeline *pipeline,
                                    const char *pass_name);
bool chimera_pass_pipeline_run(ChimeraPassPipeline *pipeline,
                               ChimeraModule *module);

// Diagnostics

ChimeraDiagnostics *chimera_context_get_diagnostics(ChimeraContext *ctx);
void chimera_diagnostics_destroy(ChimeraDiagnostics *diags);
int chimera_diagnostics_get_count(ChimeraDiagnostics *diags);
bool chimera_diagnostics_has_errors(ChimeraDiagnostics *diags);
ChimeraDiagSeverity chimera_diagnostic_get_severity(ChimeraDiagnostics *diags,
                                                    int index);
ChimeraDiagCode chimera_diagnostic_get_code(ChimeraDiagnostics *diags, int index);
const char *chimera_diagnostic_get_message(ChimeraDiagnostics *diags, int index);
const char *chimera_diagnostic_get_hint(ChimeraDiagnostics *diags, int index);
int chimera_diagnostic_get_line(ChimeraDiagnostics *diags, int index);
int chimera_diagnostic_get_column(ChimeraDiagnostics *diags, int index);

// Artifact emission

bool chimera_emit_object(ChimeraModule *module, const char *output_path);
bool chimera_emit_textual_ir(ChimeraModule *module, const char *output_path);
bool chimera_emit_metadata(ChimeraModule *module, const char *output_path);
bool chimera_emit_proof_obligations(ChimeraModule *module,
                                    const char *output_path);

// Proof bridge

typedef void (*ChimeraProofCallback)(bool success, void *user_data);

void chimera_request_proof_verification(ChimeraModule *module,
                                       const char *proof_path,
                                       ChimeraProofCallback callback,
                                       void *user_data);

// Verification

bool chimera_module_verify(ChimeraModule *module);

// Zig adapter artifact loading (Task 101)

/// Parse a `.zchmeta` Chimera metadata file into a module.
/// The metadata contains semantic signatures, ABI layouts, effects, and ownership info.
/// Returns a module with the metadata attached as attributes.
ChimeraModule *chimera_parse_zchmeta(ChimeraContext *ctx, const char *zchmeta_path);

/// Parse a `.chir` Chimera IR file (MLIR-based) into a module.
/// The IR contains typed operations for the Chimera dialect.
ChimeraModule *chimera_parse_chir(ChimeraContext *ctx, const char *chir_path);

/// Parse a `.zairpack` file (Zig AIR type/layout/function bundle) into a module.
/// This combines the AIR data with type and layout tables.
ChimeraModule *chimera_parse_zairpack(ChimeraContext *ctx, const char *zairpack_path);

/// Get the source language of a module as a string.
/// Returns "zig", "c", "rust", "mlir", or "unknown".
const char *chimera_module_get_source_lang(ChimeraModule *module);

#ifdef __cplusplus
}
#endif

#endif // CHIMERA_C_API_H
