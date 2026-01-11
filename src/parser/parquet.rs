//! Parquet file parser

use std::borrow::Cow;
use std::fs::File;
use std::path::Path;

use anyhow::{Context, Result};
use arrow::array::{
    Array, ArrayRef, BooleanArray, Date32Array, Float32Array, Float64Array, Int16Array,
    Int32Array, Int64Array, Int8Array, StringArray, TimestampMicrosecondArray,
    TimestampMillisecondArray, TimestampNanosecondArray, TimestampSecondArray, UInt16Array,
    UInt32Array, UInt64Array, UInt8Array,
};
use arrow::datatypes::DataType as ArrowType;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;

use crate::config::Config;
use crate::model::{CellType, CellValue, Column, Table};

use super::Parser;

/// Parser for Parquet files
pub struct ParquetParser;

impl Parser for ParquetParser {
    fn parse(&self, path: &Path, config: &Config) -> Result<Table> {
        let file = File::open(path)
            .with_context(|| format!("Failed to open Parquet file: {}", path.display()))?;

        let builder = ParquetRecordBatchReaderBuilder::try_new(file)
            .context("Failed to create Parquet reader")?;

        let schema = builder.schema().clone();
        let reader = builder.build().context("Failed to build Parquet reader")?;

        // Create columns from schema
        let columns: Vec<Column> = schema
            .fields()
            .iter()
            .enumerate()
            .map(|(i, field)| {
                Column::with_type(field.name().clone(), i, arrow_type_to_cell_type(field.data_type()))
            })
            .collect();

        let mut table = Table::new(columns);

        // Set key columns if specified
        if !config.key_columns.is_empty() {
            table.set_key_columns(&config.key_columns);
        }

        // Read record batches
        let mut line_num = 1usize;
        for batch_result in reader {
            let batch = batch_result.context("Failed to read Parquet batch")?;

            for row_idx in 0..batch.num_rows() {
                line_num += 1;
                let cells: Vec<CellValue> = batch
                    .columns()
                    .iter()
                    .map(|col| extract_cell_value(col, row_idx))
                    .collect();

                table.add_row(cells, line_num);
            }
        }

        // Sort if requested
        if let Some(ref sort_col) = config.sort_by {
            table.sort_by_column(sort_col);
        }

        Ok(table)
    }

    fn supports_extension(&self, ext: &str) -> bool {
        matches!(ext.to_lowercase().as_str(), "parquet" | "pq")
    }
}

fn arrow_type_to_cell_type(arrow_type: &ArrowType) -> CellType {
    match arrow_type {
        ArrowType::Null => CellType::Null,
        ArrowType::Boolean => CellType::Bool,
        ArrowType::Int8
        | ArrowType::Int16
        | ArrowType::Int32
        | ArrowType::Int64
        | ArrowType::UInt8
        | ArrowType::UInt16
        | ArrowType::UInt32
        | ArrowType::UInt64 => CellType::Int,
        ArrowType::Float16 | ArrowType::Float32 | ArrowType::Float64 => CellType::Float,
        ArrowType::Utf8 | ArrowType::LargeUtf8 => CellType::String,
        ArrowType::Date32 | ArrowType::Date64 => CellType::Date,
        ArrowType::Timestamp(_, _) => CellType::DateTime,
        _ => CellType::String, // Fallback to string for complex types
    }
}

fn extract_cell_value(array: &ArrayRef, row_idx: usize) -> CellValue {
    if array.is_null(row_idx) {
        return CellValue::Null;
    }

    match array.data_type() {
        ArrowType::Boolean => {
            let arr = array.as_any().downcast_ref::<BooleanArray>().unwrap();
            CellValue::Bool(arr.value(row_idx))
        }
        ArrowType::Int8 => {
            let arr = array.as_any().downcast_ref::<Int8Array>().unwrap();
            CellValue::Int(arr.value(row_idx) as i64)
        }
        ArrowType::Int16 => {
            let arr = array.as_any().downcast_ref::<Int16Array>().unwrap();
            CellValue::Int(arr.value(row_idx) as i64)
        }
        ArrowType::Int32 => {
            let arr = array.as_any().downcast_ref::<Int32Array>().unwrap();
            CellValue::Int(arr.value(row_idx) as i64)
        }
        ArrowType::Int64 => {
            let arr = array.as_any().downcast_ref::<Int64Array>().unwrap();
            CellValue::Int(arr.value(row_idx))
        }
        ArrowType::UInt8 => {
            let arr = array.as_any().downcast_ref::<UInt8Array>().unwrap();
            CellValue::Int(arr.value(row_idx) as i64)
        }
        ArrowType::UInt16 => {
            let arr = array.as_any().downcast_ref::<UInt16Array>().unwrap();
            CellValue::Int(arr.value(row_idx) as i64)
        }
        ArrowType::UInt32 => {
            let arr = array.as_any().downcast_ref::<UInt32Array>().unwrap();
            CellValue::Int(arr.value(row_idx) as i64)
        }
        ArrowType::UInt64 => {
            let arr = array.as_any().downcast_ref::<UInt64Array>().unwrap();
            CellValue::Int(arr.value(row_idx) as i64)
        }
        ArrowType::Float32 => {
            let arr = array.as_any().downcast_ref::<Float32Array>().unwrap();
            CellValue::Float(arr.value(row_idx) as f64)
        }
        ArrowType::Float64 => {
            let arr = array.as_any().downcast_ref::<Float64Array>().unwrap();
            CellValue::Float(arr.value(row_idx))
        }
        ArrowType::Utf8 => {
            let arr = array.as_any().downcast_ref::<StringArray>().unwrap();
            CellValue::String(Cow::Owned(arr.value(row_idx).to_string()))
        }
        ArrowType::Date32 => {
            let arr = array.as_any().downcast_ref::<Date32Array>().unwrap();
            let days = arr.value(row_idx);
            if let Some(date) = chrono::NaiveDate::from_num_days_from_ce_opt(days + 719163) {
                CellValue::Date(date)
            } else {
                CellValue::Int(days as i64)
            }
        }
        ArrowType::Timestamp(unit, _) => {
            let nanos = match unit {
                arrow::datatypes::TimeUnit::Second => {
                    let arr = array.as_any().downcast_ref::<TimestampSecondArray>().unwrap();
                    arr.value(row_idx) * 1_000_000_000
                }
                arrow::datatypes::TimeUnit::Millisecond => {
                    let arr = array.as_any().downcast_ref::<TimestampMillisecondArray>().unwrap();
                    arr.value(row_idx) * 1_000_000
                }
                arrow::datatypes::TimeUnit::Microsecond => {
                    let arr = array.as_any().downcast_ref::<TimestampMicrosecondArray>().unwrap();
                    arr.value(row_idx) * 1_000
                }
                arrow::datatypes::TimeUnit::Nanosecond => {
                    let arr = array.as_any().downcast_ref::<TimestampNanosecondArray>().unwrap();
                    arr.value(row_idx)
                }
            };
            if let Some(dt) = chrono::DateTime::from_timestamp_nanos(nanos).naive_utc().into() {
                CellValue::DateTime(dt)
            } else {
                CellValue::Int(nanos)
            }
        }
        _ => {
            // Fallback: convert to string
            let formatter = arrow::util::display::ArrayFormatter::try_new(
                array.as_ref(),
                &arrow::util::display::FormatOptions::default(),
            );
            if let Ok(fmt) = formatter {
                CellValue::String(Cow::Owned(fmt.value(row_idx).to_string()))
            } else {
                CellValue::Null
            }
        }
    }
}
