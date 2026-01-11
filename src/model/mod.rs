//! Data model for tabular data representation

mod key;
mod schema;
mod table;

pub use key::KeyBuilder;
pub use schema::{CellType, Column};
pub use table::{CellValue, Row, Table};
