//! Output formatting for diff results

mod html;
mod json;
mod terminal;
mod unified;

use std::io::Write;
use std::path::Path;

use anyhow::Result;

use crate::config::OutputFormat;
use crate::diff::DiffResult;
use crate::model::Table;

pub use html::HtmlOutput;
pub use json::JsonOutput;
pub use terminal::TerminalOutput;
pub use unified::UnifiedOutput;

/// Trait for output formatters
pub trait OutputFormatter {
    /// Render diff result to a writer
    fn render(
        &self,
        diff: &DiffResult,
        old_table: &Table,
        new_table: &Table,
        old_path: &Path,
        new_path: &Path,
        writer: &mut dyn Write,
    ) -> Result<()>;
}

/// Factory for creating output formatters
pub struct OutputFactory;

impl OutputFactory {
    /// Create an output formatter based on format type
    pub fn create(format: OutputFormat) -> Box<dyn OutputFormatter> {
        match format {
            OutputFormat::Terminal => Box::new(TerminalOutput::new()),
            OutputFormat::Json => Box::new(JsonOutput::new()),
            OutputFormat::Html => Box::new(HtmlOutput::new()),
            OutputFormat::Unified => Box::new(UnifiedOutput::new()),
        }
    }
}

/// Render diff result to stdout
pub fn render_to_stdout(
    diff: &DiffResult,
    old_table: &Table,
    new_table: &Table,
    old_path: &Path,
    new_path: &Path,
    format: OutputFormat,
) -> Result<()> {
    let formatter = OutputFactory::create(format);
    let mut stdout = std::io::stdout();
    formatter.render(diff, old_table, new_table, old_path, new_path, &mut stdout)
}
