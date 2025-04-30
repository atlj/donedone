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
    entry_comments: Vec<Vec<String>>,
    selected_entry_index: usize,
    top_index: usize,
    bottom_index: usize,
    y_size: u16,
    x_size: u16,
    previous_action: Option<Action>,
}

type Renderable = Vec<StyledContent<String>>;

const SCROLL_THRESHOLD_ITEMS: usize = 2;

impl UIState {
    pub fn new(entries: Vec<Entry>, x_size: u16, y_size: u16) -> Self {
        let entry_comments = entries
            .iter()
            .map(|entry| Self::split_entry_comments(entry, x_size))
            .collect();

        Self {
            entries,
            entry_comments,
            selected_entry_index: 0,
            top_index: 0,
            bottom_index: 0,
            y_size,
            x_size,
            previous_action: None,
        }
    }

    fn resize(&mut self, y_size: u16, x_size: u16) {
        self.x_size = x_size;
        self.y_size = y_size;

        self.entry_comments = self
            .entries
            .iter()
            .map(|entry| Self::split_entry_comments(entry, x_size))
            .collect();
    }

    fn split_entry_comments(entry: &Entry, total_width: u16) -> Vec<String> {
        let max_width = Self::get_entry_max_width(total_width) as usize;
        let mut result = Vec::new();

        if let Some(comment) = &entry.comment {
            let mut words = comment.split_whitespace().peekable();

            let mut current_line = String::with_capacity(max_width);

            while let Some(word) = words.next() {
                current_line.push_str(word);

                // Check if more chars can fit in the current line
                if let Some(next_word) = words.peek() {
                    if next_word.len() + current_line.len() <= (max_width - 1) {
                        current_line.push(' ');
                        continue;
                    }
                }
                // We reached the line's max width, yield

                // If we have a single word that's larger than the x_size, we have to truncate it.
                current_line.truncate(max_width);

                result.push(current_line.clone());

                current_line.clear();
            }
        }

        result
    }

    fn complete_render(&mut self, stdout: &mut BufWriter<Stdout>) -> Result<(), Error> {
        // Draw Entries
        let entry_render_x_size = Self::get_entry_max_width(self.x_size);
        let entry_render_y_size = self.y_size;

        let rendered_entries = self.render_entries(entry_render_y_size, entry_render_x_size);

        stdout.queue(crossterm::terminal::Clear(
            crossterm::terminal::ClearType::All,
        ))?;

        let offset_to_center_entries =
            ((entry_render_y_size as f32 - rendered_entries.len() as f32) / 2.0).round() as usize;

        for (y, contents) in rendered_entries.into_iter().enumerate() {
            assert!(y <= self.y_size.into(), "Trying to print outside of screen");

            stdout
                .queue(crossterm::cursor::MoveToColumn(5))?
                .queue(crossterm::cursor::MoveToRow(
                    (y + offset_to_center_entries) as u16,
                ))?
                .queue(crossterm::style::Print(contents))?;
        }

        // Draw Scroll bar
        let scroll_bar = self.render_scroll_bar(self.y_size - 6);

        for (y, contents) in scroll_bar.into_iter().enumerate() {
            assert!(y <= self.y_size.into(), "Trying to print outside of screen");

            stdout
                .queue(crossterm::cursor::MoveToColumn(self.x_size - 2))?
                .queue(crossterm::cursor::MoveToRow(y as u16 + 2))?
                .queue(crossterm::style::Print(contents))?;
        }

        // Draw Selected Index Indicator
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

        let rendered_entries = self
            .entries
            .iter()
            .enumerate()
            .skip(self.top_index)
            .map(|(index, entry)| self.render_entry(entry, index, y_size, x_size));

        let mut rendered_entry_count = 0;

        for rendered_entry in rendered_entries {
            if result.len() + rendered_entry.len() > y_size as usize {
                break;
            }

            result.extend(rendered_entry);

            if result.len() < y_size as usize {
                result.push("".to_string().stylize());
            }

            rendered_entry_count += 1;
        }

        self.bottom_index = self.top_index + rendered_entry_count;

        result
    }

