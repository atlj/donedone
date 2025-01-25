use std::{
    fs::{read_to_string, OpenOptions},
    io::{Error, Read, Seek, Write},
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
    file.write(&serialized.into_bytes())?;
    return Ok(());
}

pub fn remove_entry(path: &PathBuf, index_to_remove: &usize) -> Result<(), Error> {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .append(false)
        .open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let contents = dbg!(contents);

    let entries = contents
        .split("\n\n")
        .enumerate()
        .filter_map(|(index, entry)| {
            if &index == index_to_remove {
                None
            } else {
                Some(entry)
            }
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    let entries = dbg!(entries);

    file.set_len(0)?;
    file.rewind()?;
    file.write(entries.as_bytes())?;
    file.flush()?;

    return Ok(());
}
