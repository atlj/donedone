use std::{env::current_dir, path::PathBuf};

use clap::Parser;
use donedone::{
    args::{Args, Commands},
    entry::Entry,
    file::{add_entry, get_entries},
    setup::setup,
    ui::display_entries,
};

fn main() {
    let mut args = Args::parse();

    if args.file_path.is_none() {
        let mut path = current_dir().expect("No dir?");
        path.push("dndn");

        args.file_path = Some(path);
    }

    match args.command {
        None => {
            let entries = get_entries(&args.file_path.unwrap()).expect("No Entries");
            display_entries(&entries);
        }
        Some(Commands::Add {
            file_path,
            line,
            comment,
        }) => {
            let entry = Entry {
                comment,
                path: file_path,
                line,
            };

            add_entry(&args.file_path.unwrap(), &entry);
        }
        Some(Commands::Setup {}) => {
            setup();
        }
    }
}
