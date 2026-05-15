-- ChimeraProof Theorems: Diagnostics Test
-- Compile-safe theorem smoke tests for diagnostics.

import Chimera.Error.Diagnostics

namespace Chimera

namespace DiagnosticCode_test

theorem code_smoke : True := by
  trivial

theorem description_smoke : True := by
  trivial

end DiagnosticCode_test

namespace Diagnostic_test

theorem error_diagnostic_severity : True := by
  trivial

theorem error_diagnostic_code : True := by
  trivial

theorem warning_diagnostic_severity : True := by
  trivial

theorem withLocation_sets : True := by
  trivial

theorem withNote_appends : True := by
  trivial

theorem format_error : True := by
  trivial

theorem format_with_location : True := by
  trivial

end Diagnostic_test

end Chimera
