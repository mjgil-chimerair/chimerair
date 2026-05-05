//! Chimera C Adapter
//!
//! Parses C headers, validates layouts, maps errno, and generates C wrappers.
//!
//! # Safety
//!
//! This adapter works with raw C code and FFI structures.

use chimera_diagnostics::{Code, DiagnosticBag};
use chimera_meta::LayoutMetadata;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

pub mod artifact;
pub mod contract;

/// Error domain for C adapter errors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ErrorDomain {
    None,
    Io,
    Memory,
    Type,
    Ownership,
    Validation,
    Runtime,
}

/// C adapter errors
#[derive(Debug, Clone)]
pub enum AdapterError {
    ParseError(String),
    InvalidLayout(String),
    UnsupportedType(String),
    MissingHeader(String),
}

/// C type representation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CType {
    Void,
    Char,
    Short,
    Int,
    Long,
    LongLong,
    Float,
    Double,
    Pointer(Box<CType>),
    Array(Box<CType>, usize),
    Struct(String),
    Union(String),
    Enum(String),
    Typedef(String),
}

impl CType {
    /// Get the size of the type in bytes (returns None for incomplete types)
    pub fn size(&self) -> Option<u64> {
        match self {
            CType::Void => Some(1),
            CType::Char => Some(1),
            CType::Short => Some(2),
            CType::Int => Some(4),
            CType::Long => Some(8), // Assume 64-bit
            CType::LongLong => Some(8),
            CType::Float => Some(4),
            CType::Double => Some(8),
            CType::Pointer(_) => Some(8),
            CType::Array(ty, size) => ty.size().map(|s| s * (*size as u64)),
            CType::Struct(_) | CType::Union(_) | CType::Enum(_) | CType::Typedef(_) => None,
        }
    }

    /// Get the alignment of the type
    pub fn align(&self) -> Option<u64> {
        match self {
            CType::Char => Some(1),
            CType::Short => Some(2),
            CType::Int | CType::Float => Some(4),
            CType::Long | CType::LongLong | CType::Double => Some(8),
            CType::Pointer(_) => Some(8),
            CType::Array(ty, _) => ty.align(),
            _ => None,
        }
    }

    /// Parse a C type from a type string
    pub fn parse(type_str: &str) -> Option<CType> {
        let s = type_str.trim();

        // Handle pointers
        if s.ends_with('*') {
            let base = s.trim_end_matches('*').trim();
            if let Some(base_type) = CType::parse(base) {
                return Some(CType::Pointer(Box::new(base_type)));
            }
            // If base is just whitespace or empty, it's a void pointer
            if base.is_empty() || base == "void" {
                return Some(CType::Pointer(Box::new(CType::Void)));
            }
            // Unknown pointer base type
            return Some(CType::Pointer(Box::new(CType::Void)));
        }

        // Handle arrays
        if let Some(arr_idx) = s.find('[') {
            let base_type_str = s[..arr_idx].trim();
            if let Some(arr_end) = s.find(']') {
                if let Ok(size) = s[arr_idx + 1..arr_end].trim().parse::<usize>() {
                    if let Some(base_type) = CType::parse(base_type_str) {
                        return Some(CType::Array(Box::new(base_type), size));
                    }
                }
            }
        }

        // Handle basic types
        match s {
            "void" => Some(CType::Void),
            "char" => Some(CType::Char),
            "short" | "int16_t" | "uint16_t" => Some(CType::Short),
            "int" | "long" | "long int" | "int32_t" | "uint32_t" | "size_t" | "ptrdiff_t" => {
                Some(CType::Int)
            }
            "long long" | "int64_t" | "uint64_t" => Some(CType::LongLong),
            "float" => Some(CType::Float),
            "double" => Some(CType::Double),
            _ => {
                // Handle typedefs and struct/enum/unions by name
                if s.starts_with("struct ") {
                    return Some(CType::Struct(s["struct ".len()..].trim().to_string()));
                }
                if s.starts_with("union ") {
                    return Some(CType::Union(s["union ".len()..].trim().to_string()));
                }
                if s.starts_with("enum ") {
                    return Some(CType::Enum(s["enum ".len()..].trim().to_string()));
                }
                // Treat unknown identifiers as typedefs (named types)
                Some(CType::Typedef(s.to_string()))
            }
        }
    }
}

