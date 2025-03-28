use std::{io::Error, sync::mpsc::channel, thread::spawn};

use crossterm::event::Event;
use io::Action;
use render::render_loop;

use crate::{file::EntryFileHandler, log::LogError};

pub mod io;
pub mod render;

pub fn start_gui_mode(file_handler: EntryFileHandler) -> Result<(), Error> {
    let (sender, receiver) = channel::<Action>();

    spawn(move || render_loop(file_handler, receiver));

    crossterm::terminal::enable_raw_mode()?;

    while let Ok(event) = crossterm::event::read() {
        match event {
            Event::Resize(x_size, y_size) => {
                sender.send(Action::Resize { y_size, x_size }).log_if_err();
            }
            Event::Key(key_event) => {
                if let Ok(action) = key_event.try_into() {
                    if matches!(action, Action::Exit) {
                        break;
                    }

                    sender.send(action).log_if_err();
                }
            }
            _ => {}
        }
    }

    sender.send(Action::Exit).log_if_err();
    crossterm::terminal::disable_raw_mode()?;

    Ok(())
}
