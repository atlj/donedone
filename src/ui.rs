use std::io::{self, stdout, Write};

use crossterm::{
    style::Print,
    terminal::{Clear, ClearType},
    QueueableCommand,
};

use crate::entry::Entry;

pub fn display_entries(entries: &Vec<Entry>) -> Result<(), io::Error> {
    let mut stdout = stdout();
    stdout
        .queue(Clear(ClearType::All))?
        .queue(Print("Testing 123".to_string()))?;
    stdout.flush()?;

    return Ok(());
}
