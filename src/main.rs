use std::{
    path::Path,
    sync::mpsc::{self, Sender},
};

use crate::{
    events::{
        Event,
        keyboard::{Action, KeyboardHandler},
    },
    filefinder::FileFinder,
    song::Song,
    utils::selecthandler::SelectHandler,
};

mod events;
mod filefinder;
mod song;
mod ui;
mod utils;

fn main() -> Result<(), std::io::Error> {
    let mut app = App::new();
    app.run()
}

struct App {
    exit: bool,
    upcoming_media_shown: bool,
    select_handler: SelectHandler<Song>,
    file_finder: FileFinder,
    loaded_songs: Vec<Song>,
}

impl App {
    fn new() -> Self {
        App {
            exit: false,
            upcoming_media_shown: true,
            select_handler: SelectHandler::new(),
            file_finder: FileFinder::new([
                ".mp3".to_string(),
                ".ogg".to_string(),
                ".ics".to_string(),
            ]),
            loaded_songs: Vec::new(),
        }
    }

    fn run(&mut self) -> Result<(), std::io::Error> {
        let mut terminal = ratatui::init();

        let (event_tx, event_rx) = mpsc::channel::<Event>();
        self.create_threads(&event_tx);
        self.file_finder.find_paths("".to_string(), 2);
        self.loaded_songs = self.file_finder.create_songs().unwrap_or(Vec::new());

        loop {
            if self.exit {
                break;
            }
            let _ = terminal.draw(|frame| {
                ui::render(frame, self);
            });
            if let Ok(event) = event_rx.try_recv() {
                match event {
                    Event::Action(action) => match action {
                        Action::Quit => self.exit = true,
                        Action::MoveUp => self.select_handler.up(),
                        Action::MoveDown => self.select_handler.down(),
                        Action::Select => {
                            self.select_handler.select();
                        }
                    },
                }
            }
        }
        ratatui::restore();
        Ok(())
    }

    fn create_threads(&self, event_tx: &Sender<Event>) {
        let keyboard_event_tx = event_tx.clone();
        KeyboardHandler::new(keyboard_event_tx);
    }
}
