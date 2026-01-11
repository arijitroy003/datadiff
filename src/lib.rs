//! datadiff - Semantic diff for tabular data
//!
//! A high-performance library for comparing tabular data files (CSV, Excel, Parquet, JSON)
//! with semantic understanding of rows and cells.

pub mod config;
pub mod diff;
pub mod git;
pub mod model;
pub mod output;
pub mod parser;

pub use config::Config;
pub use diff::DiffResult;
pub use model::Table;
