use std::{
    env,
    path::Path,
    sync::mpsc::{self, Receiver, Sender, channel},
};

use crate::{
    events::{
        ApplicationEvent,
        keyboard::{Action, KeyboardHandler},
        musicplayer::{
            Player, PlayerInformation, PlayerReceiveEvent, PlayerSendEvent, PlayerStatus,
        },
    },
    filefinder::FileFinder,
    song::Song,
    ui::FocusedWindow,
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
        std::env::var("HOME").unwrap_or(".".to_string())
    };
    let mut app = App::new(path);
    app.run()
}

struct App {
    exit: bool,
    upcoming_media_shown: bool,
    select_handler: SelectHandler<Song>,
    queue_select_handler: SelectHandler<Song>,
    file_finder: FileFinder,
    pub player_information: PlayerInformation,
    focused_window: FocusedWindow,
}

impl App {
    fn new(path: String) -> Self {
        App {
            exit: false,
            upcoming_media_shown: true,
            select_handler: SelectHandler::new(),
            queue_select_handler: SelectHandler::new(),
            file_finder: FileFinder::new(
                [".mp3".to_string(), ".ogg".to_string(), ".wav".to_string()],
                path,
                Some(2),
            ),
            player_information: PlayerInformation::default(),
            focused_window: FocusedWindow::Main,
        }
    }

    fn run(&mut self) -> Result<(), std::io::Error> {
        let mut terminal = ratatui::init();

        let (event_tx, event_rx) = channel::<ApplicationEvent>();

        let (player_tx, player_rx) = channel::<PlayerReceiveEvent>();

        self.create_threads(&event_tx, player_rx);
        self.file_finder.find_paths(None, None);
        self.select_handler.set_items(
            self.file_finder
                .create_songs()
                .unwrap_or(&Vec::new())
                .clone(),
        );

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
                        Action::SwitchWindow => {
                            self.focused_window = match self.focused_window {
                                FocusedWindow::Main => FocusedWindow::Queue,
                                FocusedWindow::Queue => FocusedWindow::Main,
                            }
                        }
                        Action::MoveUp => self.move_cursor_up(),
                        Action::MoveDown => self.move_cursor_down(),
                        Action::Select => self.select_at_cursor(&player_tx),
                        Action::Space => {
                            player_tx
                                .send(PlayerReceiveEvent::TogglePause)
                                .expect("Failed to toggle pause");
                        }
                        Action::PreviousSong => {
                            player_tx
                                .send(PlayerReceiveEvent::Previous)
                                .expect("Failed to send previous song to player");
                        }
                        Action::NextSong => {
                            player_tx
                                .send(PlayerReceiveEvent::Next)
                                .expect("Failed to send next song to player");
                        }
                    },
                    ApplicationEvent::PlayerEvent(event) => match event {
                        PlayerSendEvent::Play(player_information) => {
                            self.player_information = player_information;
                        }
                        PlayerSendEvent::Pause => {
                            if let PlayerStatus::Playing(song) = &self.player_information.status {
                                self.player_information.status = PlayerStatus::Paused(song.clone());
                            }
                        }
                        PlayerSendEvent::TimeChanged(player_information) => {
                            self.player_information = player_information;
                        }
                        PlayerSendEvent::QueueUpdate(queue) => {
                            self.queue_select_handler.set_items(queue);
                        }
                        _ => {}
                    },
                }
            }
        }
        ratatui::restore();
        Ok(())
    }

    fn move_cursor_down(&mut self) {
        match self.focused_window {
            FocusedWindow::Queue => self.queue_select_handler.down(),
            FocusedWindow::Main => self.select_handler.down(),
        }
    }

    fn move_cursor_up(&mut self) {
        match self.focused_window {
            FocusedWindow::Queue => self.queue_select_handler.up(),
            FocusedWindow::Main => self.select_handler.up(),
        }
    }

    fn select_at_cursor(&mut self, player_tx: &Sender<PlayerReceiveEvent>) {
        match self.focused_window {
            FocusedWindow::Queue => {
                if let Some(index) = self.queue_select_handler.state().selected() {
                    player_tx
                        .send(PlayerReceiveEvent::SetAndPlaySong(index))
                        .expect("Failed to set song");
                }
            }
            FocusedWindow::Main => {
                if let Some(index) = self.select_handler.state().selected() {
                    let (queue1, queue2) = self.select_handler.items().split_at(index);
                    let mut queue2 = queue2.to_vec();
                    queue2.append(&mut queue1.to_vec());
                    player_tx
                        .send(PlayerReceiveEvent::CreateQueueAndPlay(queue2))
                        .expect("Failed to send song to player");
                }
            }
        }
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
