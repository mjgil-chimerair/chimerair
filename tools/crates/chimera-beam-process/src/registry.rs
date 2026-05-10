//! BEAM process registry.
//!
//! Named process registration via the `register/2` and `whereis/1` BIFs.
//! Registry entries survive process death (until explicitly unregistered).

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::sync::RwLock;

/// Key for registry entries (atoms are interned strings).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RegistrationKey {
    /// Atom name.
    Atom(String),
    /// PID (for aliasing).
    Pid(u64),
}

impl RegistrationKey {
    /// Create from a string.
    pub fn from_str(name: &str) -> Self {
        RegistrationKey::Atom(name.to_string())
    }

    /// Get atom name if this is an atom key.
    pub fn as_name(&self) -> Option<&str> {
        match self {
            RegistrationKey::Atom(s) => Some(s),
            _ => None,
        }
    }
}

impl From<String> for RegistrationKey {
    fn from(s: String) -> Self {
        RegistrationKey::Atom(s)
    }
}

impl From<&str> for RegistrationKey {
    fn from(s: &str) -> Self {
        RegistrationKey::Atom(s.to_string())
    }
}

/// A registry entry mapping name to PID.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registration {
    /// The key (name or alias).
    pub key: RegistrationKey,
    /// The registered PID.
    pub pid: u64,
    /// Whether this is a protected registration.
    pub protected: bool,
    /// When registered.
    pub registered_at: u64,
}

impl Registration {
    /// Create a new registration.
    pub fn new(key: RegistrationKey, pid: u64, protected: bool, registered_at: u64) -> Self {
        Registration {
            key,
            pid,
            protected,
            registered_at,
        }
    }

    /// Create a named registration.
    pub fn named(name: impl Into<String>, pid: u64, protected: bool, registered_at: u64) -> Self {
        Registration::new(
            RegistrationKey::from_str(&name.into()),
            pid,
            protected,
            registered_at,
        )
    }
}

/// Process registry for named process lookup.
#[derive(Debug, Default)]
pub struct ProcessRegistry {
    /// Name-to-PID mappings.
    entries: std::sync::RwLock<indexmap::IndexMap<String, Registration>>,
}

impl ProcessRegistry {
    /// Create a new registry.
    pub fn new() -> Self {
        ProcessRegistry {
            entries: RwLock::new(IndexMap::new()),
        }
    }

    /// Register a process under a name.
    ///
    /// Returns `Ok(())` on success, or error if name is already taken.
    pub fn register(
        &self,
        name: &str,
        pid: u64,
        protected: bool,
        timestamp: u64,
    ) -> Result<(), RegistryError> {
        let mut entries = self.entries.write().map_err(|_| RegistryError::Poisoned)?;

        // Check if already registered
        if entries.contains_key(name) {
            return Err(RegistryError::AlreadyRegistered(name.to_string()));
        }

        entries.insert(
            name.to_string(),
            Registration::named(name, pid, protected, timestamp),
        );
        Ok(())
    }

    /// Unregister a name.
    ///
    /// Returns `true` if the name was registered, `false` otherwise.
    pub fn unregister(&self, name: &str) -> bool {
        let mut entries = match self.entries.write() {
            Ok(e) => e,
            Err(_) => return false,
        };
        entries.swap_remove(name).is_some()
    }

    /// Look up a name.
    ///
    /// Returns the PID if found, `None` otherwise.
    pub fn whereis(&self, name: &str) -> Option<u64> {
        let entries = match self.entries.read() {
            Ok(e) => e,
            Err(_) => return None,
        };
        entries.get(name).map(|r| r.pid)
    }

    /// Check if a name is registered.
    pub fn is_registered(&self, name: &str) -> bool {
        let entries = match self.entries.read() {
            Ok(e) => e,
            Err(_) => return false,
        };
        entries.contains_key(name)
    }

    /// Get all registered names.
    pub fn registered_names(&self) -> Vec<String> {
        let entries = match self.entries.read() {
            Ok(e) => e,
            Err(_) => return Vec::new(),
        };
        entries.keys().cloned().collect()
    }

