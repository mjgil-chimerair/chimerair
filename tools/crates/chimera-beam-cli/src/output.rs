//! Output formatting and writing for CLI.
//!
//! Handles output in various formats (JSON, text, MLIR).

use serde_json::Value;
use std::io::{self, Write};
use std::path::PathBuf;
use std::str::FromStr;

/// Output format for CLI.
#[derive(Debug, Clone, Copy, Default)]
pub enum OutputFormat {
    /// JSON format.
    #[default]
    Json,
    /// Plain text format.
    Text,
    /// MLIR format.
    Mlir,
    /// Binary format.
    Binary,
}

impl OutputFormat {
    /// Parse from string.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "json" => Some(OutputFormat::Json),
            "text" | "txt" => Some(OutputFormat::Text),
            "mlir" => Some(OutputFormat::Mlir),
            "binary" | "bin" => Some(OutputFormat::Binary),
            _ => None,
        }
    }
}

impl FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(OutputFormat::Json),
            "text" | "txt" => Ok(OutputFormat::Text),
            "mlir" => Ok(OutputFormat::Mlir),
            "binary" | "bin" => Ok(OutputFormat::Binary),
            _ => Err(format!("Unknown format: {}", s)),
        }
    }
}

/// Output writer for CLI.
pub struct OutputWriter {
    path: Option<PathBuf>,
    format: OutputFormat,
    writer: Box<dyn Write>,
}

impl OutputWriter {
    /// Create a new output writer.
    pub fn new(path: &Option<PathBuf>, format: OutputFormat) -> anyhow::Result<Self> {
        let writer: Box<dyn Write> = match path {
            Some(p) => {
                let file = std::fs::File::create(p)
                    .map_err(|e| anyhow::anyhow!("Failed to create output file: {}", e))?;
                Box::new(file)
            }
            None => Box::new(io::stdout()),
        };

        Ok(OutputWriter {
            path: path.clone(),
            format,
            writer,
        })
    }

    /// Write JSON value.
    pub fn write_json(&mut self, value: &serde_json::Map<String, Value>) -> anyhow::Result<()> {
        match self.format {
            OutputFormat::Json => {
                let json = serde_json::to_string_pretty(value)?;
                writeln!(self.writer, "{}", json)?;
            }
            OutputFormat::Text => {
                // Convert JSON to human-readable text
                for (key, val) in value {
                    writeln!(self.writer, "{}: {}", key, val)?;
                }
            }
            OutputFormat::Mlir => {
                // Write as MLIR comment
                writeln!(self.writer, "// JSON: {}", serde_json::to_string(value)?)?;
            }
            OutputFormat::Binary => {
                anyhow::bail!("Binary output not yet supported");
            }
        }
        Ok(())
    }

    /// Write a string.
    pub fn write_str(&mut self, s: &str) -> anyhow::Result<()> {
        writeln!(self.writer, "{}", s)?;
        Ok(())
    }

    /// Flush the writer.
    pub fn flush(&mut self) -> anyhow::Result<()> {
        self.writer.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_format_parse() {
        assert!(matches!(
            OutputFormat::parse("json"),
            Some(OutputFormat::Json)
        ));
        assert!(matches!(
            OutputFormat::parse("text"),
            Some(OutputFormat::Text)
        ));
        assert!(matches!(
            OutputFormat::parse("mlir"),
            Some(OutputFormat::Mlir)
        ));
        assert!(matches!(
            OutputFormat::parse("binary"),
            Some(OutputFormat::Binary)
        ));
        assert!(OutputFormat::parse("invalid").is_none());
    }

    #[test]
    fn test_output_writer_stdout() {
        let mut writer = OutputWriter::new(&None, OutputFormat::Json).unwrap();
        let mut map = serde_json::Map::new();
        map.insert(
            "key".to_string(),
            serde_json::Value::String("value".to_string()),
        );

        writer.write_json(&map).unwrap();
        writer.flush().unwrap();
    }
}
