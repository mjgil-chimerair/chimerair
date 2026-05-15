-- ChimeraProof Wrapper: AST
-- Wrapper abstract syntax tree.

import Chimera.Foundation
import Chimera.ABI
import Chimera.ABI.Contract

namespace Chimera.Wrapper

def joinLines (lines : List String) : String :=
  String.intercalate "" lines

/--
Wrapper generator language output target.
-/
inductive WrapperLanguage where
  | c
  | rust
  | zig
deriving Repr, BEq

/--
C wrapper statement.
-/
inductive CStmt where
  | include (header : String)
  | typedef (name : String) (ty : String)
  | funcDecl (name : String) (params : List String) (ret : String)
  | funcDef (name : String) (params : List String) (ret : String) (body : List CStmt)
  | call (name : String) (args : List String)
  | assign (var : String) (value : String)
  | ret (value : Option String)
  | ifStmt (cond : String) (thenBranch : List CStmt) (elseBranch : List CStmt)
  | comment (text : String)

/--
Rust wrapper statement.
-/
inductive RustStmt where
  | useItem (path : String)
  | structDef (name : String) (fields : List (String × String))
  | implBlock (name : String) (methods : List RustStmt)
  | fnDecl (name : String) (params : List String) (ret : String)
  | fnDef (name : String) (params : List String) (ret : String) (body : List RustStmt)
  | call (name : String) (args : List String)
  | letBind (name : String) (value : String)
  | assign (var : String) (value : String)
  | ret (value : Option String)
  | comment (text : String)

/--
Zig wrapper statement.
-/
inductive ZigStmt where
  | constDecl (name : String) (value : String)
  | varDecl (name : String) (ty : String) (value : String)
  | fnDecl (name : String) (params : List (String × String)) (ret : String)
  | fnDef (name : String) (params : List (String × String)) (ret : String) (body : List ZigStmt)
  | call (name : String) (args : List String)
  | ret (value : Option String)
  | comment (text : String)

/--
Abstract wrapper statement (language-agnostic).
-/
inductive WrapperStmt where
  | c (stmt : CStmt)
  | rust (stmt : RustStmt)
  | zig (stmt : ZigStmt)

/--
Abstract wrapper function.
-/
structure WrapperFunction where
  name : Symbol
  contract : FunctionContract
  stmts : List WrapperStmt

/--
Wrapper module.
-/
structure WrapperModule where
  targetLanguage : WrapperLanguage
  functions : List WrapperFunction
  includesRuntime : Bool := false

namespace WrapperStmt

/--
Check if a statement is a comment.
-/
def isComment : WrapperStmt → Bool
  | .c (.comment _) => true
  | .rust (.comment _) => true
  | .zig (.comment _) => true
  | _ => false

/--
Get comment text if any.
-/
def getComment? : WrapperStmt → Option String
  | .c (.comment t) => some t
  | .rust (.comment t) => some t
  | .zig (.comment t) => some t
  | _ => none

end WrapperStmt

namespace WrapperFunction

/--
Check if wrapper function has no body.
-/
def isEmpty (f : WrapperFunction) : Bool :=
  f.stmts.isEmpty

end WrapperFunction

/--
Render C statement to string.
-/
def renderCStmt : CStmt → String
  | .include h => "#include " ++ h
  | .typedef n t => "typedef " ++ t ++ " " ++ n ++ ";"
  | .funcDecl n ps r => r ++ " " ++ n ++ "(" ++ String.intercalate ", " ps ++ ");"
  | .funcDef n ps r b => r ++ " " ++ n ++ "(" ++ String.intercalate ", " ps ++ ") {\n" ++ joinLines (List.map (fun s => "  " ++ s) (List.map renderCStmt b)) ++ "\n}"
  | .call n args => n ++ "(" ++ String.intercalate ", " args ++ ");"
  | .assign v val => v ++ " = " ++ val ++ ";"
  | .ret none => "return;"
  | .ret (some v) => "return " ++ v ++ ";"
  | .ifStmt c t e => "if (" ++ c ++ ") {\n" ++ joinLines (List.map (fun s => "  " ++ s) (List.map renderCStmt t)) ++ "\n}" ++ (if e.isEmpty then "" else " else {\n" ++ joinLines (List.map (fun s => "  " ++ s) (List.map renderCStmt e)) ++ "\n}")
  | .comment t => "/* " ++ t ++ " */"

/--
Render Rust statement to string.
-/
def renderRustStmt : RustStmt → String
  | .useItem p => "use " ++ p ++ ";"
  | .structDef n fs => "struct " ++ n ++ " { " ++ String.intercalate ", " (fs.map (fun field => field.1 ++ ": " ++ field.2)) ++ " }"
  | .implBlock n ms => "impl " ++ n ++ " {\n" ++ joinLines (List.map (fun s => "  " ++ s) (List.map renderRustStmt ms)) ++ "\n}"
  | .fnDecl n ps r => "fn " ++ n ++ "(" ++ String.intercalate ", " (ps.map (fun p => "_: " ++ p)) ++ ") -> " ++ r
  | .fnDef n ps r b => "fn " ++ n ++ "(" ++ String.intercalate ", " (ps.map (fun p => "_: " ++ p)) ++ ") -> " ++ r ++ " {\n" ++ joinLines (List.map (fun s => "  " ++ s) (List.map renderRustStmt b)) ++ "\n}"
  | .call n args => n ++ "(" ++ String.intercalate ", " args ++ ");"
  | .letBind n v => "let " ++ n ++ " = " ++ v ++ ";"
  | .assign v val => v ++ " = " ++ val ++ ";"
  | .ret none => "return;"
  | .ret (some v) => "return " ++ v ++ ";"
  | .comment t => "// " ++ t

/--
Render Zig statement to string.
-/
def renderZigStmt : ZigStmt → String
  | .constDecl n v => "const " ++ n ++ " = " ++ v ++ ";"
  | .varDecl n t v => "var " ++ n ++ ": " ++ t ++ " = " ++ v ++ ";"
  | .fnDecl n ps r => "fn " ++ n ++ "(" ++ String.intercalate ", " (ps.map (fun param => param.1 ++ ": " ++ param.2)) ++ ") " ++ r
  | .fnDef n ps r b => "fn " ++ n ++ "(" ++ String.intercalate ", " (ps.map (fun param => param.1 ++ ": " ++ param.2)) ++ ") " ++ r ++ " {\n" ++ joinLines (List.map (fun s => "  " ++ s) (List.map renderZigStmt b)) ++ "\n}"
  | .call n args => n ++ "(" ++ String.intercalate ", " args ++ ");"
  | .ret none => "return;"
  | .ret (some v) => "return " ++ v ++ ";"
  | .comment t => "// " ++ t

/--
Render wrapper statement to string.
-/
def renderWrapperStmt : WrapperStmt → String
  | .c s => renderCStmt s
  | .rust s => renderRustStmt s
  | .zig s => renderZigStmt s

end Chimera.Wrapper
