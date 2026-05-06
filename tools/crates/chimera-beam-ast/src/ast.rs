//! Core Erlang AST types.
//!
//! These types represent the abstract syntax tree of Core Erlang programs.

use serde::{Deserialize, Serialize};

/// A Core Erlang module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    pub name: String,
    pub exports: Vec<Export>,
    pub imports: Vec<Import>,
    pub definitions: Vec<Definition>,
    pub attributes: Vec<Attribute>,
}

impl Module {
    pub fn new(name: impl Into<String>) -> Self {
        Module {
            name: name.into(),
            exports: Vec::new(),
            imports: Vec::new(),
            definitions: Vec::new(),
            attributes: Vec::new(),
        }
    }

    pub fn with_export(mut self, export: Export) -> Self {
        self.exports.push(export);
        self
    }

    pub fn with_import(mut self, import: Import) -> Self {
        self.imports.push(import);
        self
    }

    pub fn with_definition(mut self, def: Definition) -> Self {
        self.definitions.push(def);
        self
    }

    pub fn with_attribute(mut self, attr: Attribute) -> Self {
        self.attributes.push(attr);
        self
    }
}

/// Export specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Export {
    pub function: String,
    pub arity: u32,
}

/// Import specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Import {
    pub module: String,
    pub function: String,
    pub arity: u32,
}

/// Module attribute.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attribute {
    pub key: String,
    pub value: Literal,
}

/// A definition in a Core Erlang module (function or variable).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Definition {
    /// A function definition.
    Function(Function),
    /// A variable binding.
    Variable(Variable),
}

/// A function definition with annotation, vars, and body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub arity: u32,
    pub annotation: Vec<String>,
    pub vars: Vec<(String, Type)>,
    pub body: Expr,
}

impl Function {
    pub fn new(name: impl Into<String>, arity: u32) -> Self {
        Function {
            name: name.into(),
            arity,
            annotation: Vec::new(),
            vars: Vec::new(),
            body: Expr::Literal(Literal::atom("ok")),
        }
    }
}

/// A variable binding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    pub name: String,
    pub value: Expr,
}

/// Core Erlang expression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expr {
    /// Variable reference.
    Var(String),
    /// Literal value.
    Literal(Literal),
    /// Atom literal (convenience for Literal::Atom).
    Atom(String),
    /// Function application.
    Apply(Box<Apply>),
    /// Lambda (fun expression).
    Lambda(Box<Lambda>),
    /// Let binding.
    Let(Box<Let>),
    /// Letrec (recursive let).
    LetRec(Vec<Definition>, Box<Expr>),
    /// Sequence of expressions.
    Seq(Box<Seq>),
    /// Case expression.
    Case(Box<Case>),
    /// Receive expression.
    Receive(Box<Receive>),
    /// Try-catch expression.
    Try(Box<Try>),
    /// Catch wrapper.
    Catch(Box<Expr>),
    /// PrimOp (primitive operation / BIF).
    PrimOp(PrimOp),
    /// Exit signal.
    Exit(Box<Expr>),
    /// Throw signal.
    Throw(Box<Expr>),
    /// Internal (compiler-generated).
    Internal(String),
}

/// Apply expression: module:function(args).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Apply {
    pub module: Option<String>,
    pub function: String,
    pub args: Vec<Expr>,
    pub tail: bool,
}

impl Apply {
    pub fn new(function: impl Into<String>) -> Self {
        Apply {
            module: None,
            function: function.into(),
            args: Vec::new(),
            tail: false,
        }
    }

    pub fn with_args(mut self, args: Vec<Expr>) -> Self {
        self.args = args;
        self
    }

    pub fn with_module(mut self, module: impl Into<String>) -> Self {
        self.module = Some(module.into());
        self
    }

    pub fn tail(mut self) -> Self {
        self.tail = true;
        self
    }
}

/// Lambda (fun expression).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lambda {
    pub vars: Vec<String>,
    pub body: Box<Expr>,
    pub name: Option<String>,
}

/// Let binding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Let {
    pub bindings: Vec<(String, Expr)>,
    pub body: Box<Expr>,
}

/// Sequence of expressions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Seq {
    pub first: Box<Expr>,
    pub then: Box<Expr>,
}

/// Case expression with clauses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Case {
    pub expr: Box<Expr>,
    pub clauses: Vec<Clause>,
}

/// A clause within a case or receive.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clause {
    pub patterns: Vec<Pattern>,
    pub guards: Vec<Guard>,
    pub body: Vec<Expr>,
}

impl Clause {
    pub fn new(patterns: Vec<Pattern>) -> Self {
        Clause {
            patterns,
            guards: Vec::new(),
            body: Vec::new(),
        }
    }
}

/// Guard expression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guard(pub Vec<Expr>);

