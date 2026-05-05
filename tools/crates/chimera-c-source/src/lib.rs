//! Chimera C source parser fallback crate.
//!
//! Uses tree-sitter-c for surface validation when Clang is unavailable.
//! **Important**: This parser is NON-AUTHORITATIVE for layout decisions.
//! It provides surface-level syntax validation only.
//!
//! Task 11: Stable C parser fallback

use chimera_c_schema::*;
use std::path::Path;

/// Result type for C source parsing operations
pub type Result<T> = std::result::Result<T, ParseError>;

/// Parse errors
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("tree-sitter parsing failed: {0}")]
    TreeSitterError(String),
    #[error("invalid source: {0}")]
    InvalidSource(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("unsupported feature: {0}")]
    UnsupportedFeature(String),
}

/// Parse configuration
#[derive(Debug, Clone)]
pub struct ParseConfig {
    /// Whether to include comments in the AST
    pub include_comments: bool,
    /// Maximum AST depth
    pub max_ast_depth: usize,
    /// Timeout for parsing in milliseconds
    pub timeout_ms: u64,
}

impl Default for ParseConfig {
    fn default() -> Self {
        Self {
            include_comments: false,
            max_ast_depth: 10000,
            timeout_ms: 5000,
        }
    }
}

/// C source parser using tree-sitter
pub struct CSourceParser {
    config: ParseConfig,
    parser: tree_sitter::Parser,
}

impl CSourceParser {
    /// Create a new C source parser
    pub fn new(config: ParseConfig) -> Result<Self> {
        let language = tree_sitter_c::language();
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(language.into())
            .map_err(|e| ParseError::TreeSitterError(e.to_string()))?;

        Ok(Self { config, parser })
    }

    /// Parse a C source file
    pub fn parse_file(&mut self, path: &Path) -> Result<ParseResult> {
        let content = std::fs::read_to_string(path)?;
        self.parse_source(&content, Some(path.to_string_lossy().to_string()))
    }

    /// Parse C source from string
    pub fn parse_source(
        &mut self,
        source: &str,
        source_name: Option<String>,
    ) -> Result<ParseResult> {
        // Check for unsupported features first
        self.check_unsupported_features(source)?;

        let _timeout = std::time::Duration::from_millis(self.config.timeout_ms);
        let tree = self
            .parser
            .parse(source, None)
            .ok_or_else(|| ParseError::TreeSitterError("parse returned None".to_string()))?;

        let root_node = tree.root_node();

        // Build the parse tree representation
        let declarations = self.extract_declarations(root_node, source);

        Ok(ParseResult {
            source_name: source_name.unwrap_or_else(|| "(anonymous)".to_string()),
            syntax_tree: tree,
            declarations,
            is_authoritative: false, // ALWAYS false - this is a fallback parser
            diagnostics: vec![],
        })
    }

    /// Check for unsupported features in source
    fn check_unsupported_features(&self, source: &str) -> Result<()> {
        // Check for assembly blocks (not supported by tree-sitter-c)
        if source.contains("asm") || source.contains("__asm__") {
            return Err(ParseError::UnsupportedFeature(
                "inline assembly not supported".to_string(),
            ));
        }

        // Check for complex macro patterns
        if source.contains("##") {
            // Token concatenation - not fully supported
        }

        Ok(())
    }

    /// Extract declarations from the parse tree
    fn extract_declarations(&self, node: tree_sitter::Node, source: &str) -> Vec<CDeclaration> {
        let mut declarations = vec![];

        let cursor = &mut tree_sitter::QueryCursor::new();
        let query = tree_sitter::Query::new(tree_sitter_c::language(), "(declaration) @decl").ok();

        if let Some(q) = query {
            let matches = cursor.matches(&q, node, source.as_bytes());
            for m in matches {
                for cap_node in m.captures {
                    let decl_node = cap_node.node;
                    if let Some(decl) = self.node_to_declaration(decl_node, source) {
                        declarations.push(decl);
                    }
                }
            }
        }

        // Fallback: walk children directly
        if declarations.is_empty() {
            self.walk_node(node, source, &mut declarations);
        }

        declarations
    }