impl std::fmt::Display for CType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CType::Void => write!(f, "void"),
            CType::Char => write!(f, "char"),
            CType::Short => write!(f, "short"),
            CType::Int => write!(f, "int"),
            CType::Long => write!(f, "long"),
            CType::LongLong => write!(f, "long long"),
            CType::Float => write!(f, "float"),
            CType::Double => write!(f, "double"),
            CType::Pointer(inner) => write!(f, "{}*", inner),
            CType::Array(inner, size) => write!(f, "{}[{}]", inner, size),
            CType::Struct(name) => write!(f, "struct {}", name),
            CType::Union(name) => write!(f, "union {}", name),
            CType::Enum(name) => write!(f, "enum {}", name),
            CType::Typedef(name) => write!(f, "{}", name),
        }
    }
}

/// C struct field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CStructField {
    pub name: String,
    pub offset: u64,
    pub typ: CType,
    pub size: u64,
    pub align: u64,
}

impl CStructField {
    /// Parse a field declaration string into a CStructField
    pub fn parse(decl: &str, current_offset: u64) -> Option<CStructField> {
        let decl = decl.trim();

        // Remove common suffixes
        let decl = decl.strip_suffix(';')?.trim();

        // Simple field regex: type name;
        let re = Regex::new(r"^([\w\s\*\[\]]+)\s+(\w+)").ok()?;
        let caps = re.captures(decl)?;

        let type_str = caps.get(1)?.as_str();
        let field_name = caps.get(2)?.as_str();

        let typ = CType::parse(type_str)?;
        let size = typ.size().unwrap_or(0);
        let align = typ.align().unwrap_or(8);

        Some(CStructField {
            name: field_name.to_string(),
            offset: current_offset,
            typ,
            size,
            align,
        })
    }
}

/// C struct layout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CStructLayout {
    pub name: String,
    pub size: u64,
    pub align: u64,
    pub fields: Vec<CStructField>,
    pub is_packed: bool,
}

impl CStructLayout {
    /// Compute size and alignment from fields
    pub fn compute_layout(&mut self) {
        if self.fields.is_empty() {
            self.size = 0;
            self.align = 1;
            return;
        }

        // Calculate alignment
        self.align = self.fields.iter().map(|f| f.align).max().unwrap_or(1);

        // Compute field offsets and total size
        let mut offset = 0u64;
        for field in &mut self.fields {
            // Advance offset to meet alignment requirements
            let misalignment = offset % field.align;
            if misalignment != 0 {
                offset += field.align - misalignment;
            }
            field.offset = offset;

            // Advance offset by field size
            offset += field.size;
        }

        // Align total size
        let misalignment = offset % self.align;
        if misalignment != 0 {
            offset += self.align - misalignment;
        }

        self.size = offset;
    }
}

/// C function signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CFunction {
    pub name: String,
    pub return_type: CType,
    pub params: Vec<CType>,
    pub is_variadic: bool,
}

/// C header parser state
enum ParseState {
    Idle,
    InStruct { name: String },
    InEnum { name: String },
    InFunction { name: String, return_type: CType },
}

/// C header parser
pub struct CAdapter {
    diagnostics: DiagnosticBag,
    structs: HashMap<String, CStructLayout>,
    #[allow(dead_code)]
    typedefs: HashMap<String, CType>,
    #[allow(dead_code)]
    enums: HashMap<String, HashMap<String, i64>>,
    /// Parsed function declarations from headers
    functions: Vec<CFunction>,
}

impl CAdapter {
    /// Create a new C adapter
    pub fn new() -> Self {
        Self {
            diagnostics: DiagnosticBag::new(),
            structs: HashMap::new(),
            typedefs: HashMap::new(),
            enums: HashMap::new(),
            functions: Vec::new(),
        }
    }

    /// Get diagnostics from the adapter
    pub fn diagnostics(&self) -> &DiagnosticBag {
        &self.diagnostics
    }

    /// Check if adapter has errors
    pub fn has_errors(&self) -> bool {
        self.diagnostics.has_errors()
    }

    /// Get all parsed structs
    pub fn get_structs(&self) -> &HashMap<String, CStructLayout> {
        &self.structs
    }

    /// Get all parsed function declarations
    pub fn get_functions(&self) -> &Vec<CFunction> {
        &self.functions
    }

