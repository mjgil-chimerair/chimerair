//! Exported symbol import for AIR function and global symbols.
//!
//! Task 47: Import exported symbols (name, function ID, type ID, callconv, linkage).

use serde::{Deserialize, Serialize};

/// Visibility level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Visibility {
    /// Private to the module
    Private,
    /// Publicly exported
    Public,
    /// Exported and can be referenced from other linkage units
    Exported,
}

/// Linkage type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinkageType {
    /// Internal linkage - not visible outside the module
    Internal,
    /// External linkage - visible outside the module
    External,
    /// Weak linkage - can be overridden
    Weak,
    /// Linkonce linkage - merged with other definitions
    LinkOnce,
}

/// Calling convention
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CallConv {
    /// Standard C calling convention
    C,
    /// Fast calling convention
    Fast,
    /// Cold calling convention
    Cold,
    /// Naked calling convention (no prologue/epilogue)
    Naked,
    /// Stdcall (Pascal-style)
    Stdcall,
    /// Vector call
    Vectorcall,
    /// Thiscall (C++ member functions)
    Thiscall,
    /// Zig-specific calling convention
    Zig,
    /// Unknown calling convention
    Unknown(String),
}

impl CallConv {
    /// Parse from string
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "c" => CallConv::C,
            "fast" => CallConv::Fast,
            "cold" => CallConv::Cold,
            "naked" => CallConv::Naked,
            "stdcall" => CallConv::Stdcall,
            "vectorcall" => CallConv::Vectorcall,
            "thiscall" => CallConv::Thiscall,
            "zig" => CallConv::Zig,
            other => CallConv::Unknown(other.to_string()),
        }
    }
}

/// An exported symbol from AIR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedSymbol {
    /// Symbol name
    pub name: String,
    /// Function ID (reference to function table)
    pub func_id: u64,
    /// Type ID (function signature)
    pub type_id: u64,
    /// Calling convention
    pub callconv: CallConv,
    /// Linkage type
    pub linkage: LinkageType,
    /// Visibility level
    pub visibility: Visibility,
    /// Is this an exported function (vs internal)
    pub is_exported: bool,
    /// Source location for the export
    pub source_loc: u64,
}

/// Import result for exported symbols
#[derive(Debug, Clone)]
pub struct SymbolImportResult {
    /// All imported symbols
    pub symbols: Vec<ExportedSymbol>,
    /// Symbol name to index mapping
    pub name_map: std::collections::HashMap<String, usize>,
}

impl SymbolImportResult {
    /// Create new import result
    pub fn new() -> Self {
        Self {
            symbols: Vec::new(),
            name_map: std::collections::HashMap::new(),
        }
    }

    /// Add a symbol
    pub fn add_symbol(&mut self, symbol: ExportedSymbol) {
        let idx = self.symbols.len();
        self.name_map.insert(symbol.name.clone(), idx);
        self.symbols.push(symbol);
    }

    /// Look up symbol by name
    pub fn get_by_name(&self, name: &str) -> Option<&ExportedSymbol> {
        self.name_map.get(name).and_then(|&idx| self.symbols.get(idx))
    }
}

impl Default for SymbolImportResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Symbol table importer
#[derive(Debug, Clone)]
pub struct SymbolImporter {
    /// Imported symbol results
    pub result: SymbolImportResult,
}

impl SymbolImporter {
    /// Create new importer
    pub fn new() -> Self {
        Self {
            result: SymbolImportResult::new(),
        }
    }

    /// Import a function symbol
    pub fn import_fn(
        &mut self,
        name: String,
        func_id: u64,
        type_id: u64,
        callconv: &str,
        linkage: LinkageType,
        visibility: Visibility,
        is_exported: bool,
        source_loc: u64,
    ) {
        self.result.add_symbol(ExportedSymbol {
            name,
            func_id,
            type_id,
            callconv: CallConv::from_str(callconv),
            linkage,
            visibility,
            is_exported,
            source_loc,
        });
    }

    /// Import a C-exported function
    pub fn import_c_fn(
        &mut self,
        name: String,
        func_id: u64,
        type_id: u64,
    ) {
        self.import_fn(
            name,
            func_id,
            type_id,
            "c",
            LinkageType::External,
            Visibility::Exported,
            true,
            0,
        );
    }

    /// Get the import result
    pub fn finish(self) -> SymbolImportResult {
        self.result
    }
}

impl Default for SymbolImporter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_importer_creation() {
        let importer = SymbolImporter::new();
        assert_eq!(importer.result.symbols.len(), 0);
    }

    #[test]
    fn test_import_fn() {
        let mut importer = SymbolImporter::new();
        importer.import_fn(
            "my_func".to_string(),
            1,
            2,
            "c",
            LinkageType::External,
            Visibility::Public,
            true,
            0,
        );
        assert_eq!(importer.result.symbols.len(), 1);
        let sym = &importer.result.symbols[0];
        assert_eq!(sym.name, "my_func");
        assert!(matches!(sym.callconv, CallConv::C));
    }

    #[test]
    fn test_import_c_fn() {
        let mut importer = SymbolImporter::new();
        importer.import_c_fn("printf".to_string(), 1, 2);
        assert_eq!(importer.result.symbols.len(), 1);
        let sym = &importer.result.symbols[0];
        assert_eq!(sym.name, "printf");
        assert!(matches!(sym.callconv, CallConv::C));
        assert!(matches!(sym.linkage, LinkageType::External));
        assert!(sym.is_exported);
    }

    #[test]
    fn test_get_by_name() {
        let mut importer = SymbolImporter::new();
        importer.import_fn(
            "foo".to_string(),
            1,
            2,
            "fast",
            LinkageType::Internal,
            Visibility::Private,
            false,
            0,
        );
        let found = importer.result.get_by_name("foo");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "foo");

        let not_found = importer.result.get_by_name("bar");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_callconv_parsing() {
        assert!(matches!(CallConv::from_str("c"), CallConv::C));
        assert!(matches!(CallConv::from_str("fast"), CallConv::Fast));
        assert!(matches!(CallConv::from_str("cold"), CallConv::Cold));
        assert!(matches!(CallConv::from_str("zig"), CallConv::Zig));
        let unknown = CallConv::from_str("custom");
        assert!(matches!(unknown, CallConv::Unknown(s) if s == "custom"));
    }

    #[test]
    fn test_visibility_and_linkage() {
        assert!(matches!(Visibility::Private, Visibility::Private));
        assert!(matches!(Visibility::Public, Visibility::Public));
        assert!(matches!(Visibility::Exported, Visibility::Exported));

        assert!(matches!(LinkageType::Internal, LinkageType::Internal));
        assert!(matches!(LinkageType::External, LinkageType::External));
        assert!(matches!(LinkageType::Weak, LinkageType::Weak));
        assert!(matches!(LinkageType::LinkOnce, LinkageType::LinkOnce));
    }

    #[test]
    fn test_multiple_symbols() {
        let mut importer = SymbolImporter::new();
        importer.import_c_fn("func_a".to_string(), 1, 2);
        importer.import_c_fn("func_b".to_string(), 3, 4);
        assert_eq!(importer.result.symbols.len(), 2);
        assert!(importer.result.get_by_name("func_a").is_some());
        assert!(importer.result.get_by_name("func_b").is_some());
    }
}