    /// Walk the tree and extract declarations
    fn walk_node(
        &self,
        node: tree_sitter::Node,
        source: &str,
        declarations: &mut Vec<CDeclaration>,
    ) {
        match node.kind() {
            "declaration" | "function_definition" | "type_definition" => {
                if let Some(decl) = self.node_to_declaration(node, source) {
                    declarations.push(decl);
                }
            }
            _ => {
                for i in 0..node.child_count() {
                    if let Some(child) = node.child(i) {
                        self.walk_node(child, source, declarations);
                    }
                }
            }
        }
    }

    /// Convert a tree-sitter node to a C declaration
    fn node_to_declaration(&self, node: tree_sitter::Node, source: &str) -> Option<CDeclaration> {
        let kind = node.kind();
        let span = self.node_to_span(node);

        match kind {
            "function_definition" => {
                let name = self.get_function_name(node, source)?;
                Some(CDeclaration::Function(FunctionDeclRaw {
                    name,
                    return_type: "int".to_string(), // Would need more sophisticated extraction
                    parameters: vec![],
                    span,
                }))
            }
            "declaration" => {
                let name = self.get_declarator_name(node, source)?;
                Some(CDeclaration::Variable(VarDeclRaw {
                    name,
                    typ: "auto".to_string(), // Would need more sophisticated extraction
                    span,
                }))
            }
            "type_definition" => {
                let name = self.get_typedef_name(node, source)?;
                Some(CDeclaration::Typedef(TypedefDeclRaw {
                    name,
                    underlying_type: "int".to_string(),
                    span,
                }))
            }
            _ => None,
        }
    }

    /// Get function name from a function definition node
    fn get_function_name(&self, node: tree_sitter::Node, source: &str) -> Option<String> {
        // Look for identifier in the declaration
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                return Some(self.node_text(child, source));
            }
        }
        None
    }

    /// Get declarator name from a declaration node
    fn get_declarator_name(&self, node: tree_sitter::Node, source: &str) -> Option<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "init_declarator" {
                if let Some(init) = child.child(0) {
                    if init.kind() == "identifier" {
                        return Some(self.node_text(init, source));
                    }
                }
            } else if child.kind() == "identifier" {
                return Some(self.node_text(child, source));
            }
        }
        None
    }

    /// Get typedef name
    fn get_typedef_name(&self, node: tree_sitter::Node, source: &str) -> Option<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "identifier" {
                return Some(self.node_text(child, source));
            }
        }
        None
    }

    /// Get node text
    fn node_text(&self, node: tree_sitter::Node, source: &str) -> String {
        node.utf8_text(source.as_bytes())
            .unwrap_or_default()
            .to_string()
    }

    /// Convert node to source span
    fn node_to_span(&self, node: tree_sitter::Node) -> SourceSpan {
        let start = node.start_position();
        let _end = node.end_position();
        SourceSpan {
            file: "(parsed)".to_string(),
            line: start.row as u32 + 1,
            col: start.column as u32 + 1,
            byte_offset: node.start_byte() as u64,
            byte_length: (node.end_byte() - node.start_byte()) as u64,
        }
    }
}

/// Parse result
#[derive(Debug, Clone)]
pub struct ParseResult {
    /// Source name
    pub source_name: String,
    /// The full syntax tree
    pub syntax_tree: tree_sitter::Tree,
    /// Extracted declarations
    pub declarations: Vec<CDeclaration>,
    /// Whether this parser is authoritative for layout
    /// **Always false** - this is a fallback parser
    pub is_authoritative: bool,
    /// Diagnostics from parsing
    pub diagnostics: Vec<Diagnostic>,
}

/// Raw C declaration from parser
#[derive(Debug, Clone)]
pub enum CDeclaration {
    Function(FunctionDeclRaw),
    Variable(VarDeclRaw),
    Typedef(TypedefDeclRaw),
    Struct(StructDeclRaw),
    Union(UnionDeclRaw),
    Enum(EnumDeclRaw),
}

/// Function declaration (raw)
#[derive(Debug, Clone)]
pub struct FunctionDeclRaw {
    pub name: String,
    pub return_type: String,
    pub parameters: Vec<(String, String)>, // (name, type)
    pub span: SourceSpan,
}

/// Variable declaration (raw)
#[derive(Debug, Clone)]
pub struct VarDeclRaw {
    pub name: String,
    pub typ: String,
    pub span: SourceSpan,
}

/// Typedef declaration (raw)
#[derive(Debug, Clone)]
pub struct TypedefDeclRaw {
    pub name: String,
    pub underlying_type: String,
    pub span: SourceSpan,
}

