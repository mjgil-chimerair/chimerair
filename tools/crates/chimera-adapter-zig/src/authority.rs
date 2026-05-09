use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthorityLevel {
    Authoritative,
    Fixture,
    CacheScrape,
    Unavailable,
}

impl AuthorityLevel {
    pub fn is_authoritative(&self) -> bool {
        matches!(self, AuthorityLevel::Authoritative)
    }

    pub fn is_non_authoritative(&self) -> bool {
        !self.is_authoritative()
    }

    pub fn description(&self) -> &'static str {
        match self {
            AuthorityLevel::Authoritative => {
                "Output from the patched Zig compiler with verified semantic analysis"
            }
            AuthorityLevel::Fixture => {
                "Output from pre-recorded fixture files; not verified against current source"
            }
            AuthorityLevel::CacheScrape => {
                "Output extracted from Zig build cache; may be incomplete or stale"
            }
            AuthorityLevel::Unavailable => {
                "Output when patched Zig is unavailable; use with extreme caution"
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct AuthorityConfig {
    pub require_authoritative: bool,
    pub allow_fallback: bool,
}

impl AuthorityConfig {
    pub fn production() -> Self {
        Self {
            require_authoritative: true,
            allow_fallback: false,
        }
    }

    pub fn development() -> Self {
        Self {
            require_authoritative: false,
            allow_fallback: true,
        }
    }

    pub fn check(&self, level: AuthorityLevel) -> Result<(), AuthorityError> {
        if !self.require_authoritative {
            return Ok(());
        }
        if !level.is_authoritative() && !self.allow_fallback {
            return Err(AuthorityError::NonAuthoritativeMode {
                level,
                message: format!(
                    "Authority check failed: {:?} is not authoritative. \
                     Production builds require `Authoritative` mode.",
                    level
                ),
            });
        }
        Ok(())
    }
}

impl Default for AuthorityConfig {
    fn default() -> Self {
        Self::development()
    }
}

#[derive(Debug, Clone)]
pub enum AuthorityError {
    NonAuthoritativeMode {
        level: AuthorityLevel,
        message: String,
    },
}

impl std::fmt::Display for AuthorityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthorityError::NonAuthoritativeMode { message, .. } => write!(f, "{}", message),
        }
    }
}

impl std::error::Error for AuthorityError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_production_config_rejects_fallback() {
        let config = AuthorityConfig::production();
        assert!(config.check(AuthorityLevel::Authoritative).is_ok());
        assert!(config.check(AuthorityLevel::Fixture).is_err());
        assert!(config.check(AuthorityLevel::CacheScrape).is_err());
        assert!(config.check(AuthorityLevel::Unavailable).is_err());
    }

    #[test]
    fn test_development_config_allows_all() {
        let config = AuthorityConfig::development();
        assert!(config.check(AuthorityLevel::Authoritative).is_ok());
        assert!(config.check(AuthorityLevel::Fixture).is_ok());
        assert!(config.check(AuthorityLevel::CacheScrape).is_ok());
        assert!(config.check(AuthorityLevel::Unavailable).is_ok());
    }

    #[test]
    fn test_authority_level_classification() {
        assert!(AuthorityLevel::Authoritative.is_authoritative());
        assert!(!AuthorityLevel::Fixture.is_authoritative());
        assert!(!AuthorityLevel::CacheScrape.is_authoritative());
        assert!(!AuthorityLevel::Unavailable.is_authoritative());
    }

    #[test]
    fn test_authority_level_descriptions() {
        assert!(AuthorityLevel::Authoritative
            .description()
            .contains("patched"));
        assert!(AuthorityLevel::Fixture.description().contains("fixture"));
        assert!(AuthorityLevel::CacheScrape.description().contains("cache"));
        assert!(AuthorityLevel::Unavailable
            .description()
            .contains("unavailable"));
    }

    #[test]
    fn test_production_rejects_fallback_only() {
        let config = AuthorityConfig {
            require_authoritative: true,
            allow_fallback: false,
        };
        // With allow_fallback=false, Authoritative must pass
        assert!(config.check(AuthorityLevel::Authoritative).is_ok());
        // Non-authoritative always fails
        assert!(config.check(AuthorityLevel::Unavailable).is_err());
    }

    #[test]
    fn test_default_config_is_development() {
        let config = AuthorityConfig::default();
        assert!(!config.require_authoritative);
        assert!(config.allow_fallback);
    }

    #[test]
    fn test_authority_error_display() {
        let err = AuthorityError::NonAuthoritativeMode {
            level: AuthorityLevel::Fixture,
            message: "not authoritative".to_string(),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("not authoritative"));
    }
}
