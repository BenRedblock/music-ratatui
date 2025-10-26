use ratatui::widgets::ListState;

pub struct SelectHandler<T> {
    items: Vec<T>,
    state: ListState,
}

impl<T> SelectHandler<T> {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            state: ListState::default(),
        }
    }

    pub fn set_items(&mut self, items: Vec<T>) {
        self.items = items;
        self.state.select(Some(0));
    }

    pub fn down(&mut self) {
        self.state.select_next();
    }

    pub fn up(&mut self) {
        self.state.select_previous();
    }

    pub fn select(&self) -> Option<&T> {
        if let Some(index) = self.state.selected() {
            return Some(&self.items[index]);
        }
        return None;
    }

    // Getters:
    pub fn items(&self) -> &Vec<T> {
        return &self.items;
    }

    pub fn state(&mut self) -> &mut ListState {
        &mut self.state
    }
}
