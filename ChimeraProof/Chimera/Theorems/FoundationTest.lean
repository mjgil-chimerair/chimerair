-- ChimeraProof Tests: Foundation
-- Tests for Foundation layer.

import Chimera.Foundation

namespace Chimera.Test

-- Target tests
namespace TargetTest

-- Test compatible targets have equal pointer width
theorem compatible_ptrWidth : (Target.x86_64_linux.compatible Target.x86_64_linux) = true := by
  simp [Target.compatible]

private def linux_soft_float : Target :=
  { Target.x86_64_linux with floatAbi := .soft }

private def linux_cdecl : Target :=
  { Target.x86_64_linux with callingConvention := .cdecl }

theorem windows_is_not_compatible_with_linux :
    ¬ Target.compatible Target.x86_64_linux Target.x86_64_windows := by
  simp [Target.compatible, Target.x86_64_linux, Target.x86_64_windows]

theorem soft_float_is_not_compatible_with_hard_float :
    ¬ Target.compatible Target.x86_64_linux linux_soft_float := by
  simp [Target.compatible, linux_soft_float, Target.x86_64_linux]

theorem different_default_calling_conventions_are_incompatible :
    ¬ Target.compatible Target.x86_64_linux linux_cdecl := by
  simp [Target.compatible, linux_cdecl, Target.x86_64_linux]

theorem compatible_targets_preserve_arch_and_os :
    Target.compatible Target.x86_64_linux Target.x86_64_linux ∧
      Target.x86_64_linux.arch = Target.x86_64_linux.arch ∧
      Target.x86_64_linux.os = Target.x86_64_linux.os := by
  simp [Target.compatible]

end TargetTest

namespace WordTest

theorem ofNat_is_bounded :
    (Word.ofNat 8 300).value < 2^8 := by
  exact Word.ofNat_value_bound 8 300

theorem width_zero_word_is_always_zero :
    (Word.ofNat 0 42).value = 0 := by
  exact Word.width_zero_valid 42

theorem add_preserves_bound :
    (Word.add (Word.ofNat 8 250) (Word.ofNat 8 10)).value < 2^8 := by
  exact Word.toNat_bound _

theorem mul_preserves_bound :
    (Word.mul (Word.ofNat 8 20) (Word.ofNat 8 20)).value < 2^8 := by
  exact Word.toNat_bound _

theorem shift_left_preserves_bound :
    (Word.shiftLeft (Word.ofNat 8 3) 6).value < 2^8 := by
  exact Word.toNat_bound _

theorem equal_values_imply_equal_words :
    Word.eq_of_value_eq (w := 8)
      (a := Word.ofNat 8 5)
      (b := Word.ofNat 8 261)
      (by decide) =
      rfl := by
  rfl

end WordTest

-- Symbol tests
namespace SymbolTest

theorem simple_symbol_fqn : (Symbol.simple "foo").fqn = "foo" := by rfl

theorem namespaced_symbol_fqn : (Symbol.namespaced "ns" "foo").fqn = "ns::foo" := by rfl

end SymbolTest

-- FinMap tests
namespace FinMapTest

theorem empty_size : (FinMap.empty String).size = 0 := by rfl

theorem find_empty : (FinMap.empty String).find? 0 = none := by rfl

end FinMapTest

end Chimera.Test
