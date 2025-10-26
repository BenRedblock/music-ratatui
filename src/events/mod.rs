use crate::events::keyboard::Action;

pub mod keyboard;

pub enum Event {
    Action(Action),
}
