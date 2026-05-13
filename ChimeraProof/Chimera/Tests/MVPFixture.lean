-- ChimeraProof Tests: MVP Fixture
-- End-to-end fixture for C/Rust/Zig MVP build scenarios.

import Chimera.Foundation
import Chimera.ABI
import Chimera.ABI.Contract
import Chimera.IR.Module
import Chimera.Link
import Chimera.Wrapper
import Chimera.Wrapper.AST
import Chimera.Metadata.Schema
import Chimera.Metadata.CHO
import Chimera.Metadata.CHProof
import Chimera.Metadata.ProofReport
import Chimera.Runtime.ABI

namespace Chimera.Tests.MVP

/--
Test fixture: simple C module targeting x86_64 Linux.
-/
def mvpCModule : Module := {
  moduleName := ⟨"", "mvp_c_test"⟩,
  abiVersion := 1,
  language := .c,
  target := Target.x86_64_linux,
  exports := [
    Export.mk ⟨"", "process_buffer"⟩ default {
      args := [.slice (.int 8 Signedness.unsigned)]
      returns := .result (.value (.int 64 Signedness.unsigned)) (.value (.int 32 Signedness.unsigned))
      effects := { .mayError }
      panic := .forbidden
      safety := .verified
    } true
  ],
  imports := [],
  types := [],
  layouts := [],
  allocators := [],
  contracts := [],
  panicPolicy := .forbidden,
  trustLedger := default
}

/--
Test fixture: simple Rust module targeting x86_64 Linux.
-/
def mvpRustModule : Module := {
  moduleName := ⟨"", "mvp_rust_test"⟩,
  abiVersion := 1,
  language := .rust,
  target := Target.x86_64_linux,
  exports := [
    Export.mk ⟨"", "validate_config"⟩ default {
      args := [.opaque]
      returns := .result (.value (.int 64 Signedness.unsigned)) (.value (.int 32 Signedness.unsigned))
      effects := { .mayError }
      panic := .forbidden
      safety := .generated
    } true
  ],
  imports := [],
  types := [],
  layouts := [],
  allocators := [],
  contracts := [],
  panicPolicy := .forbidden,
  trustLedger := default
}

/--
Test fixture: simple Zig module targeting x86_64 Linux.
-/
def mvpZigModule : Module := {
  moduleName := ⟨"", "mvp_zig_test"⟩,
  abiVersion := 1,
  language := .zig,
  target := Target.x86_64_linux,
  exports := [
    Export.mk ⟨"", "calculate_checksum"⟩ default {
      args := [.slice (.int 8 Signedness.unsigned)]
      returns := .result (.value (.int 64 Signedness.unsigned)) (.value (.int 32 Signedness.unsigned))
      effects := { .mayError }
      panic := .catch
      safety := .generated
    } true
  ],
  imports := [],
  types := [],
  layouts := [],
  allocators := [],
  contracts := [],
  panicPolicy := .forbidden,
  trustLedger := default
}

/--
Test fixture: C wrapper generation from contracts.
-/
def mvpCWrappers : WrapperModule :=
  let contracts := mvpCModule.contracts ++ mvpRustModule.contracts ++ mvpZigModule.contracts
  Wrapper.generateWrapperModule contracts .c

/--
Test fixture: Rust wrapper generation from contracts.
-/
def mvpRustWrappers : WrapperModule :=
  let contracts := mvpCModule.contracts ++ mvpRustModule.contracts ++ mvpZigModule.contracts
  Wrapper.generateWrapperModule contracts .rust

/--
Test fixture: Zig wrapper generation from contracts.
-/
def mvpZigWrappers : WrapperModule :=
  let contracts := mvpCModule.contracts ++ mvpRustModule.contracts ++ mvpZigModule.contracts
  Wrapper.generateWrapperModule contracts .zig

/--
Test fixture: .cho file with object payload.
-/
def mvpCHOFile : Metadata.ChimeraObjectFile :=
  Metadata.ChimeraObjectFile.empty .object 64 .little

/--
Test fixture: proof report for MVP build.
-/
def mvpProofReport : Metadata.ProofReport :=
  Metadata.ProofReport.empty "mvp-build-001" 64 .little

/--
Verify MVP C module has correct target.
-/
theorem mvp_c_module_target :
  mvpCModule.target.ptrWidth = 64 := by rfl

/--
Verify MVP Rust module has correct target.
-/
theorem mvp_rust_module_target :
  mvpRustModule.target.ptrWidth = 64 := by rfl

/--
Verify MVP Zig module has correct target.
-/
theorem mvp_zig_module_target :
  mvpZigModule.target.ptrWidth = 64 := by rfl

/--
Verify C and Rust modules have compatible targets.
-/
theorem mvp_targets_compatible :
  mvpCModule.target.ptrWidth = mvpRustModule.target.ptrWidth ∧
  mvpCModule.target.endian = mvpRustModule.target.endian := by
  simp [mvpCModule, mvpRustModule, Target.x86_64_linux]

/--
Verify wrapper module includes C runtime.
-/
theorem mvp_c_wrappers_include_runtime :
  mvpCWrappers.includesRuntime = true := by
  simp [mvpCWrappers, generateWrapperModule]

/--
Verify Rust wrapper module excludes runtime.
-/
theorem mvp_rust_wrappers_no_runtime :
  mvpRustWrappers.includesRuntime = false := by
  simp [mvpRustWrappers, generateWrapperModule]

/--
Verify .cho header is well-formed.
-/
theorem mvp_cho_header_wf :
  Metadata.CHO_header.WellFormed mvpCHOFile.header := by
    simp [mvpCHOFile, Metadata.ChimeraObjectFile.empty]

/--
Verify proof report is empty and all proved.
-/
theorem mvp_proof_report_all_proved :
  mvpProofReport.summary.all_proved = true := by rfl

/--
Verify C header content is non-empty.
-/
theorem mvp_abi_h_nonempty :
  Runtime.chimera_abi_h_content ≠ "" := by
  simp [Runtime.chimera_abi_h_content]
  decide

/--
Verify Rust ABI content is non-empty.
-/
theorem mvp_abi_rust_nonempty :
  Runtime.chimera_abi_rust_content ≠ "" := by
  simp [Runtime.chimera_abi_rust_content]
  decide

/--
Verify Zig ABI content is non-empty.
-/
theorem mvp_abi_zig_nonempty :
  Runtime.chimera_abi_zig_content ≠ "" := by
  simp [Runtime.chimera_abi_zig_content]
  decide

end Chimera.Tests.MVP

