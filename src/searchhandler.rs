use std::sync::{Arc, Mutex, mpsc::Sender};

use musicbrainz_rs::entity::recording::Recording;
use tokio::task::JoinHandle;

use crate::{
    events::ApplicationEvent, fetch::search::fetch_recording, song::Song,
    utils::selecthandler::SelectHandler,
};

pub struct SearchHandler {
    query: String,
    pub select_handler: Arc<Mutex<SelectHandler<Song>>>,
    running_search: Option<JoinHandle<()>>,
}

impl SearchHandler {
    pub fn new() -> Self {
        SearchHandler {
            query: "".to_string(),
            select_handler: Arc::new(Mutex::new(SelectHandler::new())),
            running_search: None,
        }
    }

    pub fn add_char_to_query(&mut self, char: char) {
        self.query.push(char);
    }

    pub fn remove_last_char(&mut self) {
        self.query.pop();
    }
    pub fn search(&mut self) {
        if let Some(thread) = &mut self.running_search {
            if !thread.is_finished() {
                thread.abort();
            }
        }
        let query_clone = self.query.clone();
        let select_handler_arc = self.select_handler.clone();
        let thread = tokio::spawn(async move {
            let songs = fetch_recording(&query_clone).await.unwrap_or(Vec::new());
            if let Ok(select_handler) = &mut select_handler_arc.lock() {
                select_handler.set_items(songs);
            }
        });
        self.running_search = Some(thread);
    }

    pub fn get_query(&self) -> &str {
        &self.query
    }
}
