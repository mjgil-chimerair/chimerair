//! Cross-target compatibility checks.

use super::{TargetTriple, CpuFeatures, Abi};

/// Compatibility result between two targets.
#[derive(Debug, Clone)]
pub struct TargetCompat {
    pub can_run_binary: bool,
    pub can_link: bool,
    pub reason: Option<String>,
}

impl TargetCompat {
    pub fn compatible() -> Self {
        Self {
            can_run_binary: true,
            can_link: true,
            reason: None,
        }
    }

    pub fn incompatible(reason: &str) -> Self {
        Self {
            can_run_binary: false,
            can_link: false,
            reason: Some(reason.to_string()),
        }
    }

    pub fn can_run_binary(&self) -> bool {
        self.can_run_binary
    }

    pub fn can_link(&self) -> bool {
        self.can_link
    }

    pub fn reason(&self) -> Option<&str> {
        self.reason.as_deref()
    }
}

/// Check if a binary compiled for `runner` can run on `host`.
pub fn can_run_on(host: &TargetTriple, runner: &TargetTriple) -> TargetCompat {
    if host.arch() != runner.arch() {
        return TargetCompat::incompatible("architecture mismatch");
    }
    if host.os() != runner.os() {
        return TargetCompat::incompatible("OS mismatch");
    }
    // ABI must match for binary compatibility
    if host.abi() != runner.abi() && host.abi() != runner.abi() {
        // Some ABI variations are compatible (e.g., gnu vs gnu with different ABIs)
        let compatible_abis = matches!(
            (host.abi(), runner.abi()),
            (Abi::Gnu, Abi::GnuLlvm) | (Abi::GnuLlvm, Abi::Gnu)
        );
        if !compatible_abis {
            return TargetCompat::incompatible("ABI mismatch");
        }
    }
    TargetCompat::compatible()
}

/// Check if two targets can be linked together.
pub fn can_link_to(host: &TargetTriple, other: &TargetTriple) -> TargetCompat {
    // For linking, OS and ABI must match, architecture must be compatible
    if host.os() != other.os() {
        return TargetCompat::incompatible("OS mismatch for linking");
    }
    if host.abi() != other.abi() {
        return TargetCompat::incompatible("ABI mismatch for linking");
    }
    // Architecture must match for native linking
    if host.arch() != other.arch() {
        return TargetCompat::incompatible("architecture mismatch for linking");
    }
    TargetCompat::compatible()
}

/// Check if `candidate` satisfies the requirements of `required`.
pub fn satisfies(required: &TargetTriple, candidate: &TargetTriple) -> bool {
    // Architecture must match exactly
    if required.arch() != candidate.arch() {
        return false;
    }
    // OS must match exactly
    if required.os() != candidate.os() {
        return false;
    }
    // ABI must be compatible
    match (required.abi(), candidate.abi()) {
        (Abi::None, _) => true,
        (a, b) if a == b => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::triple::Arch;
    use crate::triple::Os;
    use crate::abi::AbiFlavor;

    #[test]
    fn test_can_run_on_same() {
        let target = TargetTriple::new(Arch::X86_64, Os::Linux, AbiFlavor::Gnu);
        let compat = can_run_on(&target, &target);
        assert!(compat.can_run_binary());
    }

    #[test]
    fn test_can_run_on_arch_mismatch() {
        let host = TargetTriple::new(Arch::X86_64, Os::Linux, AbiFlavor::Gnu);
        let runner = TargetTriple::new(Arch::Aarch64, Os::Linux, AbiFlavor::Gnu);
        let compat = can_run_on(&host, &runner);
        assert!(!compat.can_run_binary());
        assert!(compat.reason().is_some());
    }

    #[test]
    fn test_can_run_on_os_mismatch() {
        let host = TargetTriple::new(Arch::X86_64, Os::Linux, AbiFlavor::Gnu);
        let runner = TargetTriple::new(Arch::X86_64, Os::Windows, AbiFlavor::Msvc);
        let compat = can_run_on(&host, &runner);
        assert!(!compat.can_run_binary());
    }

    #[test]
    fn test_can_link_to_same() {
        let target = TargetTriple::new(Arch::X86_64, Os::Linux, AbiFlavor::Gnu);
        let compat = can_link_to(&target, &target);
        assert!(compat.can_link());
    }

    #[test]
    fn test_satisfies_exact() {
        let required = TargetTriple::parse("x86_64-linux-gnu").unwrap();
        let candidate = TargetTriple::parse("x86_64-linux-gnu").unwrap();
        assert!(satisfies(&required, &candidate));
    }

    #[test]
    fn test_satisfies_abi_mismatch() {
        let required = TargetTriple::parse("x86_64-linux-gnu").unwrap();
        let candidate = TargetTriple::parse("x86_64-linux-musl").unwrap();
        assert!(!satisfies(&required, &candidate));
    }

    #[test]
    fn test_compat_incompatible() {
        let compat = TargetCompat::incompatible("arch mismatch");
        assert!(!compat.can_run_binary());
        assert!(!compat.can_link());
        assert_eq!(compat.reason(), Some("arch mismatch"));
    }

    #[test]
    fn test_compat_compatible() {
        let compat = TargetCompat::compatible();
        assert!(compat.can_run_binary());
        assert!(compat.can_link());
        assert!(compat.reason().is_none());
    }
}