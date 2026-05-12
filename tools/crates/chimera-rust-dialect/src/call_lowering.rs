//! Task 94: Lower calls to dialect operations
//!
//! Classifies direct calls, intrinsic calls, trait method calls,
//! extern calls, and function pointer calls.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Call operation in the dialect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallOp {
    /// Unique ID for this call
    pub id: String,
    /// Kind of call
    pub kind: CallKind,
    /// Target of the call
    pub callee: Callee,
    /// Arguments
    pub args: Vec<String>,
    /// Return place (where to store result)
    pub return_place: Option<String>,
    /// Cleanup/dispatch block if unwind
    pub unwind_target: Option<String>,
    /// Whether this call can unwind
    pub can_unwind: bool,
    /// Whether this call is in a `Drop` terminator
    pub is_drop: bool,
}

/// Kind of call operation
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CallKind {
    /// Direct function call: `foo(a, b)`
    Direct,
    /// Intrinsic call: `core::intrinsics::foo(a)`
    Intrinsic,
    /// Trait method call via vtable: `obj.method()`
    TraitMethod,
    /// Closure call: `(closure)(a, b)`
    Closure,
    /// Call through function pointer: `(fn_ptr)(a, b)`
    FnPointer,
    /// Extern "C" call: `extern_fn(a)`
    ExternC,
    /// Virtual call (future support, currently unsupported)
    Virtual,
}

/// Target of a call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Callee {
    /// Direct function reference
    Fn {
        /// Stable ID of the function
        stable_id: String,
        /// Symbol name
        symbol: String,
    },
    /// Intrinsic by name
    Intrinsic {
        /// Full intrinsic name
        name: String,
    },
    /// Trait method
    TraitMethod {
        /// Trait name
        trait_name: String,
        /// Method name
        method_name: String,
        /// Self type
        self_type: String,
    },
    /// Closure
    Closure {
        /// Closure expression/place
        place: String,
    },
    /// Function pointer
    FnPtr {
        /// Place containing the function pointer
        place: String,
    },
    /// Extern function
    Extern {
        /// Symbol name
        symbol: String,
        /// Library (if specified)
        library: Option<String>,
    },
}

/// Intrinsic classification
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum IntrinsicKind {
    /// Memory allocation: `alloc`, `dealloc`, `malloc`, `free`
    Alloc,
    /// Atomic operation: `atomic_*`
    Atomic,
    /// Bit manipulation: `ctlz`, `cttz`, `bswap`
    Bit,
    /// Floating point: `floor`, `ceil`, `sqrt`
    Float,
    /// SIMD: `simd_*`
    Simd,
    /// Safety: `transmute`, `offset`
    Safety,
    /// Debug: `abort`, `breakpoint`
    Debug,
    /// Other intrinsics
    Other,
}

impl Callee {
    /// Classify an intrinsic
    pub fn classify_intrinsic(name: &str) -> IntrinsicKind {
        let name_lower = name.to_lowercase();
        if name_lower.contains("alloc")
            || name_lower.contains("dealloc")
            || name_lower.contains("malloc")
        {
            IntrinsicKind::Alloc
        } else if name_lower.contains("atomic") {
            IntrinsicKind::Atomic
        } else if name_lower.contains("ctlz")
            || name_lower.contains("cttz")
            || name_lower.contains("bswap")
        {
            IntrinsicKind::Bit
        } else if name_lower.contains("floor")
            || name_lower.contains("ceil")
            || name_lower.contains("sqrt")
        {
            IntrinsicKind::Float
        } else if name_lower.contains("simd") {
            IntrinsicKind::Simd
        } else if name_lower.contains("transmute") || name_lower.contains("offset") {
            IntrinsicKind::Safety
        } else if name_lower.contains("abort") || name_lower.contains("breakpoint") {
            IntrinsicKind::Debug
        } else {
            IntrinsicKind::Other
        }
    }

