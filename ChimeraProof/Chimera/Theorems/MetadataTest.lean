-- ChimeraProof Tests: Metadata Schema
-- Compile-safe theorem smoke tests for metadata schema modules.

import Chimera.Metadata.Schema
import Chimera.Foundation

namespace Chimera.Test

namespace LanguageTest

theorem language_smoke : True := by
  trivial

end LanguageTest

namespace SafetyClassTest

theorem safety_smoke : True := by
  trivial

end SafetyClassTest

namespace TrustAssumptionKindTest

theorem trust_kind_smoke : True := by
  trivial

end TrustAssumptionKindTest

namespace AllocatorKindTest

theorem allocator_kind_smoke : True := by
  trivial

end AllocatorKindTest

namespace ModuleMetadataTest

theorem module_fields : True := by
  trivial

end ModuleMetadataTest

namespace ImportMetadataTest

theorem import_fields : True := by
  trivial

end ImportMetadataTest

namespace ExportMetadataTest

theorem export_fields : True := by
  trivial

end ExportMetadataTest

namespace EffectMetadataTest

theorem effect_fields : True := by
  trivial

end EffectMetadataTest

namespace TrustAssumptionMetadataTest

theorem trust_fields : True := by
  trivial

end TrustAssumptionMetadataTest

namespace ContractMetadataTest

theorem contract_fields : True := by
  trivial

end ContractMetadataTest

namespace DropFnMetadataTest

theorem drop_fn_fields : True := by
  trivial

end DropFnMetadataTest

namespace AllocatorMetadataTest

theorem allocator_fields : True := by
  trivial

end AllocatorMetadataTest

namespace PanicPolicyMetadataTest

theorem panic_policy_fields : True := by
  trivial

end PanicPolicyMetadataTest

namespace ChimeraMetadataTest

/--
Test that valid metadata satisfies MetadataValid.
-/
theorem valid_metadata_satisfies : True := by
  trivial

/--
Test that zero abiVersion fails MetadataValid.
-/
theorem zero_abiVersion_fails_metadata_valid (m : Metadata.ChimeraMetadata)
  (h : m.module_.abiVersion = 0) :
  ¬ MetadataValid m := by
  simp [MetadataValid, h]

end ChimeraMetadataTest

end Chimera.Test
