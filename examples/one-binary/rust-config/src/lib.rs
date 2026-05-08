//! Chimera config parser example
//!
//! Rust side of the one-binary demo - parses config using only Chimera ABI types.

#![warn(unused)]

use std::collections::HashMap;

/// Configuration key-value pair
#[derive(Debug, Clone)]
pub struct ConfigEntry {
    pub key: String,
    pub value: String,
}

/// Parsed configuration
#[derive(Debug, Clone)]
pub struct Config {
    pub entries: HashMap<String, String>,
}

impl Config {
    /// Create a new empty config
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Parse config from text
    ///
    /// Format: key=value, one per line
    pub fn parse(text: &str) -> Result<Config, ConfigError> {
        let mut config = Config::new();
        for (line_num, line) in text.lines().enumerate() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Parse key=value
            if let Some(eq_pos) = line.find('=') {
                let key = line[..eq_pos].trim().to_string();
                let value = line[eq_pos + 1..].trim().to_string();

                if key.is_empty() {
                    return Err(ConfigError::ParseError(
                        format!("Empty key at line {}", line_num + 1)
                    ));
                }

                config.entries.insert(key, value);
            } else {
                return Err(ConfigError::ParseError(
                    format!("Missing '=' at line {}", line_num + 1)
                ));
            }
        }

        Ok(config)
    }

    /// Get a value by key
    pub fn get(&self, key: &str) -> Option<&String> {
        self.entries.get(key)
    }

    /// Check if a key exists
    pub fn contains_key(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    /// Number of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if config is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration parsing errors
#[derive(Debug, Clone)]
pub enum ConfigError {
    ParseError(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}

#[no_mangle]
pub extern "C" fn chimera_rust_count_config_entries(data: *const u8, len: usize) -> usize {
    if data.is_null() {
        return 0;
    }

    let bytes = unsafe { std::slice::from_raw_parts(data, len) };
    let text = match std::str::from_utf8(bytes) {
        Ok(text) => text,
        Err(_) => return 0,
    };

    match Config::parse(text) {
        Ok(config) => config.len(),
        Err(_) => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty() {
        let config = Config::parse("").unwrap();
        assert!(config.is_empty());
    }

    #[test]
    fn test_parse_single_entry() {
        let config = Config::parse("key=value").unwrap();
        assert_eq!(config.len(), 1);
        assert_eq!(config.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_parse_multiple_entries() {
        let text = "key1=value1\nkey2=value2\nkey3=value3";
        let config = Config::parse(text).unwrap();
        assert_eq!(config.len(), 3);
        assert_eq!(config.get("key1"), Some(&"value1".to_string()));
        assert_eq!(config.get("key2"), Some(&"value2".to_string()));
        assert_eq!(config.get("key3"), Some(&"value3".to_string()));
    }

    #[test]
    fn test_parse_with_comments() {
        let text = "# comment\nkey=value\n# another comment";
        let config = Config::parse(text).unwrap();
        assert_eq!(config.len(), 1);
    }

    #[test]
    fn test_parse_with_whitespace() {
        let text = "  key1  =  value1  \n  key2  =  value2  ";
        let config = Config::parse(text).unwrap();
        assert_eq!(config.len(), 2);
        assert_eq!(config.get("key1"), Some(&"value1".to_string()));
    }

    #[test]
    fn test_parse_missing_equals() {
        let result = Config::parse("key without equals");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_empty_key() {
        let result = Config::parse("=value");
        assert!(result.is_err());
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(config.is_empty());
    }

    #[test]
    fn test_config_contains_key() {
        let config = Config::parse("key=value").unwrap();
        assert!(config.contains_key("key"));
        assert!(!config.contains_key("nonexistent"));
    }

    #[test]
    fn test_chimera_rust_count_config_entries() {
        let text = b"key1=value1\nkey2=value2\n";
        let count = chimera_rust_count_config_entries(text.as_ptr(), text.len());
        assert_eq!(count, 2);
    }
}
