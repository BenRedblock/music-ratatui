use std::{
    env,
    path::Path,
    sync::mpsc::{self, Receiver, Sender, channel},
};

use crate::{
    events::{
        ApplicationEvent,
        keyboard::{Action, KeyboardHandler},
        musicplayer::{Player, PlayerInformation, PlayerReceiveEvent, PlayerSendEvent},
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
    let args: Vec<String> = env::args().collect();
    let path = if args.len() > 1 {
        args[1].clone()
    } else {
        "~".to_string()
    };
    let mut app = App::new(path);
    app.run()
}

struct App {
    exit: bool,
    upcoming_media_shown: bool,
    select_handler: SelectHandler<Song>,
    file_finder: FileFinder,
    pub player_information: PlayerInformation,
}

impl App {
    fn new(path: String) -> Self {
        App {
            exit: false,
            upcoming_media_shown: true,
            select_handler: SelectHandler::new(),
            file_finder: FileFinder::new(
                [".mp3".to_string(), ".ogg".to_string(), ".".to_string()],
                path,
                Some(2),
            ),
            player_information: PlayerInformation::default(),
        }
    }

    fn run(&mut self) -> Result<(), std::io::Error> {
        let mut terminal = ratatui::init();

        let (event_tx, event_rx) = channel::<ApplicationEvent>();

        let (player_tx, player_rx) = channel::<PlayerReceiveEvent>();

        self.create_threads(&event_tx, player_rx);
        self.file_finder.find_paths(None, None);
        self.select_handler
            .set_items(self.file_finder.create_songs().unwrap_or(Vec::new()));

        loop {
            if self.exit {
                break;
            }
            let _ = terminal.draw(|frame| {
                ui::render(frame, self);
            });
            if let Ok(event) = event_rx.try_recv() {
                match event {
                    ApplicationEvent::Action(action) => match action {
                        Action::Quit => self.exit = true,
                        Action::MoveUp => self.select_handler.up(),
                        Action::MoveDown => self.select_handler.down(),
                        Action::Select => {
                            if let Some(song) = self.select_handler.select() {
                                player_tx
                                    .send(PlayerReceiveEvent::SetSong(song.clone()))
                                    .expect("Failed to send song to player");
                            }
                        }
                        Action::Space => {
                            player_tx
                                .send(PlayerReceiveEvent::TogglePause)
                                .expect("Failed to toggle pause");
                        }
                    },
                    ApplicationEvent::PlayerEvent(event) => match event {
                        PlayerSendEvent::Play(player_information) => {
                            self.player_information = player_information;
                        }
                        PlayerSendEvent::Pause(player_information) => {
                            self.player_information = player_information;
                        }
                        PlayerSendEvent::TimeChanged(player_information) => {
                            self.player_information = player_information;
                        }
                        _ => {}
                    },
                }
            }
        }
        ratatui::restore();
        Ok(())
    }

    fn create_threads(
        &self,
        event_tx: &Sender<ApplicationEvent>,
        player_rx: Receiver<PlayerReceiveEvent>,
    ) {
        let keyboard_event_tx = event_tx.clone();
        KeyboardHandler::new(keyboard_event_tx);
        let player_event_tx = event_tx.clone();
        Player::new(player_event_tx, player_rx);
    }
}
