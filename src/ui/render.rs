use std::{
    env::current_dir,
    fs::read_to_string,
    io::{stdout, Error, Stdout, Write},
    sync::mpsc::Receiver,
};

use crossterm::{
    style::{StyledContent, Stylize},
    ExecutableCommand, QueueableCommand,
};

use crate::entry::Entry;

pub struct UIState {
    pub entries: Vec<Entry>,
    pub selected_entry_index: usize,
    pub top_index: usize,
    pub y_size: u16,
    pub x_size: u16,
    pub previous_command: Option<UIMessage>,
}

type Renderable = Vec<StyledContent<String>>;

impl UIState {
    fn complete_render(&self, stdout: &mut Stdout) -> Result<(), Error> {
        let renderables = self.render_entries(self.y_size, self.x_size);

        stdout.queue(crossterm::terminal::Clear(
            crossterm::terminal::ClearType::All,
        ))?;

        for (y, contents) in renderables.into_iter().enumerate() {
            assert!(y <= self.y_size.into(), "Trying to print outside of screen");

            stdout
                .queue(crossterm::cursor::MoveToColumn(0))?
                .queue(crossterm::cursor::MoveToRow(y as u16))?
                .queue(crossterm::style::Print(contents))?;
        }

        stdout.flush()?;

        Ok(())
    }
    fn render_entries(&self, y_size: u16, x_size: u16) -> Renderable {
        self.entries
            .iter()
            .enumerate()
            .flat_map(|(index, entry)| {
                let mut content =
                    Self::render_entry(entry, index == self.selected_entry_index, y_size, x_size);

                if index != self.entries.len() - 1 {
                    content.push("\n".to_string().stylize());
                }
                return content;
            })
            .collect()
    }
    fn render_entry(entry: &Entry, highlight: bool, _y_size: u16, x_size: u16) -> Renderable {
        let mut result = Vec::new();

        if let Some(comment) = entry.comment.clone() {
            comment
                .chars()
                .collect::<Vec<_>>()
                .chunks(x_size.into())
                .map(|chunk| chunk.into_iter().collect::<String>())
                .for_each(|row| {
                    let mut content = row.bold();
                    if highlight {
                        content = content.magenta();
                    }
                    result.push(content);
                });
        }

        let current_dir = current_dir().unwrap();
        let relative_dir = entry
            .path
            .strip_prefix(current_dir)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        let mut line = format!("{}:{}", relative_dir, entry.line)
            .to_string()
            .underlined();

        if highlight {
            line = line.magenta();
        }

        result.push(line);

        if let Ok(file) = read_to_string(entry.path.clone()) {
            if let Some(line) = file.lines().nth(entry.line - 1) {
                result.push(line.trim().to_string().italic().dark_grey())
            }
        }

        return result;
    }
}

#[derive(Debug)]
pub enum UIMessage {
    Exit,
    MoveDown,
    MoveUp,
    SwapEntryUp,
    SwapEntryDown,
    NextPage,
    PreviousPage,
    DeleteSelectedEntry,
    SyncEntries { entries: Vec<Entry> },
    Resize { y_size: u16, x_size: u16 },
}

pub fn render_loop(mut render_state: UIState, receiver: Receiver<UIMessage>) -> Result<(), Error> {
    let mut stdout = stdout();

    stdout
        .queue(crossterm::terminal::EnterAlternateScreen)?
        .queue(crossterm::cursor::Hide)?;

    stdout.flush()?;

    render_state.complete_render(&mut stdout)?;

    while let Ok(message) = receiver.recv() {
        match message {
            UIMessage::MoveDown => {
                if render_state.selected_entry_index < render_state.entries.len() {
                    render_state.selected_entry_index += 1;
                    render_state.complete_render(&mut stdout)?;
                }
            }
            UIMessage::MoveUp => {
                if render_state.selected_entry_index > 0 {
                    render_state.selected_entry_index -= 1;
                    render_state.complete_render(&mut stdout)?;
                }
            }
            UIMessage::Exit => {
                stdout.execute(crossterm::terminal::LeaveAlternateScreen);
                return Ok(());
            }
            _ => {}
        }
    }

    stdout.execute(crossterm::terminal::LeaveAlternateScreen);
    panic!("Sender channel closed unexpectedly");
}
