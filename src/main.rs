use crate::{
    display_handlers::folder_handler::{Folder, FolderHandler, Node},
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
use log::{debug, error, info, trace, warn};
use std::{
    collections::HashMap,
    env,
    path::{Path, PathBuf},
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
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();

    info!("booting up");
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

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum CurrentScreen {
    Main(FocusedWindowMain),
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum FocusedWindowMain {
    Media,
    Queue,
    Search,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum MediaDisplayType {
    Songs,
    Folders,
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
    selected_media_display_type: MediaDisplayType,
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
        let mut folder_handler =
            FolderHandler::new(Folder::new("root".to_string(), PathBuf::from("root")));
        folder_handler.insert_songs(file_finder.songs.to_owned());
        App {
            exit: false,
            songs: HashMap::new(),
            queue_shown: true,
            folder_handler: folder_handler,
            select_handler: SelectHandler::new(),
            queue_select_handler: SelectHandler::new(),
            file_finder: file_finder,
            player_information: PlayerInformation::default(),
            current_screen: CurrentScreen::Main(FocusedWindowMain::Media),
            selected_media_display_type: MediaDisplayType::Songs,
            search_handler: SearchHandler::new(),
            player_tx,
            event_rx,
        }
    }

    async fn run(&mut self) -> Result<(), std::io::Error> {
        let mut terminal = ratatui::init();
        {
            let songs_vec = self.file_finder.create_songs();
            for song in songs_vec {
                self.songs.insert(
                    format!(
                        "{}-{}",
                        song.title.clone(),
                        song.artist.clone().unwrap_or("".to_string())
                    ),
                    song.clone(),
                );
            }
        }
        self.select_handler
            .set_items(self.songs.values().cloned().collect());
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
            thread::sleep(Duration::from_millis(5));
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
                    FocusedWindowMain::Media => FocusedWindowMain::Queue,
                    FocusedWindowMain::Queue => FocusedWindowMain::Media,
                    _ => FocusedWindowMain::Media,
                })
            }
            Action::MoveUp => match focused_window {
                FocusedWindowMain::Queue => self.queue_select_handler.up(),
                FocusedWindowMain::Media => match self.selected_media_display_type {
                    MediaDisplayType::Folders => self.folder_handler.select_handler_up(),
                    MediaDisplayType::Songs => self.select_handler.up(),
                },
                _ => {}
            },
            Action::MoveDown => match focused_window {
                FocusedWindowMain::Queue => self.queue_select_handler.down(),
                FocusedWindowMain::Media => match self.selected_media_display_type {
                    MediaDisplayType::Folders => self.folder_handler.select_handler_down(),
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
                FocusedWindowMain::Media => match self.selected_media_display_type {
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
                    MediaDisplayType::Folders => {
                        let song = self.folder_handler.select_handler_select();
                        if let Some(song) = song {
                            self.player_tx
                                .send(PlayerReceiveEvent::CreateQueueAndPlay(vec![song]))
                                .expect("Failed to send song to player");
                        }
                    }
                },
                FocusedWindowMain::Search => {
                    let _ = self.search_handler.search();
                    self.current_screen = CurrentScreen::Main(FocusedWindowMain::Media);
                }
            },
            Action::Char(char) => {
                match focused_window {
                    FocusedWindowMain::Search => {
                        self.search_handler.add_char_to_query(char);
                    }
                    FocusedWindowMain::Media => match self.selected_media_display_type {
                        MediaDisplayType::Folders => {
                            if char == 'a' {
                                if let Some(song) = self.folder_handler.select_handler_selected() {
                                    match song {
                                        Node::Folder(folder) => {
                                            let queue: Vec<Song> = folder
                                                .get_children()
                                                .iter()
                                                .filter_map(|child| match child {
                                                    Node::Song(song) => Some(song.to_owned()),
                                                    _ => None,
                                                })
                                                .collect();
                                            info!("Queue created!");
                                            self.player_tx
                                                .send(PlayerReceiveEvent::AddSongsToQueueAndPlay(
                                                    queue,
                                                ))
                                                .expect("Failed to send songs to player");
                                            info!("Playing queue!");
                                        }
                                        Node::Song(song) => self
                                            .player_tx
                                            .send(PlayerReceiveEvent::AddSongsToQueueAndPlay(vec![
                                                song.to_owned(),
                                            ]))
                                            .expect("Failed to send song to player"),
                                    };
                                }
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                }
                if !matches!(focused_window, FocusedWindowMain::Search) {
                    if char == 'f' {
                        self.current_screen = CurrentScreen::Main(FocusedWindowMain::Search);
                    }
                }
            }
            Action::Backspace => match focused_window {
                FocusedWindowMain::Search => {
                    self.search_handler.remove_last_char();
                }
                FocusedWindowMain::Media => match self.selected_media_display_type {
                    MediaDisplayType::Folders => {
                        self.folder_handler.go_to_parent();
                    }
                    _ => {}
                },
                _ => {}
            },
            Action::Esc => match focused_window {
                FocusedWindowMain::Search => {
                    self.current_screen = CurrentScreen::Main(FocusedWindowMain::Media);
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
