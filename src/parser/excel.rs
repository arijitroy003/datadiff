//! Excel file parser (xlsx, xls, ods)

use std::borrow::Cow;
use std::path::Path;

use anyhow::{bail, Context, Result};
use calamine::{open_workbook_auto, Data, Range, Reader};

use crate::config::Config;
use crate::model::{CellValue, Column, Table};

use super::Parser;

/// Parser for Excel files
pub struct ExcelParser;

impl Parser for ExcelParser {
    fn parse(&self, path: &Path, config: &Config) -> Result<Table> {
        let mut workbook = open_workbook_auto(path)
            .with_context(|| format!("Failed to open Excel file: {}", path.display()))?;

        // Get sheet name
        let sheet_name = if let Some(ref name) = config.sheet_name {
            name.clone()
        } else {
            // Use first sheet
            let sheets = workbook.sheet_names();
            if sheets.is_empty() {
                bail!("No sheets found in workbook");
            }
            sheets[0].clone()
        };

        // Get the sheet range
        let range: Range<Data> = workbook
            .worksheet_range(&sheet_name)
            .with_context(|| format!("Failed to read sheet: {}", sheet_name))?;

        // Parse range into table
        parse_range(range, config)
    }

    fn supports_extension(&self, ext: &str) -> bool {
        matches!(ext.to_lowercase().as_str(), "xlsx" | "xls" | "ods" | "xlsm")
    }
}

fn parse_range(range: Range<Data>, config: &Config) -> Result<Table> {
    let (row_count, col_count) = range.get_size();

    if row_count == 0 {
        bail!("Empty sheet");
    }

    // First row is header
    let header_row = range.rows().next().context("No header row found")?;
    let columns: Vec<Column> = header_row
        .iter()
        .enumerate()
        .map(|(i, cell)| {
            let name = cell_to_string(cell);
            Column::new(if name.is_empty() { format!("Column{}", i + 1) } else { name }, i)
        })
        .collect();

    let mut table = Table::new(columns);

    // Set key columns if specified
    if !config.key_columns.is_empty() {
        table.set_key_columns(&config.key_columns);
    }

    // Read data rows
    for (line_num, row) in range.rows().skip(1).enumerate() {
        let cells: Vec<CellValue> = row
            .iter()
            .take(col_count)
            .map(|cell| convert_cell(cell))
            .collect();

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

    // Sort if requested
    if let Some(ref sort_col) = config.sort_by {
        table.sort_by_column(sort_col);
    }

    Ok(table)
}

fn cell_to_string(cell: &Data) -> String {
    match cell {
        Data::Empty => String::new(),
        Data::String(s) => s.clone(),
        Data::Float(f) => f.to_string(),
        Data::Int(i) => i.to_string(),
        Data::Bool(b) => b.to_string(),
        Data::DateTime(dt) => {
            // Excel stores dates as days since 1899-12-30
            format!("{}", dt)
        }
        Data::DateTimeIso(s) => s.clone(),
        Data::DurationIso(s) => s.clone(),
        Data::Error(e) => format!("#{:?}", e),
    }
}

fn convert_cell(cell: &Data) -> CellValue {
    match cell {
        Data::Empty => CellValue::Null,
        Data::String(s) => {
            if s.trim().is_empty() {
                CellValue::Null
            } else {
                CellValue::String(Cow::Owned(s.clone()))
            }
        }
        Data::Float(f) => {
            // Check if it's actually an integer
            if f.fract() == 0.0 && *f >= i64::MIN as f64 && *f <= i64::MAX as f64 {
                CellValue::Int(*f as i64)
            } else {
                CellValue::Float(*f)
            }
        }
        Data::Int(i) => CellValue::Int(*i),
        Data::Bool(b) => CellValue::Bool(*b),
        Data::DateTime(ref dt) => {
            // calamine ExcelDateTime - use Display to convert and parse
            let s = format!("{}", dt);
            // Try to parse as datetime first, then date
            if let Ok(datetime) = chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S%.f") {
                CellValue::DateTime(datetime)
            } else if let Ok(datetime) = chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S%.f") {
                CellValue::DateTime(datetime)
            } else if let Ok(date) = chrono::NaiveDate::parse_from_str(&s, "%Y-%m-%d") {
                CellValue::Date(date)
            } else {
                CellValue::String(Cow::Owned(s))
            }
        }
        Data::DateTimeIso(s) => {
            if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S") {
                CellValue::DateTime(dt)
            } else if let Ok(d) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
                CellValue::Date(d)
            } else {
                CellValue::String(Cow::Owned(s.clone()))
            }
        }
        Data::DurationIso(s) => CellValue::String(Cow::Owned(s.clone())),
        Data::Error(e) => CellValue::String(Cow::Owned(format!("#{:?}", e))),
    }
}


