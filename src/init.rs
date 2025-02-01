use std::{
    env::current_dir,
    fs::{exists, OpenOptions},
    io::Write,
    process::exit,
};

pub fn init() {
    let dir = current_dir().expect("No dir?");
    let dot_git_path = dir.join(".git");

    if exists(&dot_git_path).is_err() {
        println!("The current dir doesn't contain a git project");
        exit(2);
    }

    let exclude_path = dot_git_path.join("info").join("exclude");
    match OpenOptions::new().append(true).open(exclude_path) {
        Ok(mut file) => {
            file.write(b"dndn");
        }
        Err(error) => {
            println!("Couldn't open the exclude file: {}", error);
            exit(2);
        }
    }
}
