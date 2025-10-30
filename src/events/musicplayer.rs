use std::{
    sync::mpsc::{Receiver, Sender, channel},
    thread,
    time::Duration,
};

use souvlaki::{
    MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, MediaPosition, PlatformConfig,
};
use vlc::{Event, EventType, Instance, Media, MediaPlayer, MediaPlayerAudioEx, State};

use crate::{events::ApplicationEvent, song::Song};

#[derive(Default)]
pub enum PlayerStatus {
    Playing(Song),
    Paused(Song),
    #[default]
    NoAudioSelected,
}

pub enum PlayerReceiveEvent {
    SetSong(usize),
    SetAndPlaySong(usize),
    CreateQueueAndPlay(Vec<Song>),
    AddSongsToQueueAndPlay(Vec<Song>),
    Play,
    Next,
    Previous,
    Pause,
    TogglePause,
    Update,
}

pub enum PlayerSendEvent {
    PlayerEnded,
    NextSong,
    TimeChanged(u64),
    Pause(usize),
    Play(usize),
    PlayerInformation(PlayerInformation),
    QueueUpdate(Vec<Song>),
}

enum PlayerBackendEvent {
    VLCEvent(Event),
    MediaControls(MediaControlEvent),
}

#[derive(Default)]
pub struct PlayerInformation {
    pub queue: Vec<Song>,
    pub playing_index: Option<usize>,
    pub passed_time: u64,
    pub status: PlayerStatus,
    pub volume: i32,
}

pub struct Player {
    queue: Vec<Song>,
    playing_index: Option<usize>,
    vlc_instance: Instance,
    media_player: MediaPlayer,
    event_tx: Sender<ApplicationEvent>,
    player_rx: Receiver<PlayerReceiveEvent>,
    media_controls: MediaControls,
}

impl Player {
    pub fn new(event_tx: Sender<ApplicationEvent>, player_rx: Receiver<PlayerReceiveEvent>) {
        thread::spawn(move || {
            let instance = Instance::new().unwrap();
            #[cfg(not(target_os = "windows"))]
            let hwnd = None;

            #[cfg(target_os = "windows")]
            let hwnd = {
                use std::os::raw;

                use raw_window_handle::windows::WindowsHandle;

                let handle: WindowsHandle = raw_window_handle;
                Some(handle.hwnd)
            };

            let config = PlatformConfig {
                dbus_name: "musictui",
                display_name: "Musictui",
                hwnd,
            };
            Player {
                queue: Vec::new(),
                playing_index: None,
                media_player: MediaPlayer::new(&instance).unwrap(),
                vlc_instance: instance,
                event_tx,
                player_rx,
                media_controls: MediaControls::new(config).unwrap(),
            }
            .run()
        });
    }

    pub fn run(&mut self) {
        let (vlc_tx, vlc_rx) = channel::<PlayerBackendEvent>();
        self.create_event_thread(vlc_tx);
        loop {
            if let Ok(event) = vlc_rx.try_recv() {
                match event {
                    PlayerBackendEvent::VLCEvent(event) => match event {
                        Event::MediaPlayerTimeChanged => {
                            let passed_time = self.media_player.get_time().unwrap_or(0) as u64;

                            self.event_tx
                                .send(ApplicationEvent::PlayerEvent(PlayerSendEvent::TimeChanged(
                                    passed_time,
                                )))
                                .expect("Error sending TimeChanged event");
                        }
                        Event::MediaPlayerStopped => {
                            self.next_song();
                        }
                        _ => {}
                    },
                    PlayerBackendEvent::MediaControls(event) => match event {
                        MediaControlEvent::Pause => {
                            self.pause();
                        }
                        MediaControlEvent::Next => {
                            self.next_song();
                        }
                        MediaControlEvent::Previous => {
                            self.prev_song();
                        }
                        MediaControlEvent::Play => {
                            self.play();
                        }
                        MediaControlEvent::Toggle => {
                            self.toggle_pause();
                        }
                        _ => {}
                    },
                }
            }
            if let Ok(event) = self.player_rx.try_recv() {
                match event {
                    PlayerReceiveEvent::SetSong(index) => {
                        self.set_song(index);
                    }
                    PlayerReceiveEvent::AddSongsToQueueAndPlay(songs) => {
                        self.add_songs_to_queue(songs);
                    }
                    PlayerReceiveEvent::CreateQueueAndPlay(songs) => {
                        self.create_queue_and_play(songs);
                    }
                    PlayerReceiveEvent::SetAndPlaySong(index) => {
                        self.set_and_play_song(index);
                    }
                    PlayerReceiveEvent::Play => {
                        self.play();
                    }
                    PlayerReceiveEvent::Pause => {
                        self.pause();
                    }
                    PlayerReceiveEvent::TogglePause => {
                        self.toggle_pause();
                    }
                    PlayerReceiveEvent::Update => {
                        self.event_tx
                            .send(ApplicationEvent::PlayerEvent(
                                PlayerSendEvent::PlayerInformation(self.get_player_information()),
                            ))
                            .expect("Error sending Update event");
                    }
                    PlayerReceiveEvent::Previous => {
                        self.prev_song();
                    }
                    PlayerReceiveEvent::Next => {
                        self.next_song();
                    }
                }
            }
            thread::sleep(Duration::from_millis(50));
        }
    }

    fn prev_song(&mut self) {
        if let Some(index) = self.playing_index {
            if index == 0 {
                return;
            }
            self.set_song(index - 1);
            self.play();
        }
    }

