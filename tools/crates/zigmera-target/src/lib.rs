//! Zigmera target triple, CPU, ABI, pointer width, and cross-target compatibility.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Target parsing errors
#[derive(Debug, Error)]
pub enum TargetError {
    #[error("invalid target triple: {0}")]
    InvalidTriple(String),
    #[error("unsupported OS: {0}")]
    UnsupportedOs(String),
    #[error("unsupported arch: {0}")]
    UnsupportedArch(String),
}

/// OS enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Os {
    Linux,
    Windows,
    Macos,
    FreeBsd,
    NetBsd,
    Fuchsia,
    Wasi,
    Bare,
}

impl Os {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "linux" | "glinux" => Some(Self::Linux),
            "windows" | "win32" => Some(Self::Windows),
            "macos" | "darwin" | "macosx" => Some(Self::Macos),
            "freebsd" | "freebsd" => Some(Self::FreeBsd),
            "netbsd" => Some(Self::NetBsd),
            "fuchsia" => Some(Self::Fuchsia),
            "wasi" => Some(Self::Wasi),
            "bare" | "none" | "standalone" => Some(Self::Bare),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Linux => "linux",
            Self::Windows => "windows",
            Self::Macos => "macos",
            Self::FreeBsd => "freebsd",
            Self::NetBsd => "netbsd",
            Self::Fuchsia => "fuchsia",
            Self::Wasi => "wasi",
            Self::Bare => "bare",
        }
    }
}

/// CPU architecture enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Arch {
    X86_64,
    Aarch64,
    Arm,
    Riscv64,
    Riscv32,
    Wasm32,
    Wasm64,
    Sparc64,
    PowerPc64,
}

impl Arch {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "x86_64" | "x64" | "amd64" => Some(Self::X86_64),
            "aarch64" | "arm64" | "aarch64_be" => Some(Self::Aarch64),
            "arm" | "armv7" => Some(Self::Arm),
            "riscv64" | "riscv64gc" => Some(Self::Riscv64),
            "riscv32" | "riscv32i" => Some(Self::Riscv32),
            "wasm32" | "wasm" => Some(Self::Wasm32),
            "wasm64" => Some(Self::Wasm64),
            "sparc64" => Some(Self::Sparc64),
            "powerpc64" | "ppc64" => Some(Self::PowerPc64),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::X86_64 => "x86_64",
            Self::Aarch64 => "aarch64",
            Self::Arm => "arm",
            Self::Riscv64 => "riscv64",
            Self::Riscv32 => "riscv32",
            Self::Wasm32 => "wasm32",
            Self::Wasm64 => "wasm64",
            Self::Sparc64 => "sparc64",
            Self::PowerPc64 => "powerpc64",
        }
    }

    /// Default pointer width for this architecture
    pub fn pointer_width(&self) -> u32 {
        match self {
            Self::X86_64
            | Self::Aarch64
            | Self::Riscv64
            | Self::Wasm64
            | Self::Sparc64
            | Self::PowerPc64 => 64,
            Self::Arm | Self::Riscv32 | Self::Wasm32 => 32,
        }
    }

    /// Default endianness for this architecture
    pub fn endianness(&self) -> Endianness {
        match self {
            Self::Wasm32 | Self::Wasm64 => Endianness::Little,
            Self::Sparc64 | Self::PowerPc64 => Endianness::Big,
            _ => Endianness::Little,
        }
    }
}

/// Endianness
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Endianness {
    Little,
    Big,
}

impl Endianness {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Little => "little",
            Self::Big => "big",
        }
    }
}

/// ABI enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Abi {
    StdC,
    GnuEabi,
    GnuAbi,
    Musl,
    Msvc,
    Eabi,
    GnuIlp32,
    MuslIlp32,
    X32,
    Wasm,
}

