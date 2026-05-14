-- ChimeraProof Link: Component
-- Component and ABI edge models.

import Chimera.Foundation
import Chimera.ABI
import Chimera.IR.Module

namespace Chimera

/--
Link mode for an ABI edge.
-/
inductive LinkMode where
  | directLink
  | staticLink
  | dynamicLink
  | runtimeDlopen
  | generatedWrapper
  deriving Repr, BEq

/--
ABI edge between a consumer and a provider.
-/
structure AbiEdge where
  consumer : Symbol
  provider : Symbol
  mode : LinkMode
  symbols : List Symbol
  deriving Repr, BEq

/--
Artifact set produced by a component build.
-/
structure ArtifactSet where
  objects : List String
  archives : List String
  sharedLibs : List String
  executables : List String
  metadata : List String
  proofs : List String
  deriving Repr, BEq

/--
Native link specification.
-/
structure NativeLinkSpec where
  objects : List String
  staticArchives : List String
  sharedLibraries : List String
  librarySearchPaths : List String
  linkLibraries : List String
  runtimeFiles : List String
  deriving Repr, BEq

/--
Public surface fingerprints.
-/
structure PublicSurfaceFingerprints where
  abi : Option String
  layout : Option String
  effect : Option String
  ownership : Option String
  deriving Repr, BEq

/--
Component model.
-/
structure Component where
  id : Symbol
  language : String
  kind : String
  artifacts : ArtifactSet
  linkSpec : NativeLinkSpec
  fingerprints : PublicSurfaceFingerprints
  deriving Repr, BEq

end Chimera
