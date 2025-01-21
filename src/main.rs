use std::{env::current_dir, path::PathBuf};

use clap::Parser;
use donedone::{
    args::Args,
    entry::Entry,
    file::{add_entry, get_entries},
};

fn main() {
    let mut args = Args::parse();

    if let None = args.file_path {
        let mut path = current_dir().expect("No dir?");
        path.push("dndn");

        args.file_path = Some(path);
    }

    let entry = Entry {
        path: PathBuf::from("./"),
        line: 3,
        comment: Some("Testing 123".to_string()),
    };

    let entries = get_entries(&args.file_path.unwrap());
    dbg!(entries);
}
