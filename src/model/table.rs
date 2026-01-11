//! Table, Row, and Cell data structures

use std::borrow::Cow;
use std::hash::{Hash, Hasher};

use chrono::{NaiveDate, NaiveDateTime};
use indexmap::IndexMap;
use rustc_hash::FxHasher;
use serde::{Deserialize, Serialize};

use super::schema::Column;

/// A cell value with type information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CellValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(Cow<'static, str>),
    Date(NaiveDate),
    DateTime(NaiveDateTime),
}

impl PartialEq for CellValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (CellValue::Null, CellValue::Null) => true,
            (CellValue::Bool(a), CellValue::Bool(b)) => a == b,
            (CellValue::Int(a), CellValue::Int(b)) => a == b,
            (CellValue::Float(a), CellValue::Float(b)) => {
                // Handle NaN comparison
                if a.is_nan() && b.is_nan() {
                    true
                } else {
                    a == b
                }
            }
            (CellValue::String(a), CellValue::String(b)) => a == b,
            (CellValue::Date(a), CellValue::Date(b)) => a == b,
            (CellValue::DateTime(a), CellValue::DateTime(b)) => a == b,
            // Cross-type numeric comparison
            (CellValue::Int(a), CellValue::Float(b)) => (*a as f64) == *b,
            (CellValue::Float(a), CellValue::Int(b)) => *a == (*b as f64),
            _ => false,
        }
    }
}

impl Eq for CellValue {}

impl Hash for CellValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            CellValue::Null => {}
            CellValue::Bool(b) => b.hash(state),
            CellValue::Int(i) => i.hash(state),
            CellValue::Float(f) => f.to_bits().hash(state),
            CellValue::String(s) => s.hash(state),
            CellValue::Date(d) => d.hash(state),
            CellValue::DateTime(dt) => dt.hash(state),
        }
    }
}

impl CellValue {
    /// Check if the value is null
    pub fn is_null(&self) -> bool {
        matches!(self, CellValue::Null)
    }

    /// Convert to a display string
    pub fn display(&self) -> Cow<'_, str> {
        match self {
            CellValue::Null => Cow::Borrowed("NULL"),
            CellValue::Bool(b) => Cow::Owned(b.to_string()),
            CellValue::Int(i) => Cow::Owned(i.to_string()),
            CellValue::Float(f) => Cow::Owned(f.to_string()),
            CellValue::String(s) => Cow::Borrowed(s.as_ref()),
            CellValue::Date(d) => Cow::Owned(d.to_string()),
            CellValue::DateTime(dt) => Cow::Owned(dt.to_string()),
        }
    }

    /// Compare with numeric tolerance
    pub fn equals_with_tolerance(&self, other: &Self, tolerance: f64) -> bool {
        match (self, other) {
            (CellValue::Float(a), CellValue::Float(b)) => (a - b).abs() <= tolerance,
            (CellValue::Int(a), CellValue::Float(b)) => ((*a as f64) - b).abs() <= tolerance,
            (CellValue::Float(a), CellValue::Int(b)) => (a - (*b as f64)).abs() <= tolerance,
            _ => self == other,
        }
    }

    /// Compare ignoring case (for strings)
    pub fn equals_ignore_case(&self, other: &Self) -> bool {
        match (self, other) {
            (CellValue::String(a), CellValue::String(b)) => a.eq_ignore_ascii_case(b),
            _ => self == other,
        }
    }

    /// Compare ignoring whitespace (for strings)
    pub fn equals_ignore_whitespace(&self, other: &Self) -> bool {
        match (self, other) {
            (CellValue::String(a), CellValue::String(b)) => a.trim() == b.trim(),
            _ => self == other,
        }
    }
}

impl std::fmt::Display for CellValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display())
    }
}

impl From<&str> for CellValue {
    fn from(s: &str) -> Self {
        CellValue::String(Cow::Owned(s.to_string()))
    }
}

impl From<String> for CellValue {
    fn from(s: String) -> Self {
        CellValue::String(Cow::Owned(s))
    }
}

impl From<i64> for CellValue {
    fn from(i: i64) -> Self {
        CellValue::Int(i)
    }
}

impl From<f64> for CellValue {
    fn from(f: f64) -> Self {
        CellValue::Float(f)
    }
}

impl From<bool> for CellValue {
    fn from(b: bool) -> Self {
        CellValue::Bool(b)
    }
}

impl<T> From<Option<T>> for CellValue
where
    T: Into<CellValue>,
{
    fn from(opt: Option<T>) -> Self {
        match opt {
            Some(v) => v.into(),
            None => CellValue::Null,
        }
    }
}

/// A row in the table
#[derive(Debug, Clone)]
pub struct Row {
    /// Cell values in column order
    pub cells: Vec<CellValue>,
    /// Composite key string for this row
    pub key: String,
    /// Pre-computed hash of the key for O(1) lookup
    pub key_hash: u64,
    /// Original line/row number in source file (1-indexed)
    pub source_line: usize,
}

