//! Calling convention for BEAM ABI.
//!
//! Defines how function calls are marshaled across language boundaries.

use chimera_beam_dialect::BeamType;
use serde::{Deserialize, Serialize};

/// Calling convention variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CallingConvention {
    /// Standard BEAM calling convention (NIF/C node).
    Beam,
    /// Erlang/Elixir OTP calling convention.
    Otp,
    /// Native BEAM export (for BIFs).
    Bif,
    /// Async message passing convention.
    Async,
}

impl Default for CallingConvention {
    fn default() -> Self {
        CallingConvention::Beam
    }
}

impl CallingConvention {
    /// Get the convention name.
    pub fn as_str(&self) -> &'static str {
        match self {
            CallingConvention::Beam => "beam",
            CallingConvention::Otp => "otp",
            CallingConvention::Bif => "bif",
            CallingConvention::Async => "async",
        }
    }
}

/// An argument to a function.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Argument {
    /// Argument name (if known).
    pub name: Option<String>,
    /// Argument type.
    pub arg_type: BeamType,
    /// Whether it's passed in a register.
    pub in_register: bool,
    /// Register index (if in_register).
    pub register_index: Option<u8>,
    /// Stack offset (if not in_register).
    pub stack_offset: Option<i32>,
    /// Whether it's passed by reference.
    pub by_reference: bool,
}

impl Argument {
    /// Create a new argument.
    pub fn new(name: Option<&str>, arg_type: BeamType) -> Self {
        Argument {
            name: name.map(String::from),
            arg_type,
            in_register: false,
            register_index: None,
            stack_offset: None,
            by_reference: false,
        }
    }

    /// Create with register.
    pub fn in_register(mut self, index: u8) -> Self {
        self.in_register = true;
        self.register_index = Some(index);
        self
    }

    /// Create with stack offset.
    pub fn on_stack(mut self, offset: i32) -> Self {
        self.in_register = false;
        self.stack_offset = Some(offset);
        self
    }

    /// Create passed by reference.
    pub fn by_ref(mut self) -> Self {
        self.by_reference = true;
        self
    }
}

/// Return value from a function.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnValue {
    /// Return type.
    pub ret_type: BeamType,
    /// Whether it's returned in a register.
    pub in_register: bool,
    /// Register index (if in_register).
    pub register_index: Option<u8>,
    /// Stack offset (if not in_register).
    pub stack_offset: Option<i32>,
}

impl ReturnValue {
    /// Create a new return value.
    pub fn new(ret_type: BeamType) -> Self {
        ReturnValue {
            ret_type,
            in_register: true,
            register_index: Some(0), // Return value in RAX/EAX
            stack_offset: None,
        }
    }

    /// Create returned on stack.
    pub fn on_stack(mut self, offset: i32) -> Self {
        self.in_register = false;
        self.stack_offset = Some(offset);
        self.register_index = None;
        self
    }
}

/// A slot on the stack for argument passing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackSlot {
    /// Slot index (0-based).
    pub index: usize,
    /// Size in bytes.
    pub size: usize,
    /// Alignment requirement.
    pub alignment: usize,
    /// Type of value stored.
    pub slot_type: StackSlotType,
}

impl StackSlot {
    /// Create a new stack slot.
    pub fn new(index: usize, size: usize, slot_type: StackSlotType) -> Self {
        StackSlot {
            index,
            size,
            alignment: default_alignment(size),
            slot_type,
        }
    }
}

/// Stack slot types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StackSlotType {
    /// Integer value.
    Integer,
    /// Floating point value.
    Float,
    /// Pointer/reference.
    Pointer,
    /// Aggregate (tuple, list, etc.).
    Aggregate,
}

fn default_alignment(size: usize) -> usize {
    if size <= 1 {
        1
    } else if size <= 2 {
        2
    } else if size <= 4 {
        4
    } else if size <= 8 {
        8
    } else {
        16
    }
}

/// Function signature for BEAM ABI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionSignature {
    /// Module name.
    pub module: String,
    /// Function name.
    pub function: String,
    /// Arity (number of arguments).
    pub arity: u8,
    /// Calling convention.
    pub convention: CallingConvention,
    /// Arguments.
    pub arguments: Vec<Argument>,
    /// Return value.
    pub return_value: Option<ReturnValue>,
    /// Whether function is a BIF.
    pub is_bif: bool,
}

impl FunctionSignature {
    /// Create a new signature.
    pub fn new(module: impl Into<String>, function: impl Into<String>, arity: u8) -> Self {
        FunctionSignature {
            module: module.into(),
            function: function.into(),
            arity,
            convention: CallingConvention::default(),
            arguments: vec![],
            return_value: None,
            is_bif: false,
        }
    }

    /// Set calling convention.
    pub fn with_convention(mut self, convention: CallingConvention) -> Self {
        self.convention = convention;
        self
    }

    /// Add an argument.
    pub fn add_arg(mut self, arg: Argument) -> Self {
        self.arguments.push(arg);
        self
    }

    /// Set return value.
    pub fn with_return(mut self, ret: ReturnValue) -> Self {
        self.return_value = Some(ret);
        self
    }

    /// Mark as BIF.
    pub fn as_bif(mut self) -> Self {
        self.is_bif = true;
        self
    }

    /// Get the full name (module:function/arity).
    pub fn full_name(&self) -> String {
        format!("{}:{}/{}", self.module, self.function, self.arity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calling_convention_as_str() {
        assert_eq!(CallingConvention::Beam.as_str(), "beam");
        assert_eq!(CallingConvention::Otp.as_str(), "otp");
        assert_eq!(CallingConvention::Bif.as_str(), "bif");
        assert_eq!(CallingConvention::Async.as_str(), "async");
    }

    #[test]
    fn test_argument_new() {
        let arg = Argument::new(Some("x"), BeamType::atom());
        assert_eq!(arg.name, Some("x".to_string()));
        assert!(!arg.in_register); // Default is stack-allocated
    }

    #[test]
    fn test_argument_in_register() {
        let arg = Argument::new(None, BeamType::pid()).in_register(2);
        assert!(arg.in_register);
        assert_eq!(arg.register_index, Some(2));
    }

    #[test]
    fn test_argument_on_stack() {
        let arg = Argument::new(None, BeamType::atom()).on_stack(8);
        assert!(!arg.in_register);
        assert_eq!(arg.stack_offset, Some(8));
    }

    #[test]
    fn test_return_value_new() {
        let ret = ReturnValue::new(BeamType::atom());
        assert!(ret.in_register);
    }

    #[test]
    fn test_stack_slot_alignment() {
        let slot1 = StackSlot::new(0, 4, StackSlotType::Integer);
        assert_eq!(slot1.alignment, 4);

        let slot2 = StackSlot::new(1, 8, StackSlotType::Pointer);
        assert_eq!(slot2.alignment, 8);
    }

    #[test]
    fn test_function_signature_full_name() {
        let sig = FunctionSignature::new("mod", "fun", 2);
        assert_eq!(sig.full_name(), "mod:fun/2");
    }

    #[test]
    fn test_function_signature_builder() {
        let sig = FunctionSignature::new("mod", "fun", 1)
            .with_convention(CallingConvention::Bif)
            .add_arg(Argument::new(Some("x"), BeamType::atom()))
            .with_return(ReturnValue::new(BeamType::pid()))
            .as_bif();

        assert_eq!(sig.convention, CallingConvention::Bif);
        assert_eq!(sig.arguments.len(), 1);
        assert!(sig.return_value.is_some());
        assert!(sig.is_bif);
    }
}
