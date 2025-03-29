use std::{io::Error, sync::mpsc::channel, thread::spawn};

use io::{io_loop, Action};
use render::render_loop;

use crate::file::EntryFileHandler;

pub mod io;
pub mod render;

pub fn start_gui_mode(file_handler: EntryFileHandler) -> Result<(), Error> {
    let (sender, receiver) = channel::<Action>();

    spawn(move || io_loop(sender));

    render_loop(file_handler, receiver).unwrap();

    Ok(())
}
