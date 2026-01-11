//! JSON output format

use std::io::Write;
use std::path::Path;

use anyhow::Result;
use serde::Serialize;

use crate::diff::{DiffResult, RowChange, SchemaChange};
use crate::model::{CellValue, Table};

use super::OutputFormatter;

/// JSON output formatter
pub struct JsonOutput {
    pretty: bool,
}

impl JsonOutput {
    pub fn new() -> Self {
        Self { pretty: true }
    }

    pub fn compact() -> Self {
        Self { pretty: false }
    }
}

impl Default for JsonOutput {
    fn default() -> Self {
        Self::new()
    }
}

/// Serializable row change for JSON output
#[derive(Serialize)]
struct JsonRowChange {
    #[serde(rename = "type")]
    change_type: String,
    key: String,
    source_line: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    cells: Option<Vec<JsonCell>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    changes: Option<Vec<JsonCellChange>>,
}

#[derive(Serialize)]
struct JsonCell {
    column: String,
    value: serde_json::Value,
}

#[derive(Serialize)]
struct JsonCellChange {
    column: String,
    old_value: serde_json::Value,
    new_value: serde_json::Value,
}

#[derive(Serialize)]
struct JsonDiffOutput {
    old_file: String,
    new_file: String,
    schema_changes: Vec<SchemaChange>,
    row_changes: Vec<JsonRowChange>,
    stats: JsonStats,
}

#[derive(Serialize)]
struct JsonStats {
    rows_added: usize,
    rows_removed: usize,
    rows_modified: usize,
    rows_unchanged: usize,
    cells_changed: usize,
    old_row_count: usize,
    new_row_count: usize,
}

fn cell_value_to_json(value: &CellValue) -> serde_json::Value {
    match value {
        CellValue::Null => serde_json::Value::Null,
        CellValue::Bool(b) => serde_json::Value::Bool(*b),
        CellValue::Int(i) => serde_json::json!(*i),
        CellValue::Float(f) => serde_json::json!(*f),
        CellValue::String(s) => serde_json::Value::String(s.to_string()),
        CellValue::Date(d) => serde_json::Value::String(d.to_string()),
        CellValue::DateTime(dt) => serde_json::Value::String(dt.to_string()),
    }
}

impl OutputFormatter for JsonOutput {
    fn render(
        &self,
        diff: &DiffResult,
        old_table: &Table,
        new_table: &Table,
        old_path: &Path,
        new_path: &Path,
        writer: &mut dyn Write,
    ) -> Result<()> {
        let row_changes: Vec<JsonRowChange> = diff
            .row_changes
            .iter()
            .map(|change| match change {
                RowChange::Added { key, row } => JsonRowChange {
                    change_type: "added".to_string(),
                    key: key.clone(),
                    source_line: row.source_line,
                    cells: Some(
                        row.cells
                            .iter()
                            .enumerate()
                            .map(|(i, c)| JsonCell {
                                column: new_table
                                    .columns
                                    .get(i)
                                    .map(|col| col.name.clone())
                                    .unwrap_or_else(|| format!("column_{}", i)),
                                value: cell_value_to_json(c),
                            })
                            .collect(),
                    ),
                    changes: None,
                },
                RowChange::Removed { key, row } => JsonRowChange {
                    change_type: "removed".to_string(),
                    key: key.clone(),
                    source_line: row.source_line,
                    cells: Some(
                        row.cells
                            .iter()
                            .enumerate()
                            .map(|(i, c)| JsonCell {
                                column: old_table
                                    .columns
                                    .get(i)
                                    .map(|col| col.name.clone())
                                    .unwrap_or_else(|| format!("column_{}", i)),
                                value: cell_value_to_json(c),
                            })
                            .collect(),
                    ),
                    changes: None,
                },
                RowChange::Modified {
                    key,
                    old_row,
                    changes,
                    ..
                } => JsonRowChange {
                    change_type: "modified".to_string(),
                    key: key.clone(),
                    source_line: old_row.source_line,
                    cells: None,
                    changes: Some(
                        changes
                            .iter()
                            .map(|c| JsonCellChange {
                                column: c.column.clone(),
                                old_value: cell_value_to_json(&c.old_value),
                                new_value: cell_value_to_json(&c.new_value),
                            })
                            .collect(),
                    ),
                },
            })
            .collect();

        let output = JsonDiffOutput {
            old_file: old_path.display().to_string(),
            new_file: new_path.display().to_string(),
            schema_changes: diff.schema_changes.clone(),
            row_changes,
            stats: JsonStats {
                rows_added: diff.stats.rows_added,
                rows_removed: diff.stats.rows_removed,
                rows_modified: diff.stats.rows_modified,
                rows_unchanged: diff.stats.rows_unchanged,
                cells_changed: diff.stats.cells_changed,
                old_row_count: diff.stats.old_row_count,
                new_row_count: diff.stats.new_row_count,
            },
        };

        if self.pretty {
            serde_json::to_writer_pretty(&mut *writer, &output)?;
        } else {
            serde_json::to_writer(&mut *writer, &output)?;
        }
        writeln!(writer)?;

        Ok(())
    }
}
