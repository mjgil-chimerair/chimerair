-- ChimeraProof Examples: Smoke Demo
-- Compile-safe fixture definitions for the one-binary smoke demo.

import Chimera.Foundation
import Chimera.ABI
import Chimera.Wrapper
import Chimera.Runtime.ABI

namespace Chimera.Examples.SmokeDemo

def demoCSrc : String :=
  "// smoke_demo.c\n#include <chimera_abi.h>\nch_status process_buffer(void) { return CHIMERA_STATUS_OK; }\n"

def demoRustSrc : String :=
  "// smoke_demo_lib.rs\n#[no_mangle]\npub extern \"C\" fn rust_validate() -> i32 { 0 }\n"

def demoZigSrc : String :=
  "// smoke_demo.zig\npub export fn zig_checksum() u32 { return 0; }\n"

def demoBuildScript : String :=
  "#!/bin/bash\nset -e\necho \"clang\"\necho \"rustc\"\necho \"zig\"\n"

def demoLinkScript : String :=
  "#!/bin/bash\nset -e\necho \"link\"\n"

theorem c_src_nonempty : demoCSrc ≠ "" := by
  decide

theorem rust_src_nonempty : demoRustSrc ≠ "" := by
  decide

theorem zig_src_nonempty : demoZigSrc ≠ "" := by
  decide

theorem build_script_mentions_clang :
  demoBuildScript.contains "clang" = true := by
  native_decide

theorem link_script_nonempty : demoLinkScript ≠ "" := by
  decide

end Chimera.Examples.SmokeDemo
