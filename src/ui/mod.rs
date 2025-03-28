use std::{io::Error, sync::mpsc::channel, thread::spawn};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use render::{render_loop, UIMessage};

use crate::{file::EntryFileHandler, log::LogError};

pub mod io;
pub mod render;

pub fn start_gui_mode(file_handler: EntryFileHandler) -> Result<(), Error> {
    let (sender, receiver) = channel::<UIMessage>();

    spawn(move || render_loop(file_handler, receiver));

    crossterm::terminal::enable_raw_mode()?;

    while let Ok(event) = crossterm::event::read() {
        match event {
            Event::Resize(x_size, y_size) => {
                sender
                    .send(UIMessage::Resize { y_size, x_size })
                    .log_if_err();
            }
            Event::Key(key_event) => {
                if let Ok(ui_message) = TryInto::<UIMessage>::try_into(key_event) {
                    sender.send(ui_message).log_if_err();
                }

                match key_event.code {
                    KeyCode::Esc | KeyCode::Char('q') => {
                        break;
                    }
                    KeyCode::Char('c') => {
                        if key_event.modifiers == KeyModifiers::CONTROL {
                            break;
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    sender.send(UIMessage::Exit).log_if_err();
    crossterm::terminal::disable_raw_mode()?;

    Ok(())
}

impl TryInto<UIMessage> for KeyEvent {
    type Error = ();

    fn try_into(self) -> Result<UIMessage, Self::Error> {
        match self.code {
            // Jumping
            KeyCode::Char('d') if self.modifiers == KeyModifiers::CONTROL => {
                Ok(UIMessage::JumpDown)
            }
            KeyCode::Char('u') if self.modifiers == KeyModifiers::CONTROL => Ok(UIMessage::JumpUp),
            KeyCode::Char('G') => Ok(UIMessage::JumpToBottom),
            KeyCode::Char('g') => Ok(UIMessage::JumpToTop),

            // Move up and down
            KeyCode::Char('j') => Ok(UIMessage::MoveDown),
            KeyCode::Down => Ok(UIMessage::MoveDown),
            KeyCode::Char('k') => Ok(UIMessage::MoveUp),
            KeyCode::Up => Ok(UIMessage::MoveUp),

            // Etc.
            KeyCode::Char('h') => Ok(UIMessage::SwapEntryUp),
            KeyCode::Char('l') => Ok(UIMessage::SwapEntryDown),
            KeyCode::Char('d') => Ok(UIMessage::DeleteSelectedEntry),
            _ => Err(()),
        }
    }
}