    /// Check if this call is safe (no unsafe requirements)
    pub fn is_safe(&self) -> bool {
        match self {
            Callee::Fn { .. } => true,
            Callee::Intrinsic { name } => {
                matches!(
                    Self::classify_intrinsic(name),
                    IntrinsicKind::Bit | IntrinsicKind::Float | IntrinsicKind::Debug
                )
            }
            Callee::TraitMethod { .. } => true,
            Callee::Closure { .. } => true,
            Callee::FnPtr { .. } => false,  // Depends on the pointer
            Callee::Extern { .. } => false, // Extern calls are inherently unsafe
        }
    }
}

/// Lower a call to dialect representation
pub fn lower_call(
    callee: Callee,
    args: Vec<String>,
    return_place: Option<String>,
    unwind_target: Option<String>,
) -> CallOp {
    let can_unwind = unwind_target.is_some() || !callee.is_safe();

    CallOp {
        id: format!("call_{}", callee.call_id()),
        kind: match callee {
            Callee::Fn { .. } => CallKind::Direct,
            Callee::Intrinsic { .. } => CallKind::Intrinsic,
            Callee::TraitMethod { .. } => CallKind::TraitMethod,
            Callee::Closure { .. } => CallKind::Closure,
            Callee::FnPtr { .. } => CallKind::FnPointer,
            Callee::Extern { .. } => CallKind::ExternC,
        },
        callee,
        args,
        return_place,
        unwind_target,
        can_unwind,
        is_drop: false,
    }
}

/// Lower an intrinsic call
pub fn lower_intrinsic(name: &str, args: Vec<String>) -> CallOp {
    let intrinsic_kind = Callee::classify_intrinsic(name);

    CallOp {
        id: format!("intrinsic_{}", name.replace("::", "_").to_lowercase()),
        kind: CallKind::Intrinsic,
        callee: Callee::Intrinsic {
            name: name.to_string(),
        },
        args,
        return_place: None,
        unwind_target: None,
        can_unwind: matches!(intrinsic_kind, IntrinsicKind::Safety),
        is_drop: false,
    }
}

/// Call analyzer
#[derive(Default)]
pub struct CallAnalyzer {
    calls: Vec<CallOp>,
    intrinsics_by_kind: HashMap<IntrinsicKind, Vec<String>>,
}

impl CallAnalyzer {
    /// Create a new call analyzer
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a call
    pub fn record_call(&mut self, call: CallOp) {
        let id = call.id.clone();
        if matches!(call.kind, CallKind::Intrinsic) {
            if let Callee::Intrinsic { name } = &call.callee {
                let kind = Callee::classify_intrinsic(name);
                self.intrinsics_by_kind.entry(kind).or_default().push(id);
            }
        }
        self.calls.push(call);
    }

    /// Get all recorded calls
    pub fn calls(&self) -> &[CallOp] {
        &self.calls
    }

    /// Get calls by kind
    pub fn calls_by_kind(&self, kind: CallKind) -> Vec<&CallOp> {
        self.calls.iter().filter(|c| c.kind == kind).collect()
    }

    /// Get intrinsic counts by kind
    pub fn intrinsic_stats(&self) -> HashMap<IntrinsicKind, usize> {
        self.intrinsics_by_kind
            .iter()
            .map(|(k, v)| (*k, v.len()))
            .collect()
    }

    /// Check for unsafe calls without safety annotations
    pub fn check_unsafe_calls(&self) -> Vec<String> {
        self.calls
            .iter()
            .filter(|c| !c.callee.is_safe())
            .map(|c| c.id.clone())
            .collect()
    }

