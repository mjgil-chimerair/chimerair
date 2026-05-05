//! Core Erlang module representation.
//!
//! Converts from our AST to the schema types.

use crate::ast::{self, Definition, Expr, Function, Module as AstModule};
use chimera_beam_schema::{
    Atom, Attribute, BeamModuleInfo, CompileInfo, ExportEntry, FunctionInfo, ImportEntry, Term,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("conversion error: {0}")]
    Conversion(String),
}

/// Result type for Core Erlang conversion.
pub type CoreResult<T> = Result<T, CoreError>;

/// Core Erlang module (intermediate representation for schema).
#[derive(Debug, Clone)]
pub struct CoreErlangModule {
    pub module_name: Atom,
    pub exports: Vec<ExportEntry>,
    pub imports: Vec<ImportEntry>,
    pub functions: Vec<FunctionInfo>,
    pub attributes: Vec<Attribute>,
    pub compile_info: CompileInfo,
}

impl CoreErlangModule {
    pub fn from_ast(module: &AstModule) -> CoreResult<Self> {
        let module_name = Atom::new(&module.name);

        let exports = module
            .exports
            .iter()
            .map(|e| ExportEntry {
                function: Atom::new(&e.function),
                arity: e.arity,
                label: 0,
            })
            .collect();

        let imports = module
            .imports
            .iter()
            .map(|i| ImportEntry {
                module: Atom::new(&i.module),
                function: Atom::new(&i.function),
                arity: i.arity,
            })
            .collect();

        let mut functions = Vec::new();
        for def in &module.definitions {
            if let Definition::Function(func) = def {
                functions.push(FunctionInfo {
                    name: Atom::new(&func.name),
                    arity: func.arity,
                    label: 0,
                    code_version: 1,
                    num_args: func.vars.len() as u32,
                    num_locals: 0,
                });
            }
        }

        let attributes = module
            .attributes
            .iter()
            .map(|a| Attribute {
                key: Atom::new(&a.key),
                value: Self::convert_literal(&a.value),
            })
            .collect();

        let compile_info = CompileInfo {
            options: Vec::new(),
            version: None,
            time: None,
        };

        Ok(CoreErlangModule {
            module_name,
            exports,
            imports,
            functions,
            attributes,
            compile_info,
        })
    }

    fn convert_literal(lit: &crate::ast::Literal) -> Term {
        match lit {
            crate::ast::Literal::Atom(s) => Term::Atom(Atom::new(s)),
            crate::ast::Literal::Int(i) => Term::Int(*i),
            crate::ast::Literal::Float(f) => Term::Float(*f),
            crate::ast::Literal::String(s) => Term::String(s.clone()),
            crate::ast::Literal::Char(c) => Term::Int(*c as i64),
            crate::ast::Literal::Nil => Term::List(Vec::new()),
            crate::ast::Literal::Tuple(items) => {
                Term::Tuple(items.iter().map(Self::convert_literal).collect())
            }
            crate::ast::Literal::List(_items) => Term::List(Vec::new()),
            crate::ast::Literal::Binary(bytes) => Term::Binary(bytes.clone()),
        }
    }
}

/// Convert an AST module to a schema module info.
pub fn to_module_info(ast: &AstModule) -> CoreResult<BeamModuleInfo> {
    let core = CoreErlangModule::from_ast(ast)?;

    Ok(BeamModuleInfo {
        module_name: core.module_name,
        exports: core.exports,
        imports: core.imports,
        functions: core.functions,
        attributes: core.attributes,
        compile_info: core.compile_info,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Definition, Export, Expr, Function, Literal, Module};

    #[test]
    fn test_core_erlang_module_from_ast() {
        let module = Module::new("test_module")
            .with_export(Export {
                function: "start".to_string(),
                arity: 0,
            })
            .with_definition(Definition::Function(Function::new("start", 0)));

        let core = CoreErlangModule::from_ast(&module).unwrap();
        assert_eq!(core.module_name.0, "test_module");
        assert_eq!(core.exports.len(), 1);
        assert_eq!(core.functions.len(), 1);
    }

    #[test]
    fn test_literal_conversion() {
        assert!(matches!(
            CoreErlangModule::convert_literal(&Literal::atom("test")),
            Term::Atom(_)
        ));
        assert!(matches!(
            CoreErlangModule::convert_literal(&Literal::int(42)),
            Term::Int(42)
        ));
        assert!(matches!(
            CoreErlangModule::convert_literal(&Literal::string("hello")),
            Term::String(_)
        ));
    }
}
