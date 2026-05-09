#include "chimera/IR/Dialect.h"
#include "chimera/IR/Types.h"
#include "chimera/Lowering/LLVMLowering.h"
#include "chimera/Proof/ProofBridge.h"
#include "mlir/Dialect/Arith/IR/Arith.h"
#include "mlir/Dialect/ControlFlow/IR/ControlFlow.h"
#include "mlir/Dialect/LLVMIR/LLVMDialect.h"
#include "mlir/IR/BuiltinDialect.h"
#include "mlir/IR/DialectRegistry.h"
#include "mlir/IR/MLIRContext.h"
#include "mlir/Dialect/Func/IR/FuncOps.h"
#include "mlir/IR/Verifier.h"
#include "mlir/Parser/Parser.h"
#include "mlir/Support/LogicalResult.h"
#include "llvm/Support/SourceMgr.h"
#include "llvm/Support/raw_ostream.h"
#include "llvm/Support/CommandLine.h"
#include <iostream>
#include <sstream>

namespace cl = llvm::cl;

static cl::opt<std::string> inputFile(cl::Positional,
    cl::desc("<input file>"));
static cl::opt<std::string> outputFile("o",
    cl::desc("Output file"),
    cl::value_desc("filename"));
static cl::opt<std::string> inputLang("input-lang",
    cl::desc("Input language: chimera (default), c, rust, zig"),
    cl::value_desc("lang"),
    cl::init("chimera"));
static cl::opt<bool> verify("verify",
    cl::desc("Verify the module after parsing"),
    cl::init(true));
static cl::opt<bool> verbose("v",
    cl::desc("Verbose output"));
static cl::opt<std::string> target("target",
    cl::desc("Target triple"),
    cl::value_desc("triple"));
static cl::opt<bool> emitMetadata("emit-metadata",
    cl::desc("Emit .chmeta metadata sidecar"),
    cl::init(false));
static cl::opt<std::string> metadataOutput("metadata-output",
    cl::desc("Metadata output file"),
    cl::value_desc("filename"));
static cl::opt<bool> emitProof("emit-proof",
    cl::desc("Emit .chproof proof sidecar"),
    cl::init(false));
static cl::opt<std::string> proofOutput("proof-output",
    cl::desc("Proof output file"),
    cl::value_desc("filename"));
static cl::opt<bool> emitObject("emit-object",
    cl::desc("Emit .cho object file"),
    cl::init(false));
static cl::opt<std::string> objectOutput("object-output",
    cl::desc("Object output file"),
    cl::value_desc("filename"));
static cl::opt<bool> lowerToLLVM("lower-llvm",
    cl::desc("Lower parsed MLIR to LLVM dialect before printing/emitting"),
    cl::init(false));

void printHelp() {
    std::cout << "chimerac - Chimera compiler driver\n\n";
    std::cout << "Usage: chimerac [options] <input>\n\n";
    std::cout << "Options:\n";
    std::cout << "  -o <file>           Output file\n";
    std::cout << "  --input-lang <lang> Input language: chimera, c, rust, zig (default: chimera)\n";
    std::cout << "  --target <triple>  Target triple\n";
    std::cout << "  --verify           Verify module after parsing (default: true)\n";
    std::cout << "  --emit-metadata    Emit metadata sidecar (.chmeta)\n";
    std::cout << "  --metadata-output  Metadata output file\n";
    std::cout << "  --emit-proof       Emit proof sidecar (.chproof)\n";
    std::cout << "  --proof-output     Proof output file\n";
    std::cout << "  --emit-object      Emit object file (.cho)\n";
    std::cout << "  --object-output    Object output file\n";
    std::cout << "  --lower-llvm       Lower parsed MLIR to LLVM dialect\n";
    std::cout << "  -v                 Verbose output\n";
    std::cout << "  --help             Show this help\n";
}

