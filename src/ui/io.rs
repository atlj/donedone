use std::{io::Error, sync::mpsc::Sender};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent};

use crate::{entry::Entry, log::LogError};

pub fn io_loop(render_loop_sender: Sender<Action>) -> Result<(), Error> {
    crossterm::terminal::enable_raw_mode()?;

    while let Ok(event) = crossterm::event::read() {
        match event {
            Event::Resize(x_size, y_size) => {
                render_loop_sender
                    .send(Action::Resize { y_size, x_size })
                    .log_if_err();
            }
            Event::Mouse(mouse_event) => {
                if let Ok(action) = mouse_event.try_into() {
                    render_loop_sender.send(action).log_if_err();
                }
            }
            Event::Key(key_event) => {
                if let Ok(action) = key_event.try_into() {
                    if matches!(action, Action::Exit) {
                        break;
                    }

                    render_loop_sender.send(action).log_if_err();
                }
            }
            _ => {}
        }
    }

    render_loop_sender.send(Action::Exit).log_if_err();
    crossterm::terminal::disable_raw_mode()?;

    Ok(())
}

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
            KeyCode::Left => Ok(Action::SwapEntryUp),
            KeyCode::Char('l') => Ok(Action::SwapEntryDown),
            KeyCode::Right => Ok(Action::SwapEntryDown),
            KeyCode::Char('d') => Ok(Action::DeleteSelectedEntry),
            _ => Err(()),
        }
    }
}
