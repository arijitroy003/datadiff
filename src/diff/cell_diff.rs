//! Cell-level comparison logic

use crate::model::CellValue;

/// Cell comparator with configurable options
pub struct CellComparator {
    ignore_case: bool,
    ignore_whitespace: bool,
    numeric_tolerance: Option<f64>,
}

impl CellComparator {
    /// Create a new cell comparator
    pub fn new(ignore_case: bool, ignore_whitespace: bool, numeric_tolerance: Option<f64>) -> Self {
        Self {
            ignore_case,
            ignore_whitespace,
            numeric_tolerance,
        }
    }

    /// Compare two cell values for equality
    pub fn equal(&self, a: &CellValue, b: &CellValue) -> bool {
        // Handle tolerance for numeric values
        if let Some(tolerance) = self.numeric_tolerance {
            if a.equals_with_tolerance(b, tolerance) {
                return true;
            }
        }

        // Handle case-insensitive comparison
        if self.ignore_case {
            if a.equals_ignore_case(b) {
                return true;
            }
        }

        // Handle whitespace-insensitive comparison
        if self.ignore_whitespace {
            if a.equals_ignore_whitespace(b) {
                return true;
            }
        }

        // Standard equality
        a == b
    }
}

impl Default for CellComparator {
    fn default() -> Self {
        Self::new(false, false, None)
    }
}

/// Calculate percentage change for numeric values
pub fn percentage_change(old: &CellValue, new: &CellValue) -> Option<f64> {
    let old_num = match old {
        CellValue::Int(i) => *i as f64,
        CellValue::Float(f) => *f,
        _ => return None,
    };

    let new_num = match new {
        CellValue::Int(i) => *i as f64,
        CellValue::Float(f) => *f,
        _ => return None,
    };

    if old_num == 0.0 {
        if new_num == 0.0 {
            Some(0.0)
        } else {
            None // Infinite change
        }
    } else {
        Some((new_num - old_num) / old_num * 100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Cow;

    #[test]
    fn test_exact_equality() {
        let comparator = CellComparator::default();
        
        assert!(comparator.equal(&CellValue::Int(42), &CellValue::Int(42)));
        assert!(!comparator.equal(&CellValue::Int(42), &CellValue::Int(43)));
        
        assert!(comparator.equal(
            &CellValue::String(Cow::Owned("hello".into())),
            &CellValue::String(Cow::Owned("hello".into()))
        ));
    }

    #[test]
    fn test_case_insensitive() {
        let comparator = CellComparator::new(true, false, None);
        
        assert!(comparator.equal(
            &CellValue::String(Cow::Owned("Hello".into())),
            &CellValue::String(Cow::Owned("hello".into()))
        ));
    }

    #[test]
    fn test_numeric_tolerance() {
        let comparator = CellComparator::new(false, false, Some(0.01));
        
        assert!(comparator.equal(&CellValue::Float(1.0), &CellValue::Float(1.005)));
        assert!(!comparator.equal(&CellValue::Float(1.0), &CellValue::Float(1.02)));
    }

    #[test]
    fn test_percentage_change() {
        assert_eq!(
            percentage_change(&CellValue::Int(100), &CellValue::Int(150)),
            Some(50.0)
        );
        assert_eq!(
            percentage_change(&CellValue::Float(100.0), &CellValue::Float(80.0)),
            Some(-20.0)
        );
    }
}
