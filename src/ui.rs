use std::{
    io::{self, stdout, Stdout, Write},
    path::PathBuf,
    process::exit,
    usize,
};

use crossterm::{
    cursor,
    event::{Event, KeyCode, KeyEvent, KeyModifiers},
    style::{self, Print, PrintStyledContent},
    terminal::{self, enable_raw_mode},
    QueueableCommand,
};

use crate::{
    entry::Entry,
    file::{get_entries, remove_entry},
};

pub fn display_entries(path: &PathBuf) -> Result<(), io::Error> {
    let mut entries = get_entries(path).expect("No Entries");
    let mut stdout = stdout();
    crossterm::terminal::enable_raw_mode()?;
    stdout
        .queue(terminal::EnterAlternateScreen)?
        .queue(terminal::Clear(terminal::ClearType::All))?
        .queue(cursor::Hide)?;
    stdout.flush()?;

    let mut selected_entry_index = 0;

    render(&mut stdout, &entries, &selected_entry_index);

    loop {
        match crossterm::event::read()? {
            Event::Resize(y, x) => {
                render(&mut stdout, &entries, &selected_entry_index);
            }
            Event::Key(key_event) => match key_event.code {
                KeyCode::Char('c') => {
                    if key_event.modifiers == KeyModifiers::CONTROL {
                        exit(0);
                    }
                }
                KeyCode::Char('j') => {
                    if selected_entry_index < entries.len() {
                        selected_entry_index += 1;
                        render(&mut stdout, &entries, &selected_entry_index)?;
                    }
                }
                KeyCode::Char('k') => {
                    if selected_entry_index > 0 {
                        selected_entry_index -= 1;
                        render(&mut stdout, &entries, &selected_entry_index)?;
                    }
                }
                KeyCode::Char('d') => {
                    if matches!(
                        crossterm::event::read()?,
                        Event::Key(KeyEvent {
                            code: KeyCode::Char('d'),
                            ..
                        })
                    ) {
                        remove_entry(path, &selected_entry_index);
                        entries = get_entries(path).expect("No Entries");
                        render(&mut stdout, &entries, &selected_entry_index)?;
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    return Ok(());
}

fn render(
    stdout: &mut Stdout,
    entries: &Vec<Entry>,
    selected_entry_index: &usize,
) -> Result<(), io::Error> {
    let (x_size, y_size) = terminal::size()?;
    let comments = entries
        .into_iter()
        .map(|entry| entry.comment.clone().unwrap_or("".to_string()));
    let count = comments.clone().count();
    stdout.queue(terminal::Clear(terminal::ClearType::All))?;

    for (index, comment) in comments.enumerate() {
        if index == *selected_entry_index {
            stdout.queue(style::SetForegroundColor(style::Color::Green))?;
        }
        stdout
            .queue(cursor::MoveToRow(
                ((y_size - (count) as u16) / 2) + index as u16,
            ))?
            .queue(cursor::MoveToColumn((x_size - (comment.len() as u16)) / 2))?
            .queue(style::Print(comment))?
            .queue(style::ResetColor);
    }

    stdout.flush();

    return Ok(());
}
