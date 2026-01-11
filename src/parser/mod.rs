//! Parser layer for reading various tabular data formats

mod csv;
mod excel;
mod json;
mod parquet;

use std::path::Path;

use anyhow::{bail, Result};

use crate::config::Config;
use crate::model::Table;

pub use self::csv::CsvParser;
pub use self::excel::ExcelParser;
pub use self::json::JsonParser;
pub use self::parquet::ParquetParser;

/// Trait for parsing tabular data files
pub trait Parser: Send + Sync {
    /// Parse a file and return a Table
    fn parse(&self, path: &Path, config: &Config) -> Result<Table>;

    /// Check if this parser can handle the given file extension
    fn supports_extension(&self, ext: &str) -> bool;
}

/// Factory for creating parsers based on file extension
pub struct ParserFactory {
    parsers: Vec<Box<dyn Parser>>,
}

impl Default for ParserFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl ParserFactory {
    /// Create a new parser factory with all supported parsers
    pub fn new() -> Self {
        Self {
            parsers: vec![
                Box::new(CsvParser),
                Box::new(ExcelParser),
                Box::new(ParquetParser),
                Box::new(JsonParser),
            ],
        }
    }

    /// Get a parser for the given file path
    pub fn get_parser(&self, path: &Path) -> Result<&dyn Parser> {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        for parser in &self.parsers {
            if parser.supports_extension(&ext) {
                return Ok(parser.as_ref());
            }
        }

        bail!(
            "Unsupported file format: {}",
            path.extension()
                .and_then(|e| e.to_str())
                .unwrap_or("unknown")
        )
    }

    /// Parse a file using the appropriate parser
    pub fn parse(&self, path: &Path, config: &Config) -> Result<Table> {
        let parser = self.get_parser(path)?;
        parser.parse(path, config)
    }
}

/// Detect file format from content (for files without extension)
pub fn detect_format(path: &Path) -> Option<&'static str> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let file = File::open(path).ok()?;
    let mut reader = BufReader::new(file);
    let mut buffer = [0u8; 8];
    let bytes_read = std::io::Read::read(&mut reader, &mut buffer).ok()?;

    if bytes_read < 4 {
        return None;
    }

    // Check for Parquet magic bytes
    if &buffer[0..4] == b"PAR1" {
        return Some("parquet");
    }

    // Check for Excel ZIP format (xlsx)
    if &buffer[0..4] == b"PK\x03\x04" {
        return Some("xlsx");
    }

    // Check for old Excel format (xls)
    if &buffer[0..4] == b"\xD0\xCF\x11\xE0" {
        return Some("xls");
    }

    // Try to detect JSON
    reader.seek_relative(-(bytes_read as i64)).ok()?;
    let mut line = String::new();
    reader.read_line(&mut line).ok()?;
    let trimmed = line.trim_start();
    if trimmed.starts_with('[') || trimmed.starts_with('{') {
        return Some("json");
    }

    // Default to CSV
    Some("csv")
}
