//! Domain tags for schema-specific hashing.

/// Domain tags used to namespace hash inputs per schema type.
/// This prevents cross-domain hash collisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DomainTag {
    Zsnap,
    Zdep,
    Zairpack,
    Zchmeta,
    Zchproof,
    Chobject,
    Chir,
    Fingerprint,
    CacheKey,
}

impl DomainTag {
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            DomainTag::Zsnap => b"zsnap-v1",
            DomainTag::Zdep => b"zdep-v1",
            DomainTag::Zairpack => b"zairpack-v1",
            DomainTag::Zchmeta => b"zchmeta-v1",
            DomainTag::Zchproof => b"zchproof-v1",
            DomainTag::Chobject => b"chobject-v1",
            DomainTag::Chir => b"chir-v1",
            DomainTag::Fingerprint => b"fingerprint-v1",
            DomainTag::CacheKey => b"cachekey-v1",
        }
    }
}

/// Schema domains for versioned hashing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaDomain {
    V0 { schema: &'static str },
    V1 { schema: &'static str },
}

impl SchemaDomain {
    pub fn new(schema: &'static str, version: u32) -> Self {
        match version {
            0 => Self::V0 { schema },
            _ => Self::V1 { schema },
        }
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        match self {
            Self::V0 { schema } => format!("{}-v0", schema).into_bytes(),
            Self::V1 { schema } => format!("{}-v1", schema).into_bytes(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_tag_as_bytes() {
        assert_eq!(DomainTag::Zsnap.as_bytes(), b"zsnap-v1");
        assert_eq!(DomainTag::Zdep.as_bytes(), b"zdep-v1");
    }

    #[test]
    fn test_schema_domain_v1() {
        let domain = SchemaDomain::new("zsnap", 1);
        assert_eq!(domain.as_bytes(), b"zsnap-v1");
    }

    #[test]
    fn test_schema_domain_v0() {
        let domain = SchemaDomain::new("zsnap", 0);
        assert_eq!(domain.as_bytes(), b"zsnap-v0");
    }
}