    /// Get all registrations.
    pub fn all_registrations(&self) -> Vec<Registration> {
        let entries = match self.entries.read() {
            Ok(e) => e,
            Err(_) => return Vec::new(),
        };
        entries.values().cloned().collect()
    }

    /// Get the registration for a name, if any.
    pub fn get(&self, name: &str) -> Option<Registration> {
        let entries = match self.entries.read() {
            Ok(e) => e,
            Err(_) => return None,
        };
        entries.get(name).cloned()
    }

    /// Unregister all entries for a given PID.
    ///
    /// Returns the number of entries removed.
    pub fn unregister_pid(&self, pid: u64) -> usize {
        let mut entries = match self.entries.write() {
            Ok(e) => e,
            Err(_) => return 0,
        };

        let to_remove: Vec<String> = entries
            .iter()
            .filter(|(_, r)| r.pid == pid)
            .map(|(name, _)| name.clone())
            .collect();

        for name in &to_remove {
            entries.swap_remove(name);
        }

        to_remove.len()
    }

    /// Number of registrations.
    pub fn len(&self) -> usize {
        let entries = match self.entries.read() {
            Ok(e) => e,
            Err(_) => return 0,
        };
        entries.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Registry errors.
#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("name '{0}' is already registered")]
    AlreadyRegistered(String),
    #[error("registry is poisoned")]
    Poisoned,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_whereis() {
        let registry = ProcessRegistry::new();

        assert!(registry.register("my_process", 100, false, 1000).is_ok());
        assert_eq!(registry.whereis("my_process"), Some(100));
        assert!(registry.is_registered("my_process"));
    }

    #[test]
    fn test_register_duplicate() {
        let registry = ProcessRegistry::new();

        assert!(registry.register("dup", 100, false, 1000).is_ok());
        let result = registry.register("dup", 200, false, 1001);
        assert!(result.is_err());
        assert_eq!(registry.whereis("dup"), Some(100)); // Original still there
    }

    #[test]
    fn test_unregister() {
        let registry = ProcessRegistry::new();

        registry.register("test", 100, false, 1000);
        assert!(registry.unregister("test"));
        assert!(!registry.is_registered("test"));
        assert_eq!(registry.whereis("test"), None);

        // Unregister non-existent returns false
        assert!(!registry.unregister("non_existent"));
    }

    #[test]
    fn test_unregister_pid() {
        let registry = ProcessRegistry::new();

        registry.register("proc1", 100, false, 1000);
        registry.register("proc2", 100, false, 1001);
        registry.register("proc3", 200, false, 1002);

        let removed = registry.unregister_pid(100);
        assert_eq!(removed, 2);
        assert!(!registry.is_registered("proc1"));
        assert!(!registry.is_registered("proc2"));
        assert!(registry.is_registered("proc3"));
    }

    #[test]
    fn test_all_registrations() {
        let registry = ProcessRegistry::new();

        registry.register("a", 1, false, 1000);
        registry.register("b", 2, true, 1001);

        let regs = registry.all_registrations();
        assert_eq!(regs.len(), 2);
        assert!(regs
            .iter()
            .any(|r| r.key.as_name() == Some("a") && r.pid == 1));
        assert!(regs
            .iter()
            .any(|r| r.key.as_name() == Some("b") && r.pid == 2 && r.protected));
    }

    #[test]
    fn test_registry_len() {
        let registry = ProcessRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);

        registry.register("p1", 1, false, 1000);
        registry.register("p2", 2, false, 1001);
        assert!(!registry.is_empty());
        assert_eq!(registry.len(), 2);
    }

    #[test]
    fn test_registration_key() {
        let key: RegistrationKey = "test_name".into();
        assert!(matches!(key, RegistrationKey::Atom(_)));

        let key2: RegistrationKey = "another".to_string().into();
        assert!(matches!(key2, RegistrationKey::Atom(_)));
    }
}
