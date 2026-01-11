//! Row matching algorithm

use rustc_hash::FxHashSet;

use crate::model::{Row, Table};

/// Row matcher using hash-based lookup
pub struct RowMatcher {
    #[allow(dead_code)]
    ignore_columns: FxHashSet<String>,
}

impl RowMatcher {
    /// Create a new row matcher
    pub fn new(ignore_columns: &[String]) -> Self {
        Self {
            ignore_columns: ignore_columns.iter().cloned().collect(),
        }
    }

    /// Match rows between old and new tables
    /// Returns iterator of (Option<old_row>, Option<new_row>) pairs
    pub fn match_rows<'a>(
        &self,
        old_table: &'a Table,
        new_table: &'a Table,
    ) -> Vec<(Option<&'a Row>, Option<&'a Row>)> {
        let mut matches = Vec::new();
        let mut matched_new_hashes = FxHashSet::default();

        // Match old rows to new rows
        for old_row in &old_table.rows {
            if let Some(new_row) = new_table.get_row_by_hash(old_row.key_hash) {
                // Verify keys actually match (handle hash collisions)
                if old_row.key == new_row.key {
                    matches.push((Some(old_row), Some(new_row)));
                    matched_new_hashes.insert(new_row.key_hash);
                } else {
                    // Hash collision: treat as removed
                    matches.push((Some(old_row), None));
                }
            } else {
                // Row was removed
                matches.push((Some(old_row), None));
            }
        }

        // Find added rows (new rows not matched to any old row)
        for new_row in &new_table.rows {
            if !matched_new_hashes.contains(&new_row.key_hash) {
                matches.push((None, Some(new_row)));
            }
        }

        matches
    }
}

/// Match rows with key column override
pub fn match_rows_with_keys<'a>(
    old_table: &'a Table,
    new_table: &'a Table,
    _key_columns: &[String],
) -> Vec<(Option<&'a Row>, Option<&'a Row>)> {
    // Note: Tables should already have key columns set via set_key_columns()
    // This function is a convenience wrapper
    let matcher = RowMatcher::new(&[]);
    matcher.match_rows(old_table, new_table)
}
