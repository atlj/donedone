use std::{
    env::current_dir,
    fs::read_to_string,
    io::{stdout, Error, Stdout, Write},
    path::PathBuf,
    sync::mpsc::Receiver,
};

use crossterm::{
    style::{StyledContent, Stylize},
    ExecutableCommand, QueueableCommand,
};

use crate::{
    entry::Entry,
    file::{get_entries, remove_entry, swap_entries},
};

pub struct UIState {
    entries: Vec<Entry>,
    selected_entry_index: usize,
    top_index: usize,
    bottom_index: usize,
    y_size: u16,
    x_size: u16,
    previous_command: Option<UIMessage>,
}

type Renderable = Vec<StyledContent<String>>;

impl UIState {
    pub fn new(entries: Vec<Entry>, x_size: u16, y_size: u16) -> Self {
        Self {
            entries,
            selected_entry_index: 0,
            top_index: 0,
            bottom_index: 0,
            y_size,
            x_size,
            previous_command: None,
        }
    }
    fn complete_render(&mut self, stdout: &mut Stdout) -> Result<(), Error> {
        let renderables = self.render_entries(self.y_size, self.x_size - 10);

        stdout.queue(crossterm::terminal::Clear(
            crossterm::terminal::ClearType::All,
        ))?;

        for (y, contents) in renderables.into_iter().enumerate() {
            assert!(y <= self.y_size.into(), "Trying to print outside of screen");

            stdout
                .queue(crossterm::cursor::MoveToColumn(5))?
                .queue(crossterm::cursor::MoveToRow(y as u16))?
                .queue(crossterm::style::Print(contents))?;
        }

        stdout.flush()?;

        Ok(())
    }

    fn render_entries(&mut self, y_size: u16, x_size: u16) -> Renderable {
        let mut result: Renderable = vec![];

        let mut current_index = self.top_index;
        while let Some(entry) = self.entries.get(current_index) {
            if result.len() >= y_size.into() {
                break;
            }

            let mut content = Self::render_entry(
                entry,
                current_index == self.selected_entry_index,
                y_size,
                x_size,
            );

            if current_index != self.entries.len() - 1 {
                content.push("\n".to_string().stylize());
            }

            if result.len() + content.len() > y_size.into() {
                break;
            }

            result.append(&mut content);
            current_index += 1;
        }

        self.bottom_index = self.top_index + result.len();

        return result;
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

#[derive(Debug, Clone)]
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

pub fn render_loop(
    entry_path: &PathBuf,
    mut render_state: UIState,
    receiver: Receiver<UIMessage>,
) -> Result<(), Error> {
    let mut stdout = stdout();

    stdout
        .queue(crossterm::cursor::Hide)?
        .queue(crossterm::terminal::EnterAlternateScreen)?;

    stdout.flush()?;

    render_state.complete_render(&mut stdout)?;

    while let Ok(message) = receiver.recv() {
        match message {
            UIMessage::MoveDown => {
                if render_state.selected_entry_index < render_state.entries.len() - 1 {
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
            UIMessage::SwapEntryUp => {
                if render_state.selected_entry_index > 0 {
                    swap_entries(
                        entry_path,
                        &render_state.selected_entry_index,
                        &(render_state.selected_entry_index - 1),
                    )?;

                    render_state.entries.swap(
                        render_state.selected_entry_index,
                        render_state.selected_entry_index - 1,
                    );

                    render_state.selected_entry_index -= 1;

                    render_state.complete_render(&mut stdout)?;
                }
            }
            UIMessage::SwapEntryDown => {
                if render_state.selected_entry_index < render_state.entries.len() - 1 {
                    swap_entries(
                        entry_path,
                        &render_state.selected_entry_index,
                        &(render_state.selected_entry_index + 1),
                    )?;

                    render_state.entries.swap(
                        render_state.selected_entry_index,
                        render_state.selected_entry_index + 1,
                    );

                    render_state.selected_entry_index += 1;

                    render_state.complete_render(&mut stdout)?;
                }
            }
            UIMessage::DeleteSelectedEntry => {
                if matches!(
                    render_state.previous_command,
                    Some(UIMessage::DeleteSelectedEntry)
                ) {
                    remove_entry(entry_path, &render_state.selected_entry_index)?;
                    if let Some(entries) = get_entries(entry_path) {
                        render_state.entries = entries;

                        if render_state.selected_entry_index > 0 {
                            render_state.selected_entry_index -= 1;
                        }
                    }
                    render_state.complete_render(&mut stdout)?;
                }
            }
            UIMessage::Exit => {
                stdout.execute(crossterm::terminal::LeaveAlternateScreen)?;
                return Ok(());
            }
            _ => {}
        }

        render_state.previous_command = Some(message.clone());
    }

    stdout.execute(crossterm::terminal::LeaveAlternateScreen)?;
    panic!("Sender channel closed unexpectedly");
}
