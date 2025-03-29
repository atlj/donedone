use std::env::current_dir;

use clap::Parser;
use donedone::{
    args::{Args, Commands},
    entry::Entry,
    file::EntryFileHandler,
    init::init,
    log::LogError,
    ui::start_gui_mode,
};

fn main() {
    let mut args = Args::parse();

    if args.file_path.is_none() {
        let mut path = current_dir().expect("No dir?");
        path.push("dndn");

        args.file_path = Some(path);
    }

    let mut file_handler = EntryFileHandler::from_file_path(&args.file_path.unwrap()).unwrap();

    match args.command {
        None => {
            start_gui_mode(file_handler).log_if_err();
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

            file_handler.add_entry(&entry).log_if_err();
        }
        Some(Commands::Init {}) => {
            init().log_if_err();
        }
        Some(Commands::Remove { index }) => {
            file_handler.remove_entry(&index).unwrap();
        }
    }
}
