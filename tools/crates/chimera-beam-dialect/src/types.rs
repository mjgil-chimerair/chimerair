//! BEAM dialect types.
//!
//! Defines the type system for BEAM semantics in MLIR.

use serde::{Deserialize, Serialize};

/// BEAM type kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BeamTypeKind {
    /// Process type (lightweight actor).
    Process,
    /// Port type for external I/O.
    Port,
    /// Reference type for uniquified references.
    Reference,
    /// Atom type (interned string).
    Atom,
    /// Tuple type (fixed-size heterogeneous).
    Tuple,
    /// List type (cons-cell, nil-terminated).
    List,
    /// Binary type (heap binary or sub-binary slice).
    Binary,
    /// Closure type (code + environment).
    Closure,
    /// PID type (process identifier).
    Pid,
    /// Map type (key-value dictionary).
    Map,
    /// Catch marker (for exception handling).
    Catch,
    /// NoReturn type (unreachable/always throws).
    NoReturn,
}

impl Default for BeamTypeKind {
    fn default() -> Self {
        BeamTypeKind::Atom
    }
}

/// A BEAM type in MLIR.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeamType {
    /// Kind of BEAM type.
    pub kind: BeamTypeKind,
    /// For tuple: number of elements.
    pub tuple_len: Option<usize>,
    /// For list: element type (None means any).
    pub element_type: Option<Box<BeamType>>,
    /// For binary: size in bytes (None means variable).
    pub binary_size: Option<usize>,
}

impl BeamType {
    /// Create a process type.
    pub fn process() -> Self {
        BeamType {
            kind: BeamTypeKind::Process,
            tuple_len: None,
            element_type: None,
            binary_size: None,
        }
    }

    /// Create a pid type.
    pub fn pid() -> Self {
        BeamType {
            kind: BeamTypeKind::Pid,
            tuple_len: None,
            element_type: None,
            binary_size: None,
        }
    }

    /// Create an atom type.
    pub fn atom() -> Self {
        BeamType {
            kind: BeamTypeKind::Atom,
            tuple_len: None,
            element_type: None,
            binary_size: None,
        }
    }

    /// Create a tuple type.
    pub fn tuple(len: usize) -> Self {
        BeamType {
            kind: BeamTypeKind::Tuple,
            tuple_len: Some(len),
            element_type: None,
            binary_size: None,
        }
    }

    /// Create a list type.
    pub fn list(elem: Option<BeamType>) -> Self {
        BeamType {
            kind: BeamTypeKind::List,
            tuple_len: None,
            element_type: elem.map(Box::new),
            binary_size: None,
        }
    }

    /// Create a binary type.
    pub fn binary(size: Option<usize>) -> Self {
        BeamType {
            kind: BeamTypeKind::Binary,
            tuple_len: None,
            element_type: None,
            binary_size: size,
        }
    }

    /// Create a closure type.
    pub fn closure() -> Self {
        BeamType {
            kind: BeamTypeKind::Closure,
            tuple_len: None,
            element_type: None,
            binary_size: None,
        }
    }

    /// Create a reference type.
    pub fn reference() -> Self {
        BeamType {
            kind: BeamTypeKind::Reference,
            tuple_len: None,
            element_type: None,
            binary_size: None,
        }
    }

    /// Create a port type.
    pub fn port() -> Self {
        BeamType {
            kind: BeamTypeKind::Port,
            tuple_len: None,
            element_type: None,
            binary_size: None,
        }
    }

    /// Create a map type.
    pub fn map() -> Self {
        BeamType {
            kind: BeamTypeKind::Map,
            tuple_len: None,
            element_type: None,
            binary_size: None,
        }
    }

    /// Create a catch type.
    pub fn catch() -> Self {
        BeamType {
            kind: BeamTypeKind::Catch,
            tuple_len: None,
            element_type: None,
            binary_size: None,
        }
    }

    /// Create a noreturn type.
    pub fn noreturn() -> Self {
        BeamType {
            kind: BeamTypeKind::NoReturn,
            tuple_len: None,
            element_type: None,
            binary_size: None,
        }
    }

    /// Get the MLIR type name.
    pub fn type_name(&self) -> &'static str {
        match self.kind {
            BeamTypeKind::Process => "beam.process",
            BeamTypeKind::Port => "beam.port",
            BeamTypeKind::Reference => "beam.reference",
            BeamTypeKind::Atom => "beam.atom",
            BeamTypeKind::Tuple => "beam.tuple",
            BeamTypeKind::List => "beam.list",
            BeamTypeKind::Binary => "beam.binary",
            BeamTypeKind::Closure => "beam.closure",
            BeamTypeKind::Pid => "beam.pid",
            BeamTypeKind::Map => "beam.map",
            BeamTypeKind::Catch => "beam.catch",
            BeamTypeKind::NoReturn => "beam.noreturn",
        }
    }
}

impl BeamType {
    /// Check if this is a singleton type (atom, pid, reference, port).
    pub fn is_singleton(&self) -> bool {
        matches!(
            self.kind,
            BeamTypeKind::Atom | BeamTypeKind::Pid | BeamTypeKind::Reference | BeamTypeKind::Port
        )
    }

    /// Check if this is a compound type (tuple, list, map, binary).
    pub fn is_compound(&self) -> bool {
        matches!(
            self.kind,
            BeamTypeKind::Tuple | BeamTypeKind::List | BeamTypeKind::Map | BeamTypeKind::Binary
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_type() {
        let t = BeamType::process();
        assert_eq!(t.kind, BeamTypeKind::Process);
        assert_eq!(t.type_name(), "beam.process");
    }

    #[test]
    fn test_pid_type() {
        let t = BeamType::pid();
        assert_eq!(t.kind, BeamTypeKind::Pid);
        assert!(t.is_singleton());
    }

    #[test]
    fn test_atom_type() {
        let t = BeamType::atom();
        assert_eq!(t.kind, BeamTypeKind::Atom);
        assert!(t.is_singleton());
    }

    #[test]
    fn test_tuple_type() {
        let t = BeamType::tuple(3);
        assert_eq!(t.kind, BeamTypeKind::Tuple);
        assert_eq!(t.tuple_len, Some(3));
        assert!(!t.is_singleton());
    }

    #[test]
    fn test_list_type() {
        let t = BeamType::list(Some(BeamType::atom()));
        assert_eq!(t.kind, BeamTypeKind::List);
        assert!(t.element_type.is_some());
    }

    #[test]
    fn test_binary_type() {
        let t = BeamType::binary(Some(1024));
        assert_eq!(t.kind, BeamTypeKind::Binary);
        assert_eq!(t.binary_size, Some(1024));
    }

    #[test]
    fn test_closure_type() {
        let t = BeamType::closure();
        assert_eq!(t.kind, BeamTypeKind::Closure);
    }

    #[test]
    fn test_reference_type() {
        let t = BeamType::reference();
        assert!(t.is_singleton());
    }

    #[test]
    fn test_port_type() {
        let t = BeamType::port();
        assert!(t.is_singleton());
    }

    #[test]
    fn test_map_type() {
        let t = BeamType::map();
        assert!(t.is_compound());
    }

    #[test]
    fn test_catch_type() {
        let t = BeamType::catch();
        assert_eq!(t.kind, BeamTypeKind::Catch);
    }

    #[test]
    fn test_noreturn_type() {
        let t = BeamType::noreturn();
        assert_eq!(t.kind, BeamTypeKind::NoReturn);
    }
}
