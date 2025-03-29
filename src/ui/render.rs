use std::{
    cmp::min,
    env::current_dir,
    fs::OpenOptions,
    io::{stdout, BufRead, BufReader, BufWriter, Error, Stdout, Write},
    sync::mpsc::Receiver,
};

use crossterm::{
    style::{StyledContent, Stylize},
    ExecutableCommand, QueueableCommand,
};

use crate::{entry::Entry, file::EntryFileHandler};

use super::io::Action;

pub struct UIState {
    entries: Vec<Entry>,
    selected_entry_index: usize,
    top_index: usize,
    bottom_index: usize,
    y_size: u16,
    x_size: u16,
    previous_action: Option<Action>,
}

type Renderable = Vec<StyledContent<String>>;

const AVERAGE_ENTRY_LENGTH: usize = 3;
const JUMP_AMOUNT: usize = 5;
const SCROLL_THRESHOLD_ITEMS: usize = 2;
const MAXIMUM_HORIZONTAL_CHARACTERS: usize = 80;

impl UIState {
    pub fn new(entries: Vec<Entry>, x_size: u16, y_size: u16) -> Self {
        Self {
            entries,
            selected_entry_index: 0,
            top_index: 0,
            bottom_index: 0,
            y_size,
            x_size,
            previous_action: None,
        }
    }

    fn complete_render(&mut self, stdout: &mut BufWriter<Stdout>) -> Result<(), Error> {
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

        let scroll_bar = self.render_scroll_bar(self.y_size - 6, self.y_size);

        for (y, contents) in scroll_bar.into_iter().enumerate() {
            assert!(y <= self.y_size.into(), "Trying to print outside of screen");

            stdout
                .queue(crossterm::cursor::MoveToColumn(self.x_size - 2))?
                .queue(crossterm::cursor::MoveToRow(y as u16 + 2))?
                .queue(crossterm::style::Print(contents))?;
        }

        let selected_index_indicator = self.render_selected_index();

        let mut column = self.x_size - 1;

        for contents in selected_index_indicator.into_iter().rev() {
            column -= contents.content().len() as u16;

            stdout
                .queue(crossterm::cursor::MoveToColumn(column))?
                .queue(crossterm::cursor::MoveToRow(0))?
                .queue(crossterm::style::Print(contents))?;
        }

        stdout.flush()?;

        Ok(())
    }

    fn render_entries(&mut self, y_size: u16, x_size: u16) -> Renderable {
        let mut result: Renderable = vec![];

        let mut current_index = self.top_index;
        let mut displayed_entries = 0;
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
            displayed_entries += 1;
            current_index += 1;
        }

        self.bottom_index = self.top_index + displayed_entries;

        result
    }

    fn render_entry(entry: &Entry, highlight: bool, _y_size: u16, x_size: u16) -> Renderable {
        let mut result = Vec::new();

        if let Some(comment) = entry.comment.clone() {
            comment
                .chars()
                .collect::<Vec<_>>()
                .chunks(x_size.into())
                .map(|chunk| chunk.iter().collect::<String>())
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

        if let Ok(file_handle) = OpenOptions::new().read(true).open(&entry.path) {
            let buf_reader = BufReader::new(file_handle);
            if let Some(Ok(line)) = buf_reader.lines().nth(entry.line - 1) {
                let mut line = line.trim().to_string();
                line.truncate(MAXIMUM_HORIZONTAL_CHARACTERS);
                result.push(line.italic().dark_grey());
            }
        }

        result
    }

    fn render_scroll_bar(&self, y_size: u16, entry_window_y_size: u16) -> Renderable {
        let mut result: Renderable = vec![];

        if (self.entries.len() * AVERAGE_ENTRY_LENGTH) < (entry_window_y_size + 1).into() {
            return result;
        }

        let displayed_entries =
            (entry_window_y_size as f32 / AVERAGE_ENTRY_LENGTH as f32).round() as usize - 1;

        let thumb_size_ratio = displayed_entries as f32 / self.entries.len() as f32;
        let thumb_size = (thumb_size_ratio * y_size as f32).round() as usize;

        let thumb_start_y = min(
            (y_size as f32 * (self.top_index as f32 / (self.entries.len() as f32))).round()
                as usize,
            (y_size as usize) - thumb_size,
        );

        for _ in 0..thumb_start_y {
            result.push("┃".to_string().stylize());
        }

        for _ in 0..thumb_size {
            result.push("▓".to_string().stylize());
        }

        for _ in 0..(y_size as usize - thumb_size - thumb_start_y) {
            result.push("┃".to_string().stylize());
        }

        result
    }

    fn render_selected_index(&self) -> Renderable {
        vec![
            (self.selected_entry_index + 1).to_string().magenta(),
            "/".to_string().stylize(),
            self.entries.len().to_string().stylize(),
        ]
    }
}