    fn render_entry(
        &self,
        entry: &Entry,
        entry_index: usize,
        _y_size: u16,
        x_size: u16,
    ) -> Renderable {
        let should_highlight = self.selected_entry_index == entry_index;
        let mut result = Vec::new();

        if let Some(comment) = self.entry_comments.get(entry_index) {
            let comment_to_push = comment.clone().into_iter().map(|comment| {
                if should_highlight {
                    comment.magenta()
                } else {
                    comment.stylize()
                }
            });

            result.extend(comment_to_push);
        }

        let current_dir = current_dir().unwrap();
        let mut relative_dir = entry
            .path
            .strip_prefix(current_dir)
            .map(|dir| dir.to_str().unwrap_or_default().to_string())
            .unwrap_or_default();

        relative_dir.truncate(x_size as usize);

        let mut line = format!("{}:{}", relative_dir, entry.line)
            .to_string()
            .underlined();

        if should_highlight {
            line = line.magenta();
        }

        result.push(line);

        if let Ok(file_handle) = OpenOptions::new().read(true).open(&entry.path) {
            let buf_reader = BufReader::new(file_handle);
            if let Some(Ok(line)) = buf_reader.lines().nth(entry.line - 1) {
                let mut line = line.trim().to_string();
                line.truncate(x_size as usize);
                result.push(line.italic().dark_grey());
            }
        }

        result
    }

    fn render_scroll_bar(&self, y_size: u16) -> Renderable {
        let mut result: Renderable = vec![];
        let displayed_entry_count = self.bottom_index - self.top_index;

        if displayed_entry_count == self.entries.len() {
            return result;
        }

        let thumb_size_ratio = displayed_entry_count as f32 / self.entries.len() as f32;
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
            (min(self.selected_entry_index + 1, self.entries.len()))
                .to_string()
                .magenta(),
            "/".to_string().stylize(),
            self.entries.len().to_string().stylize(),
        ]
    }

    fn select_entry(&mut self, index: usize, window_height: usize, edge_threshold: usize) {
        self.selected_entry_index = min(self.entries.len() - 1, index);

        // Scroll up
        if self.selected_entry_index.saturating_sub(edge_threshold) < self.top_index {
            self.top_index = self.selected_entry_index.saturating_sub(edge_threshold);
            return;
        }

        // Scroll down
        if self.selected_entry_index + edge_threshold >= self.bottom_index {
            let bottom_index_goal = min(
                self.selected_entry_index + edge_threshold,
                self.entries.len() - 1,
            );

            let mut height_to_fill = window_height;
            let comments = self
                .entry_comments
                .iter()
                .rev()
                .skip(self.entries.len() - 1 - bottom_index_goal);

            let mut top_index = bottom_index_goal;
            for entry_comments in comments {
                let entry_height = entry_comments.len() // comments
                    + 2; // code and path

                if entry_height > height_to_fill {
                    break;
                }

                height_to_fill -= entry_height;

                // Gap
                height_to_fill = height_to_fill.saturating_sub(1);

                top_index -= 1;
            }

            self.top_index = top_index + 1
        }
    }

    fn get_entry_max_width(total_width: u16) -> u16 {
        total_width - 10
    }
}

