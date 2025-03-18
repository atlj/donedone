use std::{env::current_dir, fs::OpenOptions, io::Write};

pub fn log(message: &str) {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(current_dir().unwrap().join("dndn_log.txt"))
        .unwrap();

    write!(file, "\n{}", message);
}
