// Chimera Fuzz Target Infrastructure
// Provides fuzzing entry points for parser, verifier, metadata loader, and C API

#include "chimera/Fuzz/FuzzTargets.h"
#include "chimera/IR/Dialect.h"
#include "chimera-c/ChimeraCAPI.h"
#include "mlir/Dialect/Func/IR/FuncOps.h"
#include "mlir/IR/BuiltinOps.h"
#include "mlir/IR/MLIRContext.h"
#include "mlir/IR/BuiltinDialect.h"
#include "mlir/IR/Verifier.h"
#include "mlir/Parser/Parser.h"
#include "mlir/Support/LogicalResult.h"
#include <cstdint>
#include <cstring>
#include <filesystem>
#include <fstream>
#include <string>
#include <vector>

namespace chimera {
namespace fuzz {

namespace {

std::string copyBytesToString(const uint8_t *data, size_t size) {
    return std::string(reinterpret_cast<const char *>(data), size);
}

} // namespace

int runParserFuzzInput(const uint8_t* data, size_t size) {
    mlir::MLIRContext context;
    context.getOrLoadDialect<mlir::BuiltinDialect>();
    context.getOrLoadDialect<mlir::func::FuncDialect>();
    context.getOrLoadDialect<ChimeraDialect>();

    std::string source = copyBytesToString(data, size);
    mlir::ParserConfig parser_config(&context);
    auto module = mlir::parseSourceString<mlir::ModuleOp>(source, parser_config);
    if (module) {
        (void)mlir::verify(*module);
    }
    return 0;
}

int runMetadataFuzzInput(const uint8_t* data, size_t size) {
    ChimeraContext* ctx = chimera_context_create();
    if (!ctx) return 0;

    std::string source = copyBytesToString(data, size);
    ChimeraModule* module = chimera_parse_string(ctx, source.data(), source.size());
    if (module) {
        auto tempPath = std::filesystem::temp_directory_path() / "chimera-fuzz-meta.json";
        chimera_emit_metadata(module, tempPath.string().c_str());
        std::error_code ec;
        std::filesystem::remove(tempPath, ec);
        chimera_module_destroy(module);
    }

    chimera_context_destroy(ctx);
    return 0;
}

int runCapiFuzzInput(const uint8_t* data, size_t size) {
    if (size < 4) {
        return 0;
    }

    uint32_t selector = 0;
    std::memcpy(&selector, data, sizeof(selector));
    size_t offset = 4;
    const char *payload = reinterpret_cast<const char *>(data + offset);
    size_t payloadSize = size - offset;

    switch (selector % 6) {
        case 0: {
            if (offset < size) {
                ChimeraContext* ctx = chimera_context_create();
                if (ctx) {
                    ChimeraModule* module = chimera_parse_string(ctx, payload, payloadSize);
                    if (module) {
                        chimera_module_verify(module);
                        chimera_module_destroy(module);
                    }
                    chimera_context_destroy(ctx);
                }
            }
            break;
        }
        case 1: {
            ChimeraContext* ctx = chimera_context_create();
            if (ctx) {
                int version = chimera_context_get_version();
                (void)version;
                chimera_context_destroy(ctx);
            }
            break;
        }
        case 2: {
            ChimeraContext* ctx = chimera_context_create();
            if (ctx) {
                ChimeraPassPipeline* pipeline = chimera_pass_pipeline_create(ctx, "default");
                if (pipeline) {
                    chimera_pass_pipeline_destroy(pipeline);
                }
                chimera_context_destroy(ctx);
            }
            break;
        }
        case 3: {
            ChimeraContext* ctx = chimera_context_create();
            if (ctx) {
                ChimeraDiagnostics* diags = chimera_context_get_diagnostics(ctx);
                if (diags) {
                    int count = chimera_diagnostics_get_count(diags);
                    bool hasErrors = chimera_diagnostics_has_errors(diags);
                    (void)count;
                    (void)hasErrors;
                    chimera_diagnostics_destroy(diags);
                }
                chimera_context_destroy(ctx);
            }
            break;
        }
        case 4: {
            ChimeraContext* ctx = chimera_context_create();
            if (ctx && offset < size) {
                ChimeraModule* module = chimera_parse_string(ctx, payload, payloadSize);
                if (module) {
                    chimera_module_verify(module);
                    chimera_module_destroy(module);
                }
                chimera_context_destroy(ctx);
            }
            break;
        }
        case 5: {
            int version = chimera_context_get_version();
            (void)version;
            break;
        }
    }

    return 0;
}

//===----------------------------------------------------------------------===//
// Initialize/Shutdown
//===----------------------------------------------------------------------===//

void initializeFuzzing() {
    // Initialize any global state needed for fuzzing
}

void shutdownFuzzing() {
    // Cleanup any global state
}

} // namespace fuzz
} // namespace chimera