impl Row {
    /// Create a new row with computed key
    pub fn new(cells: Vec<CellValue>, key_column_indices: &[usize], source_line: usize) -> Self {
        let key = Self::compute_key(&cells, key_column_indices);
        let key_hash = Self::hash_key(&key);
        Self {
            cells,
            key,
            key_hash,
            source_line,
        }
    }

    /// Compute composite key from specified columns
    fn compute_key(cells: &[CellValue], key_column_indices: &[usize]) -> String {
        if key_column_indices.is_empty() {
            // If no key columns specified, use all columns
            cells
                .iter()
                .map(|c| c.display().into_owned())
                .collect::<Vec<_>>()
                .join("|")
        } else {
            key_column_indices
                .iter()
                .filter_map(|&i| cells.get(i))
                .map(|c| c.display().into_owned())
                .collect::<Vec<_>>()
                .join("|")
        }
    }

    /// Hash the key using FxHasher for performance
    fn hash_key(key: &str) -> u64 {
        let mut hasher = FxHasher::default();
        key.hash(&mut hasher);
        hasher.finish()
    }

    /// Get a cell value by column index
    pub fn get(&self, index: usize) -> Option<&CellValue> {
        self.cells.get(index)
    }

    /// Recompute the key with new key column indices
    pub fn recompute_key(&mut self, key_column_indices: &[usize]) {
        self.key = Self::compute_key(&self.cells, key_column_indices);
        self.key_hash = Self::hash_key(&self.key);
    }
}

/// A table containing columns and rows
#[derive(Debug)]
pub struct Table {
    /// Column definitions
    pub columns: Vec<Column>,
    /// All rows in the table
    pub rows: Vec<Row>,
    /// Indices of columns used as primary key
    pub key_columns: Vec<usize>,
    /// Index from key hash to row index for O(1) lookup
    pub row_index: IndexMap<u64, usize>,
}

impl Table {
    /// Create a new empty table with column definitions
    pub fn new(columns: Vec<Column>) -> Self {
        Self {
            columns,
            rows: Vec::new(),
            key_columns: Vec::new(),
            row_index: IndexMap::new(),
        }
    }

    /// Add a row to the table
    pub fn add_row(&mut self, cells: Vec<CellValue>, source_line: usize) {
        let row = Row::new(cells, &self.key_columns, source_line);
        let hash = row.key_hash;
        let idx = self.rows.len();
        self.rows.push(row);
        self.row_index.insert(hash, idx);
    }

    /// Set key columns by name
    pub fn set_key_columns(&mut self, key_names: &[String]) {
        self.key_columns = key_names
            .iter()
            .filter_map(|name| self.columns.iter().position(|c| &c.name == name))
            .collect();

        // Recompute keys for all rows
        for row in &mut self.rows {
            row.recompute_key(&self.key_columns);
        }

        // Rebuild index
        self.rebuild_row_index();
    }

    /// Set key columns by index
    pub fn set_key_column_indices(&mut self, indices: Vec<usize>) {
        self.key_columns = indices;

        // Recompute keys for all rows
        for row in &mut self.rows {
            row.recompute_key(&self.key_columns);
        }

        // Rebuild index
        self.rebuild_row_index();
    }

    /// Rebuild the row index
    fn rebuild_row_index(&mut self) {
        self.row_index.clear();
        for (idx, row) in self.rows.iter().enumerate() {
            self.row_index.insert(row.key_hash, idx);
        }
    }

    /// Look up a row by key hash
    pub fn get_row_by_hash(&self, hash: u64) -> Option<&Row> {
        self.row_index.get(&hash).map(|&idx| &self.rows[idx])
    }

    /// Get column index by name
    pub fn column_index(&self, name: &str) -> Option<usize> {
        self.columns.iter().position(|c| c.name == name)
    }

    /// Get column by name
    pub fn column(&self, name: &str) -> Option<&Column> {
        self.columns.iter().find(|c| c.name == name)
    }

    /// Number of rows
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }

    /// Number of columns
    pub fn column_count(&self) -> usize {
        self.columns.len()
    }

    /// Sort rows by a column
    pub fn sort_by_column(&mut self, column_name: &str) {
        if let Some(col_idx) = self.column_index(column_name) {
            self.rows.sort_by(|a, b| {
                let va = a.get(col_idx);
                let vb = b.get(col_idx);
                match (va, vb) {
                    (Some(CellValue::Int(a)), Some(CellValue::Int(b))) => a.cmp(b),
                    (Some(CellValue::Float(a)), Some(CellValue::Float(b))) => {
                        a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
                    }
                    (Some(CellValue::String(a)), Some(CellValue::String(b))) => a.cmp(b),
                    _ => std::cmp::Ordering::Equal,
                }
            });
            self.rebuild_row_index();
        }
    }
}
