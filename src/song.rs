use std::path::Path;

#[derive(Clone)]
pub struct Song {
    pub title: String,
    pub author: Option<String>,
    pub album: Option<String>,
    pub total_time: u32,
    pub file_path: String,
}
