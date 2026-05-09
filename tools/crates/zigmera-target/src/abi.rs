//! ABI classification and representation.

use serde::{Deserialize, Serialize};

/// ABI classification for calling conventions and data representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Abi {
    None,
    Gnu,
    GnuIlp32,
    GnuLp64,
    GnuLlvm,
    Musl,
    MuslIlp32,
    MuslLp64,
    Apple,
    AppleSimulator,
    Msvc,
    Ptx,
    Itanium,
    Other(String),
}

impl Abi {
    pub fn pointer_width(&self) -> u32 {
        match self {
            Abi::GnuIlp32 | Abi::MuslIlp32 => 32,
            Abi::GnuLp64 | Abi::MuslLp64 | Abi::Gnu | Abi::Musl | Abi::GnuLlvm => 64,
            Abi::Msvc => 64,
            Abi::Apple | Abi::AppleSimulator => 64,
            Abi::Ptx => 64,
            Abi::Itanium => 64,
            Abi::None => 64,
            Abi::Other(_) => 64,
        }
    }

    pub fn is_msvc(&self) -> bool {
        matches!(self, Abi::Msvc)
    }

    pub fn is_gnu(&self) -> bool {
        matches!(self, Abi::Gnu | Abi::GnuIlp32 | Abi::GnuLp64 | Abi::GnuLlvm)
    }

    pub fn is_musl(&self) -> bool {
        matches!(self, Abi::Musl | Abi::MuslIlp32 | Abi::MuslLp64)
    }

    pub fn is_apple(&self) -> bool {
        matches!(self, Abi::Apple | Abi::AppleSimulator)
    }

    pub fn long_width(&self) -> u32 {
        match self {
            Abi::GnuIlp32 | Abi::MuslIlp32 => 32,
            _ => 64,
        }
    }

    pub fn long_long_width(&self) -> u32 {
        64
    }
}

impl Default for Abi {
    fn default() -> Self {
        Self::None
    }
}

impl std::fmt::Display for Abi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Abi::None => write!(f, "none"),
            Abi::Gnu => write!(f, "gnu"),
            Abi::GnuIlp32 => write!(f, "gnuilp32"),
            Abi::GnuLp64 => write!(f, "gnulp64"),
            Abi::GnuLlvm => write!(f, "gnullvm"),
            Abi::Musl => write!(f, "musl"),
            Abi::MuslIlp32 => write!(f, "muslip32"),
            Abi::MuslLp64 => write!(f, "musllp64"),
            Abi::Apple => write!(f, "apple"),
            Abi::AppleSimulator => write!(f, "apple-simulator"),
            Abi::Msvc => write!(f, "msvc"),
            Abi::Ptx => write!(f, "ptx"),
            Abi::Itanium => write!(f, "itanium"),
            Abi::Other(s) => write!(f, "other:{}", s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abi_pointer_width() {
        assert_eq!(Abi::GnuIlp32.pointer_width(), 32);
        assert_eq!(Abi::GnuLp64.pointer_width(), 64);
        assert_eq!(Abi::Msvc.pointer_width(), 64);
    }

    #[test]
    fn test_abi_is_msvc() {
        assert!(Abi::Msvc.is_msvc());
        assert!(!Abi::Gnu.is_msvc());
        assert!(!Abi::Apple.is_msvc());
    }

    #[test]
    fn test_abi_is_gnu() {
        assert!(Abi::Gnu.is_gnu());
        assert!(Abi::GnuLlvm.is_gnu());
        assert!(!Abi::Msvc.is_gnu());
    }

    #[test]
    fn test_abi_is_apple() {
        assert!(Abi::Apple.is_apple());
        assert!(Abi::AppleSimulator.is_apple());
        assert!(!Abi::Gnu.is_apple());
    }

    #[test]
    fn test_abi_long_width() {
        assert_eq!(Abi::GnuIlp32.long_width(), 32);
        assert_eq!(Abi::Gnu.long_width(), 64);
    }

    #[test]
    fn test_abi_display() {
        assert_eq!(Abi::Msvc.to_string(), "msvc");
        assert_eq!(Abi::GnuLlvm.to_string(), "gnullvm");
    }
}