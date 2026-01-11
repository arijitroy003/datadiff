//! Schema comparison logic

use serde::{Deserialize, Serialize};

use crate::model::Table;

/// Types of schema changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SchemaChange {
    /// Column was added
    ColumnAdded { name: String, index: usize },
    /// Column was removed
    ColumnRemoved { name: String, index: usize },
    /// Column was renamed (detected by position if types match)
    ColumnRenamed {
        old_name: String,
        new_name: String,
        index: usize,
    },
    /// Column was moved to different position
    ColumnMoved {
        name: String,
        from_index: usize,
        to_index: usize,
    },
    /// Column type changed
    ColumnTypeChanged {
        name: String,
        old_type: String,
        new_type: String,
    },
}

impl std::fmt::Display for SchemaChange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SchemaChange::ColumnAdded { name, index } => {
                write!(f, "+ {} (new column at position {})", name, index)
            }
            SchemaChange::ColumnRemoved { name, index } => {
                write!(f, "- {} (removed from position {})", name, index)
            }
            SchemaChange::ColumnRenamed {
                old_name,
                new_name,
                index,
            } => {
                write!(f, "~ {} → {} (renamed at position {})", old_name, new_name, index)
            }
            SchemaChange::ColumnMoved {
                name,
                from_index,
                to_index,
            } => {
                write!(f, "↔ {} (moved from {} to {})", name, from_index, to_index)
            }
            SchemaChange::ColumnTypeChanged {
                name,
                old_type,
                new_type,
            } => {
                write!(f, "⚡ {} (type {} → {})", name, old_type, new_type)
            }
        }
    }
}

/// Schema comparison engine
pub struct SchemaDiff;

impl SchemaDiff {
    /// Compare schemas of two tables
    pub fn compare(old_table: &Table, new_table: &Table) -> Vec<SchemaChange> {
        let mut changes = Vec::new();

        let old_names: Vec<_> = old_table.columns.iter().map(|c| &c.name).collect();
        let new_names: Vec<_> = new_table.columns.iter().map(|c| &c.name).collect();

        // Find removed columns
        for (old_idx, old_name) in old_names.iter().enumerate() {
            if !new_names.contains(old_name) {
                changes.push(SchemaChange::ColumnRemoved {
                    name: (*old_name).clone(),
                    index: old_idx,
                });
            }
        }

        // Find added columns
        for (new_idx, new_name) in new_names.iter().enumerate() {
            if !old_names.contains(new_name) {
                changes.push(SchemaChange::ColumnAdded {
                    name: (*new_name).clone(),
                    index: new_idx,
                });
            }
        }

        // Find moved columns
        for (old_idx, old_name) in old_names.iter().enumerate() {
            if let Some(new_idx) = new_names.iter().position(|n| n == old_name) {
                if old_idx != new_idx {
                    changes.push(SchemaChange::ColumnMoved {
                        name: (*old_name).clone(),
                        from_index: old_idx,
                        to_index: new_idx,
                    });
                }
            }
        }

        // Find type changes
        for old_col in &old_table.columns {
            if let Some(new_col) = new_table.columns.iter().find(|c| c.name == old_col.name) {
                if old_col.inferred_type != new_col.inferred_type {
                    changes.push(SchemaChange::ColumnTypeChanged {
                        name: old_col.name.clone(),
                        old_type: old_col.inferred_type.to_string(),
                        new_type: new_col.inferred_type.to_string(),
                    });
                }
            }
        }

        changes
    }
}