    /// Validate call consistency
    pub fn validate(&self) -> Result<(), CallError> {
        for call in &self.calls {
            // Check for calls with unwind targets on safe functions
            if call.unwind_target.is_some() && call.callee.is_safe() {
                return Err(CallError::InvalidUnwind(call.id.clone()));
            }
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CallError {
    #[error("call '{0}' has invalid unwind target for safe function")]
    InvalidUnwind(String),
    #[error("missing return place for non-unit call")]
    MissingReturnPlace,
    #[error("extern call without symbol")]
    ExternWithoutSymbol,
}

impl Callee {
    fn call_id(&self) -> String {
        match self {
            Callee::Fn { symbol, .. } => symbol.clone(),
            Callee::Intrinsic { name } => name.replace("::", "_"),
            Callee::TraitMethod {
                trait_name,
                method_name,
                ..
            } => {
                format!("{}_{}", trait_name, method_name)
            }
            Callee::Closure { place } => place.clone(),
            Callee::FnPtr { place } => place.clone(),
            Callee::Extern { symbol, .. } => symbol.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lower_direct_call() {
        let call = lower_call(
            Callee::Fn {
                stable_id: "fn123".to_string(),
                symbol: "test_fn".to_string(),
            },
            vec!["x".to_string(), "y".to_string()],
            Some("_0".to_string()),
            None,
        );
        assert_eq!(call.kind, CallKind::Direct);
        assert!(!call.can_unwind);
    }

    #[test]
    fn test_lower_extern_call() {
        let call = lower_call(
            Callee::Extern {
                symbol: "printf".to_string(),
                library: Some("libc".to_string()),
            },
            vec!["fmt".to_string()],
            None,
            None,
        );
        assert_eq!(call.kind, CallKind::ExternC);
        assert!(!call.callee.is_safe());
    }

    #[test]
    fn test_intrinsic_classification() {
        assert_eq!(
            Callee::classify_intrinsic("core::intrinsics::abort"),
            IntrinsicKind::Debug
        );
        assert_eq!(
            Callee::classify_intrinsic("core::intrinsics::ctlz"),
            IntrinsicKind::Bit
        );
        assert_eq!(
            Callee::classify_intrinsic("core::intrinsics::sqrtf32"),
            IntrinsicKind::Float
        );
    }

    #[test]
    fn test_lower_intrinsic() {
        let call = lower_intrinsic("core::intrinsics::ctlz", vec!["x".to_string()]);
        assert_eq!(call.kind, CallKind::Intrinsic);
        assert!(!call.can_unwind); // Bit intrinsics don't unwind
    }

    #[test]
    fn test_call_analyzer_intrinsic_stats() {
        let mut analyzer = CallAnalyzer::new();
        analyzer.record_call(lower_intrinsic("abort", vec![]));
        analyzer.record_call(lower_intrinsic("ctlz", vec!["x".to_string()]));
        analyzer.record_call(lower_intrinsic("ctlz", vec!["y".to_string()]));

        let stats = analyzer.intrinsic_stats();
        assert_eq!(stats[&IntrinsicKind::Debug], 1);
        assert_eq!(stats[&IntrinsicKind::Bit], 2);
    }

    #[test]
    fn test_unsafe_call_detection() {
        let mut analyzer = CallAnalyzer::new();
        analyzer.record_call(lower_call(
            Callee::Fn {
                stable_id: "fn1".to_string(),
                symbol: "safe_fn".to_string(),
            },
            vec![],
            None,
            None,
        ));
        analyzer.record_call(lower_call(
            Callee::Extern {
                symbol: "ffi_call".to_string(),
                library: None,
            },
            vec![],
            None,
            None,
        ));

        let unsafe_calls = analyzer.check_unsafe_calls();
        assert_eq!(unsafe_calls.len(), 1);
    }

    #[test]
    fn test_call_validation() {
        let analyzer = CallAnalyzer::new();
        assert!(analyzer.validate().is_ok());
    }

    #[test]
    fn test_trait_method_call() {
        let call = lower_call(
            Callee::TraitMethod {
                trait_name: "Iterator".to_string(),
                method_name: "next".to_string(),
                self_type: "Vec<i32>".to_string(),
            },
            vec!["self".to_string()],
            Some("result".to_string()),
            None,
        );
        assert_eq!(call.kind, CallKind::TraitMethod);
        assert!(call.callee.is_safe());
    }

    #[test]
    fn test_closure_call() {
        let call = lower_call(
            Callee::Closure {
                place: "closure_var".to_string(),
            },
            vec!["x".to_string()],
            Some("_0".to_string()),
            None,
        );
        assert_eq!(call.kind, CallKind::Closure);
    }
}
