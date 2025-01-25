use std::{
    env::current_dir,
    fs::read_to_string,
    io::{self, stdout, Stdout, Write},
    os::unix::fs::FileExt,
    path::PathBuf,
    usize,
};

use clap::builder::StyledStr;
use crossterm::{
    cursor,
    event::{Event, KeyCode, KeyEvent, KeyModifiers},
    style::{self, StyledContent, Stylize},
    terminal::{self, disable_raw_mode},
    ExecutableCommand, QueueableCommand,
};

use crate::{
    entry::Entry,
    file::{get_entries, remove_entry, swap_entries},
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

    render(&mut stdout, &entries, &selected_entry_index)?;

    loop {
        match crossterm::event::read()? {
            Event::Resize(y, x) => {
                render(&mut stdout, &entries, &selected_entry_index)?;
            }
            Event::Key(key_event) => match key_event.code {
                KeyCode::Char('c') => {
                    if key_event.modifiers == KeyModifiers::CONTROL {
                        break;
                    }
                }
                KeyCode::Char('q') => {
                    break;
                }
                KeyCode::Esc => {
                    break;
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
                KeyCode::Char('h') => {
                    if selected_entry_index > 0 {
                        swap_entries(path, &selected_entry_index, &(selected_entry_index - 1))?;
                        selected_entry_index -= 1;
                        entries = get_entries(path).expect("No Entries");
                        render(&mut stdout, &entries, &selected_entry_index)?;
                    }
                }
                KeyCode::Char('l') => {
                    if selected_entry_index < entries.len() {
                        swap_entries(path, &selected_entry_index, &(selected_entry_index + 1))?;
                        selected_entry_index += 1;
                        entries = get_entries(path).expect("No Entries");
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
                        remove_entry(path, &selected_entry_index)?;
                        entries = get_entries(path).expect("No Entries");
                        render(&mut stdout, &entries, &selected_entry_index)?;
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    disable_raw_mode()?;
    stdout.execute(terminal::LeaveAlternateScreen)?;
    Ok(())
}

fn render(
    stdout: &mut Stdout,
    entries: &Vec<Entry>,
    selected_entry_index: &usize,
) -> Result<(), io::Error> {
    let (x_size, y_size) = terminal::size()?;
    let count = entries.len();
    stdout.queue(terminal::Clear(terminal::ClearType::All))?;

    for (entry_index, entry) in entries.into_iter().enumerate() {
        for (index, ui_content) in entry.get_ui_contents().into_iter().enumerate() {
            if entry_index == *selected_entry_index {
                stdout.queue(style::SetForegroundColor(style::Color::Magenta))?;
            }
            stdout
                .queue(cursor::MoveToRow(
                    ((y_size - (count * 4) as u16) / 2) + ((entry_index * 4) + index) as u16,
                ))?
                .queue(cursor::MoveToColumn(4))?
                .queue(style::Print(ui_content))?;
        }

        stdout.queue(style::ResetColor)?;
    }

    stdout.flush()?;

    Ok(())
}

impl Entry {
    fn get_ui_contents(&self) -> Vec<StyledContent<String>> {
        let mut result = Vec::new();

        if let Some(comment) = self.comment.clone() {
            result.push(comment.bold());
        }

        let current_dir = current_dir().unwrap();
        let relative_dir = self
            .path
            .strip_prefix(current_dir)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        result.push(
            format!("{}:{}", relative_dir, self.line)
                .to_string()
                .underlined(),
        );

        if let Ok(file) = read_to_string(self.path.clone()) {
            if let Some(line) = file.lines().nth(self.line) {
                result.push(line.trim().to_string().italic().dark_grey())
            }
        }

        return result;
    }
}
