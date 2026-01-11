//! Diff engine for comparing tables

pub mod cell_diff;
mod row_diff;
mod schema_diff;

use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::model::{CellValue, Row, Table};

pub use cell_diff::CellComparator;
pub use row_diff::RowMatcher;
pub use schema_diff::{SchemaChange, SchemaDiff};

/// A change to a single cell
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellChange {
    /// Column name
    pub column: String,
    /// Column index
    pub column_index: usize,
    /// Old value
    pub old_value: CellValue,
    /// New value
    pub new_value: CellValue,
}

/// A change to a row
#[derive(Debug)]
pub enum RowChange {
    /// Row was added in the new table
    Added { key: String, row: Row },
    /// Row was removed from the old table
    Removed { key: String, row: Row },
    /// Row was modified
    Modified {
        key: String,
        old_row: Row,
        new_row: Row,
        changes: Vec<CellChange>,
    },
}

impl RowChange {
    /// Get the key for this change
    pub fn key(&self) -> &str {
        match self {
            RowChange::Added { key, .. } => key,
            RowChange::Removed { key, .. } => key,
            RowChange::Modified { key, .. } => key,
        }
    }
}

/// Statistics about the diff
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DiffStats {
    pub rows_added: usize,
    pub rows_removed: usize,
    pub rows_modified: usize,
    pub rows_unchanged: usize,
    pub cells_changed: usize,
    pub old_row_count: usize,
    pub new_row_count: usize,
}

impl DiffStats {
    /// Check if there are any changes
    pub fn has_changes(&self) -> bool {
        self.rows_added > 0 || self.rows_removed > 0 || self.rows_modified > 0
    }
}

/// Result of comparing two tables
#[derive(Debug)]
pub struct DiffResult {
    /// Schema changes between tables
    pub schema_changes: Vec<SchemaChange>,
    /// Row changes (added, removed, modified)
    pub row_changes: Vec<RowChange>,
    /// Statistics
    pub stats: DiffStats,
}

impl DiffResult {
    /// Create a new empty diff result
    pub fn new() -> Self {
        Self {
            schema_changes: Vec::new(),
            row_changes: Vec::new(),
            stats: DiffStats::default(),
        }
    }

    /// Check if there are any changes
    pub fn has_changes(&self) -> bool {
        !self.schema_changes.is_empty() || self.stats.has_changes()
    }

    /// Get only added rows
    pub fn added_rows(&self) -> impl Iterator<Item = &Row> {
        self.row_changes.iter().filter_map(|c| match c {
            RowChange::Added { row, .. } => Some(row),
            _ => None,
        })
    }

    /// Get only removed rows
    pub fn removed_rows(&self) -> impl Iterator<Item = &Row> {
        self.row_changes.iter().filter_map(|c| match c {
            RowChange::Removed { row, .. } => Some(row),
            _ => None,
        })
    }

    /// Get only modified rows
    pub fn modified_rows(&self) -> impl Iterator<Item = (&Row, &Row, &Vec<CellChange>)> {
        self.row_changes.iter().filter_map(|c| match c {
            RowChange::Modified {
                old_row,
                new_row,
                changes,
                ..
            } => Some((old_row, new_row, changes)),
            _ => None,
        })
    }
}

impl Default for DiffResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Main diff engine
pub struct DiffEngine {
    config: Config,
    cell_comparator: CellComparator,
}

impl DiffEngine {
    /// Create a new diff engine with configuration
    pub fn new(config: Config) -> Self {
        let cell_comparator = CellComparator::new(
            config.ignore_case,
            config.ignore_whitespace,
            config.numeric_tolerance,
        );
        Self {
            config,
            cell_comparator,
        }
    }

    /// Compare two tables
    pub fn diff(&self, old_table: &Table, new_table: &Table) -> DiffResult {
        let mut result = DiffResult::new();

        // Set stats
        result.stats.old_row_count = old_table.row_count();
        result.stats.new_row_count = new_table.row_count();

        // Compare schemas
        result.schema_changes = SchemaDiff::compare(old_table, new_table);

        // Match rows
        let row_matcher = RowMatcher::new(&self.config.ignore_columns);
        let matches = row_matcher.match_rows(old_table, new_table);

        // Process matches
        for (old_row_opt, new_row_opt) in matches {
            match (old_row_opt, new_row_opt) {
                (Some(old_row), Some(new_row)) => {
                    // Check if row is modified
                    let changes = self.compare_row_cells(old_row, new_row, old_table, new_table);
                    if !changes.is_empty() {
                        result.stats.rows_modified += 1;
                        result.stats.cells_changed += changes.len();
                        result.row_changes.push(RowChange::Modified {
                            key: old_row.key.clone(),
                            old_row: old_row.clone(),
                            new_row: new_row.clone(),
                            changes,
                        });
                    } else {
                        result.stats.rows_unchanged += 1;
                    }
                }
                (Some(old_row), None) => {
                    result.stats.rows_removed += 1;
                    result.row_changes.push(RowChange::Removed {
                        key: old_row.key.clone(),
                        row: old_row.clone(),
                    });
                }
                (None, Some(new_row)) => {
                    result.stats.rows_added += 1;
                    result.row_changes.push(RowChange::Added {
                        key: new_row.key.clone(),
                        row: new_row.clone(),
                    });
                }
                (None, None) => unreachable!(),
            }
        }

        result
    }

    /// Compare cells between two rows
    fn compare_row_cells(
        &self,
        old_row: &Row,
        new_row: &Row,
        old_table: &Table,
        new_table: &Table,
    ) -> Vec<CellChange> {
        let mut changes = Vec::new();

        // Build column name mapping for both tables
        let old_columns: Vec<_> = old_table.columns.iter().map(|c| &c.name).collect();
        let new_columns: Vec<_> = new_table.columns.iter().map(|c| &c.name).collect();

        // Compare columns that exist in both tables
        for (old_idx, old_col_name) in old_columns.iter().enumerate() {
            // Skip ignored columns
            if self.config.ignore_columns.contains(*old_col_name) {
                continue;
            }

            // Find matching column in new table
            if let Some(new_idx) = new_columns.iter().position(|n| n == old_col_name) {
                let old_value = old_row.cells.get(old_idx).cloned().unwrap_or(CellValue::Null);
                let new_value = new_row.cells.get(new_idx).cloned().unwrap_or(CellValue::Null);

                if !self.cell_comparator.equal(&old_value, &new_value) {
                    changes.push(CellChange {
                        column: (*old_col_name).clone(),
                        column_index: old_idx,
                        old_value,
                        new_value,
                    });
                }
            }
        }

        changes
    }
}

/// Convenience function to compute diff
pub fn compute_diff(old_table: &Table, new_table: &Table, config: &Config) -> DiffResult {
    let engine = DiffEngine::new(config.clone());
    engine.diff(old_table, new_table)
}
