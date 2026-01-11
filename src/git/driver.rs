//! Git diff driver mode

use std::path::PathBuf;

use anyhow::Result;

use crate::config::{Config, OutputFormat};
use crate::diff::compute_diff;
use crate::output::render_to_stdout;
use crate::parser::ParserFactory;

/// Arguments for git diff driver mode
/// Git calls: tool old_file old_hex old_mode new_file new_hex new_mode
#[derive(Debug)]
pub struct GitDriverArgs {
    pub old_file: PathBuf,
    pub old_hex: String,
    pub old_mode: String,
    pub new_file: PathBuf,
    pub new_hex: String,
    pub new_mode: String,
}

impl GitDriverArgs {
    /// Parse git driver arguments
    pub fn parse(args: &[String]) -> Option<Self> {
        if args.len() < 7 {
            return None;
        }

        Some(Self {
            old_file: PathBuf::from(&args[1]),
            old_hex: args[2].clone(),
            old_mode: args[3].clone(),
            new_file: PathBuf::from(&args[4]),
            new_hex: args[5].clone(),
            new_mode: args[6].clone(),
        })
    }
}

/// Run datadiff as a git diff driver
pub fn run_git_driver(args: &GitDriverArgs) -> Result<()> {
    let config = Config {
        old_file: args.old_file.clone(),
        new_file: args.new_file.clone(),
        output_format: OutputFormat::Unified,
        git_driver_mode: true,
        ..Default::default()
    };

    let factory = ParserFactory::new();

    // Parse both files
    let old_table = factory.parse(&args.old_file, &config)?;
    let new_table = factory.parse(&args.new_file, &config)?;

    // Compute diff
    let diff = compute_diff(&old_table, &new_table, &config);

    // Output in unified format for git
    render_to_stdout(
        &diff,
        &old_table,
        &new_table,
        &args.old_file,
        &args.new_file,
        OutputFormat::Unified,
    )?;

    Ok(())
}
