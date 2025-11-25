use std::{sync::mpsc::Sender, thread};

use crossterm::event::KeyCode;

use crate::events::ApplicationEvent;

pub enum Action {
    Quit,
    SwitchWindow,
    MoveUp,
    MoveDown,
    Select,
    Space,
    NextSong,
    PreviousSong,
    Char(char),
    Backspace,
    Esc,
}

pub struct KeyboardHandler {
    event_tx: Sender<ApplicationEvent>,
}

impl KeyboardHandler {
    pub fn new(event_tx: Sender<ApplicationEvent>) {
        tokio::spawn(async move {
            KeyboardHandler { event_tx }.run().await;
        });
    }

    async fn run(&self) {
        loop {
            match crossterm::event::read().unwrap() {
                crossterm::event::Event::Key(key_event) => {
                    if key_event.modifiers.is_empty() {
                        self.handle(key_event);
                    } else {
                        self.handle_with_modifier(key_event);
                    }
                }
                _ => {}
            }
        }
    }

    fn handle_with_modifier(&self, key_event: crossterm::event::KeyEvent) {
        if key_event
            .modifiers
            .contains(crossterm::event::KeyModifiers::CONTROL)
        {
            if let crossterm::event::KeyCode::Char('c') = key_event.code {
                let _ = self.event_tx.send(ApplicationEvent::Action(Action::Quit));
            }
        }
        if key_event
            .modifiers
            .contains(crossterm::event::KeyModifiers::SHIFT)
        {
            if let crossterm::event::KeyCode::Char(c) = key_event.code {
                let _ = self
                    .event_tx
                    .send(ApplicationEvent::Action(Action::Char(c)));
            }
        }
    }

    fn handle(&self, key_event: crossterm::event::KeyEvent) {
        match key_event.code {
            KeyCode::Up | KeyCode::PageUp => {
                let _ = self.event_tx.send(ApplicationEvent::Action(Action::MoveUp));
            }
            KeyCode::Down | KeyCode::PageDown => {
                let _ = self
                    .event_tx
                    .send(ApplicationEvent::Action(Action::MoveDown));
            }
            KeyCode::Enter => {
                let _ = self.event_tx.send(ApplicationEvent::Action(Action::Select));
            }
            KeyCode::Char(' ') => {
                let _ = self.event_tx.send(ApplicationEvent::Action(Action::Space));
            }
            KeyCode::Left => {
                let _ = self
                    .event_tx
                    .send(ApplicationEvent::Action(Action::PreviousSong));
            }
            KeyCode::Right => {
                let _ = self
                    .event_tx
                    .send(ApplicationEvent::Action(Action::NextSong));
            }
            KeyCode::Tab => {
                let _ = self
                    .event_tx
                    .send(ApplicationEvent::Action(Action::SwitchWindow));
            }
            KeyCode::Char(char) => {
                let _ = self
                    .event_tx
                    .send(ApplicationEvent::Action(Action::Char(char)));
            }
            KeyCode::Backspace => {
                let _ = self
                    .event_tx
                    .send(ApplicationEvent::Action(Action::Backspace));
            }
            KeyCode::Esc => {
                let _ = self.event_tx.send(ApplicationEvent::Action(Action::Esc));
            }
            _ => {}
        }
    }
}
