use std::sync::mpsc::{channel, Receiver};

use crossterm::event::KeyEvent;

use crate::entry::Entry;

pub struct UIState {
    pub entries: Vec<Entry>,
    pub selected_entry_index: usize,
    pub top_index: usize,
    pub y_size: u16,
    pub x_size: u16,
    pub previous_command: Option<UIMessage>,
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

pub fn render_loop(initial_state: UIState, receiver: Receiver<UIMessage>) {
    while let Ok(message) = receiver.recv() {
        match message {
            UIMessage::Exit => return,
            _ => {}
        }
    }

    panic!("Sender channel closed unexpectedly");
}
