use crate::{
    events::{keyboard::Action, musicplayer::PlayerSendEvent},
    song::Song,
};

pub mod keyboard;
pub mod musicplayer;

pub enum ApplicationEvent {
    Action(Action),
    PlayerEvent(PlayerSendEvent),
}

pub fn format_ms_to_duration_string(ms: u64) -> String {
    let seconds = ms / 1000;
    let minutes = seconds / 60;
    let hours = minutes / 60;

    let mut formatted = String::new();

    if hours > 0 {
        formatted.push_str(&format!("{:02}:", hours));
    }

    formatted.push_str(&format!("{:02}:{:02}", minutes % 60, seconds % 60));

    formatted
}