pub fn render_loop(
    mut file_handler: EntryFileHandler,
    io_action_receiver: Receiver<Action>,
) -> Result<(), Error> {
    let (initial_x_size, initial_y_size) = crossterm::terminal::size()?;
    let mut render_state = UIState::new(file_handler.get_entries(), initial_x_size, initial_y_size);
    let mut stdout = BufWriter::new(stdout());

    stdout
        .queue(crossterm::cursor::Hide)?
        .queue(crossterm::terminal::EnterAlternateScreen)?;

    stdout.flush()?;

    render_state.complete_render(&mut stdout)?;

    while let Ok(action) = io_action_receiver.recv() {
        let mut previous_action_to_save = Some(action.clone());

        match action {
            Action::MoveDown => {
                if render_state.selected_entry_index < render_state.entries.len() - 1 {
                    if render_state.selected_entry_index == render_state.bottom_index - 1 {
                        render_state.top_index += 1;
                    }

                    render_state.selected_entry_index += 1;
                    render_state.complete_render(&mut stdout)?;
                }
            }
            Action::MoveUp => {
                if render_state.selected_entry_index > 0 {
                    if render_state.selected_entry_index == render_state.top_index {
                        render_state.top_index -= 1;
                    }

                    render_state.selected_entry_index -= 1;
                    render_state.complete_render(&mut stdout)?;
                }
            }
            Action::SwapEntryUp => {
                if render_state.selected_entry_index > 0 {
                    file_handler.swap_entries(
                        &render_state.selected_entry_index,
                        &(render_state.selected_entry_index - 1),
                    )?;

                    render_state.entries.swap(
                        render_state.selected_entry_index,
                        render_state.selected_entry_index - 1,
                    );

                    if render_state.selected_entry_index == render_state.top_index {
                        render_state.top_index -= 1;
                    }

                    render_state.selected_entry_index -= 1;

                    render_state.complete_render(&mut stdout)?;
                }
            }
            Action::SwapEntryDown => {
                if render_state.selected_entry_index < render_state.entries.len() - 1 {
                    file_handler.swap_entries(
                        &render_state.selected_entry_index,
                        &(render_state.selected_entry_index + 1),
                    )?;

                    render_state.entries.swap(
                        render_state.selected_entry_index,
                        render_state.selected_entry_index + 1,
                    );

                    if render_state.selected_entry_index == render_state.bottom_index - 1 {
                        render_state.top_index += 1;
                    }

                    render_state.selected_entry_index += 1;

                    render_state.complete_render(&mut stdout)?;
                }
            }
            Action::DeleteSelectedEntry => {
                if matches!(
                    render_state.previous_action,
                    Some(Action::DeleteSelectedEntry)
                ) {
                    file_handler.remove_entry(&render_state.selected_entry_index)?;
                    previous_action_to_save = None;

                    render_state.entries = file_handler.get_entries();

                    if render_state.selected_entry_index > 0 {
                        render_state.selected_entry_index -= 1;
                    }

                    render_state.complete_render(&mut stdout)?;
                }
            }
            Action::JumpDown => {
                render_state.selected_entry_index = min(
                    render_state.selected_entry_index + JUMP_AMOUNT,
                    render_state.entries.len() - 1,
                );

                if render_state.selected_entry_index - render_state.top_index
                    > SCROLL_THRESHOLD_ITEMS
                {
                    render_state.top_index = min(
                        render_state.selected_entry_index - SCROLL_THRESHOLD_ITEMS,
                        render_state.entries.len() - 1,
                    );
                }

                render_state.complete_render(&mut stdout)?;
            }
            Action::JumpUp => {
                if JUMP_AMOUNT > render_state.selected_entry_index {
                    render_state.selected_entry_index = 0;
                } else {
                    render_state.selected_entry_index -= JUMP_AMOUNT;
                }

                if render_state.selected_entry_index >= render_state.top_index
                    || SCROLL_THRESHOLD_ITEMS >= render_state.selected_entry_index
                {
                    render_state.top_index = 0;
                } else if render_state.top_index - render_state.selected_entry_index
                    >= SCROLL_THRESHOLD_ITEMS
                {
                    render_state.top_index =
                        render_state.selected_entry_index - SCROLL_THRESHOLD_ITEMS;
                }

                render_state.complete_render(&mut stdout)?;
            }
            Action::JumpToTop => {
                render_state.selected_entry_index = 0;
                render_state.top_index = 0;
                render_state.complete_render(&mut stdout)?;
            }
            Action::JumpToBottom => {
                render_state.selected_entry_index = render_state.entries.len() - 1;
                render_state.top_index = render_state.entries.len() - 1;
                render_state.complete_render(&mut stdout)?;
            }
            Action::Exit => {
                stdout.execute(crossterm::terminal::LeaveAlternateScreen)?;
                return Ok(());
            }
            _ => {}
        }

        render_state.previous_action = previous_action_to_save;
    }

    stdout.execute(crossterm::terminal::LeaveAlternateScreen)?;
    panic!("Sender channel closed unexpectedly");
}
