use std::{
    env::current_dir,
    fs::{exists, OpenOptions},
    io::{BufRead, BufReader, Error, Write},
    process::exit,
};

pub fn init() -> Result<(), Error> {
    let dir = current_dir().expect("No dir?");
    let dot_git_path = dir.join(".git");

    if exists(&dot_git_path).is_err() {
        println!("The current dir doesn't contain a git project");
        exit(2);
    }

    let exclude_path = dot_git_path.join("info").join("exclude");
    let mut git_exclude_file = OpenOptions::new()
        .read(true)
        .append(true)
        .open(exclude_path)?;
    let mut reader = BufReader::new(&git_exclude_file);

    let mut line = String::new();
    while reader.read_line(&mut line)? != 0 {
        if line.trim() == "dndn" {
            println!("⚠ dndn has already been initialized in this project");
            exit(2);
        }
        line.clear();
    }

    git_exclude_file.write(b"\ndndn")?;

    println!("✅ Successfully initialized dndn");

    return Ok(());
}
