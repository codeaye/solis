use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct SolisArgs {
    /// The log level to use
    #[arg(short, long, default_value = "INFO")]
    pub log_level: String,
    /// The path of the file to run (optional)
    #[arg(short, long)]
    pub file_path: Option<PathBuf>,
}
