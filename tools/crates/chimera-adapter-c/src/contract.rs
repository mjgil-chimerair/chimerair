use chimera_component::Symbol;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CallingConvention {
    Cdecl,
    Stdcall,
    Fastcall,
    Thiscall,
    Aapcs,
    Vectorcall,
}

impl CallingConvention {
    pub fn as_str(&self) -> &'static str {
        match self {
            CallingConvention::Cdecl => "cdecl",
            CallingConvention::Stdcall => "stdcall",
            CallingConvention::Fastcall => "fastcall",
            CallingConvention::Thiscall => "thiscall",
            CallingConvention::Aapcs => "aapcs",
            CallingConvention::Vectorcall => "vectorcall",
        }
    }
}

impl Default for CallingConvention {
    fn default() -> Self {
        CallingConvention::Cdecl
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AllocatorContract {
    CallerAllocates,
    CalleeAllocatesFree,
    CallerAllocatesCalleeFrees,
    NoHeapAllocation,
}

impl AllocatorContract {
    pub fn as_str(&self) -> &'static str {
        match self {
            AllocatorContract::CallerAllocates => "caller-allocates",
            AllocatorContract::CalleeAllocatesFree => "callee-allocates-frees",
            AllocatorContract::CallerAllocatesCalleeFrees => "caller-allocates-calleep-frees",
            AllocatorContract::NoHeapAllocation => "no-heap-allocation",
        }
    }
}

impl Default for AllocatorContract {
    fn default() -> Self {
        AllocatorContract::CallerAllocates
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrnoContract {
    MaySetErrno,
    DoesNotSetErrno,
}

impl ErrnoContract {
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrnoContract::MaySetErrno => "may-set-errno",
            ErrnoContract::DoesNotSetErrno => "does-not-set-errno",
        }
    }
}

impl Default for ErrnoContract {
    fn default() -> Self {
        ErrnoContract::MaySetErrno
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CFunctionExport {
    pub name: String,
    pub params: Vec<CParamContract>,
    pub return_type: CTypeContract,
    pub calling_convention: CallingConvention,
    pub allocator: AllocatorContract,
    pub errno: ErrnoContract,
    pub variadic: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CFunctionImport {
    pub name: String,
    pub params: Vec<CParamContract>,
    pub return_type: CTypeContract,
    pub calling_convention: CallingConvention,
    pub allocator: AllocatorContract,
    pub errno: ErrnoContract,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CParamContract {
    pub name: String,
    pub typ: CTypeContract,
    pub is_const: bool,
    pub is_output: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CTypeContract {
    Void,
    Bool,
    Char,
    Short,
    Int,
    Long,
    LongLong,
    UnsignedChar,
    UnsignedShort,
    UnsignedInt,
    UnsignedLong,
    UnsignedLongLong,
    Float,
    Double,
    Pointer(Box<CTypeContract>),
    ConstPointer(Box<CTypeContract>),
    StringPointer,
    Struct(String),
    Union(String),
    Enum(String),
    Opaque(String),
    SizeT,
    SsizeT,
}

impl CTypeContract {
    pub fn is_integer(&self) -> bool {
        matches!(
            self,
            CTypeContract::Char
                | CTypeContract::Short
                | CTypeContract::Int
                | CTypeContract::Long
                | CTypeContract::LongLong
                | CTypeContract::UnsignedChar
                | CTypeContract::UnsignedShort
                | CTypeContract::UnsignedInt
                | CTypeContract::UnsignedLong
                | CTypeContract::UnsignedLongLong
                | CTypeContract::Bool
                | CTypeContract::SizeT
                | CTypeContract::SsizeT
        )
    }

    pub fn is_float(&self) -> bool {
        matches!(self, CTypeContract::Float | CTypeContract::Double)
    }

    pub fn is_pointer(&self) -> bool {
        matches!(
            self,
            CTypeContract::Pointer(_)
                | CTypeContract::ConstPointer(_)
                | CTypeContract::StringPointer
        )
    }

    pub fn pointee_type(&self) -> Option<&CTypeContract> {
        match self {
            CTypeContract::Pointer(inner) | CTypeContract::ConstPointer(inner) => Some(inner),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CStructContract {
    pub name: String,
    pub size: u64,
    pub alignment: u32,
    pub fields: Vec<CFieldContract>,
    pub is_packed: bool,
    pub is_opaque: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CFieldContract {
    pub name: String,
    pub typ: CTypeContract,
    pub offset: u64,
    pub size: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CABIEdgeContract {
    pub exported_functions: Vec<CFunctionExport>,
    pub imported_functions: Vec<CFunctionImport>,
    pub exported_structs: Vec<CStructContract>,
    pub callbacks: Vec<CCallbackContract>,
    pub symbols: Vec<Symbol>,
}

impl CABIEdgeContract {
    pub fn new() -> Self {
        Self {
            exported_functions: Vec::new(),
            imported_functions: Vec::new(),
            exported_structs: Vec::new(),
            callbacks: Vec::new(),
            symbols: Vec::new(),
        }
    }

    pub fn add_export(&mut self, export: CFunctionExport) {
        self.symbols.push(Symbol::new(export.name.clone()));
        self.exported_functions.push(export);
    }

    pub fn add_import(&mut self, import: CFunctionImport) {
        self.symbols.push(Symbol::new(import.name.clone()));
        self.imported_functions.push(import);
    }

    pub fn add_struct(&mut self, s: CStructContract) {
        self.exported_structs.push(s);
    }

    pub fn add_callback(&mut self, cb: CCallbackContract) {
        self.callbacks.push(cb);
    }

    pub fn has_exports(&self) -> bool {
        !self.exported_functions.is_empty()
    }

    pub fn has_imports(&self) -> bool {
        !self.imported_functions.is_empty()
    }

    pub fn total_symbols(&self) -> usize {
        self.symbols.len()
    }

    pub fn merge(&mut self, other: &CABIEdgeContract) {
        self.exported_functions
            .extend_from_slice(&other.exported_functions);
        self.imported_functions
            .extend_from_slice(&other.imported_functions);
        self.exported_structs
            .extend_from_slice(&other.exported_structs);
        self.callbacks.extend_from_slice(&other.callbacks);
        self.symbols.extend_from_slice(&other.symbols);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CCallbackContract {
    pub name: String,
    pub params: Vec<CParamContract>,
    pub return_type: CTypeContract,
    pub calling_convention: CallingConvention,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_contract() {
        let contract = CABIEdgeContract::new();
        assert!(!contract.has_exports());
        assert!(!contract.has_imports());
        assert_eq!(contract.total_symbols(), 0);
    }

    #[test]
    fn test_add_export_function() {
        let mut contract = CABIEdgeContract::new();
        contract.add_export(CFunctionExport {
            name: "c_helper".to_string(),
            params: vec![CParamContract {
                name: "x".to_string(),
                typ: CTypeContract::Int,
                is_const: false,
                is_output: false,
            }],
            return_type: CTypeContract::Int,
            calling_convention: CallingConvention::Cdecl,
            allocator: AllocatorContract::CallerAllocates,
            errno: ErrnoContract::DoesNotSetErrno,
            variadic: false,
        });
        assert!(contract.has_exports());
        assert_eq!(contract.total_symbols(), 1);
    }

    #[test]
    fn test_add_import_function() {
        let mut contract = CABIEdgeContract::new();
        contract.add_import(CFunctionImport {
            name: "rust_helper".to_string(),
            params: vec![CParamContract {
                name: "buf".to_string(),
                typ: CTypeContract::Pointer(Box::new(CTypeContract::Void)),
                is_const: false,
                is_output: true,
            }],
            return_type: CTypeContract::Int,
            calling_convention: CallingConvention::Cdecl,
            allocator: AllocatorContract::CalleeAllocatesFree,
            errno: ErrnoContract::MaySetErrno,
        });
        assert!(contract.has_imports());
        assert_eq!(contract.total_symbols(), 1);
    }

    #[test]
    fn test_add_struct_contract() {
        let mut contract = CABIEdgeContract::new();
        contract.add_struct(CStructContract {
            name: "Point".to_string(),
            size: 8,
            alignment: 4,
            fields: vec![
                CFieldContract {
                    name: "x".to_string(),
                    typ: CTypeContract::Int,
                    offset: 0,
                    size: 4,
                },
                CFieldContract {
                    name: "y".to_string(),
                    typ: CTypeContract::Int,
                    offset: 4,
                    size: 4,
                },
            ],
            is_packed: false,
            is_opaque: false,
        });
        assert_eq!(contract.exported_structs.len(), 1);
        assert_eq!(contract.exported_structs[0].size, 8);
    }

    #[test]
    fn test_merge_contracts() {
        let mut a = CABIEdgeContract::new();
        a.add_export(CFunctionExport {
            name: "foo".to_string(),
            params: vec![],
            return_type: CTypeContract::Void,
            calling_convention: CallingConvention::Cdecl,
            allocator: AllocatorContract::CallerAllocates,
            errno: ErrnoContract::DoesNotSetErrno,
            variadic: false,
        });

        let mut b = CABIEdgeContract::new();
        b.add_import(CFunctionImport {
            name: "bar".to_string(),
            params: vec![],
            return_type: CTypeContract::Int,
            calling_convention: CallingConvention::Cdecl,
            allocator: AllocatorContract::CallerAllocates,
            errno: ErrnoContract::DoesNotSetErrno,
        });

        a.merge(&b);
        assert_eq!(a.total_symbols(), 2);
        assert!(a.has_exports());
        assert!(a.has_imports());
    }

    #[test]
    fn test_calling_convention_default() {
        assert_eq!(CallingConvention::default(), CallingConvention::Cdecl);
    }

    #[test]
    fn test_calling_convention_as_str() {
        assert_eq!(CallingConvention::Cdecl.as_str(), "cdecl");
        assert_eq!(CallingConvention::Stdcall.as_str(), "stdcall");
    }

    #[test]
    fn test_type_classification() {
        assert!(CTypeContract::Int.is_integer());
        assert!(CTypeContract::Float.is_float());
        assert!(CTypeContract::Pointer(Box::new(CTypeContract::Void)).is_pointer());
        assert!(!CTypeContract::Struct("Foo".to_string()).is_integer());
    }

    #[test]
    fn test_pointee_type() {
        let ptr = CTypeContract::Pointer(Box::new(CTypeContract::Char));
        assert_eq!(*ptr.pointee_type().unwrap(), CTypeContract::Char);
    }

    #[test]
    fn test_callback_contract() {
        let cb = CCallbackContract {
            name: "comparator".to_string(),
            params: vec![
                CParamContract {
                    name: "a".to_string(),
                    typ: CTypeContract::ConstPointer(Box::new(CTypeContract::Void)),
                    is_const: true,
                    is_output: false,
                },
                CParamContract {
                    name: "b".to_string(),
                    typ: CTypeContract::ConstPointer(Box::new(CTypeContract::Void)),
                    is_const: true,
                    is_output: false,
                },
            ],
            return_type: CTypeContract::Int,
            calling_convention: CallingConvention::Cdecl,
        };
        assert_eq!(cb.name, "comparator");
        assert_eq!(cb.params.len(), 2);
    }

    #[test]
    fn test_allocator_contract_as_str() {
        assert_eq!(
            AllocatorContract::CallerAllocates.as_str(),
            "caller-allocates"
        );
        assert_eq!(
            AllocatorContract::NoHeapAllocation.as_str(),
            "no-heap-allocation"
        );
    }

    #[test]
    fn test_errno_contract_as_str() {
        assert_eq!(ErrnoContract::MaySetErrno.as_str(), "may-set-errno");
        assert_eq!(
            ErrnoContract::DoesNotSetErrno.as_str(),
            "does-not-set-errno"
        );
    }

    #[test]
    fn test_contract_serialization() {
        let mut contract = CABIEdgeContract::new();
        contract.add_export(CFunctionExport {
            name: "compute".to_string(),
            params: vec![CParamContract {
                name: "n".to_string(),
                typ: CTypeContract::SizeT,
                is_const: false,
                is_output: false,
            }],
            return_type: CTypeContract::Double,
            calling_convention: CallingConvention::Cdecl,
            allocator: AllocatorContract::CallerAllocates,
            errno: ErrnoContract::DoesNotSetErrno,
            variadic: false,
        });

        let json = serde_json::to_string_pretty(&contract).unwrap();
        let deserialized: CABIEdgeContract = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.exported_functions.len(), 1);
        assert_eq!(deserialized.exported_functions[0].name, "compute");
    }

    #[test]
    fn test_opaque_struct_contract() {
        let s = CStructContract {
            name: "FILE".to_string(),
            size: 0,
            alignment: 0,
            fields: vec![],
            is_packed: false,
            is_opaque: true,
        };
        assert!(s.is_opaque);
        assert!(s.fields.is_empty());
    }

    #[test]
    fn test_type_contract_string_pointer() {
        let s = CTypeContract::StringPointer;
        assert!(s.is_pointer());
        assert_eq!(s.pointee_type(), None);
    }

    #[test]
    fn test_add_callback_to_contract() {
        let mut contract = CABIEdgeContract::new();
        contract.add_callback(CCallbackContract {
            name: "on_event".to_string(),
            params: vec![CParamContract {
                name: "event_id".to_string(),
                typ: CTypeContract::Int,
                is_const: false,
                is_output: false,
            }],
            return_type: CTypeContract::Void,
            calling_convention: CallingConvention::Cdecl,
        });
        assert_eq!(contract.callbacks.len(), 1);
        assert_eq!(contract.callbacks[0].name, "on_event");
    }
}
