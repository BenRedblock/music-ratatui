use std::{
    sync::mpsc::{Receiver, Sender, channel},
    thread,
};

use vlc::{Event, EventType, Instance, Media, MediaPlayer, State};

use crate::{events::ApplicationEvent, song::Song};

enum PlayerStatus {
    Playing,
    Paused,
    NoAudioSelected,
}

pub enum PlayerReceiveEvent {
    SetSong(Song),
    Play,
    Pause,
}

pub enum PlayerSendEvent {
    SongEnded,
}

pub struct PlayerInformation {
    queue: Vec<Song>,
    playing_index: Option<usize>,
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
                            if let Some(current_index) = self.playing_index {
                                let next_index = current_index + 1;
                                if next_index < self.queue.len() {
                                    let next_song = &self.queue[next_index];
                                    self.set_song(next_song);
                                    self.media_player.play().unwrap();
                                    self.playing_index = Some(next_index);
                                } else {
                                    self.playing_index = None;
                                }
                            }
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
                }
            }
        }
    }

    fn play_song(&mut self, song: &Song) {
        self.set_song(song);
        self.queue = Vec::new();
        self.playing_index = Some(0);
        self.media_player.play().expect("Failed to play media");
    }

    fn set_song(&self, song: &Song) {
        let media = Media::new_path(&self.vlc_instance, &song.file_path).unwrap();
        self.media_player.set_media(&media);
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
