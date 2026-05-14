-- ChimeraProof
-- A Lean 4 formalization of the ChimeraIR MVP proof system
--
-- This module defines the Target model for ChimeraIR,
-- representing the compilation target (architecture, OS, ABI).

import Chimera.Foundation.Word
import Chimera.Foundation.Alignment

namespace Chimera

/--
Endianness represents the byte ordering of memory.
-/
inductive Endianness where
  | little  -- Little-endian (least significant byte first)
  | big     -- Big-endian (most significant byte first)
deriving Repr, BEq

/--
AbiFamily represents the calling convention family.
-/
inductive AbiFamily where
  | sysv      -- System V AMD64 ABI (Linux, macOS, BSD)
  | windows   -- Windows x64 ABI (Microsoft)
  | aapcs     -- ARM AAPCS ABI (mobile, embedded)
  | wasm      -- WebAssembly ABI
  | unknown   -- Unknown or custom ABI
deriving Repr, BEq

/--
Architecture of the compilation target.
-/
inductive Architecture where
  | x86_64
  | aarch64
  | wasm32
  | unknown
deriving Repr, BEq

/--
Operating system of the compilation target.
-/
inductive OperatingSystem where
  | linux
  | windows
  | macos
  | wasm
  | unknown
deriving Repr, BEq

/--
Floating-point ABI mode.
-/
inductive FloatAbi where
  | hard
  | soft
  | none
deriving Repr, BEq

/--
Default calling-convention family for top-level exports/imports on the target.
-/
inductive TargetCallingConvention where
  | sysv
  | windows
  | wasm
  | cdecl
  | unknown
deriving Repr, BEq

/--
SourceLanguage represents the source language of a module.
-/
inductive SourceLanguage where
  | c
  | rust
  | zig
  | ocaml
  | erlang
  | elixir
  | chimera  -- Native Chimera syntax (future)
deriving Repr, BEq

/--
Target represents a compilation target with all ABI-relevant properties.
-/
structure Target where
  /-- Target triple string (e.g., "x86_64-unknown-linux-gnu") -/
  triple     : String
  /-- Architecture of the target. -/
  arch       : Architecture
  /-- Operating system of the target. -/
  os         : OperatingSystem
  /-- Pointer width in bits -/
  ptrWidth   : Nat
  /-- Endianness of the target -/
  endian     : Endianness
  /-- ABI family -/
  abi        : AbiFamily
  /-- Float ABI mode. -/
  floatAbi   : FloatAbi
  /-- Default calling convention for cross-language ABI lowering. -/
  callingConvention : TargetCallingConvention
  /-- Width of usize/isize in bits -/
  usizeWidth : Nat
  /-- Alignment table for primitive types -/
  alignments : AlignmentTable
deriving Repr, BEq

namespace Target

/--
Default alignment for a given width.
-/
def defaultAlign (w : Nat) : Nat := w / 8

/--
Check if two targets are compatible (can be linked together).
For layout safety, must have matching architecture, OS, pointer width, endianness,
ABI family, float ABI, default calling convention, usize width, and alignments.
-/
def compatible (a b : Target) : Prop :=
  a.arch = b.arch ∧
  a.os = b.os ∧
  a.ptrWidth = b.ptrWidth ∧
  a.endian = b.endian ∧
  a.abi = b.abi ∧
  a.floatAbi = b.floatAbi ∧
  a.callingConvention = b.callingConvention ∧
  a.usizeWidth = b.usizeWidth ∧
  a.alignments = b.alignments

/--
Target compatibility theorem: compatible targets have equal architecture.
-/
theorem compatible_arch_eq {a b : Target} (h : compatible a b) :
  a.arch = b.arch := match h with | ⟨arch_eq, _, _, _, _, _, _, _, _⟩ => arch_eq

/--
Target compatibility theorem: compatible targets have equal operating system.
-/
theorem compatible_os_eq {a b : Target} (h : compatible a b) :
  a.os = b.os := match h with | ⟨_, os_eq, _, _, _, _, _, _, _⟩ => os_eq