    /// Parse a C header string and extract struct layouts and function declarations
    pub fn parse_header(
        &mut self,
        header_content: &str,
    ) -> Result<Vec<CStructLayout>, AdapterError> {
        let mut layouts = Vec::new();
        let mut state = ParseState::Idle;

        let mut _current_struct_name: Option<String> = None;
        let mut current_struct_is_packed = false;
        let mut current_fields: Vec<CStructField> = Vec::new();
        let mut brace_depth = 0;
        let mut pending_field_offset = 0u64;

        for line in header_content.lines() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with("//") || line.starts_with("/*") {
                continue;
            }

            // Handle preprocessor directives - ignore for now
            if line.starts_with('#') {
                continue;
            }

            match state {
                ParseState::Idle => {
                    // Look for struct or enum definitions
                    if line.contains("struct") || line.contains("typedef struct") {
                        if let Some((name, _is_typedef, is_packed)) = self.parse_struct_start(line)
                        {
                            state = ParseState::InStruct { name };
                            current_struct_is_packed = is_packed;
                            current_fields.clear();
                            pending_field_offset = 0;

                            // Check if struct is on one line: struct Name { ... };
                            if line.contains('}') && line.contains("};") {
                                if let Some(layout) =
                                    self.finish_struct(line, &current_fields, false, is_packed)
                                {
                                    layouts.push(layout.clone());
                                    self.structs.insert(layout.name.clone(), layout);
                                }
                                state = ParseState::Idle;
                            }
                        }
                    } else if line.contains("enum ") {
                        if let Some(name) = self.parse_enum_start(line) {
                            state = ParseState::InEnum { name };
                        }
                    } else if line.contains("typedef ") && !line.contains("struct") {
                        // typedef of non-struct type
                        if let Some((name, typ)) = self.parse_typedef(line) {
                            self.typedefs.insert(name, typ);
                        }
                    } else if line.contains('(') && line.contains(");") {
                        // Try to parse as function declaration
                        if let Some(func) = self.parse_function_decl(line) {
                            self.functions.push(func);
                        }
                    }
                }

                ParseState::InStruct { ref name } => {
                    let struct_name = name.clone();

                    // Count braces to track when struct ends
                    brace_depth += line.matches('{').count() as i32;
                    brace_depth -= line.matches('}').count() as i32;

                    // Parse field declarations
                    if brace_depth > 0 {
                        // Inside struct body - parse fields
                        let fields = self.parse_fields_from_line(line, &mut pending_field_offset);
                        current_fields.extend(fields);
                    }

                    // Struct ended
                    if brace_depth <= 0 {
                        let mut layout = CStructLayout {
                            name: struct_name,
                            size: 0,
                            align: 8,
                            fields: current_fields.clone(),
                            is_packed: current_struct_is_packed,
                        };
                        layout.compute_layout();

                        layouts.push(layout.clone());
                        self.structs.insert(layout.name.clone(), layout);

                        current_fields.clear();
                        state = ParseState::Idle;
                    }
                }

                ParseState::InEnum { ref name } => {
                    let enum_name = name.clone();

                    brace_depth += line.matches('{').count() as i32;
                    brace_depth -= line.matches('}').count() as i32;

                    // Parse enum values
                    let values = self.parse_enum_values(line);
                    if !values.is_empty() {
                        self.enums.insert(enum_name, values);
                    }

                    if brace_depth <= 0 {
                        state = ParseState::Idle;
                    }
                }

                ParseState::InFunction { .. } => {
                    // Function body parsing not needed for declarations
                    // Just skip until we hit semicolon or newline
                }
            }
        }

