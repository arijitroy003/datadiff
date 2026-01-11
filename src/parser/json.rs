//! JSON array parser

use std::borrow::Cow;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use anyhow::{bail, Context, Result};
use indexmap::IndexSet;
use serde_json::Value;

use crate::config::Config;
use crate::model::{CellValue, Column, Table};

use super::Parser;

/// Parser for JSON array files
pub struct JsonParser;

impl Parser for JsonParser {
    fn parse(&self, path: &Path, config: &Config) -> Result<Table> {
        let file = File::open(path)
            .with_context(|| format!("Failed to open JSON file: {}", path.display()))?;
        let reader = BufReader::new(file);

        let value: Value =
            serde_json::from_reader(reader).context("Failed to parse JSON file")?;

        // Handle both arrays and single objects
        let array = match value {
            Value::Array(arr) => arr,
            Value::Object(_) => vec![value],
            _ => bail!("JSON must be an array or object"),
        };

        if array.is_empty() {
            bail!("JSON array is empty");
        }

        // Collect all unique keys across all objects to build column list
        let mut column_names: IndexSet<String> = IndexSet::new();
        for item in &array {
            if let Value::Object(obj) = item {
                for key in obj.keys() {
                    column_names.insert(key.clone());
                }
            }
        }

        let columns: Vec<Column> = column_names
            .iter()
            .enumerate()
            .map(|(i, name)| Column::new(name.clone(), i))
            .collect();

        let mut table = Table::new(columns);

        // Set key columns if specified
        if !config.key_columns.is_empty() {
            table.set_key_columns(&config.key_columns);
        }

        // Convert each object to a row
        for (line_num, item) in array.iter().enumerate() {
            let cells = match item {
                Value::Object(obj) => column_names
                    .iter()
                    .map(|key| json_value_to_cell(obj.get(key)))
                    .collect(),
                _ => {
                    // Non-object item in array: put in first column
                    let mut cells = vec![json_value_to_cell(Some(item))];
                    cells.resize(column_names.len(), CellValue::Null);
                    cells
                }
            };

            table.add_row(cells, line_num + 1);
        }

        // Sort if requested
        if let Some(ref sort_col) = config.sort_by {
            table.sort_by_column(sort_col);
        }

        Ok(table)
    }

    fn supports_extension(&self, ext: &str) -> bool {
        matches!(ext.to_lowercase().as_str(), "json" | "jsonl" | "ndjson")
    }
}

fn json_value_to_cell(value: Option<&Value>) -> CellValue {
    match value {
        None | Some(Value::Null) => CellValue::Null,
        Some(Value::Bool(b)) => CellValue::Bool(*b),
        Some(Value::Number(n)) => {
            if let Some(i) = n.as_i64() {
                CellValue::Int(i)
            } else if let Some(f) = n.as_f64() {
                CellValue::Float(f)
            } else {
                CellValue::String(Cow::Owned(n.to_string()))
            }
        }
        Some(Value::String(s)) => {
            // Try parsing as date/datetime
            if let Ok(date) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
                return CellValue::Date(date);
            }
            if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S") {
                return CellValue::DateTime(dt);
            }
            if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S") {
                return CellValue::DateTime(dt);
            }
            CellValue::String(Cow::Owned(s.clone()))
        }
        Some(Value::Array(arr)) => {
            // Serialize array back to JSON string
            CellValue::String(Cow::Owned(serde_json::to_string(arr).unwrap_or_default()))
        }
        Some(Value::Object(obj)) => {
            // Serialize object back to JSON string
            CellValue::String(Cow::Owned(serde_json::to_string(obj).unwrap_or_default()))
        }
    }
}