/// Pattern for pattern matching.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Pattern {
    Wildcard,
    Var(String),
    Atom(String),
    Int(i64),
    Char(char),
    Float(f64),
    Tuple(Vec<Pattern>),
    Cons(Box<ConsPattern>),
    Nil,
    Binary(Vec<Pattern>),
}

/// Cons cell pattern (head : tail).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsPattern {
    pub head: Box<Pattern>,
    pub tail: Box<Pattern>,
}

/// Receive expression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receive {
    pub clauses: Vec<Clause>,
    pub timeout: Option<Box<Expr>>,
    pub after: Option<Box<Expr>>,
}

impl Receive {
    pub fn with_clauses(mut self, clauses: Vec<Clause>) -> Self {
        self.clauses = clauses;
        self
    }

    pub fn with_timeout(mut self, timeout: Expr) -> Self {
        self.timeout = Some(Box::new(timeout));
        self
    }
}

/// Try-catch expression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Try {
    pub expr: Box<Expr>,
    pub vars: Vec<String>,
    pub body: Box<Expr>,
    pub catch_vars: Vec<String>,
    pub handler: Box<Expr>,
}

/// Primitive operation (BIF or NIF).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimOp {
    pub name: String,
    pub args: Vec<Expr>,
}

impl PrimOp {
    pub fn new(name: impl Into<String>) -> Self {
        PrimOp {
            name: name.into(),
            args: Vec::new(),
        }
    }

    pub fn with_args(mut self, args: Vec<Expr>) -> Self {
        self.args = args;
        self
    }
}

/// Literal values in Core Erlang.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Literal {
    Atom(String),
    Int(i64),
    Float(f64),
    Char(char),
    String(String),
    Nil,
    Tuple(Vec<Literal>),
    List(Vec<Expr>),
    Binary(Vec<u8>),
}

impl Literal {
    pub fn atom(s: impl Into<String>) -> Self {
        Literal::Atom(s.into())
    }

    pub fn int(i: i64) -> Self {
        Literal::Int(i)
    }

    pub fn string(s: impl Into<String>) -> Self {
        Literal::String(s.into())
    }

    pub fn nil() -> Self {
        Literal::Nil
    }
}

/// Send expression: dest ! msg.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Send {
    pub dest: Box<Expr>,
    pub msg: Box<Expr>,
}

/// Link expression: link(pid).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    pub pid: Box<Expr>,
}

/// Monitor expression: monitor(pid | name).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Monitor {
    pub target: MonitorTarget,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MonitorTarget {
    Pid(Box<Expr>),
    Name(Box<Expr>),
}

/// Spawn expression: spawn(module, function, args).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spawn {
    pub module: Box<Expr>,
    pub function: Box<Expr>,
    pub args: Vec<Expr>,
}

/// Type annotation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Type {
    pub kind: TypeKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TypeKind {
    Any,
    Atom,
    Integer,
    Float,
    Binary,
    List(Box<Type>),
    Tuple(Vec<Type>),
    Function { args: Vec<Type>, result: Box<Type> },
    Var(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_creation() {
        let module = Module::new("my_module")
            .with_export(Export {
                function: "start".to_string(),
                arity: 0,
            })
            .with_definition(Definition::Function(Function::new("start", 0)));
        assert_eq!(module.name, "my_module");
        assert_eq!(module.exports.len(), 1);
    }

    #[test]
    fn test_apply_creation() {
        let apply = Apply::new("spawn")
            .with_module("erlang")
            .with_args(vec![Expr::Literal(Literal::int(1))]);
        assert_eq!(apply.function, "spawn");
        assert!(apply.module.is_some());
    }

    #[test]
    fn test_literal_constructors() {
        assert!(matches!(Literal::atom("test"), Literal::Atom(_)));
        assert!(matches!(Literal::int(42), Literal::Int(42)));
        assert!(matches!(Literal::string("hello"), Literal::String(_)));
        assert!(matches!(Literal::nil(), Literal::Nil));
    }

    #[test]
    fn test_clause_creation() {
        let clause = Clause::new(vec![Pattern::Wildcard]);
        assert!(clause.patterns.len() == 1);
    }

    #[test]
    fn test_primop_creation() {
        let primop = PrimOp::new("erlang:spawn").with_args(vec![
            Expr::Literal(Literal::atom("module")),
            Expr::Literal(Literal::atom("init")),
            Expr::Literal(Literal::Nil),
        ]);
        assert_eq!(primop.name, "erlang:spawn");
        assert_eq!(primop.args.len(), 3);
    }

    #[test]
    fn test_type_kind_serialization() {
        let kind = TypeKind::Integer;
        let json = serde_json::to_string(&kind).unwrap();
        assert_eq!(json, "\"Integer\"");
    }
}