        Ok(layouts)
    }

    /// Parse a C function declaration from a line
    /// Handles patterns like: int main(int argc, char* argv[]);
    /// Returns the function signature if parseable
    fn parse_function_decl(&mut self, line: &str) -> Option<CFunction> {
        // Clean up the line - remove storage class specifiers and comments
        let line = line.trim();
        let line = line.strip_prefix("extern").unwrap_or(line).trim();
        let line = line.strip_prefix("static").unwrap_or(line).trim();
        let line = line.strip_prefix("inline").unwrap_or(line).trim();

        // Must have a semicolon at the end for a declaration
        if !line.ends_with(';') {
            return None;
        }

        // Remove trailing semicolon
        let decl = line.trim_end_matches(';').trim();

        // Skip function pointers and complex declarations for now
        if decl.contains('(') && decl.contains("(*)") {
            return None;
        }

        // Find the opening parenthesis for parameter list
        let paren_idx = decl.find('(')?;
        let before_paren = &decl[..paren_idx].trim();

        // Find the closing parenthesis
        let close_paren_idx = decl.find(')')?;
        let params_str = &decl[paren_idx + 1..close_paren_idx];

        // Split return type and function name
        // Last word before '(' is function name, rest is return type
        let before_paren_parts: Vec<&str> = before_paren.split_whitespace().collect();
        if before_paren_parts.len() < 2 {
            return None;
        }

        let func_name = before_paren_parts.last()?;
        let return_type_str = before_paren_parts[..before_paren_parts.len() - 1].join(" ");
        let return_type = CType::parse(&return_type_str)?;

        // Parse parameters
        let params = self.parse_function_params(params_str);

        Some(CFunction {
            name: func_name.to_string(),
            return_type,
            params,
            is_variadic: params_str.contains("..."),
        })
    }

    /// Parse function parameters string into CType list
    fn parse_function_params(&self, params_str: &str) -> Vec<CType> {
        let mut params = Vec::new();

        // Handle empty params (void or nothing)
        let params_str = params_str.trim();
        if params_str.is_empty() || params_str == "void" {
            return params;
        }

        // Split by comma, but be careful with nested parens
        let mut depth = 0;
        let mut current_param = String::new();

        for ch in params_str.chars() {
            match ch {
                '(' | '[' => {
                    depth += 1;
                    current_param.push(ch);
                }
                ')' | ']' => {
                    if depth == 0 {
                        // End of parameters
                        if !current_param.trim().is_empty() && current_param.trim() != "void" {
                            if let Some(typ) = CType::parse(current_param.trim()) {
                                params.push(typ);
                            }
                        }
                    } else {
                        depth -= 1;
                        current_param.push(ch);
                    }
                }
                ',' if depth == 0 => {
                    // End of current parameter
                    if !current_param.trim().is_empty() && current_param.trim() != "void" {
                        if let Some(typ) = CType::parse(current_param.trim()) {
                            params.push(typ);
                        }
                    }
                    current_param.clear();
                }
                _ => {
                    current_param.push(ch);
                }
            }
        }

        // Handle last parameter if any
        if !current_param.trim().is_empty() && current_param.trim() != "void" {
            if let Some(typ) = CType::parse(current_param.trim()) {
                params.push(typ);
            }
        }

        params
    }

    fn parse_struct_start(&self, line: &str) -> Option<(String, bool, bool)> {
        let line = line.trim();

        let is_typedef = line.starts_with("typedef");
        let struct_part = if is_typedef {
            line.strip_prefix("typedef")?.trim()
        } else {
            line
        };

        if !struct_part.starts_with("struct") {
            return None;
        }

        let struct_rest = struct_part.strip_prefix("struct")?.trim();

        // Check for packed attribute
        let is_packed = line.contains("__attribute__((packed))")
            || line.contains("#pragma pack")
            || line.contains("_Static_assert");

        // Extract name (up to { or ;)
        let name = if let Some(idx) = struct_rest.find('{') {
            struct_rest[..idx].trim().to_string()
        } else if let Some(idx) = struct_rest.find(';') {
            struct_rest[..idx].trim().to_string()
        } else {
            struct_rest.to_string()
        };

        // Skip anonymous structs
        if name.is_empty() {
            return None;
        }

        Some((name, is_typedef, is_packed))
    }

    fn parse_enum_start(&self, line: &str) -> Option<String> {
        let line = line.trim();
        let enum_part = line.strip_prefix("enum")?.trim();

        let name = if let Some(idx) = enum_part.find('{') {
            enum_part[..idx].trim().to_string()
        } else if let Some(idx) = enum_part.find(';') {
            enum_part[..idx].trim().to_string()
        } else {
            enum_part.to_string()
        };

        if name.is_empty() {
            return None;
        }

        Some(name)
    }

    fn parse_typedef(&self, line: &str) -> Option<(String, CType)> {
        let line = line.trim();
        let typedef_part = line.strip_prefix("typedef")?.trim();

        // Format: typedef OldType NewType;
        let parts: Vec<&str> = typedef_part.rsplitn(2, ' ').collect();
        if parts.len() < 2 {
            return None;
        }

        let new_name = parts[0].trim().to_string();
        let old_type_str = parts[1].trim();

        let typ = CType::parse(old_type_str)?;

        Some((new_name, typ))
    }

    fn parse_fields_from_line(&self, line: &str, offset: &mut u64) -> Vec<CStructField> {
        let mut fields = Vec::new();

        // Split by semicolons to get individual field declarations
        for decl in line.split(';') {
            let decl = decl.trim();
            if decl.is_empty() || decl.starts_with("struct") || decl.starts_with("union") {
                continue;
            }

            if let Some(field) = CStructField::parse(decl, *offset) {
                *offset += field.size;
                // Account for alignment padding
                if field.size > 0 {
                    let misalignment = field.size % field.align;
                    if misalignment != 0 {
                        *offset += field.align - misalignment;
                    }
                }
                fields.push(field);
            }
        }

        fields
    }

    fn parse_enum_values(&self, line: &str) -> HashMap<String, i64> {
        let mut values = HashMap::new();
        let mut current_value = 0i64;

        for part in line.split(',') {
            let part = part.trim().trim_end_matches('}');
            if part.is_empty() {
                continue;
            }

            // Check for = to get explicit value
            let (name, value) = if let Some(idx) = part.find('=') {
                let name = part[..idx].trim().to_string();
                let value_str = part[idx + 1..].trim();
                let value = value_str.parse().unwrap_or(current_value);
                (name, value)
            } else {
                (part.to_string(), current_value)
            };

            if !name.is_empty() {
                values.insert(name, value);
                current_value = value + 1;
            }
        }

        values
    }

    fn finish_struct(
        &self,
        line: &str,
        _fields: &[CStructField],
        _is_typedef: bool,
        is_packed: bool,
    ) -> Option<CStructLayout> {
        // Parse fields from the one-liner if possible
        let mut fields = Vec::new();
        let mut offset = 0u64;

        if let Some(start) = line.find('{') {
            if let Some(end) = line.find('}') {
                let body = &line[start + 1..end];
                fields = self.parse_fields_from_line(body, &mut offset);
            }
        }

        // Extract struct name
        let name = if let Some(name_end) = line.find('{') {
            let before = line[..name_end].trim();
            if before.contains("typedef") {
                if let Some(struct_idx) = before.find("struct") {
                    let after_struct = before[struct_idx + 6..].trim();
                    after_struct
                        .split_whitespace()
                        .next()
                        .unwrap_or("")
                        .to_string()
                } else {
                    before.split_whitespace().last().unwrap_or("").to_string()
                }
            } else {
                before.strip_prefix("struct")?.trim().to_string()
            }
        } else {
            return None;
        };

        let mut layout = CStructLayout {
            name,
            size: 0,
            align: 8,
            fields,
            is_packed,
        };
        layout.compute_layout();

        Some(layout)
    }

    /// Validate that a C layout matches the expected Chimera layout
    pub fn validate_layout(&mut self, c_layout: &CStructLayout, expected: &LayoutMetadata) -> bool {
        let mut valid = true;

        if c_layout.size != expected.size {
            self.diagnostics.error(
                Code::TypeMismatch,
                &format!(
                    "struct {} has size {} but expected {}",
                    c_layout.name, c_layout.size, expected.size
                ),
            );
            valid = false;
        }

        if c_layout.align != expected.align {
            self.diagnostics.error(
                Code::TypeMismatch,
                &format!(
                    "struct {} has alignment {} but expected {}",
                    c_layout.name, c_layout.align, expected.align
                ),
            );
            valid = false;
        }

        // Validate field offsets
        for expected_field in &expected.fields {
            if let Some(c_field) = c_layout
                .fields
                .iter()
                .find(|f| f.name == expected_field.name)
            {
                if c_field.offset != expected_field.offset {
                    self.diagnostics.error(
                        Code::TypeMismatch,
                        &format!(
                            "field {} in {} has offset {} but expected {}",
                            expected_field.name,
                            c_layout.name,
                            c_field.offset,
                            expected_field.offset
                        ),
                    );
                    valid = false;
                }
            }
        }

        valid
    }

    /// Map a standard errno value to a Chimera error domain
    pub fn map_errno(errno: i32) -> (&'static str, ErrorDomain) {
        match errno {
            1 => ("EPERM", ErrorDomain::Validation), // Operation not permitted
            2 => ("ENOENT", ErrorDomain::Io),        // No such file or directory
            5 => ("EIO", ErrorDomain::Io),           // I/O error
            12 => ("ENOMEM", ErrorDomain::Memory),   // Cannot allocate memory
            17 => ("EEXIST", ErrorDomain::Validation), // File exists
            22 => ("EINVAL", ErrorDomain::Validation), // Invalid argument
            28 => ("ENOSPC", ErrorDomain::Io),       // No space left on device
            _ => ("UNKNOWN", ErrorDomain::Runtime),
        }
    }
}

