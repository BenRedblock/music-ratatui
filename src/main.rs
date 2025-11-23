use std::{
    env,
    path::Path,
    sync::mpsc::{self, Receiver, Sender, channel},
    thread,
    time::Duration,
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
    searchhandler::SearchHandler,
    song::Song,
    utils::selecthandler::SelectHandler,
};
mod config;
mod events;
mod fetch;
mod filefinder;
mod searchhandler;
mod song;
mod ui;
mod utils;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let args: Vec<String> = env::args().collect();
    let path = if args.len() > 1 {
        args[1].clone()
    } else {
        std::env::var("HOME").unwrap_or(".".to_string())
    };
    let mut app = App::new(path);
    let res = app.run().await;
    res
}

struct App {
    exit: bool,
    upcoming_media_shown: bool,
    select_handler: SelectHandler<Song>,
    queue_select_handler: SelectHandler<Song>,
    file_finder: FileFinder,
    pub player_information: PlayerInformation,
    current_screen: CurrentScreen,
    search_handler: SearchHandler,
    player_tx: Sender<PlayerReceiveEvent>,
    event_rx: Receiver<ApplicationEvent>,
}

pub enum CurrentScreen {
    Main(FocusedWindowMain),
    SearchMain,
}

pub enum FocusedWindowMain {
    Main,
    Queue,
    Search,
}

impl App {
    fn new(path: String) -> Self {
        let (player_tx, player_rx) = channel::<PlayerReceiveEvent>();
        let (event_tx, event_rx) = channel::<ApplicationEvent>();
        App::create_threads(event_tx, player_rx);
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
            current_screen: CurrentScreen::Main(FocusedWindowMain::Main),
            search_handler: SearchHandler::new(),
            player_tx,
            event_rx,
        }
    }

    async fn run(&mut self) -> Result<(), std::io::Error> {
        let mut terminal = ratatui::init();

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
            if let Ok(event) = self.event_rx.try_recv() {
                match event {
                    ApplicationEvent::Action(action) => match action {
                        Action::Quit => self.exit = true,

                        _ => match &self.current_screen {
                            CurrentScreen::Main(_) => self.main_screen_events(action).await,
                            CurrentScreen::SearchMain => {}
                        },
                    },
                    ApplicationEvent::PlayerEvent(event) => match event {
                        PlayerSendEvent::Play(playing_index) => {
                            self.player_information.playing_index = Some(playing_index);
                            if let Some(song) = self.get_current_song() {
                                self.player_information.status =
                                    PlayerStatus::Playing(song.clone());
                            }
                        }
                        PlayerSendEvent::Pause(playing_index) => {
                            self.player_information.playing_index = Some(playing_index);
                            if let Some(song) = self.get_current_song() {
                                self.player_information.status =
                                    PlayerStatus::Playing(song.clone());
                            }
                        }
                        PlayerSendEvent::TimeChanged(passed_time) => {
                            self.player_information.passed_time = passed_time;
                        }
                        PlayerSendEvent::QueueUpdate(queue) => {
                            self.player_information.queue = queue.clone();
                            self.queue_select_handler.set_items(queue);
                        }
                        PlayerSendEvent::NextSong => {
                            self.player_information.playing_index =
                                self.player_information.playing_index.map(|index| index + 1);
                            if let Some(song) = self.get_current_song() {
                                self.player_information.status =
                                    PlayerStatus::Playing(song.clone());
                            }
                        }
                        PlayerSendEvent::PlayerEnded => {
                            self.player_information.playing_index = None;
                            self.player_information.status = PlayerStatus::NoAudioSelected;
                        }
                        PlayerSendEvent::PlayerInformation(player_information) => {
                            self.player_information = player_information;
                        }
                    },
                }
            }
            thread::sleep(Duration::from_millis(20));
        }
        ratatui::restore();
        Ok(())
    }

    async fn main_screen_events(&mut self, action: Action) {
        let focused_window = match &self.current_screen {
            CurrentScreen::Main(focused_window) => focused_window,
            _ => &FocusedWindowMain::Main,
        };
        match action {
            Action::SwitchWindow => {
                self.current_screen = CurrentScreen::Main(match focused_window {
                    FocusedWindowMain::Main => FocusedWindowMain::Queue,
                    FocusedWindowMain::Queue => FocusedWindowMain::Main,
                    _ => FocusedWindowMain::Main,
                })
            }
            Action::MoveUp => match focused_window {
                FocusedWindowMain::Queue => self.queue_select_handler.up(),
                FocusedWindowMain::Main => self.select_handler.up(),
                _ => {}
            },
            Action::MoveDown => match focused_window {
                FocusedWindowMain::Queue => self.queue_select_handler.down(),
                FocusedWindowMain::Main => self.select_handler.down(),
                _ => {}
            },
            Action::Select => match focused_window {
                FocusedWindowMain::Queue => {
                    if let Some(index) = self.queue_select_handler.state().selected() {
                        self.player_tx
                            .send(PlayerReceiveEvent::SetAndPlaySong(index))
                            .expect("Failed to set song");
                    }
                }
                FocusedWindowMain::Main => {
                    if let Some(index) = self.select_handler.state().selected() {
                        let (queue1, queue2) = self.select_handler.items().split_at(index);
                        let mut queue2 = queue2.to_vec();
                        queue2.append(&mut queue1.to_vec());
                        self.player_tx
                            .send(PlayerReceiveEvent::CreateQueueAndPlay(queue2))
                            .expect("Failed to send song to player");
                    }
                }
                FocusedWindowMain::Search => {
                    let _ = self.search_handler.search().await;
                }
            },
            Action::Char(char) => match focused_window {
                FocusedWindowMain::Search => {
                    self.search_handler.add_char_to_query(char);
                }
                _ => {
                    if char == 'f' {
                        self.current_screen = CurrentScreen::Main(FocusedWindowMain::Search);
                    }
                }
            },
            Action::Backspace => match focused_window {
                FocusedWindowMain::Search => {
                    self.search_handler.remove_last_char();
                }
                _ => {}
            },
            Action::Esc => match focused_window {
                FocusedWindowMain::Search => {
                    self.current_screen = CurrentScreen::Main(FocusedWindowMain::Main);
                }
                _ => {}
            },
            Action::Space => match focused_window {
                FocusedWindowMain::Search => {
                    self.search_handler.add_char_to_query(' ');
                }
                _ => {
                    self.player_tx
                        .send(PlayerReceiveEvent::TogglePause)
                        .expect("Failed to toggle pause");
                }
            },
            Action::PreviousSong => {
                self.player_tx
                    .send(PlayerReceiveEvent::Previous)
                    .expect("Failed to send previous song to player");
            }
            Action::NextSong => {
                self.player_tx
                    .send(PlayerReceiveEvent::Next)
                    .expect("Failed to send next song to player");
            }
            _ => {}
        }
    }

    fn get_current_song(&self) -> Option<&Song> {
        if let Some(index) = self.player_information.playing_index {
            self.player_information.queue.get(index)
        } else {
            None
        }
    }

    fn create_threads(event_tx: Sender<ApplicationEvent>, player_rx: Receiver<PlayerReceiveEvent>) {
        let keyboard_event_tx = event_tx.clone();
        KeyboardHandler::new(keyboard_event_tx);
        let player_event_tx = event_tx.clone();
        Player::new(player_event_tx, player_rx);
    }
}
