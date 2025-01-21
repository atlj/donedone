use clap::Parser;
use donedone::{args::Args, hello};

fn main() {
    let args = Args::parse();
    dbg!(args);
}
