use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};

use crate::entry::Entry;

#[derive(Debug, Clone)]
pub enum Action {
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
    JumpDown,
    JumpUp,
    JumpToTop,
    JumpToBottom,
}

impl TryInto<Action> for MouseEvent {
    type Error = ();

    fn try_into(self) -> Result<Action, Self::Error> {
        match self.kind {
            crossterm::event::MouseEventKind::ScrollDown => Ok(Action::MoveDown),
            crossterm::event::MouseEventKind::ScrollUp => Ok(Action::MoveUp),
            _ => Err(()),
        }
    }
}

impl TryInto<Action> for KeyEvent {
    type Error = ();

    fn try_into(self) -> Result<Action, Self::Error> {
        match self.code {
            // Quit
            KeyCode::Esc | KeyCode::Char('q') => Ok(Action::Exit),
            KeyCode::Char('c') if self.modifiers == KeyModifiers::CONTROL => Ok(Action::Exit),

            // Jumping
            KeyCode::Char('d') if self.modifiers == KeyModifiers::CONTROL => Ok(Action::JumpDown),
            KeyCode::Char('u') if self.modifiers == KeyModifiers::CONTROL => Ok(Action::JumpUp),
            KeyCode::Char('G') => Ok(Action::JumpToBottom),
            KeyCode::Char('g') => Ok(Action::JumpToTop),

            // Move up and down
            KeyCode::Char('j') => Ok(Action::MoveDown),
            KeyCode::Down => Ok(Action::MoveDown),
            KeyCode::Char('k') => Ok(Action::MoveUp),
            KeyCode::Up => Ok(Action::MoveUp),

            // Etc.
            KeyCode::Char('h') => Ok(Action::SwapEntryUp),
            KeyCode::Char('l') => Ok(Action::SwapEntryDown),
            KeyCode::Char('d') => Ok(Action::DeleteSelectedEntry),
            _ => Err(()),
        }
    }
}