impl Abi {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "stdc" | "c" | "gnu" => Some(Self::StdC),
            "gnueabi" | "gnu_eabi" => Some(Self::GnuEabi),
            "gnuabi" | "gnu_abi" => Some(Self::GnuAbi),
            "musl" => Some(Self::Musl),
            "msvc" => Some(Self::Msvc),
            "eabi" => Some(Self::Eabi),
            "gnuilp32" | "gnu_ilp32" => Some(Self::GnuIlp32),
            "muslilp32" | "musl_ilp32" => Some(Self::MuslIlp32),
            "x32" | "x32abi" => Some(Self::X32),
            "wasm" | "wasm32" => Some(Self::Wasm),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::StdC => "stdc",
            Self::GnuEabi => "gnueabi",
            Self::GnuAbi => "gnuabi",
            Self::Musl => "musl",
            Self::Msvc => "msvc",
            Self::Eabi => "eabi",
            Self::GnuIlp32 => "gnuilp32",
            Self::MuslIlp32 => "muslilp32",
            Self::X32 => "x32",
            Self::Wasm => "wasm",
        }
    }
}

/// Backend selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Backend {
    Llvm,
    CBE,
    Wasm,
}

impl Backend {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "llvm" | "llvm_ir" => Some(Self::Llvm),
            "cbe" | "c_backend" => Some(Self::CBE),
            "wasm" | "wasm_backend" => Some(Self::Wasm),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Llvm => "llvm",
            Self::CBE => "cbe",
            Self::Wasm => "wasm",
        }
    }
}

/// Target triple representation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TargetTriple {
    pub arch: Arch,
    pub os: Os,
    pub abi: Abi,
    pub backend: Backend,
}

impl TargetTriple {
    /// Parse a target triple string (e.g., "x86_64-linux-gnu-llvm")
    pub fn parse(triple: &str) -> Result<Self, TargetError> {
        let parts: Vec<&str> = triple.split('-').collect();
        if parts.len() < 3 {
            return Err(TargetError::InvalidTriple(triple.to_string()));
        }

        let arch = Arch::parse(parts[0])
            .ok_or_else(|| TargetError::UnsupportedArch(parts[0].to_string()))?;
        let os =
            Os::parse(parts[1]).ok_or_else(|| TargetError::UnsupportedOs(parts[1].to_string()))?;
        let abi = Abi::parse(parts[2]).unwrap_or(Abi::StdC);
        let backend = parts
            .get(3)
            .and_then(|b| Backend::parse(*b))
            .unwrap_or(Backend::Llvm);

        Ok(Self {
            arch,
            os,
            abi,
            backend,
        })
    }

    /// Get the canonical triple string
    pub fn as_str(&self) -> String {
        format!(
            "{}-{}-{}-{}",
            self.arch.as_str(),
            self.os.as_str(),
            self.abi.as_str(),
            self.backend.as_str()
        )
    }

    /// Get pointer width for this target
    pub fn pointer_width(&self) -> u32 {
        self.arch.pointer_width()
    }

    /// Get endianness for this target
    pub fn endianness(&self) -> Endianness {
        self.arch.endianness()
    }

    /// Check if this target is compatible with another
    pub fn is_compatible_with(&self, other: &TargetTriple) -> bool {
        self.arch == other.arch && self.os == other.os
    }

    /// Check if this target supports the same ABI as another
    pub fn same_abi(&self, other: &TargetTriple) -> bool {
        self.abi == other.abi && self.pointer_width() == other.pointer_width()
    }
}

impl std::fmt::Display for TargetTriple {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// CPU feature flags
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CpuFeatures {
    flags: Vec<String>,
}

impl CpuFeatures {
    pub fn new() -> Self {
        Self { flags: Vec::new() }
    }

    pub fn with_feature(mut self, feature: &str) -> Self {
        self.flags.push(feature.to_string());
        self
    }

    pub fn has_feature(&self, feature: &str) -> bool {
        self.flags.iter().any(|f| f == feature)
    }

