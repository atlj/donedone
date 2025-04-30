use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, BufWriter, Error, Read, Seek, Write},
    path::Path,
};

use crate::entry::Entry;

#[derive(Debug)]
pub struct EntryFileHandler {
    reader: BufReader<File>,
    writer: BufWriter<File>,
}

impl EntryFileHandler {
    pub fn from_file_path(path: &Path) -> Result<EntryFileHandler, Error> {
        let read_handle = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;

        let reader = BufReader::new(read_handle);

        let write_handler = OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(path)?;

        let writer = BufWriter::new(write_handler);

        Ok(EntryFileHandler { reader, writer })
    }

    pub fn get_entries(&mut self) -> Vec<Entry> {
        let mut entries = Vec::new();

        let mut entry_string = String::with_capacity(100);
        let mut read_head = String::with_capacity(100);

        while self
            .reader
            .read_line(&mut read_head)
            .is_ok_and(|read_count| read_count != 0)
        {
            entry_string.push_str(&read_head);
            if read_head == "\n" {
                if let Ok(entry) = Entry::deserialize(&entry_string) {
                    entries.push(entry);
                };

                entry_string.clear();
            }

            read_head.clear();
        }

        _ = self.reader.rewind().inspect_err(|err| {
            dbg!(err);
        });

        entries
    }

    pub fn add_entry(&mut self, entry: &Entry) -> Result<(), Error> {
        let serialized = entry.serialize();
        self.writer.seek(std::io::SeekFrom::End(0))?;
        _ = self.writer.write(&serialized.into_bytes())?;
        self.writer.flush()?;
        self.writer.rewind()?;
        Ok(())
    }

    pub fn remove_entry(&mut self, index_to_remove: &usize) -> Result<(), Error> {
        let mut file_contents = String::new();
        self.reader.read_to_string(&mut file_contents)?;

        let filtered_entries_string = file_contents
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

        self.writer.get_ref().set_len(0)?;
        self.writer.rewind()?;
        self.writer.write_all(filtered_entries_string.as_bytes())?;
        self.writer.flush()?;

        self.reader.rewind()?;

        Ok(())
    }

    pub fn swap_entries(&mut self, index_a: &usize, index_b: &usize) -> Result<(), Error> {
        let mut file_contents = String::new();
        self.reader.read_to_string(&mut file_contents)?;

        let mut entry_strings = file_contents.split("\n\n").collect::<Vec<_>>();

        entry_strings.swap(*index_a, *index_b);
        let swapped_entries_string = entry_strings.join("\n\n");

        self.writer.get_ref().set_len(0)?;
        self.writer.rewind()?;
        self.writer.write_all(swapped_entries_string.as_bytes())?;
        self.writer.flush()?;

        self.reader.rewind()?;

        Ok(())
    }
}