impl Default for CAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for AdapterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AdapterError::ParseError(s) => write!(f, "C parse error: {}", s),
            AdapterError::InvalidLayout(s) => write!(f, "Invalid layout: {}", s),
            AdapterError::UnsupportedType(s) => write!(f, "Unsupported type: {}", s),
            AdapterError::MissingHeader(s) => write!(f, "Missing header: {}", s),
        }
    }
}

impl std::error::Error for AdapterError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ctype_size() {
        assert_eq!(CType::Char.size(), Some(1));
        assert_eq!(CType::Int.size(), Some(4));
        assert_eq!(CType::Long.size(), Some(8));
        assert_eq!(CType::Double.size(), Some(8));
        assert_eq!(CType::Pointer(Box::new(CType::Int)).size(), Some(8));
    }

    #[test]
    fn test_ctype_alignment() {
        assert_eq!(CType::Char.align(), Some(1));
        assert_eq!(CType::Short.align(), Some(2));
        assert_eq!(CType::Int.align(), Some(4));
        assert_eq!(CType::Long.align(), Some(8));
    }

    #[test]
    fn test_adapter_creation() {
        let adapter = CAdapter::new();
        assert!(!adapter.has_errors());
    }

    #[test]
    fn test_parse_simple_struct() {
        let mut adapter = CAdapter::new();
        let header = "struct Simple { int a; int b; };";
        let layouts = adapter.parse_header(header).unwrap();
        assert_eq!(layouts.len(), 1);
        assert_eq!(layouts[0].name, "Simple");
    }

    #[test]
    fn test_errno_mapping() {
        let (name, domain) = CAdapter::map_errno(2);
        assert_eq!(name, "ENOENT");
        assert_eq!(domain, ErrorDomain::Io);
        let (name, domain) = CAdapter::map_errno(12);
        assert_eq!(name, "ENOMEM");
        assert_eq!(domain, ErrorDomain::Memory);
    }

    #[test]
    fn test_validate_layout_size_mismatch() {
        let mut adapter = CAdapter::new();
        let c_layout = CStructLayout {
            name: "Test".to_string(),
            size: 16,
            align: 8,
            fields: vec![],
            is_packed: false,
        };
        let expected = LayoutMetadata {
            name: "Test".to_string(),
            size: 8,
            align: 8,
            fields: vec![],
            is_packed: false,
        };
        let valid = adapter.validate_layout(&c_layout, &expected);
        assert!(!valid);
        assert!(adapter.has_errors());
    }

    #[test]
    fn test_validate_layout_success() {
        let mut adapter = CAdapter::new();
        let c_layout = CStructLayout {
            name: "Test".to_string(),
            size: 16,
            align: 8,
            fields: vec![],
            is_packed: false,
        };
        let expected = LayoutMetadata {
            name: "Test".to_string(),
            size: 16,
            align: 8,
            fields: vec![],
            is_packed: false,
        };
        let valid = adapter.validate_layout(&c_layout, &expected);
        assert!(valid);
    }

    #[test]
    fn test_ctype_parse_basic() {
        assert_eq!(CType::parse("int"), Some(CType::Int));
        assert_eq!(CType::parse("char"), Some(CType::Char));
        assert_eq!(CType::parse("void"), Some(CType::Void));
    }

    #[test]
    fn test_ctype_parse_pointer() {
        assert_eq!(
            CType::parse("int*"),
            Some(CType::Pointer(Box::new(CType::Int)))
        );
        assert_eq!(
            CType::parse("void*"),
            Some(CType::Pointer(Box::new(CType::Void)))
        );
    }

    #[test]
    fn test_ctype_parse_array() {
        assert_eq!(
            CType::parse("int[10]"),
            Some(CType::Array(Box::new(CType::Int), 10))
        );
    }

    #[test]
    fn test_ctype_display() {
        assert_eq!(CType::Int.to_string(), "int");
        assert_eq!(CType::Pointer(Box::new(CType::Int)).to_string(), "int*");
        assert_eq!(
            CType::Array(Box::new(CType::Int), 10).to_string(),
            "int[10]"
        );
    }

    #[test]
    fn test_compute_struct_layout() {
        let mut layout = CStructLayout {
            name: "Test".to_string(),
            size: 0,
            align: 8,
            fields: vec![
                CStructField {
                    name: "a".to_string(),
                    offset: 0,
                    typ: CType::Int,
                    size: 4,
                    align: 4,
                },
                CStructField {
                    name: "b".to_string(),
                    offset: 0,
                    typ: CType::LongLong,
                    size: 8,
                    align: 8,
                },
            ],
            is_packed: false,
        };
        layout.compute_layout();
        assert_eq!(layout.fields[0].offset, 0);
        assert_eq!(layout.fields[1].offset, 8);
        assert!(layout.size >= 16);
    }

    #[test]
    fn test_struct_field_parse_basic() {
        let field = CStructField::parse("int value;", 0);
        assert!(field.is_some());
        let f = field.unwrap();
        assert_eq!(f.name, "value");
        assert_eq!(f.typ, CType::Int);
    }

    #[test]
    fn test_struct_field_parse_pointer() {
        let field = CStructField::parse("int* ptr;", 0);
        assert!(field.is_some());
        let f = field.unwrap();
        assert_eq!(f.name, "ptr");
        assert_eq!(f.typ, CType::Pointer(Box::new(CType::Int)));
    }

    #[test]
    fn test_parse_function_decl_simple() {
        let mut adapter = CAdapter::new();
        let header = "int main(int argc, char* argv[]);";
        let _layouts = adapter.parse_header(header).unwrap();
        let funcs = adapter.get_functions();
        assert!(funcs.len() >= 1, "Should parse at least one function");

        // Find main function
        let main_func = funcs.iter().find(|f| f.name == "main");
        assert!(main_func.is_some(), "Should find main function");
        let main = main_func.unwrap();
        assert_eq!(main.name, "main");
        assert_eq!(main.return_type, CType::Int);
        assert!(!main.is_variadic);
    }

    #[test]
    fn test_parse_function_decl_with_params() {
        let mut adapter = CAdapter::new();
        let header = "int add(int a, int b);";
        let _layouts = adapter.parse_header(header).unwrap();
        let funcs = adapter.get_functions();

        let add_func = funcs.iter().find(|f| f.name == "add");
        assert!(add_func.is_some(), "Should find add function");
        let add = add_func.unwrap();
        assert_eq!(add.params.len(), 2);
        assert_eq!(add.return_type, CType::Int);
    }

    #[test]
    fn test_parse_function_decl_pointer_return() {
        let mut adapter = CAdapter::new();
        let header = "void* malloc(size_t size);";
        let _layouts = adapter.parse_header(header).unwrap();
        let funcs = adapter.get_functions();

        let malloc_func = funcs.iter().find(|f| f.name == "malloc");
        assert!(malloc_func.is_some(), "Should find malloc function");
        let malloc = malloc_func.unwrap();
        assert_eq!(malloc.return_type, CType::Pointer(Box::new(CType::Void)));
    }

    #[test]
    fn test_parse_function_decl_no_params() {
        let mut adapter = CAdapter::new();
        let header = "int get_value(void);";
        let _layouts = adapter.parse_header(header).unwrap();
        let funcs = adapter.get_functions();

        let get_val = funcs.iter().find(|f| f.name == "get_value");
        assert!(get_val.is_some(), "Should find get_value function");
        assert_eq!(get_val.unwrap().params.len(), 0);
    }

    #[test]
    fn test_parse_function_decl_extern() {
        let mut adapter = CAdapter::new();
        let header = "extern int init(void);";
        let _layouts = adapter.parse_header(header).unwrap();
        let funcs = adapter.get_functions();

        let init_func = funcs.iter().find(|f| f.name == "init");
        assert!(init_func.is_some(), "Should find init function");
    }
}
