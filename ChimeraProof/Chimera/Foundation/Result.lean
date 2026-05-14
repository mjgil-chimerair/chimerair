-- ChimeraProof Foundation: Result
-- Result and error handling for the proof system.

import Chimera.Error.ChError

namespace Chimera

/--
Result type alias using the canonical ChError from Error.ChError.
-/
abbrev Result α := Except ChError α

namespace Result

/--
Return successfully.
-/
def ok {α : Type} (a : α) : Result α :=
  Except.ok a

/--
Return an error.
-/
def err {α : Type} (e : ChError) : Result α :=
  Except.error e

/--
Map over the success value.
-/
def map {α β : Type} (f : α → β) (r : Result α) : Result β :=
  match r with
  | Except.ok a => Except.ok (f a)
  | Except.error e => Except.error e

/--
Bind two results.
-/
def bind {α β : Type} (r : Result α) (f : α → Result β) : Result β :=
  match r with
  | Except.ok a => f a
  | Except.error e => Except.error e

/--
Get the value or a default.
-/
def getOr {α : Type} (r : Result α) (default : α) : α :=
  match r with
  | Except.ok a => a
  | Except.error _ => default

end Result

end Chimera