use std::{
    fs::{read_to_string, File, OpenOptions},
    io::{Error, Write},
    path::PathBuf,
};

use crate::entry::Entry;

pub fn get_entries(path: &PathBuf) -> Option<Vec<Entry>> {
    let mut entries = Vec::new();

    for entry_string in read_to_string(path).ok()?.split("\n\n") {
        match Entry::deserialize(entry_string) {
            Err(error) => println!("Couldn't parse the dndn file: {}", error),
            Ok(entry) => entries.push(entry),
        }
    }

    return Some(entries);
}

pub fn add_entry(path: &PathBuf, entry: &Entry) -> Result<(), Error> {
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    let serialized = entry.serialize();
    file.write(&serialized.into_bytes());
    return Ok(());
}