/// Struct declaration (raw)
#[derive(Debug, Clone)]
pub struct StructDeclRaw {
    pub name: Option<String>,
    pub fields: Vec<(String, String)>, // (name, type)
    pub span: SourceSpan,
}

/// Union declaration (raw)
#[derive(Debug, Clone)]
pub struct UnionDeclRaw {
    pub name: Option<String>,
    pub fields: Vec<(String, String)>,
    pub span: SourceSpan,
}

/// Enum declaration (raw)
#[derive(Debug, Clone)]
pub struct EnumDeclRaw {
    pub name: Option<String>,
    pub constants: Vec<String>,
    pub span: SourceSpan,
}

impl ParseResult {
    /// Get all diagnostic codes
    pub fn diagnostic_codes(&self) -> Vec<CDiagnosticCode> {
        self.diagnostics.iter().map(|d| d.code).collect()
    }
}

/// C header parser (same as source parser but for headers)
pub struct CHeaderParser {
    source_parser: CSourceParser,
}

impl CHeaderParser {
    /// Create a new header parser
    pub fn new(config: ParseConfig) -> Result<Self> {
        Ok(Self {
            source_parser: CSourceParser::new(config)?,
        })
    }

    /// Parse a header file
    pub fn parse_file(&mut self, path: &Path) -> Result<ParseResult> {
        self.source_parser.parse_file(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_config_default() {
        let config = ParseConfig::default();
        assert!(!config.include_comments);
        assert_eq!(config.max_ast_depth, 10000);
        assert_eq!(config.timeout_ms, 5000);
    }

    #[test]
    fn test_parse_result_not_authoritative() {
        let mut parser = CSourceParser::new(ParseConfig::default()).unwrap();
        let result = parser
            .parse_source("int foo();", Some("test.c".to_string()))
            .unwrap();
        assert!(!result.is_authoritative); // MUST be false
    }

    #[test]
    fn test_unsupported_inline_asm() {
        let mut parser = CSourceParser::new(ParseConfig::default()).unwrap();
        let result = parser.parse_source("void foo() { asm(\"nop\"); }", None);
        assert!(matches!(result, Err(ParseError::UnsupportedFeature(_))));
    }

    #[test]
    fn test_simple_function_parsing() {
        let mut parser = CSourceParser::new(ParseConfig::default()).unwrap();
        let result = parser.parse_source("int main() { return 0; }", Some("test.c".to_string()));
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(!result.is_authoritative);
    }

    #[test]
    fn test_simple_declaration_parsing() {
        let mut parser = CSourceParser::new(ParseConfig::default()).unwrap();
        let result = parser
            .parse_source("int x;", Some("test.c".to_string()))
            .unwrap();
        assert!(!result.is_authoritative);
    }

    #[test]
    fn test_source_span_creation() {
        let span = SourceSpan {
            file: "test.c".to_string(),
            line: 1,
            col: 1,
            byte_offset: 0,
            byte_length: 10,
        };
        assert_eq!(span.line, 1);
        assert_eq!(span.col, 1);
    }

    #[test]
    fn test_diagnostic_code_codes() {
        assert_eq!(CDiagnosticCode::ParseUnexpectedToken.code(), 1000);
        assert_eq!(CDiagnosticCode::ClangExtractFailed.code(), 2000);
        assert_eq!(CDiagnosticCode::IncludeNotFound.code(), 3000);
    }

    #[test]
    fn test_cdeclaration_variants() {
        let decl = CDeclaration::Function(FunctionDeclRaw {
            name: "foo".to_string(),
            return_type: "int".to_string(),
            parameters: vec![],
            span: SourceSpan {
                file: "test.c".to_string(),
                line: 1,
                col: 1,
                byte_offset: 0,
                byte_length: 10,
            },
        });

        match decl {
            CDeclaration::Function(f) => assert_eq!(f.name, "foo"),
            _ => panic!("expected Function"),
        }
    }

    #[test]
    fn test_parse_error_display() {
        let err = ParseError::UnsupportedFeature("inline asm".to_string());
        assert!(err.to_string().contains("inline asm"));

        let err = ParseError::TreeSitterError("parse failed".to_string());
        assert!(err.to_string().contains("tree-sitter"));
    }
}
