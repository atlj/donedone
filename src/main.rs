use std::env::current_dir;

use clap::Parser;
use donedone::{
    args::{Args, Commands},
    entry::Entry,
    file::{add_entry, remove_entry},
    init::init,
    ui::start_gui_mode,
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
            start_gui_mode(&args.file_path.unwrap());
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
        Some(Commands::Init {}) => {
            init();
        }
        Some(Commands::Remove { index }) => {
            remove_entry(&args.file_path.unwrap(), &index).unwrap();
        }
    }
}
