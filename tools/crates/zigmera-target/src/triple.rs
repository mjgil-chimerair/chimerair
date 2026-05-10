//! Target triple parsing and representation.

use std::fmt;
use serde::{Serialize, Serializer};

/// A parsed target triple (arch-os-abi).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TargetTriple {
    arch: Arch,
    os: Os,
    abi: AbiFlavor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Arch {
    X86_64,
    Aarch64,
    Arm,
    Riscv64,
    Wasm32,
    Wasm64,
    Other(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Os {
    Linux,
    Windows,
    Macos,
    FreeBSD,
    NetBSD,
    None,
    Other(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AbiFlavor {
    Gnu,
    Musl,
    Msvc,
    Glibc,
    Apple,
    Eabihf,
    None,
    Other(String),
}

impl TargetTriple {
    pub fn new(arch: Arch, os: Os, abi: AbiFlavor) -> Self {
        Self { arch, os, abi }
    }

    pub fn parse(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() < 3 {
            return None;
        }
        let arch = match parts[0] {
            "x86_64" => Arch::X86_64,
            "aarch64" | "arm64" => Arch::Aarch64,
            "arm" => Arch::Arm,
            "riscv64" => Arch::Riscv64,
            "wasm32" => Arch::Wasm32,
            "wasm64" => Arch::Wasm64,
            other => Arch::Other(other.to_string()),
        };
        let os = match parts[1] {
            "linux" => Os::Linux,
            "windows" => Os::Windows,
            "macos" | "darwin" => Os::Macos,
            "freebsd" => Os::FreeBSD,
            "netbsd" => Os::NetBSD,
            "none" => Os::None,
            other => Os::Other(other.to_string()),
        };
        let abi = match parts[2] {
            "gnu" => AbiFlavor::Gnu,
            "musl" => AbiFlavor::Musl,
            "msvc" => AbiFlavor::Msvc,
            "glibc" => AbiFlavor::Glibc,
            "apple" => AbiFlavor::Apple,
            "eabihf" => AbiFlavor::Eabihf,
            "none" => AbiFlavor::None,
            other => AbiFlavor::Other(other.to_string()),
        };
        Some(Self { arch, os, abi })
    }

    pub fn arch(&self) -> Arch {
        self.arch
    }

    pub fn os(&self) -> Os {
        self.os
    }

    pub fn abi(&self) -> AbiFlavor {
        self.abi
    }

    pub fn pointer_width(&self) -> u32 {
        match self.arch {
            Arch::X86_64 | Arch::Aarch64 | Arch::Riscv64 => 64,
            Arch::Arm => 32,
            Arch::Wasm32 => 32,
            Arch::Wasm64 => 64,
            Arch::Other(_) => 64,
        }
    }

    pub fn endian(&self) -> Endian {
        match self.arch {
            Arch::Arm => Endian::Little,
            Arch::Other(_) => Endian::Little,
            _ => Endian::Little,
        }
    }

    pub fn is_apple(&self) -> bool {
        self.os == Os::Macos
    }

    pub fn is_windows(&self) -> bool {
        self.os == Os::Windows
    }

    pub fn is_linux(&self) -> bool {
        self.os == Os::Linux
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endian {
    Little,
    Big,
}

impl fmt::Display for TargetTriple {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let arch_str = match self.arch {
            Arch::X86_64 => "x86_64",
            Arch::Aarch64 => "aarch64",
            Arch::Arm => "arm",
            Arch::Riscv64 => "riscv64",
            Arch::Wasm32 => "wasm32",
            Arch::Wasm64 => "wasm64",
            Arch::Other(s) => &s,
        };
        let os_str = match self.os {
            Os::Linux => "linux",
            Os::Windows => "windows",
            Os::Macos => "macos",
            Os::FreeBSD => "freebsd",
            Os::NetBSD => "netbsd",
            Os::None => "none",
            Os::Other(s) => &s,
        };
        let abi_str = match self.abi {
            AbiFlavor::Gnu => "gnu",
            AbiFlavor::Musl => "musl",
            AbiFlavor::Msvc => "msvc",
            AbiFlavor::Glibc => "glibc",
            AbiFlavor::Apple => "apple",
            AbiFlavor::Eabihf => "eabihf",
            AbiFlavor::None => "none",
            AbiFlavor::Other(s) => &s,
        };
        write!(f, "{}-{}-{}", arch_str, os_str, abi_str)
    }
}

impl Serialize for TargetTriple {
    fn serialize<S>(&self, serializer: S) -> S::Result
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_x86_64_linux() {
        let triple = TargetTriple::parse("x86_64-linux-gnu").unwrap();
        assert!(matches!(triple.arch, Arch::X86_64));
        assert!(matches!(triple.os, Os::Linux));
        assert!(matches!(triple.abi, AbiFlavor::Gnu));
        assert_eq!(triple.pointer_width(), 64);
    }

    #[test]
    fn test_parse_aarch64_macos() {
        let triple = TargetTriple::parse("aarch64-macos-apple").unwrap();
        assert!(matches!(triple.arch, Arch::Aarch64));
        assert!(matches!(triple.os, Os::Macos));
        assert!(matches!(triple.abi, AbiFlavor::Apple));
    }

    #[test]
    fn test_parse_wasm32() {
        let triple = TargetTriple::parse("wasm32-wasm-none").unwrap();
        assert!(matches!(triple.arch, Arch::Wasm32));
        assert_eq!(triple.pointer_width(), 32);
    }

    #[test]
    fn test_triple_display() {
        let triple = TargetTriple::parse("x86_64-linux-gnu").unwrap();
        assert_eq!(triple.to_string(), "x86_64-linux-gnu");
    }

    #[test]
    fn test_endian() {
        let triple = TargetTriple::parse("aarch64-linux-gnu").unwrap();
        assert_eq!(triple.endian(), Endian::Little);
    }

    #[test]
    fn test_os_helpers() {
        let linux = TargetTriple::parse("x86_64-linux-gnu").unwrap();
        assert!(linux.is_linux());
        assert!(!linux.is_windows());
        assert!(!linux.is_apple());

        let windows = TargetTriple::parse("x86_64-windows-msvc").unwrap();
        assert!(windows.is_windows());
        assert!(!windows.is_linux());
    }
}