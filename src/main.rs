//! datadiff - Semantic diff for tabular data

use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};

use datadiff::config::{Config, OutputFormat};
use datadiff::diff::compute_diff;
use datadiff::git::{run_git_driver, GitDriverArgs};
use datadiff::output::render_to_stdout;
use datadiff::parser::ParserFactory;

#[derive(Debug, Clone, Copy, ValueEnum)]
enum CliOutputFormat {
    Terminal,
    Json,
    Html,
    Unified,
}

impl From<CliOutputFormat> for OutputFormat {
    fn from(f: CliOutputFormat) -> Self {
        match f {
            CliOutputFormat::Terminal => OutputFormat::Terminal,
            CliOutputFormat::Json => OutputFormat::Json,
            CliOutputFormat::Html => OutputFormat::Html,
            CliOutputFormat::Unified => OutputFormat::Unified,
        }
    }
}

/// Semantic diff for tabular data (CSV, Excel, Parquet, JSON)
#[derive(Parser, Debug)]
#[command(name = "datadiff")]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Old/original file to compare
    #[arg(required_unless_present = "git_driver")]
    old_file: Option<PathBuf>,

    /// New file to compare
    #[arg(required_unless_present = "git_driver")]
    new_file: Option<PathBuf>,

    /// Column(s) to use as primary key for row matching (comma-separated)
    #[arg(short, long, value_delimiter = ',')]
    key: Vec<String>,

    /// Output format
    #[arg(short, long, value_enum, default_value = "terminal")]
    format: CliOutputFormat,

    /// Ignore case when comparing string values
    #[arg(long)]
    ignore_case: bool,

    /// Tolerance for numeric comparisons (e.g., 0.001)
    #[arg(long)]
    numeric_tolerance: Option<f64>,

    /// Ignore leading/trailing whitespace in string values
    #[arg(long)]
    ignore_whitespace: bool,

    /// Column(s) to ignore in comparison (comma-separated)
    #[arg(long, value_delimiter = ',')]
    ignore_column: Vec<String>,

    /// Column to sort by before diffing (normalizes order)
    #[arg(long)]
    sort_by: Option<String>,

    /// For Excel files: which sheet to compare
    #[arg(long)]
    sheet: Option<String>,

    /// Only show statistics, not detailed changes
    #[arg(long)]
    stats_only: bool,

    /// Run as git diff driver (internal use)
    #[arg(long, hide = true)]
    git_driver: bool,

    /// Additional arguments for git driver mode
    #[arg(trailing_var_arg = true, hide = true)]
    git_args: Vec<String>,
}

fn main() -> ExitCode {
    match run() {
        Ok(has_changes) => {
            if has_changes {
                ExitCode::from(1) // Differences found
            } else {
                ExitCode::SUCCESS // No differences
            }
        }
        Err(e) => {
            eprintln!("Error: {:#}", e);
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<bool> {
    let cli = Cli::parse();

    // Handle git driver mode
    if cli.git_driver {
        let mut args = vec![String::new()]; // Placeholder for program name
        args.extend(cli.git_args);
        
        if let Some(git_args) = GitDriverArgs::parse(&args) {
            run_git_driver(&git_args)?;
            return Ok(true);
        } else {
            anyhow::bail!("Invalid git driver arguments");
        }
    }

    // Normal diff mode
    let old_file = cli.old_file.context("old_file is required")?;
    let new_file = cli.new_file.context("new_file is required")?;

    let config = Config {
        old_file: old_file.clone(),
        new_file: new_file.clone(),
        key_columns: cli.key,
        output_format: cli.format.into(),
        ignore_case: cli.ignore_case,
        numeric_tolerance: cli.numeric_tolerance,
        ignore_whitespace: cli.ignore_whitespace,
        ignore_columns: cli.ignore_column,
        sort_by: cli.sort_by,
        sheet_name: cli.sheet,
        stats_only: cli.stats_only,
        git_driver_mode: false,
    };

    // Parse files
    let factory = ParserFactory::new();
    
    let mut old_table = factory
        .parse(&old_file, &config)
        .with_context(|| format!("Failed to parse old file: {}", old_file.display()))?;
    
    let mut new_table = factory
        .parse(&new_file, &config)
        .with_context(|| format!("Failed to parse new file: {}", new_file.display()))?;

    // Set key columns if specified
    if !config.key_columns.is_empty() {
        old_table.set_key_columns(&config.key_columns);
        new_table.set_key_columns(&config.key_columns);
    }

    // Compute diff
    let diff = compute_diff(&old_table, &new_table, &config);

    // Handle stats-only mode
    if config.stats_only {
        println!("Old file: {} ({} rows)", old_file.display(), diff.stats.old_row_count);
        println!("New file: {} ({} rows)", new_file.display(), diff.stats.new_row_count);
        println!();
        println!("Added:     {}", diff.stats.rows_added);
        println!("Removed:   {}", diff.stats.rows_removed);
        println!("Modified:  {}", diff.stats.rows_modified);
        println!("Unchanged: {}", diff.stats.rows_unchanged);
        println!("Cells changed: {}", diff.stats.cells_changed);
        return Ok(diff.has_changes());
    }

    // Render output
    render_to_stdout(
        &diff,
        &old_table,
        &new_table,
        &old_file,
        &new_file,
        config.output_format,
    )?;

    Ok(diff.has_changes())
}
