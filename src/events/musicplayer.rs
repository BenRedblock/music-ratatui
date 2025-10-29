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
    PlayerEnded(PlayerInformation),
    NextSong(PlayerInformation),
    TimeChanged(PlayerInformation),
    Pause(PlayerInformation),
    Unpause(PlayerInformation),
    Play(PlayerInformation),
    PlayerInformation(PlayerInformation),
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
                        todo!("Does not get called");
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
                    Event::MediaPlayerTimeChanged => {
                        let playerinfo = self.get_player_information();
                        self.event_tx
                            .send(ApplicationEvent::PlayerEvent(PlayerSendEvent::TimeChanged(
                                playerinfo,
                            )))
                            .expect("Error sending TimeChanged event");
                    }
                    _ => {}
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
                        self.media_player.pause();
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
                return;
            }
            self.set_song(index + 1);
            self.play();
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

    fn add_song_to_queue(&mut self, song: Song) {
        self.queue.push(song);
    }

    fn set_and_play_song(&mut self, index: usize) {
        self.set_song(index);
        self.play();
    }

    fn create_queue_and_play(&mut self, songs: Vec<Song>) {
        self.queue = songs;
        if !self.queue.is_empty() {
            self.set_song(0);
            self.play();
        }
    }

    fn add_songs_to_queue(&mut self, songs: Vec<Song>) {
        for song in songs {
            self.add_song_to_queue(song);
        }
    }

    fn add_to_queue_and_play_song(&mut self, song: Song) {
        self.add_song_to_queue(song);
        self.set_song(self.queue.len() - 1);
        self.play();
    }

    fn play(&mut self) {
        self.media_player.play().expect("Failed to play media");
    }

    fn set_song(&mut self, index: usize) {
        if let Some(song) = self.queue.get(index) {
            let media = Media::new_path(&self.vlc_instance, &song.file_path).unwrap();
            self.playing_index = Some(index);
            self.media_player.set_media(&media);
        }
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
