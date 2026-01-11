//! Configuration handling for datadiff

use std::path::PathBuf;

/// Output format for diff results
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum OutputFormat {
    #[default]
    Terminal,
    Json,
    Html,
    Unified,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "terminal" => Ok(OutputFormat::Terminal),
            "json" => Ok(OutputFormat::Json),
            "html" => Ok(OutputFormat::Html),
            "unified" => Ok(OutputFormat::Unified),
            _ => Err(format!("Unknown output format: {}", s)),
        }
    }
}

/// Configuration for diff operations
#[derive(Debug, Clone)]
pub struct Config {
    /// Path to the old/original file
    pub old_file: PathBuf,
    /// Path to the new file
    pub new_file: PathBuf,
    /// Columns to use as primary key for row matching
    pub key_columns: Vec<String>,
    /// Output format
    pub output_format: OutputFormat,
    /// Ignore case when comparing string values
    pub ignore_case: bool,
    /// Tolerance for numeric comparisons
    pub numeric_tolerance: Option<f64>,
    /// Ignore leading/trailing whitespace in string values
    pub ignore_whitespace: bool,
    /// Columns to ignore in comparison
    pub ignore_columns: Vec<String>,
    /// Column to sort by before diffing (normalizes order)
    pub sort_by: Option<String>,
    /// For Excel files: which sheet to compare
    pub sheet_name: Option<String>,
    /// Only show statistics, not detailed changes
    pub stats_only: bool,
    /// Git diff driver mode
    pub git_driver_mode: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            old_file: PathBuf::new(),
            new_file: PathBuf::new(),
            key_columns: Vec::new(),
            output_format: OutputFormat::default(),
            ignore_case: false,
            numeric_tolerance: None,
            ignore_whitespace: false,
            ignore_columns: Vec::new(),
            sort_by: None,
            sheet_name: None,
            stats_only: false,
            git_driver_mode: false,
        }
    }
}

impl Config {
    /// Create a new Config with file paths
    pub fn new(old_file: PathBuf, new_file: PathBuf) -> Self {
        Self {
            old_file,
            new_file,
            ..Default::default()
        }
    }

    /// Set key columns for row matching
    pub fn with_key_columns(mut self, keys: Vec<String>) -> Self {
        self.key_columns = keys;
        self
    }

    /// Set output format
    pub fn with_output_format(mut self, format: OutputFormat) -> Self {
        self.output_format = format;
        self
    }

    /// Enable case-insensitive comparison
    pub fn with_ignore_case(mut self, ignore: bool) -> Self {
        self.ignore_case = ignore;
        self
    }

    /// Set numeric tolerance for float comparisons
    pub fn with_numeric_tolerance(mut self, tolerance: f64) -> Self {
        self.numeric_tolerance = Some(tolerance);
        self
    }

    /// Enable whitespace-insensitive comparison
    pub fn with_ignore_whitespace(mut self, ignore: bool) -> Self {
        self.ignore_whitespace = ignore;
        self
    }

    /// Set columns to ignore
    pub fn with_ignore_columns(mut self, columns: Vec<String>) -> Self {
        self.ignore_columns = columns;
        self
    }

    /// Set sort column for normalization
    pub fn with_sort_by(mut self, column: String) -> Self {
        self.sort_by = Some(column);
        self
    }

    /// Set Excel sheet name
    pub fn with_sheet_name(mut self, name: String) -> Self {
        self.sheet_name = Some(name);
        self
    }

    /// Enable stats-only mode
    pub fn with_stats_only(mut self, stats_only: bool) -> Self {
        self.stats_only = stats_only;
        self
    }
}
