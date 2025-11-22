use std::path::Path;

use crate::utils::selecthandler::SelectHandlerItem;

#[derive(Clone)]
pub struct Song {
    pub title: String,
    pub author: Option<String>,
    pub album: Option<String>,
    pub total_time: u32,
    pub file_path: String,
}

impl SelectHandlerItem for Song {
    fn title(&self) -> String {
        self.title.clone()
    }
}