/--
Target compatibility theorem: compatible targets have equal pointer width.
-/
theorem compatible_ptrWidth_eq {a b : Target} (h : compatible a b) :
  a.ptrWidth = b.ptrWidth := match h with | ⟨_, _, ptrWidth_eq, _, _, _, _, _, _⟩ => ptrWidth_eq

/--
Target compatibility theorem: compatible targets have equal usize width.
-/
theorem compatible_usizeWidth_eq {a b : Target} (h : compatible a b) :
  a.usizeWidth = b.usizeWidth := match h with | ⟨_, _, _, _, _, _, _, usizeWidth_eq, _⟩ => usizeWidth_eq

/--
Target compatibility theorem: compatible targets have equal endianness.
-/
theorem compatible_endian_eq {a b : Target} (h : compatible a b) :
  a.endian = b.endian := match h with | ⟨_, _, _, endian_eq, _, _, _, _, _⟩ => endian_eq

/--
Target compatibility theorem: compatible targets have equal ABI family.
-/
theorem compatible_abi_eq {a b : Target} (h : compatible a b) :
  a.abi = b.abi := match h with | ⟨_, _, _, _, abi_eq, _, _, _, _⟩ => abi_eq

/--
Target compatibility theorem: compatible targets have equal float ABI.
-/
theorem compatible_floatAbi_eq {a b : Target} (h : compatible a b) :
  a.floatAbi = b.floatAbi := match h with | ⟨_, _, _, _, _, floatAbi_eq, _, _, _⟩ => floatAbi_eq

/--
Target compatibility theorem: compatible targets have equal default calling convention.
-/
theorem compatible_callingConvention_eq {a b : Target} (h : compatible a b) :
  a.callingConvention = b.callingConvention := match h with | ⟨_, _, _, _, _, _, cc_eq, _, _⟩ => cc_eq

/--
Target compatibility theorem: compatible targets compute identical ABI layouts.
Since we now include alignments in the compatible predicate, layouts are identical.
-/
theorem compatible_layout_stable {a b : Target} (h : compatible a b) :
  a.alignments = b.alignments := match h with | ⟨_, _, _, _, _, _, _, _, alignments_eq⟩ => alignments_eq

/--
Create the standard x86_64 Linux target.
-/
def x86_64_linux : Target := {
  triple     := "x86_64-unknown-linux-gnu"
  arch       := .x86_64
  os         := .linux
  ptrWidth   := 64
  endian     := .little
  abi        := .sysv
  floatAbi   := .hard
  callingConvention := .sysv
  usizeWidth := 64
  alignments := {
    i8  := 1, i16 := 2, i32 := 4, i64 := 8,
    u8  := 1, u16 := 2, u32 := 4, u64 := 8,
    f32 := 4, f64 := 8,
    ptr := 8
  }
}

/--
Create the standard x86_64 Windows target.
-/
def x86_64_windows : Target := {
  triple     := "x86_64-pc-windows-msvc"
  arch       := .x86_64
  os         := .windows
  ptrWidth   := 64
  endian     := .little
  abi        := .windows
  floatAbi   := .hard
  callingConvention := .windows
  usizeWidth := 64
  alignments := {
    i8  := 1, i16 := 2, i32 := 4, i64 := 8,
    u8  := 1, u16 := 2, u32 := 4, u64 := 8,
    f32 := 4, f64 := 8,
    ptr := 8
  }
}

/--
Create a generic wasm target.
-/
def wasm32 : Target := {
  triple     := "wasm32-unknown-unknown"
  arch       := .wasm32
  os         := .wasm
  ptrWidth   := 32
  endian     := .little
  abi        := .wasm
  floatAbi   := .none
  callingConvention := .wasm
  usizeWidth := 32
  alignments := {
    i8  := 1, i16 := 2, i32 := 4, i64 := 8,
    u8  := 1, u16 := 2, u32 := 4, u64 := 8,
    f32 := 4, f64 := 8,
    ptr := 4
  }
}

end Target

end Chimera