    pub fn features(&self) -> &[String] {
        &self.flags
    }
}

/// Target data for a specific compilation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetData {
    pub triple: TargetTriple,
    pub cpu_features: CpuFeatures,
    pub cpu_name: String,
}

impl TargetData {
    pub fn new(triple: TargetTriple) -> Self {
        Self {
            triple,
            cpu_features: CpuFeatures::new(),
            cpu_name: "generic".to_string(),
        }
    }

    pub fn with_cpu(mut self, cpu_name: &str) -> Self {
        self.cpu_name = cpu_name.to_string();
        self
    }

    pub fn with_feature(mut self, feature: &str) -> Self {
        self.cpu_features.flags.push(feature.to_string());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arch_parsing() {
        assert_eq!(Arch::parse("x86_64"), Some(Arch::X86_64));
        assert_eq!(Arch::parse("aarch64"), Some(Arch::Aarch64));
        assert_eq!(Arch::parse("arm"), Some(Arch::Arm));
        assert_eq!(Arch::parse("unknown"), None);
    }

    #[test]
    fn test_arch_pointer_width() {
        assert_eq!(Arch::X86_64.pointer_width(), 64);
        assert_eq!(Arch::Arm.pointer_width(), 32);
    }

    #[test]
    fn test_target_triple_parse() {
        let triple = TargetTriple::parse("x86_64-linux-gnu-llvm").unwrap();
        assert_eq!(triple.arch, Arch::X86_64);
        assert_eq!(triple.os, Os::Linux);
        assert_eq!(triple.abi, Abi::StdC);
        assert_eq!(triple.backend, Backend::Llvm);
    }

    #[test]
    fn test_target_triple_parse_aarch64() {
        let triple = TargetTriple::parse("aarch64-linux-gnu").unwrap();
        assert_eq!(triple.arch, Arch::Aarch64);
        assert_eq!(triple.os, Os::Linux);
    }

    #[test]
    fn test_target_triple_display() {
        let triple = TargetTriple::parse("x86_64-linux-musl-llvm").unwrap();
        assert_eq!(triple.to_string(), "x86_64-linux-musl-llvm");
    }

    #[test]
    fn test_target_compatibility() {
        let t1 = TargetTriple::parse("x86_64-linux-gnu-llvm").unwrap();
        let t2 = TargetTriple::parse("x86_64-linux-musl-llvm").unwrap();
        let t3 = TargetTriple::parse("aarch64-linux-gnu-llvm").unwrap();

        assert!(t1.is_compatible_with(&t2));
        assert!(!t1.is_compatible_with(&t3));
    }

    #[test]
    fn test_abi_parsing() {
        assert_eq!(Abi::parse("gnu"), Some(Abi::StdC));
        assert_eq!(Abi::parse("musl"), Some(Abi::Musl));
        assert_eq!(Abi::parse("msvc"), Some(Abi::Msvc));
    }

    #[test]
    fn test_backend_parsing() {
        assert_eq!(Backend::parse("llvm"), Some(Backend::Llvm));
        assert_eq!(Backend::parse("cbe"), Some(Backend::CBE));
        assert_eq!(Backend::parse("wasm"), Some(Backend::Wasm));
    }

    #[test]
    fn test_cpu_features() {
        let features = CpuFeatures::new()
            .with_feature("sse4.2")
            .with_feature("avx");
        assert!(features.has_feature("sse4.2"));
        assert!(!features.has_feature("sse3"));
    }

    #[test]
    fn test_target_data() {
        let target = TargetData::new(TargetTriple::parse("x86_64-linux-gnu-llvm").unwrap())
            .with_cpu("haswell")
            .with_feature("avx2");
        assert_eq!(target.cpu_name, "haswell");
        assert!(target.cpu_features.has_feature("avx2"));
    }

    #[test]
    fn test_target_error() {
        let result = TargetTriple::parse("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_endianness() {
        assert_eq!(Arch::X86_64.endianness(), Endianness::Little);
        assert_eq!(Arch::Sparc64.endianness(), Endianness::Big);
    }
}
