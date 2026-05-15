// RUN: chimerac --target x86_64-unknown-linux-gnu %s 2>&1 | FileCheck %s

// Test fixture for Zig FFI-safe code through the Chimera pipeline.
// This represents the expected MLIR output from Zig lowering for:
// - extern struct with no slices crossing FFI boundary
// - export fn with primitive types only
// - no error unions in exported function signatures

// The chimera.source_lang attribute marks this as coming from Zig
module @zig_ffi_demo attributes { chimera.source_lang = "zig" } {
  // Function with primitive types only - FFI-safe
  // The chimera.export marks this as crossing the FFI boundary
  func.func @add(%arg0: i32, %arg1: i32) -> i32 {
    %result = arith.addi %arg0, %arg1 : i32
    return %result : i32
  }

  // CHECK: chimera.source_lang = "zig"
  // CHECK: func.func

  // Function with result type - represents Zig's !T error union
  // chimera.result operations would be used in real Zig output
  func.func private @might_fail(%val: i32) -> i64 {
    %c42 = arith.constant 42 : i64
    return %c42 : i64
  }
}
