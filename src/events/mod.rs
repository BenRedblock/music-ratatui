use crate::events::keyboard::Action;

pub mod keyboard;

pub enum Events {
    Action(Action),
}
