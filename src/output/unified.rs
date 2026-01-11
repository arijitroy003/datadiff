//! Git-style unified diff output

use std::io::Write;
use std::path::Path;

use anyhow::Result;

use crate::diff::{DiffResult, RowChange};
use crate::model::Table;

use super::OutputFormatter;

/// Unified diff output (Git-style)
pub struct UnifiedOutput {
    #[allow(dead_code)]
    context_lines: usize,
}

impl UnifiedOutput {
    pub fn new() -> Self {
        Self { context_lines: 3 }
    }

    pub fn with_context(context_lines: usize) -> Self {
        Self { context_lines }
    }
}

impl Default for UnifiedOutput {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputFormatter for UnifiedOutput {
    fn render(
        &self,
        diff: &DiffResult,
        old_table: &Table,
        new_table: &Table,
        old_path: &Path,
        new_path: &Path,
        writer: &mut dyn Write,
    ) -> Result<()> {
        // File headers
        writeln!(writer, "--- {}", old_path.display())?;
        writeln!(writer, "+++ {}", new_path.display())?;

        if !diff.has_changes() {
            return Ok(());
        }

        // Output header row if different
        let old_headers: Vec<_> = old_table.columns.iter().map(|c| c.name.as_str()).collect();
        let new_headers: Vec<_> = new_table.columns.iter().map(|c| c.name.as_str()).collect();

        if old_headers != new_headers {
            writeln!(writer, "@@ -1 +1 @@ header")?;
            writeln!(writer, "-{}", old_headers.join(","))?;
            writeln!(writer, "+{}", new_headers.join(","))?;
        }

        // Output changes
        for change in &diff.row_changes {
            match change {
                RowChange::Added { row, .. } => {
                    let cells: Vec<_> = row.cells.iter().map(|c| c.display().to_string()).collect();
                    writeln!(writer, "@@ +{} @@", row.source_line)?;
                    writeln!(writer, "+{}", cells.join(","))?;
                }
                RowChange::Removed { row, .. } => {
                    let cells: Vec<_> = row.cells.iter().map(|c| c.display().to_string()).collect();
                    writeln!(writer, "@@ -{} @@", row.source_line)?;
                    writeln!(writer, "-{}", cells.join(","))?;
                }
                RowChange::Modified {
                    old_row, new_row, ..
                } => {
                    let old_cells: Vec<_> = old_row
                        .cells
                        .iter()
                        .map(|c| c.display().to_string())
                        .collect();
                    let new_cells: Vec<_> = new_row
                        .cells
                        .iter()
                        .map(|c| c.display().to_string())
                        .collect();
                    writeln!(
                        writer,
                        "@@ -{},{} +{},{} @@",
                        old_row.source_line, 1, new_row.source_line, 1
                    )?;
                    writeln!(writer, "-{}", old_cells.join(","))?;
                    writeln!(writer, "+{}", new_cells.join(","))?;
                }
            }
        }

        Ok(())
    }
}
