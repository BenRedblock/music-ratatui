use std::{
    sync::mpsc::{Receiver, Sender, channel},
    thread,
    time::Duration,
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
    SetSong(Song),
    Play,
    Pause,
    TogglePause,
}

pub enum PlayerSendEvent {
    PlayerEnded(PlayerInformation),
    NextSong(PlayerInformation),
    TimeChanged(PlayerInformation),
    Pause(PlayerInformation),
    Unpause(PlayerInformation),
    Play(PlayerInformation),
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
}

impl Player {
    pub fn new(event_tx: Sender<ApplicationEvent>, player_rx: Receiver<PlayerReceiveEvent>) {
        thread::spawn(move || {
            let instance = Instance::new().unwrap();
            Player {
                queue: Vec::new(),
                playing_index: None,
                media_player: MediaPlayer::new(&instance).unwrap(),
                vlc_instance: instance,
                event_tx,
                player_rx,
            }
            .run()
        });
    }

    pub fn run(&mut self) {
        let (vlc_tx, vlc_rx) = channel::<Event>();
        self.create_event_thread(vlc_tx);
        loop {
            if let Ok(event) = vlc_rx.try_recv() {
                match event {
                    Event::MediaStateChanged(state) => {
                        if state == State::Ended {
                            self.song_ended();
                        }
                        if state == State::Playing {
                            self.event_tx
                                .send(ApplicationEvent::PlayerEvent(PlayerSendEvent::Play(
                                    self.get_player_information(),
                                )))
                                .expect("Error sending Play event");
                        }
                    }
                    _ => {}
                }
            }
            if let Ok(event) = self.player_rx.try_recv() {
                match event {
                    PlayerReceiveEvent::SetSong(song) => {
                        self.play_song(&song);
                    }
                    PlayerReceiveEvent::Play => {
                        self.media_player.play().expect("Failed to play media");
                    }
                    PlayerReceiveEvent::Pause => {
                        self.media_player.pause();
                    }
                    PlayerReceiveEvent::TogglePause => {
                        self.toggle_pause();
                    }
                }
            }
            thread::sleep(Duration::from_millis(50));
            self.event_tx
                .send(ApplicationEvent::PlayerEvent(PlayerSendEvent::TimeChanged(
                    self.get_player_information(),
                )))
                .expect("Error sending Event");
        }
    }

    fn song_ended(&mut self) {
        if let Some(current_index) = self.playing_index {
            let next_index = current_index + 1;
            if next_index < self.queue.len() {
                self.set_song(next_index);
                self.media_player.play().unwrap();
                self.playing_index = Some(next_index);
                self.event_tx
                    .send(ApplicationEvent::PlayerEvent(PlayerSendEvent::NextSong(
                        self.get_player_information(),
                    )))
                    .unwrap();
            } else {
                self.playing_index = None;
                self.event_tx
                    .send(ApplicationEvent::PlayerEvent(PlayerSendEvent::PlayerEnded(
                        self.get_player_information(),
                    )))
                    .unwrap();
            }
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
            self.media_player.pause();
            self.event_tx
                .send(ApplicationEvent::PlayerEvent(PlayerSendEvent::Pause(
                    self.get_player_information(),
                )))
                .expect("Error sending pause event");
        } else {
            self.media_player.play().expect("Error playing media");
            self.event_tx
                .send(ApplicationEvent::PlayerEvent(PlayerSendEvent::Unpause(
                    self.get_player_information(),
                )))
                .expect("Error sending unpause event");
        }
    }

    fn play_song(&mut self, song: &Song) {
        self.queue.push(song.clone());
        self.set_song(self.queue.len() - 1);
        self.media_player.play().expect("Failed to play media");
        self.event_tx
            .send(ApplicationEvent::PlayerEvent(PlayerSendEvent::Play(
                self.get_player_information(),
            )))
            .expect("Error sending play event");
    }

    fn set_song(&mut self, index: usize) {
        if let Some(song) = self.queue.get(index) {
            let media = Media::new_path(&self.vlc_instance, &song.file_path).unwrap();
            self.playing_index = Some(index);
            self.media_player.set_media(&media);
        }
    }

    fn play(&self) {
        self.media_player.play().unwrap();
    }

    fn create_event_thread(&self, vlc_event_tx: Sender<Event>) {
        let event_manager = self.media_player.event_manager();

        let event_tx = vlc_event_tx.clone();
        let _ = event_manager.attach(EventType::MediaPlayerTimeChanged, move |_, _| {
            let _ = event_tx.send(Event::MediaPlayerTimeChanged);
        });
        let event_tx = vlc_event_tx.clone();
        let _ = event_manager.attach(EventType::MediaStateChanged, move |event, _| {
            if let Event::MediaStateChanged(state) = event {
                let _ = event_tx.send(Event::MediaStateChanged(state));
            }
        });
    }
}
