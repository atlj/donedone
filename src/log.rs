use std::{env::current_dir, fs::OpenOptions, io::Write};

pub fn log(message: &str) {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(current_dir().unwrap().join("dndn_log.txt"))
        .unwrap();

    _ = write!(file, "\n{}", message);
}

pub trait LogError<T, E> {
    fn log_if_err(self);
}

impl<T, E: std::fmt::Debug> LogError<T, E> for Result<T, E> {
    fn log_if_err(self) {
        if let Err(ref e) = self {
            log(format!("Error occurred: {:?}", e).as_str());
        }
    }
}
