use musicbrainz_rs::entity::recording::Recording;

use crate::{fetch::search::fetch_recording, utils::selecthandler::SelectHandler};

pub struct SearchHandler {
    query: String,
    pub selecthandler: SelectHandler<Recording>,
}

impl SearchHandler {
    pub fn new() -> Self {
        SearchHandler {
            query: "".to_string(),
            selecthandler: SelectHandler::new(),
        }
    }

    pub async fn add_char_to_query(&mut self, char: char) {
        self.query.push(char);
        let _ = self.search();
    }

    pub async fn remove_last_char(&mut self) {
        self.query.pop();
        let _ = self.search();
    }

    async fn search(&mut self) {
        self.selecthandler
            .set_items(fetch_recording(&self.query).await.unwrap_or(Vec::new()));
    }

    pub fn get_query(&self) -> &str {
        &self.query
    }
}
