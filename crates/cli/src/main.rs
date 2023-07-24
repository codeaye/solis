use args::*;
use commands::*;

use clap::Parser;
use colored::Colorize;
use std::io::Write;

mod args;
mod commands;
mod utils;

fn main() {
    let args = SolisArgs::parse();

    std::env::set_var("RUST_LOG", args.log_level.to_uppercase());

    let mut builder = env_logger::Builder::from_default_env();
    builder
        .format(|buf, record| {
            writeln!(
                buf,
                "{}: {}",
                match record.level() {
                    log::Level::Error => "error".red(),
                    log::Level::Warn => "warn".yellow(),
                    log::Level::Info => "info".green(),
                    log::Level::Debug => "debug".blue(),
                    log::Level::Trace => "trace".purple(),
                }
                .italic(),
                record.args()
            )
        })
        .init();

    match args.file_path {
        Some(file_path) => run(file_path),
        _ => repl(),
    }
}
