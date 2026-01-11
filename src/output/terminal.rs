//! Colored terminal output

use std::io::Write;
use std::path::Path;

use anyhow::Result;
use termcolor::ColorChoice;

use crate::diff::{cell_diff::percentage_change, CellChange, DiffResult, SchemaChange};
use crate::model::{Row, Table};

use super::OutputFormatter;

/// Terminal output with colors
pub struct TerminalOutput {
    #[allow(dead_code)]
    color_choice: ColorChoice,
}

impl TerminalOutput {
    pub fn new() -> Self {
        Self {
            color_choice: ColorChoice::Auto,
        }
    }

    pub fn with_color_choice(color_choice: ColorChoice) -> Self {
        Self { color_choice }
    }

    fn write_header(&self, writer: &mut dyn Write, old_path: &Path, new_path: &Path) -> Result<()> {
        writeln!(writer, "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")?;
        writeln!(
            writer,
            " datadiff: {} → {}",
            old_path.display(),
            new_path.display()
        )?;
        writeln!(writer, "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━")?;
        writeln!(writer)?;
        Ok(())
    }

    fn write_schema_changes(&self, changes: &[SchemaChange], writer: &mut dyn Write) -> Result<()> {
        if changes.is_empty() {
            return Ok(());
        }

        writeln!(writer, "Schema Changes:")?;
        for change in changes {
            writeln!(writer, "  {}", change)?;
        }
        writeln!(writer)?;
        Ok(())
    }

    fn write_summary(&self, diff: &DiffResult, writer: &mut dyn Write) -> Result<()> {
        writeln!(
            writer,
            "Summary: +{} added, -{} removed, ~{} modified (out of {} → {} rows)",
            diff.stats.rows_added,
            diff.stats.rows_removed,
            diff.stats.rows_modified,
            diff.stats.old_row_count,
            diff.stats.new_row_count
        )?;
        writeln!(writer)?;
        Ok(())
    }

    fn write_added_rows(&self, diff: &DiffResult, table: &Table, writer: &mut dyn Write) -> Result<()> {
        let added: Vec<_> = diff.added_rows().collect();
        if added.is_empty() {
            return Ok(());
        }

        writeln!(writer, "Added Rows:")?;
        self.write_rows_table(&added, table, writer)?;
        writeln!(writer)?;
        Ok(())
    }

    fn write_removed_rows(&self, diff: &DiffResult, table: &Table, writer: &mut dyn Write) -> Result<()> {
        let removed: Vec<_> = diff.removed_rows().collect();
        if removed.is_empty() {
            return Ok(());
        }

        writeln!(writer, "Removed Rows:")?;
        self.write_rows_table(&removed, table, writer)?;
        writeln!(writer)?;
        Ok(())
    }

    fn write_rows_table(&self, rows: &[&Row], table: &Table, writer: &mut dyn Write) -> Result<()> {
        if rows.is_empty() {
            return Ok(());
        }

        // Build table data
        let headers: Vec<String> = table.columns.iter().map(|c| c.name.clone()).collect();
        
        let mut table_data: Vec<Vec<String>> = Vec::new();
        table_data.push(headers);
        
        for row in rows {
            let row_data: Vec<String> = row
                .cells
                .iter()
                .map(|c| format!("{}", c.display()))
                .collect();
            table_data.push(row_data);
        }

        // Use tabled for formatting
        let display = build_table(&table_data);
        writeln!(writer, "{}", display)?;
        Ok(())
    }

    fn write_modified_rows(&self, diff: &DiffResult, writer: &mut dyn Write) -> Result<()> {
        let modified: Vec<_> = diff.modified_rows().collect();
        if modified.is_empty() {
            return Ok(());
        }

        writeln!(writer, "Modified Rows:")?;
        for (old_row, _new_row, changes) in modified {
            writeln!(writer, "  {}:", old_row.key)?;
            for change in changes {
                self.write_cell_change(change, writer)?;
            }
        }
        writeln!(writer)?;
        Ok(())
    }

    fn write_cell_change(&self, change: &CellChange, writer: &mut dyn Write) -> Result<()> {
        let pct = percentage_change(&change.old_value, &change.new_value);
        let pct_str = pct
            .map(|p| format!(" ({:+.1}%)", p))
            .unwrap_or_default();

        writeln!(
            writer,
            "    {}: {} → {}{}",
            change.column,
            change.old_value.display(),
            change.new_value.display(),
            pct_str
        )?;
        Ok(())
    }
}

impl Default for TerminalOutput {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputFormatter for TerminalOutput {
    fn render(
        &self,
        diff: &DiffResult,
        old_table: &Table,
        new_table: &Table,
        old_path: &Path,
        new_path: &Path,
        writer: &mut dyn Write,
    ) -> Result<()> {
        self.write_header(writer, old_path, new_path)?;

        if !diff.has_changes() {
            writeln!(writer, "No differences found.")?;
            return Ok(());
        }

        self.write_schema_changes(&diff.schema_changes, writer)?;
        self.write_summary(diff, writer)?;
        self.write_added_rows(diff, new_table, writer)?;
        self.write_removed_rows(diff, old_table, writer)?;
        self.write_modified_rows(diff, writer)?;

        Ok(())
    }
}

/// Build a formatted table from data
fn build_table(data: &[Vec<String>]) -> String {
    if data.is_empty() || data[0].is_empty() {
        return String::new();
    }

    let col_count = data[0].len();
    
    // Build column-aligned output manually
    let mut col_widths: Vec<usize> = vec![0; col_count];
    for row in data {
        for (i, cell) in row.iter().enumerate() {
            if i < col_widths.len() {
                col_widths[i] = col_widths[i].max(cell.len());
            }
        }
    }

    let mut output = String::new();
    
    // Top border
    output.push('┌');
    for (i, width) in col_widths.iter().enumerate() {
        output.push_str(&"─".repeat(*width + 2));
        if i < col_widths.len() - 1 {
            output.push('┬');
        }
    }
    output.push_str("┐\n");

    // Header row
    if let Some(header) = data.first() {
        output.push('│');
        for (i, cell) in header.iter().enumerate() {
            let width = col_widths.get(i).copied().unwrap_or(0);
            output.push_str(&format!(" {:width$} │", cell, width = width));
        }
        output.push('\n');
    }

    // Header separator
    output.push('├');
    for (i, width) in col_widths.iter().enumerate() {
        output.push_str(&"─".repeat(*width + 2));
        if i < col_widths.len() - 1 {
            output.push('┼');
        }
    }
    output.push_str("┤\n");

    // Data rows
    for row in data.iter().skip(1) {
        output.push('│');
        for (i, cell) in row.iter().enumerate() {
            let width = col_widths.get(i).copied().unwrap_or(0);
            output.push_str(&format!(" {:width$} │", cell, width = width));
        }
        output.push('\n');
    }

    // Bottom border
    output.push('└');
    for (i, width) in col_widths.iter().enumerate() {
        output.push_str(&"─".repeat(*width + 2));
        if i < col_widths.len() - 1 {
            output.push('┴');
        }
    }
    output.push_str("┘\n");

    output
}
