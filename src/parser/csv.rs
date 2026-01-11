//! CSV file parser

use std::borrow::Cow;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use anyhow::{Context, Result};

use crate::config::Config;
use crate::model::{CellType, CellValue, Column, Table};

use super::Parser;

/// Parser for CSV files
pub struct CsvParser;

impl Parser for CsvParser {
    fn parse(&self, path: &Path, config: &Config) -> Result<Table> {
        let file =
            File::open(path).with_context(|| format!("Failed to open file: {}", path.display()))?;
        let reader = BufReader::new(file);
        let mut csv_reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .flexible(true)
            .from_reader(reader);

        // Read headers
        let headers = csv_reader
            .headers()
            .context("Failed to read CSV headers")?
            .clone();

        let columns: Vec<Column> = headers
            .iter()
            .enumerate()
            .map(|(i, name)| Column::new(name.to_string(), i))
            .collect();

        let mut table = Table::new(columns);

        // Set key columns if specified in config
        if !config.key_columns.is_empty() {
            table.set_key_columns(&config.key_columns);
        }

        // Read rows
        for (line_num, result) in csv_reader.records().enumerate() {
            let record = result.with_context(|| format!("Failed to read CSV row {}", line_num + 2))?; // +2 for 1-indexing and header

            let cells: Vec<CellValue> = record.iter().map(|s| parse_cell_value(s)).collect();

            // Pad with nulls if row has fewer columns
            let cells = if cells.len() < table.column_count() {
                let mut padded = cells;
                padded.resize(table.column_count(), CellValue::Null);
                padded
            } else {
                cells
            };

            table.add_row(cells, line_num + 2); // +2 for 1-indexing and header
        }

        // Infer column types
        infer_column_types(&mut table);

        // Sort if requested
        if let Some(ref sort_col) = config.sort_by {
            table.sort_by_column(sort_col);
        }

        Ok(table)
    }

    fn supports_extension(&self, ext: &str) -> bool {
        matches!(ext.to_lowercase().as_str(), "csv" | "tsv" | "txt")
    }
}

/// Parse a string value into a CellValue with type inference
fn parse_cell_value(s: &str) -> CellValue {
    let trimmed = s.trim();

    // Check for empty/null
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("null") || trimmed == "NA" {
        return CellValue::Null;
    }

    // Try parsing as boolean
    if trimmed.eq_ignore_ascii_case("true") || trimmed.eq_ignore_ascii_case("yes") {
        return CellValue::Bool(true);
    }
    if trimmed.eq_ignore_ascii_case("false") || trimmed.eq_ignore_ascii_case("no") {
        return CellValue::Bool(false);
    }

    // Try parsing as integer
    if let Ok(i) = trimmed.parse::<i64>() {
        return CellValue::Int(i);
    }

    // Try parsing as float
    if let Ok(f) = trimmed.parse::<f64>() {
        return CellValue::Float(f);
    }

    // Try parsing as date
    if let Ok(date) = chrono::NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
        return CellValue::Date(date);
    }

    // Try parsing as datetime (ISO 8601)
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%dT%H:%M:%S") {
        return CellValue::DateTime(dt);
    }
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%d %H:%M:%S") {
        return CellValue::DateTime(dt);
    }

    // Default to string
    CellValue::String(Cow::Owned(trimmed.to_string()))
}

/// Infer column types from data
fn infer_column_types(table: &mut Table) {
    for col_idx in 0..table.column_count() {
        let mut inferred = CellType::Null;

        for row in &table.rows {
            if let Some(cell) = row.cells.get(col_idx) {
                let cell_type = match cell {
                    CellValue::Null => CellType::Null,
                    CellValue::Bool(_) => CellType::Bool,
                    CellValue::Int(_) => CellType::Int,
                    CellValue::Float(_) => CellType::Float,
                    CellValue::String(_) => CellType::String,
                    CellValue::Date(_) => CellType::Date,
                    CellValue::DateTime(_) => CellType::DateTime,
                };

                inferred = inferred.widen(cell_type);
            }
        }

        if let Some(col) = table.columns.get_mut(col_idx) {
            col.inferred_type = inferred;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cell_value() {
        assert_eq!(parse_cell_value(""), CellValue::Null);
        assert_eq!(parse_cell_value("null"), CellValue::Null);
        assert_eq!(parse_cell_value("true"), CellValue::Bool(true));
        assert_eq!(parse_cell_value("false"), CellValue::Bool(false));
        assert_eq!(parse_cell_value("42"), CellValue::Int(42));
        assert_eq!(parse_cell_value("3.14"), CellValue::Float(3.14));
        assert_eq!(
            parse_cell_value("hello"),
            CellValue::String(Cow::Owned("hello".to_string()))
        );
    }
}
