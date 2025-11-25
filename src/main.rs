use crate::{
    display_handlers::folder_handler::{Folder, FolderHandler},
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
use std::{
    collections::HashMap,
    env,
    path::Path,
    sync::mpsc::{Receiver, Sender, channel},
    thread,
    time::Duration,
};
mod config;
mod display_handlers;
mod events;
mod fetch;
mod filefinder;
mod searchhandler;
mod song;
mod songs;
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

pub enum CurrentScreen {
    Main(FocusedWindowMain),
}

pub enum FocusedWindowMain {
    Media(MediaDisplayType),
    Queue,
    Search,
}

pub enum MediaDisplayType {
    Songs,
    Folder,
}

struct App {
    exit: bool,
    songs: HashMap<String, Song>,
    queue_shown: bool,
    folder_handler: FolderHandler,
    select_handler: SelectHandler<Song>,
    queue_select_handler: SelectHandler<Song>,
    file_finder: FileFinder,
    pub player_information: PlayerInformation,
    current_screen: CurrentScreen,
    search_handler: SearchHandler,
    player_tx: Sender<PlayerReceiveEvent>,
    event_rx: Receiver<ApplicationEvent>,
}

impl App {
    fn new(path: String) -> Self {
        let (player_tx, player_rx) = channel::<PlayerReceiveEvent>();
        let (event_tx, event_rx) = channel::<ApplicationEvent>();
        App::create_threads(event_tx.clone(), player_rx);

        let mut file_finder = FileFinder::new(
            [".mp3".to_string(), ".ogg".to_string(), ".wav".to_string()],
            path,
            Some(2),
        );

        file_finder.find_paths(None, None);
        let folder_handler = FolderHandler::new(file_finder.folder.clone());

        App {
            exit: false,
            songs: HashMap::new(),
            queue_shown: true,
            folder_handler: folder_handler,
            select_handler: SelectHandler::new(),
            queue_select_handler: SelectHandler::new(),
            file_finder: file_finder,
            player_information: PlayerInformation::default(),
            current_screen: CurrentScreen::Main(FocusedWindowMain::Media(MediaDisplayType::Folder)),
            search_handler: SearchHandler::new(),
            player_tx,
            event_rx,
        }
    }

    async fn run(&mut self) -> Result<(), std::io::Error> {
        let mut terminal = ratatui::init();

        self.file_finder.create_songs().iter().map(|songs| {
            songs.iter().for_each(|song| {
                self.songs.insert(
                    format!(
                        "{}-{}",
                        song.title.clone(),
                        song.artist.clone().unwrap_or("".to_string())
                    ),
                    song.clone(),
                );
            })
        });
        let _ = terminal.draw(|frame| {
            ui::render(frame, self);
        });
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
        };
        match action {
            Action::SwitchWindow => {
                self.current_screen = CurrentScreen::Main(match focused_window {
                    FocusedWindowMain::Media(_) => FocusedWindowMain::Queue,
                    FocusedWindowMain::Queue => FocusedWindowMain::Media(MediaDisplayType::Folder),
                    _ => FocusedWindowMain::Media(MediaDisplayType::Folder),
                })
            }
            Action::MoveUp => match focused_window {
                FocusedWindowMain::Queue => self.queue_select_handler.up(),
                FocusedWindowMain::Media(display_type) => match display_type {
                    MediaDisplayType::Folder => self.select_handler.up(),
                    MediaDisplayType::Songs => self.select_handler.up(),
                },
                _ => {}
            },
            Action::MoveDown => match focused_window {
                FocusedWindowMain::Queue => self.queue_select_handler.down(),
                FocusedWindowMain::Media(display_type) => match display_type {
                    MediaDisplayType::Folder => self.select_handler.down(),
                    MediaDisplayType::Songs => self.select_handler.down(),
                },
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
                FocusedWindowMain::Media(display_type) => match display_type {
                    MediaDisplayType::Songs => {
                        if let Some(index) = self.select_handler.state().selected() {
                            let (queue1, queue2) = self.select_handler.items().split_at(index);
                            let mut queue2 = queue2.to_vec();
                            queue2.append(&mut queue1.to_vec());
                            self.player_tx
                                .send(PlayerReceiveEvent::CreateQueueAndPlay(queue2))
                                .expect("Failed to send song to player");
                        }
                    }
                    MediaDisplayType::Folder => {}
                },
                FocusedWindowMain::Search => {
                    let _ = self.search_handler.search();
                    self.current_screen =
                        CurrentScreen::Main(FocusedWindowMain::Media(MediaDisplayType::Folder));
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
                    self.current_screen =
                        CurrentScreen::Main(FocusedWindowMain::Media(MediaDisplayType::Songs));
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
