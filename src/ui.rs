use std::io::{self, stdout, Stdout, Write};

use crossterm::{
    cursor,
    event::Event,
    style::{self, Print, PrintStyledContent},
    terminal::{self, enable_raw_mode},
    QueueableCommand,
};

use crate::entry::Entry;

pub fn display_entries(entries: &Vec<Entry>) -> Result<(), io::Error> {
    let mut stdout = stdout();
    stdout
        .queue(terminal::EnterAlternateScreen)?
        .queue(terminal::Clear(terminal::ClearType::All))?
        .queue(cursor::Hide)?;
    stdout.flush()?;

    render(&mut stdout, entries);
    stdout.flush();

    loop {
        match crossterm::event::read()? {
            Event::Resize(y, x) => {}
            _ => {}
        }
    }

    return Ok(());
}

fn render(stdout: &mut Stdout, entries: &Vec<Entry>) -> Result<(), io::Error> {
    let (x_size, y_size) = terminal::size()?;
    let comment = entries.get(0).unwrap().comment.clone().unwrap();

    stdout
        .queue(cursor::SavePosition)?
        .queue(cursor::MoveToRow(y_size / 2))?
        .queue(cursor::MoveToColumn((x_size - (comment.len() as u16)) / 2))?
        .queue(style::Print(comment))?;

    return Ok(());
}