int main(int argc, char **argv) {
    // Load our Chimera dialect and builtin types
    mlir::DialectRegistry registry;
    chimera::lowering::registerLLVMDialects(registry);
    registry.insert<mlir::BuiltinDialect, chimera::ChimeraDialect>();
    mlir::MLIRContext context(registry);
    context.getOrLoadDialect<mlir::BuiltinDialect>();
    context.getOrLoadDialect<mlir::arith::ArithDialect>();
    context.getOrLoadDialect<mlir::cf::ControlFlowDialect>();
    context.getOrLoadDialect<mlir::func::FuncDialect>();
    context.getOrLoadDialect<chimera::ChimeraDialect>();
    context.getOrLoadDialect<mlir::LLVM::LLVMDialect>();

    // Parse command line
    cl::ParseCommandLineOptions(argc, argv, "Chimera compiler driver\n");

    if (verbose) {
        llvm::errs() << "Chimera compiler driver\n";
        llvm::errs() << "=======================\n";
        if (!inputFile.empty()) {
            llvm::errs() << "Input: " << inputFile << "\n";
        }
        if (!inputLang.empty()) {
            llvm::errs() << "Input language: " << inputLang << "\n";
        }
        if (!outputFile.empty()) {
            llvm::errs() << "Output: " << outputFile << "\n";
        }
        llvm::errs() << "\n";
    }

    // Validate input language
    if (inputLang != "chimera" && inputLang != "c" &&
        inputLang != "rust" && inputLang != "zig") {
        llvm::errs() << "Error: --input-lang must be one of: chimera, c, rust, zig\n";
        return 1;
    }

    // If no input file, just print help and exit
    if (inputFile.empty()) {
        printHelp();
        return 0;
    }

    // Parse input file using the non-template version
    llvm::SourceMgr sourceMgr;
    auto module = mlir::parseSourceFile(inputFile, sourceMgr, &context);

    if (!module) {
        llvm::errs() << "Error: failed to parse input file: " << inputFile << "\n";
        return 1;
    }

    if (verify) {
        // Verify the module
        if (mlir::failed(mlir::verify(*module))) {
            llvm::errs() << "Error: module verification failed\n";
            return 1;
        }
        if (verbose) {
            llvm::errs() << "Verification passed\n";
        }
    }

    // Log input language mode
    if (verbose && inputLang != "chimera") {
        llvm::errs() << "Input language mode: " << inputLang << "\n";
    }

    if (lowerToLLVM) {
        auto moduleOp = mlir::cast<mlir::ModuleOp>(module.get());
        if (mlir::failed(chimera::lowering::lowerModuleToLLVM(moduleOp))) {
            llvm::errs() << "Error: LLVM lowering failed\n";
            return 1;
        }
        if (verbose) {
            llvm::errs() << "LLVM lowering passed\n";
        }
    }

    // Emit metadata sidecar if requested
    if (emitMetadata) {
        std::string metadataPath = metadataOutput.empty()
            ? outputFile.empty() ? "output.chmeta" : outputFile + ".chmeta"
            : metadataOutput;

        // Count functions in the module
        int funcCount = 0;
        (*module)->walk([&funcCount](mlir::Operation *op) {
            if (mlir::isa<mlir::func::FuncOp>(op))
                funcCount++;
        });

        // Build JSON metadata matching chimera-meta schema
        std::ostringstream os;
        os << "{\n";
        os << "  \"version\": {\"major\": 0, \"minor\": 1, \"patch\": 0},\n";
        os << "  \"module\": {\n";
        os << "    \"name\": \"" << inputFile << "\",\n";
        os << "    \"target\": \"" << (target.empty() ? "unknown" : target.getValue()) << "\",\n";
        os << "    \"source_lang\": \"" << inputLang << "\"\n";
        os << "  },\n";
        os << "  \"functions\": [],\n";
        os << "  \"proof_obligations\": [],\n";
        os << "  \"wrappers\": []\n";
        os << "}\n";

        std::error_code ec;
        llvm::raw_fd_ostream fos(metadataPath, ec);
        if (ec) {
            llvm::errs() << "Error: failed to open metadata file: " << ec.message() << "\n";
            return 1;
        }
        fos << os.str();
        if (verbose) {
            llvm::errs() << "Metadata written to: " << metadataPath << "\n";
        }
    }

    // Emit proof sidecar if requested
    if (emitProof) {
        std::string proofPath = proofOutput.empty()
            ? outputFile.empty() ? "output.chproof" : outputFile + ".chproof"
            : proofOutput;

        chimera::proof::ProofExportConfig proofConfig;
        proofConfig.targetTriple = target.empty() ? "unknown" : target.getValue();
        proofConfig.pointerWidth = 64;
        proofConfig.endian = "little";

        chimera::proof::ProofExporter exporter(proofConfig);
        auto moduleOp = mlir::cast<mlir::ModuleOp>(module.get());
        auto proofJson = exporter.exportToJson(moduleOp);

        std::error_code ec;
        llvm::raw_fd_ostream fos(proofPath, ec);
        if (ec) {
            llvm::errs() << "Error: failed to open proof file: " << ec.message() << "\n";
            return 1;
        }
        fos << proofJson;
        if (verbose) {
            llvm::errs() << "Proof written to: " << proofPath << "\n";
        }
    }

    // Emit object file if requested
    if (emitObject) {
        std::string objectPath = objectOutput.empty()
            ? outputFile.empty() ? "output.cho" : outputFile + ".cho"
            : objectOutput;

        // Count functions and operations in the module
        int funcCount = 0;
        int opCount = 0;
        (*module)->walk([&funcCount, &opCount](mlir::Operation *op) {
            opCount++;
            if (mlir::isa<mlir::func::FuncOp>(op))
                funcCount++;
        });

        // Get the target triple string
        std::string targetStr = target.empty() ? "unknown" : target.getValue();

        // Build JSON metadata for the object
        std::ostringstream metadataJson;
        metadataJson << "{\n";
        metadataJson << "  \"version\": {\"major\": 0, \"minor\": 1, \"patch\": 0},\n";
        metadataJson << "  \"module\": {\n";
        metadataJson << "    \"name\": \"" << inputFile << "\",\n";
        metadataJson << "    \"target\": \"" << targetStr << "\",\n";
        metadataJson << "    \"source_lang\": \"" << inputLang << "\"\n";
        metadataJson << "  },\n";
        metadataJson << "  \"functions\": [],\n";
        metadataJson << "  \"proof_obligations\": [],\n";
        metadataJson << "  \"wrappers\": []\n";
        metadataJson << "}\n";

        // Build binary object file
        // Format: MAGIC(4) + VERSION_MAJOR(2) + VERSION_MINOR(2) +
        //         TARGET_LEN(4) + TARGET(target_len) +
        //         PAYLOAD_KIND(1) + PAYLOAD_SIZE(8) + METADATA_SIZE(8) +
        //         PAYLOAD(variable) + METADATA(variable)
        std::string irOutput;
        {
            llvm::raw_string_ostream os(irOutput);
            (*module)->print(os);
        }

        uint32_t targetLen = targetStr.size();
        uint8_t payloadKind = 2; // TextualIR
        uint64_t payloadSize = irOutput.size();
        uint64_t metadataSize = metadataJson.str().size();

        std::ostringstream objectData;
        // MAGIC
        objectData << "CHOB";
        // VERSION
        objectData << static_cast<char>(0) << static_cast<char>(1);
        // TARGET_LEN
        objectData << static_cast<char>(targetLen & 0xff)
                   << static_cast<char>((targetLen >> 8) & 0xff)
                   << static_cast<char>((targetLen >> 16) & 0xff)
                   << static_cast<char>((targetLen >> 24) & 0xff);
        // TARGET
        objectData << targetStr;
        // PAYLOAD_KIND
        objectData << static_cast<char>(payloadKind);
        // PAYLOAD_SIZE
        for (int i = 0; i < 8; i++)
            objectData << static_cast<char>((payloadSize >> (i * 8)) & 0xff);
        // METADATA_SIZE
        for (int i = 0; i < 8; i++)
            objectData << static_cast<char>((metadataSize >> (i * 8)) & 0xff);
        // PAYLOAD
        objectData << irOutput;
        // METADATA
        objectData << metadataJson.str();

        std::error_code ec;
        llvm::raw_fd_ostream fos(objectPath, ec);
        if (ec) {
            llvm::errs() << "Error: failed to open object file: " << ec.message() << "\n";
            return 1;
        }
        fos << objectData.str();
        if (verbose) {
            llvm::errs() << "Object written to: " << objectPath << "\n";
        }
    }

    // Output the module
    if (!outputFile.empty()) {
        std::string output;
        {
            llvm::raw_string_ostream os(output);
            (*module)->print(os);
        }
        std::error_code ec;
        llvm::raw_fd_ostream fos(outputFile, ec);
        if (ec) {
            llvm::errs() << "Error: failed to open output file: " << ec.message() << "\n";
            return 1;
        }
        fos << output;
        if (verbose) {
            llvm::errs() << "Output written to: " << outputFile << "\n";
        }
    } else {
        (*module)->print(llvm::outs());
    }

    return 0;
}
