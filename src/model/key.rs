//! Primary key handling utilities

use super::table::{CellValue, Table};

/// Builder for computing composite keys
pub struct KeyBuilder {
    column_indices: Vec<usize>,
    separator: String,
}

impl Default for KeyBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyBuilder {
    /// Create a new key builder
    pub fn new() -> Self {
        Self {
            column_indices: Vec::new(),
            separator: "|".to_string(),
        }
    }

    /// Set the key columns by index
    pub fn with_columns(mut self, indices: Vec<usize>) -> Self {
        self.column_indices = indices;
        self
    }

    /// Set the key columns by name from a table
    pub fn with_column_names(mut self, table: &Table, names: &[String]) -> Self {
        self.column_indices = names
            .iter()
            .filter_map(|name| table.column_index(name))
            .collect();
        self
    }

    /// Set the separator between key components
    pub fn with_separator(mut self, sep: impl Into<String>) -> Self {
        self.separator = sep.into();
        self
    }

    /// Build a key string from cell values
    pub fn build_key(&self, cells: &[CellValue]) -> String {
        if self.column_indices.is_empty() {
            // Use all columns if no key columns specified
            cells
                .iter()
                .map(|c| c.display().into_owned())
                .collect::<Vec<_>>()
                .join(&self.separator)
        } else {
            self.column_indices
                .iter()
                .filter_map(|&i| cells.get(i))
                .map(|c| c.display().into_owned())
                .collect::<Vec<_>>()
                .join(&self.separator)
        }
    }

    /// Get the column indices
    pub fn column_indices(&self) -> &[usize] {
        &self.column_indices
    }

    /// Check if key columns are set
    pub fn has_key_columns(&self) -> bool {
        !self.column_indices.is_empty()
    }
}

/// Auto-detect potential key columns based on uniqueness
pub fn detect_key_columns(table: &Table) -> Vec<usize> {
    use rustc_hash::FxHashSet;

    // Try each column to see if it has unique values
    for col_idx in 0..table.column_count() {
        let mut seen: FxHashSet<u64> = FxHashSet::default();
        let mut all_unique = true;

        for row in &table.rows {
            if let Some(cell) = row.cells.get(col_idx) {
                use std::hash::{Hash, Hasher};
                let mut hasher = rustc_hash::FxHasher::default();
                cell.hash(&mut hasher);
                let hash = hasher.finish();

                if !seen.insert(hash) {
                    all_unique = false;
                    break;
                }
            }
        }

        if all_unique {
            return vec![col_idx];
        }
    }

    // If no single unique column found, return empty (will use all columns)
    Vec::new()
}
