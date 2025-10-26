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
