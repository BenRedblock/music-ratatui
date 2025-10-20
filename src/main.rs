use std::sync::mpsc::{self, Sender};

use crate::{
    events::{
        Events,
        keyboard::{Action, KeyboardHandler},
    },
    song::Song,
};

mod events;
mod song;
mod ui;
fn main() -> Result<(), std::io::Error> {
    let mut app = App::new();
    app.run()
}

struct App {
    exit: bool,
    current_song: Song,
    upcoming_media_shown: bool,
}

impl App {
    fn new() -> Self {
        let current_song = Song {
            title: String::from(""),
            author: String::from(""),
            playing: false,
            time_played: 50,
            total_time: 100,
        };
        App {
            exit: false,
            current_song,
            upcoming_media_shown: true,
        }
    }

    fn run(&mut self) -> Result<(), std::io::Error> {
        let mut terminal = ratatui::init();

        let (event_tx, event_rx) = mpsc::channel::<Events>();
        self.create_threads(&event_tx);
        while !self.exit {
            let _ = terminal.draw(|frame| {
                ui::render(frame, &self);
            });
            if let Ok(event) = event_rx.try_recv() {
                match event {
                    Events::Action(action) => match action {
                        Action::Quit => self.exit = true,
                    },
                }
            }
        }
        ratatui::restore();
        Ok(())
    }

    fn create_threads(&self, event_tx: &Sender<Events>) {
        let keyboard_event_tx = event_tx.clone();
        KeyboardHandler::new(keyboard_event_tx);
    }
}