pub fn render_loop(
    mut file_handler: EntryFileHandler,
    io_action_receiver: Receiver<Action>,
    on_initial_render: impl FnOnce(),
) -> Result<(), Error> {
    let (initial_x_size, initial_y_size) = crossterm::terminal::size()?;
    let mut render_state = UIState::new(file_handler.get_entries(), initial_x_size, initial_y_size);
    let mut stdout = BufWriter::new(stdout());

    stdout
        .queue(crossterm::cursor::Hide)?
        .queue(crossterm::terminal::EnterAlternateScreen)?;

    stdout.flush()?;

    render_state.complete_render(&mut stdout)?;

    on_initial_render();

    while let Ok(action) = io_action_receiver.recv() {
        let mut previous_action_to_save = Some(action.clone());

        match action {
            Action::MoveDown => {
                if render_state.selected_entry_index < render_state.entries.len() - 1 {
                    render_state.select_entry(
                        render_state.selected_entry_index + 1,
                        render_state.y_size as usize,
                        SCROLL_THRESHOLD_ITEMS,
                    );
                    render_state.complete_render(&mut stdout)?;
                }
            }
            Action::MoveUp => {
                if render_state.selected_entry_index > 0 {
                    render_state.select_entry(
                        render_state.selected_entry_index - 1,
                        render_state.y_size as usize,
                        SCROLL_THRESHOLD_ITEMS,
                    );
                    render_state.complete_render(&mut stdout)?;
                }
            }
            Action::SwapEntryUp => {
                if render_state.entries.len() > 1 && render_state.selected_entry_index > 0 {
                    file_handler.swap_entries(
                        &render_state.selected_entry_index,
                        &(render_state.selected_entry_index - 1),
                    )?;

                    render_state.entries.swap(
                        render_state.selected_entry_index,
                        render_state.selected_entry_index - 1,
                    );
                    render_state.entry_comments.swap(
                        render_state.selected_entry_index,
                        render_state.selected_entry_index - 1,
                    );

                    render_state.select_entry(
                        render_state.selected_entry_index - 1,
                        render_state.y_size as usize,
                        SCROLL_THRESHOLD_ITEMS,
                    );
                    render_state.complete_render(&mut stdout)?;
                }
            }
            Action::SwapEntryDown => {
                if render_state.entries.len() > 1
                    && render_state.selected_entry_index < render_state.entries.len() - 1
                {
                    file_handler.swap_entries(
                        &render_state.selected_entry_index,
                        &(render_state.selected_entry_index + 1),
                    )?;

                    render_state.entries.swap(
                        render_state.selected_entry_index,
                        render_state.selected_entry_index + 1,
                    );
                    render_state.entry_comments.swap(
                        render_state.selected_entry_index,
                        render_state.selected_entry_index + 1,
                    );

                    render_state.select_entry(
                        render_state.selected_entry_index + 1,
                        render_state.y_size as usize,
                        SCROLL_THRESHOLD_ITEMS,
                    );

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

                    render_state
                        .entries
                        .remove(render_state.selected_entry_index);
                    render_state
                        .entry_comments
                        .remove(render_state.selected_entry_index);

                    render_state.select_entry(
                        render_state.selected_entry_index,
                        render_state.y_size as usize,
                        0,
                    );
                    render_state.complete_render(&mut stdout)?;
                }
            }
            Action::JumpDown => {
                let jump_amount = (render_state.bottom_index - render_state.top_index) / 2;
                render_state.select_entry(
                    min(
                        render_state.selected_entry_index + jump_amount,
                        render_state.entries.len() - 1,
                    ),
                    render_state.y_size as usize,
                    SCROLL_THRESHOLD_ITEMS,
                );
                render_state.complete_render(&mut stdout)?;
            }
            Action::JumpUp => {
                let jump_amount = (render_state.bottom_index - render_state.top_index) / 2;
                render_state.select_entry(
                    render_state
                        .selected_entry_index
                        .saturating_sub(jump_amount),
                    render_state.y_size as usize,
                    SCROLL_THRESHOLD_ITEMS,
                );
                render_state.complete_render(&mut stdout)?;
            }
            Action::JumpToTop => {
                render_state.select_entry(0, render_state.y_size as usize, SCROLL_THRESHOLD_ITEMS);
                render_state.complete_render(&mut stdout)?;
            }
            Action::JumpToBottom => {
                render_state.select_entry(
                    render_state.entries.len() - 1,
                    render_state.y_size as usize,
                    SCROLL_THRESHOLD_ITEMS,
                );
                render_state.complete_render(&mut stdout)?;
            }
            Action::Exit => {
                stdout.execute(crossterm::terminal::LeaveAlternateScreen)?;
                return Ok(());
            }
            Action::Resize { y_size, x_size } => {
                render_state.resize(y_size, x_size);
                render_state.complete_render(&mut stdout)?;
            }
            _ => {}
        }

        render_state.previous_action = previous_action_to_save;
    }

    stdout.execute(crossterm::terminal::LeaveAlternateScreen)?;
    panic!("Sender channel closed unexpectedly");
}
