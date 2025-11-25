use ratatui::widgets::{ListItem, ListState};

use crate::{
    display_handlers::folder_handler::{Folder, Node},
    song::Song,
};

pub trait SelectHandlerItem
where
    Self: Clone,
{
    fn list_item(&self) -> ListItem;
}

#[derive(Clone)]
pub enum Selectable {
    Song(Song),
    Node(Node),
}

impl SelectHandlerItem for Selectable {
    fn list_item(&self) -> ListItem {
        match self {
            Selectable::Song(song) => song.list_item(),
            Selectable::Node(node) => node.list_item(),
        }
    }
}

pub struct SelectHandler<T: SelectHandlerItem> {
    items: Vec<T>,
    state: ListState,
}

impl<T: SelectHandlerItem> SelectHandler<T> {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            state: ListState::default(),
        }
    }

    pub fn set_items(&mut self, items: Vec<T>) {
        self.items = items;
        if !self.items.is_empty() {
            self.state.select(Some(0));
        } else {
            self.state.select(None);
        }
    }

    pub fn down(&mut self) {
        self.state.select_next();
    }

    pub fn up(&mut self) {
        self.state.select_previous();
    }

    pub fn select(&self) -> Option<&T> {
        self.state.selected().map(|i| &self.items[i])
    }

    // Getters:
    pub fn items(&self) -> &Vec<T> {
        return &self.items;
    }

    pub fn state(&mut self) -> &mut ListState {
        &mut self.state
    }

    pub fn select_handler_state_and_items(&mut self) -> (&mut ListState, Vec<T>) {
        let list_items: Vec<T>;
        {
            let items = self.items();
            list_items = items.clone();
        }
        (self.state(), list_items)
    }
}
