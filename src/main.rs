use std::sync::mpsc::{self, Sender};

use crate::{
    events::{
        Events,
        keyboard::{Action, KeyboardHandler},
    },
    selecthandler::SelectHandler,
    song::Song,
};

mod events;
mod selecthandler;
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
    select_handler: SelectHandler<String>,
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
            select_handler: SelectHandler::new(vec![
                "Option 1".to_string(),
                "Option 2".to_string(),
                "Option 3".to_string(),
            ]),
        }
    }

    fn run(&mut self) -> Result<(), std::io::Error> {
        let mut terminal = ratatui::init();

        let (event_tx, event_rx) = mpsc::channel::<Events>();
        self.create_threads(&event_tx);
        loop {
            if self.exit {
                break;
            }
            let _ = terminal.draw(|frame| {
                ui::render(frame, self);
            });
            if let Ok(event) = event_rx.try_recv() {
                match event {
                    Events::Action(action) => match action {
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

    fn create_threads(&self, event_tx: &Sender<Events>) {
        let keyboard_event_tx = event_tx.clone();
        KeyboardHandler::new(keyboard_event_tx);
    }
}
