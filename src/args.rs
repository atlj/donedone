use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    /// Defaults to `./dndn`
    #[arg(short, long, value_name = "destination")]
    pub file_path: Option<PathBuf>,
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Add a new item
    Add {
        /// Path to the file
        file_path: PathBuf,
        /// Which line to add the todo to
        line: usize,
        /// Comment for the entry
        comment: Option<String>,
    },
    /// Sets up dndn in a way to not affect git
    Setup {},
    Remove {
        index: usize,
    },
}
