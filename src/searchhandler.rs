use musicbrainz_rs::entity::recording::Recording;

use crate::{fetch::search::fetch_recording, song::Song, utils::selecthandler::SelectHandler};

pub struct SearchHandler {
    query: String,
    pub select_handler: SelectHandler<Song>,
}

impl SearchHandler {
    pub fn new() -> Self {
        SearchHandler {
            query: "".to_string(),
            select_handler: SelectHandler::new(),
        }
    }

    pub fn add_char_to_query(&mut self, char: char) {
        self.query.push(char);
    }

    pub fn remove_last_char(&mut self) {
        self.query.pop();
    }

    pub async fn search(&mut self) {
        self.select_handler
            .set_items(fetch_recording(&self.query).await.unwrap_or(Vec::new()));
    }

    pub fn get_query(&self) -> &str {
        &self.query
    }
}
