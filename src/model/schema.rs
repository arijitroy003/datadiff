//! Column metadata and type information

use serde::{Deserialize, Serialize};

/// Inferred cell type for a column
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CellType {
    Null,
    Bool,
    Int,
    Float,
    String,
    Date,
    DateTime,
    Mixed,
}

impl Default for CellType {
    fn default() -> Self {
        CellType::Null
    }
}

impl CellType {
    /// Widen the type to accommodate another type
    pub fn widen(self, other: CellType) -> CellType {
        if self == other {
            return self;
        }

        match (self, other) {
            (CellType::Null, t) | (t, CellType::Null) => t,
            (CellType::Int, CellType::Float) | (CellType::Float, CellType::Int) => CellType::Float,
            (CellType::Date, CellType::DateTime) | (CellType::DateTime, CellType::Date) => {
                CellType::DateTime
            }
            _ => CellType::Mixed,
        }
    }
}

impl std::fmt::Display for CellType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CellType::Null => write!(f, "null"),
            CellType::Bool => write!(f, "bool"),
            CellType::Int => write!(f, "int"),
            CellType::Float => write!(f, "float"),
            CellType::String => write!(f, "string"),
            CellType::Date => write!(f, "date"),
            CellType::DateTime => write!(f, "datetime"),
            CellType::Mixed => write!(f, "mixed"),
        }
    }
}

/// Column metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    /// Column name (from header)
    pub name: String,
    /// Column index (0-based position)
    pub index: usize,
    /// Inferred type from data
    pub inferred_type: CellType,
}

impl Column {
    /// Create a new column with name and index
    pub fn new(name: impl Into<String>, index: usize) -> Self {
        Self {
            name: name.into(),
            index,
            inferred_type: CellType::Null,
        }
    }

    /// Create a column with a specified type
    pub fn with_type(name: impl Into<String>, index: usize, cell_type: CellType) -> Self {
        Self {
            name: name.into(),
            index,
            inferred_type: cell_type,
        }
    }
}
