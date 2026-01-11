# datadiff

A high-performance CLI tool for semantic diffing of tabular data files with native Git integration.

## Features

- **Multi-format support**: CSV, Excel (.xlsx, .xls, .ods), Parquet, and JSON
- **Semantic comparison**: Understands rows and cells, not just lines
- **Key-based matching**: Match rows by primary key columns for accurate diffs
- **Multiple output formats**: Terminal (colored), JSON, HTML, Unified (Git-style)
- **Smart type inference**: Automatically detects numeric, date, and boolean types
- **Configurable comparison**: Case-insensitive, whitespace-insensitive, numeric tolerance
- **Git integration**: Use as an external diff driver for Git

## Installation

```bash
cargo install --path .
```

Or build from source:

```bash
cargo build --release
./target/release/datadiff --help
```

## Usage

### Basic Usage

```bash
# Compare two CSV files
datadiff old.csv new.csv

# Specify key columns for row matching
datadiff old.csv new.csv --key=id
datadiff old.csv new.csv --key=name,date
```

### Output Formats

```bash
# Colored terminal output (default)
datadiff old.csv new.csv --format=terminal

# JSON output for programmatic use
datadiff old.csv new.csv --format=json

# HTML report
datadiff old.csv new.csv --format=html > diff.html

# Git-style unified diff
datadiff old.csv new.csv --format=unified
```

### Comparison Options

```bash
# Ignore case when comparing strings
datadiff old.csv new.csv --ignore-case

# Set numeric tolerance for float comparisons
datadiff old.csv new.csv --numeric-tolerance=0.001

# Ignore leading/trailing whitespace
datadiff old.csv new.csv --ignore-whitespace

# Ignore specific columns
datadiff old.csv new.csv --ignore-column=timestamp

# Normalize row order before diffing
datadiff old.csv new.csv --sort-by=id
```

### Excel Files

```bash
# Compare specific sheet
datadiff old.xlsx new.xlsx --sheet="Sales Data"
```

### Statistics Only

```bash
# Show summary statistics without detailed changes
datadiff old.csv new.csv --stats-only
```

## Example Output

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
 datadiff: sales_q1.csv → sales_q2.csv
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Schema Changes:
  + discount_rate (new column at position 5)

Summary: +2 added, -1 removed, ~5 modified (out of 150 → 156 rows)

Added Rows:
┌────────────┬─────────────────┬─────────┬──────────┐
│ product_id │ name            │ price   │ quantity │
├────────────┼─────────────────┼─────────┼──────────┤
│ SKU-201    │ New Widget      │ 29.99   │ 100      │
│ SKU-202    │ Super Gadget    │ 149.99  │ 50       │
└────────────┴─────────────────┴─────────┴──────────┘

Removed Rows:
┌────────────┬─────────────────┬─────────┬──────────┐
│ product_id │ name            │ price   │ quantity │
├────────────┼─────────────────┼─────────┼──────────┤
│ SKU-001    │ Old Product     │ 19.99   │ 0        │
└────────────┴─────────────────┴─────────┴──────────┘

Modified Rows:
  SKU-042:
    price: 29.99 → 34.99 (+16.7%)
    quantity: 100 → 85 (-15.0%)
  
  SKU-108:
    name: Gadget v1 → Gadget v2
```

## Git Integration

### Setup as External Diff Driver

1. Install datadiff:
```bash
cargo install --path .
```

2. Configure Git:
```bash
git config --global diff.csv.command "datadiff --git-driver"
git config --global diff.xlsx.command "datadiff --git-driver"
git config --global diff.parquet.command "datadiff --git-driver"
```

3. Add to `.gitattributes`:
```
*.csv diff=csv
*.xlsx diff=xlsx
*.parquet diff=parquet
```

Now `git diff` will automatically use datadiff for tabular files!

## Exit Codes

- `0`: No differences found
- `1`: Differences found
- `2`: Error occurred

## Supported File Formats

| Format | Extensions | Notes |
|--------|------------|-------|
| CSV | `.csv`, `.tsv`, `.txt` | Auto-detects delimiter |
| Excel | `.xlsx`, `.xls`, `.xlsm`, `.ods` | Use `--sheet` to specify sheet |
| Parquet | `.parquet`, `.pq` | Uses Arrow for efficient reading |
| JSON | `.json`, `.jsonl`, `.ndjson` | Arrays of objects |

## License

MIT