    fn next_song(&mut self) {
        if let Some(index) = self.playing_index {
            if index + 1 >= self.queue.iter().len() {
                self.playing_index = None;
                self.event_tx
                    .send(ApplicationEvent::PlayerEvent(PlayerSendEvent::PlayerEnded))
                    .expect("Failed to send player ended event");
                return;
            }
            self.set_song(index + 1);
            self.event_tx
                .send(ApplicationEvent::PlayerEvent(PlayerSendEvent::NextSong))
                .expect("Failed to send next song event");
            self.play();
        }
    }

    fn get_player_information(&self) -> PlayerInformation {
        let passed_time = self.media_player.get_time().unwrap_or(0) as u64;
        PlayerInformation {
            queue: self.queue.clone(),
            playing_index: self.playing_index,
            passed_time: passed_time,
            status: self.get_player_status(),
            volume: self.media_player.get_volume(),
        }
    }

    fn get_player_status(&self) -> PlayerStatus {
        match self.media_player.state() {
            State::Playing => {
                let current_song = self.queue.get(self.playing_index.unwrap_or(0));
                if let Some(song) = current_song {
                    PlayerStatus::Playing(song.clone())
                } else {
                    PlayerStatus::NoAudioSelected
                }
            }
            State::Paused => {
                let current_song = self.queue.get(self.playing_index.unwrap_or(0));
                if let Some(song) = current_song {
                    PlayerStatus::Paused(song.clone())
                } else {
                    PlayerStatus::NoAudioSelected
                }
            }
            _ => PlayerStatus::NoAudioSelected,
        }
    }

    fn toggle_pause(&mut self) {
        if self.media_player.is_playing() {
            self.pause();
        } else {
            self.pause();
        }
    }

    fn add_song_to_queue(&mut self, song: Song) {
        self.queue.push(song);
        self.event_tx
            .send(ApplicationEvent::PlayerEvent(PlayerSendEvent::QueueUpdate(
                self.queue.clone(),
            )))
            .expect("Error sending queue update event");
    }

    fn set_and_play_song(&mut self, index: usize) {
        self.set_song(index);
        self.play();
    }

    fn create_queue_and_play(&mut self, songs: Vec<Song>) {
        self.queue = songs;
        self.event_tx
            .send(ApplicationEvent::PlayerEvent(PlayerSendEvent::QueueUpdate(
                self.queue.clone(),
            )))
            .expect("Error sending queue update event");
        if !self.queue.is_empty() {
            self.set_song(0);
            self.play();
        }
    }

    fn add_songs_to_queue(&mut self, songs: Vec<Song>) {
        for song in songs {
            self.add_song_to_queue(song);
        }
        self.event_tx
            .send(ApplicationEvent::PlayerEvent(PlayerSendEvent::QueueUpdate(
                self.queue.clone(),
            )))
            .expect("Error sending queue update event");
    }

    fn add_to_queue_and_play_song(&mut self, song: Song) {
        self.add_song_to_queue(song);
        self.set_song(self.queue.len() - 1);
        self.play();
    }

    fn play(&mut self) {
        if let Some((_song, index)) = self.get_current_song() {
            self.media_player.play().expect("Failed to play media");
            self.event_tx
                .send(ApplicationEvent::PlayerEvent(PlayerSendEvent::Play(index)))
                .expect("Error sending Play event");
            self.media_controls
                .set_playback(MediaPlayback::Playing { progress: None })
                .unwrap();
        }
    }

    fn pause(&mut self) {
        if let Some((_song, index)) = self.get_current_song() {
            self.media_player.pause();
            self.event_tx
                .send(ApplicationEvent::PlayerEvent(PlayerSendEvent::Pause(index)))
                .expect("Error sending Play event");
            self.media_controls
                .set_playback(MediaPlayback::Paused { progress: None })
                .unwrap();
        }
    }

    fn set_song(&mut self, index: usize) {
        if let Some(song) = self.queue.get(index) {
            let media = Media::new_path(&self.vlc_instance, &song.file_path).unwrap();
            self.playing_index = Some(index);
            self.media_player.set_media(&media);
            self.media_controls
                .set_metadata(MediaMetadata {
                    title: Some(&song.title),
                    artist: Some("Slowdive"),
                    album: Some("Souvlaki"),
                    ..Default::default()
                })
                .unwrap();
        }
    }

    fn get_current_song(&self) -> Option<(&Song, usize)> {
        if let Some(index) = self.playing_index {
            if let Some(song) = self.queue.get(index) {
                return Some((song, index));
            }
        }
        None
    }

    fn create_event_thread(&mut self, player_backend_event_tx: Sender<PlayerBackendEvent>) {
        let event_manager = self.media_player.event_manager();

        let event_tx = player_backend_event_tx.clone();
        let _ = event_manager.attach(EventType::MediaPlayerTimeChanged, move |_, _| {
            let _ = event_tx.send(PlayerBackendEvent::VLCEvent(Event::MediaPlayerTimeChanged));
        });
        let event_tx = player_backend_event_tx.clone();
        let _ = event_manager.attach(EventType::MediaPlayerStopped, move |event, _| {
            let _ = event_tx.send(PlayerBackendEvent::VLCEvent(Event::MediaPlayerStopped));
        });

        // MediaControlls
        let event_tx = player_backend_event_tx.clone();
        let _ = self.media_controls.attach(move |event: MediaControlEvent| {
            let _ = event_tx.send(PlayerBackendEvent::MediaControls(event));
        });
    }
}
