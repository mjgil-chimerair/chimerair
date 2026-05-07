//! Zig memory operations modeling.

use serde::{Deserialize, Serialize};

/// Memory address space
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AddressSpace {
    /// Default address space
    Default,
    /// Code address space
    Code,
    /// Stack address space
    Stack,
    /// Global address space
    Global,
    /// Thread-local address space
    ThreadLocal,
    /// Custom address space (with ID)
    Custom(u32),
}

impl AddressSpace {
    /// Get the numeric representation
    pub fn as_u32(&self) -> u32 {
        match self {
            AddressSpace::Default => 0,
            AddressSpace::Code => 1,
            AddressSpace::Stack => 2,
            AddressSpace::Global => 3,
            AddressSpace::ThreadLocal => 4,
            AddressSpace::Custom(id) => *id,
        }
    }

    /// Parse from string
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "default" => Some(AddressSpace::Default),
            "code" => Some(AddressSpace::Code),
            "stack" => Some(AddressSpace::Stack),
            "global" => Some(AddressSpace::Global),
            "thread_local" => Some(AddressSpace::ThreadLocal),
            _ => None,
        }
    }
}

/// Pointer model with address space and alignment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointerModel {
    /// Address space
    pub address_space: AddressSpace,
    /// Pointer width in bits
    pub width_bits: u32,
    /// Alignment requirement in bytes
    pub alignment: u32,
    /// Whether this pointer is const
    pub is_const: bool,
    /// Whether this pointer is volatile
    pub is_volatile: bool,
}

impl PointerModel {
    /// Create a new pointer model
    pub fn new(address_space: AddressSpace, width_bits: u32) -> Self {
        Self {
            address_space,
            width_bits,
            alignment: width_bits / 8,
            is_const: false,
            is_volatile: false,
        }
    }

    /// Set const flag
    pub fn with_const(mut self, is_const: bool) -> Self {
        self.is_const = is_const;
        self
    }

    /// Set volatile flag
    pub fn with_volatile(mut self, is_volatile: bool) -> Self {
        self.is_volatile = is_volatile;
        self
    }
}

/// Memory model for Zig operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryModel {
    /// Default pointer model
    pub default_pointer: PointerModel,
    /// Maximum alignment seen
    pub max_alignment: u32,
    /// Stack alignment requirement
    pub stack_alignment: u32,
}

impl Default for MemoryModel {
    fn default() -> Self {
        Self {
            default_pointer: PointerModel::new(AddressSpace::Default, 64),
            max_alignment: 16,
            stack_alignment: 16,
        }
    }
}

impl MemoryModel {
    /// Create a new memory model
    pub fn new() -> Self {
        Self::default()
    }

    /// Set max alignment
    pub fn with_max_alignment(mut self, align: u32) -> Self {
        self.max_alignment = align;
        self
    }

    /// Set stack alignment
    pub fn with_stack_alignment(mut self, align: u32) -> Self {
        self.stack_alignment = align;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_address_space_as_u32() {
        assert_eq!(AddressSpace::Default.as_u32(), 0);
        assert_eq!(AddressSpace::Code.as_u32(), 1);
        assert_eq!(AddressSpace::Global.as_u32(), 3);
    }

    #[test]
    fn test_address_space_parse() {
        assert_eq!(AddressSpace::parse("default"), Some(AddressSpace::Default));
        assert_eq!(AddressSpace::parse("global"), Some(AddressSpace::Global));
        assert_eq!(AddressSpace::parse("unknown"), None);
    }

    #[test]
    fn test_pointer_model() {
        let ptr = PointerModel::new(AddressSpace::Default, 64)
            .with_const(true)
            .with_volatile(false);
        assert_eq!(ptr.width_bits, 64);
        assert!(ptr.is_const);
        assert!(!ptr.is_volatile);
    }

    #[test]
    fn test_memory_model_default() {
        let model = MemoryModel::default();
        assert_eq!(model.max_alignment, 16);
        assert_eq!(model.stack_alignment, 16);
    }

    #[test]
    fn test_memory_model_custom() {
        let model = MemoryModel::new()
            .with_max_alignment(32)
            .with_stack_alignment(8);
        assert_eq!(model.max_alignment, 32);
        assert_eq!(model.stack_alignment, 8);
    }

    #[test]
    fn test_pointer_model_alignment() {
        let ptr64 = PointerModel::new(AddressSpace::Default, 64);
        assert_eq!(ptr64.alignment, 8);

        let ptr32 = PointerModel::new(AddressSpace::Default, 32);
        assert_eq!(ptr32.alignment, 4);
    }

    #[test]
    fn test_pointer_model_const_volatile() {
        let ptr = PointerModel::new(AddressSpace::Default, 64)
            .with_const(true)
            .with_volatile(true);
        assert!(ptr.is_const);
        assert!(ptr.is_volatile);
    }

    #[test]
    fn test_pointer_model_thread_local() {
        let ptr = PointerModel::new(AddressSpace::ThreadLocal, 64);
        assert_eq!(ptr.address_space, AddressSpace::ThreadLocal);
        assert_eq!(ptr.width_bits, 64);
    }

    #[test]
    fn test_address_space_custom() {
        let custom = AddressSpace::Custom(42);
        assert_eq!(custom.as_u32(), 42);
    }

    #[test]
    fn test_address_space_all() {
        assert_eq!(AddressSpace::Default.as_u32(), 0);
        assert_eq!(AddressSpace::Code.as_u32(), 1);
        assert_eq!(AddressSpace::Stack.as_u32(), 2);
        assert_eq!(AddressSpace::Global.as_u32(), 3);
        assert_eq!(AddressSpace::ThreadLocal.as_u32(), 4);
    }

    #[test]
    fn test_memory_model_chained() {
        let model = MemoryModel::new()
            .with_max_alignment(64)
            .with_stack_alignment(16);
        assert_eq!(model.max_alignment, 64);
        assert_eq!(model.stack_alignment, 16);
    }

    #[test]
    fn test_pointer_model_all_address_spaces() {
        for aspace in &[
            AddressSpace::Default,
            AddressSpace::Code,
            AddressSpace::Stack,
            AddressSpace::Global,
            AddressSpace::ThreadLocal,
        ] {
            let ptr = PointerModel::new(*aspace, 64);
            assert_eq!(ptr.width_bits, 64);
            assert_eq!(ptr.address_space, *aspace);
        }
    }
}
