use std::env::current_dir;

use clap::Parser;
use donedone::args::Args;

fn main() {
    let mut args = Args::parse();

    if let None = args.file_path {
        let mut path = current_dir().expect("No dir?");
        path.push("dndn");

        args.file_path = Some(path);
    }

    dbg!(args);
}
